use crate::{
    git,
    model::*,
    repository::{self, Sessions},
    stack::{NodeParser, StackTraceParser},
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

pub fn investigate(
    sessions: &Sessions,
    token: &str,
    stack: &str,
    logs: &str,
    tests: &str,
) -> Result<Investigation> {
    if stack.len() + logs.len() + tests.len() > 128_000 {
        return Err("Incident inputs exceed the 128 KB limit".into());
    }
    if stack.trim().is_empty() {
        return Err("Paste a JavaScript / TypeScript stack trace".into());
    }
    let root = sessions.root(token)?;
    let repository = repository::metadata(&root, token)?;
    let case_id = id();
    let mut frames = NodeParser.parse(stack);
    if frames.is_empty() {
        return Err(
            "No Node.js stack locations found. Expected: at function (src/file.ts:line:column)"
                .into(),
        );
    }
    let title = stack
        .lines()
        .find(|l| {
            l.trim()
                .split(':')
                .next()
                .map(|s| s.ends_with("Error") || s.ends_with("Exception"))
                .unwrap_or(false)
        })
        .or_else(|| stack.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("Incident")
        .trim()
        .chars()
        .take(120)
        .collect::<String>();
    let mut case = Investigation {
        id: case_id.clone(),
        title: title.clone(),
        created_at: now(),
        repository,
        stack: stack.into(),
        logs: logs.into(),
        tests: tests.into(),
        frames: vec![],
        nodes: vec![],
        edges: vec![],
        warnings: vec![],
        source_hashes: BTreeMap::new(),
        stale: false,
    };
    let p = Provenance::input("stack_input", stack, &case_id);
    let incident = case.node(
        "incident",
        Status::Observed,
        &title,
        "User-supplied incident; reproduction has not been independently verified",
        "parser",
        p.clone(),
        json!({}),
    );
    let mut commits: HashMap<String, String> = HashMap::new();
    for f in frames.iter_mut().take(8) {
        let fp = Provenance::input("stack_input", &f.raw, &case_id);
        let frame = case.node(
            "stack_frame",
            Status::Observed,
            f.function_name.as_deref().unwrap_or("Stack frame"),
            &f.raw,
            "parser",
            fp.clone(),
            json!(f),
        );
        case.edge(
            &incident,
            &frame,
            "occurred_at",
            Status::Derived,
            fp.clone(),
        );
        let src = match repository::source(&root, &f.file_path, f.line, f.column) {
            Ok(s) => s,
            Err(e) => {
                case.warnings.push(format!("Frame {}: {e}", f.index));
                if let Some(n) = case.nodes.iter_mut().find(|n| n.id == frame) {
                    n.validity = "unresolved".into();
                }
                continue;
            }
        };
        f.resolved = true;
        case.source_hashes
            .insert(src.path.clone(), src.hash.clone());
        let mut sp = Provenance::input("source", &src.lines.join("\n"), &case_id);
        sp.file_path = Some(src.path.clone());
        sp.line_start = Some(src.start);
        sp.line_end = Some(src.start + src.lines.len() - 1);
        sp.source_hash = Some(src.hash.clone());
        let source = case.node(
            "source_line",
            Status::Derived,
            &format!("{}:{}", src.path, f.line),
            &src.lines[f.line - src.start],
            "static_analysis",
            sp.clone(),
            json!(src),
        );
        case.edge(&frame, &source, "occurred_at", Status::Derived, sp.clone());
        if case.repository.head.is_none() {
            case.warnings.push(
                "Repository has no commits; source is inspectable but history is unavailable"
                    .into(),
            );
            continue;
        }
        let dirty_args = vec![
            "diff".into(),
            "HEAD".into(),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "--".into(),
            src.path.clone(),
        ];
        let dirty = git::run_git(&root, &dirty_args)?;
        if !dirty.is_empty() {
            let p = git::provenance(dirty_args, &dirty, Some(&src.path), None, None);
            let n = case.node(
                "working_diff",
                Status::Derived,
                "Working tree changes",
                "Source differs from HEAD; inspect alongside committed history",
                "git",
                p.clone(),
                json!({"patch":dirty}),
            );
            case.edge(&source, &n, "modified_by", Status::Derived, p);
        }
        let args = vec![
            "log".into(),
            "-12".into(),
            "--format=%H%x1f%an%x1f%aI%x1f%s%x00".into(),
            "--".into(),
            src.path.clone(),
        ];
        let history_raw = git::run_git(&root, &args)?;
        let history = git::parse_commits(&history_raw);
        let hp = git::provenance(args, &history_raw, Some(&src.path), None, None);
        let hnode = case.node(
            "file_history",
            Status::Derived,
            "Relevant file history",
            &format!("{} recent commits", history.len()),
            "git",
            hp.clone(),
            json!(history),
        );
        case.edge(&source, &hnode, "contains", Status::Derived, hp);
        let range = format!("{},{}", f.line, f.line);
        let args = vec![
            "blame".into(),
            "--no-textconv".into(),
            "--line-porcelain".into(),
            "-L".into(),
            range,
            "--".into(),
            src.path.clone(),
        ];
        let raw = match git::run_git(&root, &args) {
            Ok(r) => r,
            Err(e) => {
                case.warnings.push(e);
                continue;
            }
        };
        let blame = git::parse_blame(&raw)?;
        let bp = git::provenance(
            args,
            &raw,
            Some(&src.path),
            Some(f.line),
            if blame.uncommitted {
                None
            } else {
                Some(&blame.sha)
            },
        );
        let bn = case.node(
            "blame_result",
            Status::Derived,
            if blame.uncommitted {
                "Uncommitted line"
            } else {
                "Last changed line"
            },
            "Blame records last modification. It does not establish the cause of this incident.",
            "git",
            bp.clone(),
            json!(blame),
        );
        case.edge(&source, &bn, "modified_by", Status::Derived, bp.clone());
        if blame.uncommitted {
            case.warnings.push(format!(
                "{}:{} has uncommitted changes; no committed suspect assigned",
                src.path, f.line
            ));
            continue;
        }
        let cn = if let Some(n) = commits.get(&blame.sha) {
            n.clone()
        } else {
            let args = vec![
                "show".into(),
                "-s".into(),
                "--format=%H%x1f%an%x1f%aI%x1f%s%x00".into(),
                blame.sha.clone(),
            ];
            let raw = git::run_git(&root, &args)?;
            let commit = git::parse_commits(&raw)
                .into_iter()
                .next()
                .ok_or("Cannot parse Git commit metadata")?;
            let cp = git::provenance(args, &raw, None, None, Some(&blame.sha));
            let n = case.node(
                "commit",
                Status::Derived,
                &commit.subject,
                &commit.sha,
                "git",
                cp,
                json!(commit),
            );
            commits.insert(blame.sha.clone(), n.clone());
            n
        };
        case.edge(&bn, &cn, "modified_by", Status::Derived, bp);
        let args = vec![
            "show".into(),
            "--format=fuller".into(),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "--no-renames".into(),
            "--unified=6".into(),
            blame.sha.clone(),
            "--".into(),
            src.path.clone(),
        ];
        let patch = git::run_git(&root, &args)?;
        let dp = git::provenance(
            args,
            &patch,
            Some(&src.path),
            Some(f.line),
            Some(&blame.sha),
        );
        let guards: Vec<_> = patch
            .lines()
            .filter(|l| {
                l.starts_with('-')
                    && !l.starts_with("---")
                    && l.contains("if")
                    && (l.contains("null") || l.contains('!') || l.contains("undefined"))
            })
            .map(str::to_string)
            .collect();
        let dn = case.node(
            "diff_hunk",
            Status::Derived,
            "Actual commit patch",
            &src.path,
            "git",
            dp.clone(),
            json!({"patch":patch,"removedGuardLines":guards}),
        );
        case.edge(&cn, &dn, "contains", Status::Derived, dp.clone());
        let mut signals = vec![
            "Commit last modified the current crash line".to_string(),
            "Commit touched the crash file".to_string(),
        ];
        if !guards.is_empty() {
            signals.push("Patch contains removed lines matching a guard-text heuristic".into());
        }
        let signal = case.node(
            "candidate_signal",
            Status::Correlated,
            "Investigation candidate",
            "Line modification is correlation; root cause remains unverified",
            "static_analysis",
            dp.clone(),
            json!({"signals":signals,"score":if guards.is_empty(){2}else{3}}),
        );
        case.edge(
            &dn,
            &signal,
            "correlates_with",
            Status::Correlated,
            dp.clone(),
        );
        case.edge(&source, &signal, "correlates_with", Status::Correlated, dp);
    }
    if frames.len() > 8 {
        case.warnings.push("Graph includes the first eight frames; all parsed frames remain in the stack inspector".into());
    }
    case.frames = frames;
    for (kind, raw, title) in [
        ("log_event", logs, "Attached logs"),
        ("test_result", tests, "Attached test output"),
    ] {
        if raw.trim().is_empty() {
            continue;
        }
        let p = Provenance::input(
            if kind == "test_result" {
                "pasted_test_output"
            } else {
                "pasted_logs"
            },
            raw,
            &case_id,
        );
        let n = case.node(
            kind,
            Status::Observed,
            title,
            "User-supplied output; TRACE did not execute this command",
            "parser",
            p.clone(),
            json!({"origin":"pasted", "independentlyExecuted":false}),
        );
        case.edge(&incident, &n, "contains", Status::Observed, p);
    }
    let after = repository::metadata(&root, token)?;
    if after.head != case.repository.head || after.status != case.repository.status {
        return Err(
            "Repository changed during analysis. Retry to capture a consistent snapshot.".into(),
        );
    }
    for (path, h) in &case.source_hashes {
        let p = repository::resolve(&root, path)?;
        if hash(&repository::read_source(&p)?) != *h {
            return Err("Source changed during analysis. Retry the investigation.".into());
        }
    }
    Ok(case)
}

pub fn check_stale(case: &mut Investigation, sessions: &Sessions) -> Result<()> {
    let repo = sessions.open(&case.repository.root)?;
    let root = sessions.root(&repo.token)?;
    let repo_changed = repo.head != case.repository.head || repo.status != case.repository.status;
    case.repository.token = repo.token;
    let mut changed = BTreeMap::new();
    for (file, h) in &case.source_hashes {
        let state = match repository::resolve(&root, file) {
            Ok(p) => match repository::read_source(&p) {
                Ok(b) => {
                    if hash(&b) == *h {
                        "snapshot"
                    } else {
                        "stale"
                    }
                }
                Err(_) => "invalid",
            },
            Err(_) => "invalid",
        };
        changed.insert(file.clone(), state);
    }
    case.stale = repo_changed || changed.values().any(|v| *v != "snapshot");
    for n in &mut case.nodes {
        n.validity = if let Some(f) = &n.provenance.file_path {
            changed.get(f).copied().unwrap_or("snapshot").into()
        } else {
            "snapshot".into()
        };
    }
    if case.stale {
        case.warnings.push("Snapshot is stale: repository HEAD, status or source content changed. Inspect retained evidence or start a fresh investigation.".into());
    }
    Ok(())
}
