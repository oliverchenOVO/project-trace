//! Native-owned approved experiments, isolated checkout mutations and retained proof.
use crate::{
    execution::{self, Execution, ResolvedCommand, TestCommand},
    git,
    model::*,
    repository, store,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub case_id: String,
    pub command: TestCommand,
    pub known_good: Option<String>,
    pub known_bad: String,
    pub mode: String,
    pub hypothesis_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceState {
    pub head: Option<String>,
    pub branch: String,
    pub status: String,
    pub files_hash: String,
    pub file_count: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub sequence: usize,
    pub commit_sha: String,
    pub test_execution_id: String,
    pub classification: String,
    pub role: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRun {
    pub id: String,
    pub case_id: String,
    pub repository: String,
    pub worktree: String,
    pub artifacts: String,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub status: String,
    pub mode: String,
    pub command: ResolvedCommand,
    pub known_good: Option<String>,
    pub known_bad: String,
    pub first_bad: Option<String>,
    pub suspect: Option<String>,
    pub hypothesis_id: Option<String>,
    pub executions: Vec<Execution>,
    pub steps: Vec<Step>,
    pub git_operations: Vec<Execution>,
    pub before: SourceState,
    pub after: Option<SourceState>,
    pub source_unchanged: Option<bool>,
    pub cleanup_status: String,
    pub error: Option<String>,
    pub approval_token: Option<String>,
    pub integrity_hash: String,
}
pub fn run_digest(r: &VerificationRun) -> String {
    let mut v = r.clone();
    v.integrity_hash.clear();
    hash(&serde_json::to_vec(&v).unwrap())
}
pub fn source_state(root: &Path) -> Result<SourceState> {
    let (head, branch, status, _) = repository::snapshot(root)?;
    let files = git::git(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )?;
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(head.as_deref().unwrap_or(""));
    digest.update(&branch);
    digest.update(&status);
    let mut count = 0;
    let mut names: Vec<_> = files.split('\0').filter(|s| !s.is_empty()).collect();
    names.sort();
    names.dedup();
    for name in names {
        let path = root.join(name);
        digest.update(name.as_bytes());
        match std::fs::symlink_metadata(&path) {
            Ok(m) if m.file_type().is_symlink() => {
                digest.update(
                    std::fs::read_link(&path)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .as_bytes(),
                );
            }
            Ok(m) if m.is_file() => {
                let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
                let mut b = [0u8; 65536];
                loop {
                    let n = f.read(&mut b).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    digest.update(&b[..n]);
                }
            }
            Ok(_) => digest.update(b"DIRECTORY"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => digest.update(b"MISSING"),
            Err(e) => return Err(e.to_string()),
        };
        count += 1;
    }
    // Detect staging-only modifications, including index changes with unchanged bytes.
    digest.update(git::git(
        root,
        &[
            "diff",
            "--cached",
            "--binary",
            "--no-ext-diff",
            "--no-textconv",
        ],
    )?);
    Ok(SourceState {
        head,
        branch,
        status,
        files_hash: format!("{:x}", digest.finalize()),
        file_count: count,
    })
}
pub struct Service {
    db: PathBuf,
    base: PathBuf,
    active: Mutex<HashMap<String, Arc<AtomicBool>>>,
}
impl Service {
    pub fn new(db: PathBuf, base: PathBuf) -> Result<Arc<Self>> {
        std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
        let base = base.canonicalize().map_err(|e| e.to_string())?;
        let s = Arc::new(Self {
            db,
            base,
            active: Mutex::new(HashMap::new()),
        });
        for mut r in store::runs(&s.db, None)? {
            if ["running", "cancelling", "cleaning_up"].contains(&r.status.as_str()) {
                r.status = "interrupted".into();
                r.error=Some("INTERRUPTED VERIFICATION. Execution will not resume automatically. Inspect retained steps and clean up the owned worktree.".into());
                r.approval_token = None;
                r.completed_at = Some(now());
                s.persist(&mut r)?;
            }
        }
        Ok(s)
    }
    pub fn list(&self, case_id: &str) -> Result<Vec<VerificationRun>> {
        let mut runs = store::runs(&self.db, Some(case_id))?;
        for r in &mut runs {
            if let Err(err) = validate_run(r) {
                r.status = "integrity_error".into();
                r.error = Some(err);
            }
        }
        Ok(runs)
    }
    pub fn busy(&self, case_id: &str) -> bool {
        store::runs(&self.db, Some(case_id))
            .unwrap_or_default()
            .iter()
            .any(|r| r.status == "running" || r.status == "cancelling" || r.status == "cleaning_up")
    }
    pub fn prepare(&self, request: Request) -> Result<VerificationRun> {
        if self.busy(&request.case_id) {
            return Err("An experiment is already running for this case".into());
        }
        let case = store::load(&self.db, &request.case_id)?;
        let root = Path::new(&case.repository.root)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if self.base.starts_with(&root) {
            return Err("Temporary verification storage is inside the selected repository. Select a narrower repository or relocate TRACE local data.".into());
        }
        if root.join(".gitmodules").exists() {
            return Err("Submodule repositories are not supported by verification yet; no worktree was created.".into());
        }
        if !["test_bad", "test_good", "bisect"].contains(&request.mode.as_str()) {
            return Err("Unknown verification mode".into());
        }
        if request.hypothesis_id.as_ref().is_some_and(|id| {
            !case
                .nodes
                .iter()
                .any(|n| n.id == *id && n.kind == "hypothesis")
        }) {
            return Err("Select an actual case hypothesis".into());
        }
        let before = source_state(&root)?;
        let bad = resolve_revision(&root, &request.known_bad)?;
        if before.head.as_ref() != Some(&bad) {
            return Err("Known bad must equal the current repository HEAD. Reopen or start a fresh case after switching revisions.".into());
        }
        let good = request
            .known_good
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| resolve_revision(&root, s))
            .transpose()?;
        if request.mode != "test_bad" && good.is_none() {
            return Err("Select a known-good commit".into());
        }
        if good.as_ref() == Some(&bad) {
            return Err("Known-good and known-bad revisions must differ".into());
        }
        let command = execution::resolve_command(&request.command, &root)?;
        let rid = id();
        let runbase = self.base.join(&rid);
        let worktree = runbase.join("tree");
        let artifacts = runbase.join("artifacts");
        std::fs::create_dir(&runbase).map_err(|e| e.to_string())?;
        let mut run = VerificationRun {
            id: rid,
            case_id: case.id,
            repository: repository::display_path(&root),
            worktree: repository::display_path(&worktree),
            artifacts: repository::display_path(&artifacts),
            created_at: now(),
            started_at: None,
            completed_at: None,
            status: "awaiting_approval".into(),
            mode: request.mode,
            command,
            known_good: good,
            known_bad: bad,
            first_bad: None,
            suspect: case
                .nodes
                .iter()
                .find(|n| n.kind == "blame_result")
                .and_then(|n| n.data["sha"].as_str())
                .map(str::to_string),
            hypothesis_id: request.hypothesis_id,
            executions: vec![],
            steps: vec![],
            git_operations: vec![],
            before,
            after: None,
            source_unchanged: None,
            cleanup_status: "not_created".into(),
            error: None,
            approval_token: Some(id()),
            integrity_hash: String::new(),
        };
        std::fs::write(
            runbase.join("owner.json"),
            serde_json::to_vec(
                &json!({"id":run.id,"repository":run.repository,"caseId":run.case_id}),
            )
            .unwrap(),
        )
        .map_err(|e| e.to_string())?;
        self.persist(&mut run)?;
        Ok(run)
    }
    pub fn approve(self: &Arc<Self>, rid: &str, token: &str) -> Result<VerificationRun> {
        let mut active = self.active.lock().map_err(|_| "Verification lock failed")?;
        let mut r = store::load_run(&self.db, rid)?;
        validate_run(&r)?;
        if r.status != "awaiting_approval" || r.approval_token.as_deref() != Some(token) {
            return Err("Execution requires the current explicit approval token".into());
        }
        if self.busy(&r.case_id) {
            return Err("An experiment is already running for this case".into());
        }
        if source_state(Path::new(&r.repository))? != r.before {
            return Err(
                "SOURCE REPOSITORY CHANGED BEFORE APPROVAL. Review a fresh execution plan.".into(),
            );
        }
        r.approval_token = None;
        r.status = "running".into();
        r.started_at = Some(now());
        self.persist(&mut r)?;
        let flag = Arc::new(AtomicBool::new(false));
        active.insert(r.id.clone(), flag.clone());
        let service = self.clone();
        let worker_id = r.id.clone();
        std::thread::spawn(move || {
            service.worker(r, flag);
            service.active.lock().unwrap().remove(&worker_id);
        });
        Ok(store::load_run(&self.db, rid)?)
    }
    pub fn cancel(&self, rid: &str) -> Result<()> {
        if let Some(flag) = self
            .active
            .lock()
            .map_err(|_| "Verification lock failed")?
            .get(rid)
        {
            flag.store(true, Ordering::SeqCst);
            return Ok(());
        }
        let mut r = store::load_run(&self.db, rid)?;
        if r.status == "awaiting_approval" {
            r.status = "cancelled".into();
            r.approval_token = None;
            r.completed_at = Some(now());
            self.persist(&mut r)?;
            return Ok(());
        }
        Err("No active verification to cancel".into())
    }
    fn persist(&self, r: &mut VerificationRun) -> Result<()> {
        r.integrity_hash = run_digest(r);
        let mut snapshot = r.clone();
        if snapshot.started_at.is_some()
            && snapshot.completed_at.is_none()
            && snapshot.status != "running"
        {
            snapshot.status = "cleaning_up".into();
            snapshot.integrity_hash = run_digest(&snapshot);
        }
        let mut c = store::load(&self.db, &r.case_id)?;
        if snapshot.source_unchanged==Some(false){c.stale=true;}
        attach(&mut c, &snapshot);
        store::save_bundle(&self.db, &c, &snapshot)
    }
    fn git_op(
        &self,
        r: &mut VerificationRun,
        cwd: &Path,
        args: &[&str],
        flag: Arc<AtomicBool>,
    ) -> Result<Execution> {
        let program = git::installed_git(
            Path::new(&r.repository),
            std::env::var_os("PATH").as_deref(),
        )?;
        let hooks = self.base.join(&r.id).join("disabled-hooks");
        std::fs::create_dir_all(&hooks).map_err(|e| e.to_string())?;
        let mut safe = vec![
            "--no-optional-locks".into(),
            "-c".into(),
            "core.fsmonitor=false".into(),
            "-c".into(),
            "core.pager=cat".into(),
            "-c".into(),"core.longpaths=true".into(),
            "-c".into(),
            format!("core.hooksPath={}", repository::display_path(&hooks)),
            "-c".into(),
            "submodule.recurse=false".into(),
            "-c".into(),
            "protocol.file.allow=never".into(),
        ];
        // Disable every configured checkout filter: repository configuration is untrusted.
        let filters = self.config_filters(r, flag.clone())?;
        for name in filters {
            for field in ["process", "smudge", "clean"] {
                safe.extend(["-c".into(), format!("filter.{name}.{field}=")]);
            }
            safe.extend(["-c".into(), format!("filter.{name}.required=false")]);
        }
        safe.extend(args.iter().map(|s| s.to_string()));
        let command = ResolvedCommand {
            requested: TestCommand {
                executable: repository::display_path(&program),
                args: safe.clone(),
                working_directory: ".".into(),
                timeout_ms: 60000,
            },
            executable: repository::display_path(&program),
            args: safe,
            environment: json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"policy":"Native-controlled Git mutation, hooks/filters/submodules disabled"}),
        };
        let result = execution::execute(
            &command,
            cwd,
            "git-administration",
            Path::new(&r.artifacts),
            flag,
        )?;
        r.git_operations.push(result.clone());
        self.persist(r)?;
        Ok(result)
    }
    fn config_filters(
        &self,
        r: &mut VerificationRun,
        flag: Arc<AtomicBool>,
    ) -> Result<Vec<String>> {
        let root = PathBuf::from(&r.repository);
        let program = git::installed_git(&root, std::env::var_os("PATH").as_deref())?;
        let args: Vec<String> = [
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "config",
            "--name-only",
            "--get-regexp",
            r"^filter\.",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let cmd = ResolvedCommand {
            requested: TestCommand {
                executable: repository::display_path(&program),
                args: args.clone(),
                working_directory: ".".into(),
                timeout_ms: 60000,
            },
            executable: repository::display_path(&program),
            args,
            environment: json!({"policy":"Bounded read-only Git configuration names; no filter execution"}),
        };
        let output = execution::execute(
            &cmd,
            &root,
            "git-configuration",
            Path::new(&r.artifacts),
            flag,
        )?;
        r.git_operations.push(output.clone());
        self.persist(r)?;
        if output.status != "passed" && !(output.status == "failed" && output.exit_code == Some(1))
        {
            return Err(format!(
                "Git filter inspection failed: {} {}",
                output.status, output.stderr
            ));
        }
        if output.output_truncated {
            return Err("Too many Git filter configuration entries".into());
        }
        let mut names = vec![];
        for key in output.stdout.lines() {
            if let Some(key) = key.strip_prefix("filter.") {
                if let Some((name, _)) = key.rsplit_once('.') {
                    if name.contains('\n') || name.len() > 200 {
                        return Err("Invalid Git filter name".into());
                    }
                    if !names.contains(&name.to_string()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
        Ok(names)
    }
    fn successful_git(
        &self,
        r: &mut VerificationRun,
        cwd: &Path,
        args: &[&str],
        flag: Arc<AtomicBool>,
    ) -> Result<String> {
        let result = self.git_op(r, cwd, args, flag)?;
        if result.status != "passed" {
            return Err(format!(
                "Git {}: {} {}",
                args.join(" "),
                result.status,
                result.stderr
            ));
        }
        Ok(result.stdout)
    }
    fn evaluate(
        &self,
        r: &mut VerificationRun,
        sha: &str,
        role: &str,
        flag: Arc<AtomicBool>,
    ) -> Result<String> {
        if flag.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        let tree = PathBuf::from(&r.worktree);
        // Reset only the owned isolated worktree, never the active checkout. Untracked outputs
        // are removed by rebuilding the worktree between candidates to prevent contamination.
        if tree.exists() {
            self.successful_git(
                r,
                Path::new(&r.repository).to_path_buf().as_path(),
                &["worktree", "remove", "--force", &r.worktree.clone()],
                flag.clone(),
            )?;
        }
        let repo = PathBuf::from(&r.repository);
        let path = r.worktree.clone();
        self.successful_git(
            r,
            &repo,
            &["worktree", "add", "--detach", &path, sha],
            flag.clone(),
        )?;
        r.cleanup_status = "pending".into();
        self.persist(r)?;
        let actual = resolve_revision(&tree, "HEAD")?;
        if actual != sha {
            return Err("Temporary worktree revision does not match experiment".into());
        }
        let e = execution::execute(&r.command, &tree, sha, Path::new(&r.artifacts), flag)?;
        let class = execution::classification(&e).unwrap_or("error").to_string();
        r.steps.push(Step {
            sequence: r.steps.len() + 1,
            commit_sha: sha.into(),
            test_execution_id: e.id.clone(),
            classification: class.clone(),
            role: role.into(),
        });
        r.executions.push(e.clone());
        self.persist(r)?;
        if class == "error" {
            return Err(format!(
                "VERIFICATION INCONCLUSIVE: {} {}",
                e.status,
                e.error.unwrap_or_default()
            ));
        }
        Ok(class)
    }
    fn experiment(&self, r: &mut VerificationRun, flag: Arc<AtomicBool>) -> Result<()> {
        let bad = r.known_bad.clone();
        let good = r.known_good.clone();
        if let Some(ref good) = good {
            let repo = PathBuf::from(&r.repository);
            let relationship = self.git_op(
                r,
                &repo,
                &["merge-base", "--is-ancestor", good, &bad],
                flag.clone(),
            )?;
            if relationship.status != "passed" {
                return Err("Known good must be an ancestor of known bad".into());
            }
        }
        if r.mode == "test_good" {
            if self.evaluate(r, good.as_deref().unwrap(), "known_good", flag)? != "good" {
                return Err("INVALID GOOD BOUNDARY: selected good revision fails".into());
            }
            r.status = "good_confirmed".into();
            return Ok(());
        }
        if self.evaluate(r, &bad, "known_bad", flag.clone())? != "bad" {
            return Err("INVALID BAD BOUNDARY: selected bad revision passes".into());
        }
        if r.mode == "test_bad" {
            r.status = "bad_confirmed".into();
            return Ok(());
        }
        let good = good.unwrap();
        if self.evaluate(r, &good, "known_good", flag.clone())? != "good" {
            return Err("INVALID GOOD BOUNDARY: selected good revision fails".into());
        }
        // Git supplies DAG-aware bisect candidates. Each candidate is tested in a newly
        // created clean worktree; bisect state lives in a separate owned coordinator.
        let repo = PathBuf::from(&r.repository);
        let coordinator = self.base.join(&r.id).join("bisect");
        let coord = repository::display_path(&coordinator);
        self.successful_git(
            r,
            &repo,
            &["worktree", "add", "--detach", &coord, &bad],
            flag.clone(),
        )?;
        let result = (|| -> Result<()> {
            self.successful_git(
                r,
                &coordinator,
                &["bisect", "start", &bad, &good, "--"],
                flag.clone(),
            )?;
            for _ in 0..128 {
                if flag.load(Ordering::SeqCst) {
                    return Err("cancelled".into());
                }
                let sha = resolve_revision(&coordinator, "HEAD")?;
                let class = self.evaluate(r, &sha, "bisect_candidate", flag.clone())?;
                self.successful_git(r, &coordinator, &["bisect", &class, &sha], flag.clone())?;
                let gitdir = git::git(&coordinator, &["rev-parse", "--absolute-git-dir"])?;
                let marker = Path::new(gitdir.trim()).join("BISECT_EXPECTED_REV");
                // BISECT_EXPECTED_REV is removed by Git when narrowing completes. BISECT_LOG
                // carries the final first-bad comment; independently require a full SHA there.
                let log = std::fs::read_to_string(Path::new(gitdir.trim()).join("BISECT_LOG"))
                    .map_err(|e| e.to_string())?;
                if let Some(line) = log.lines().find(|s| s.starts_with("# first bad commit: [")) {
                    let first = line
                        .strip_prefix("# first bad commit: [")
                        .unwrap()
                        .split(']')
                        .next()
                        .unwrap();
                    if !git::valid_sha(first) {
                        return Err("Git bisect returned invalid revision".into());
                    }
                    let first = first.to_string();
                    if self.evaluate(r, &first, "boundary_bad_confirmation", flag.clone())? != "bad"
                    {
                        return Err(
                            "VERIFICATION INCONCLUSIVE: boundary failed repeat check".into()
                        );
                    }
                    let parents = git::git(&repo, &["show", "-s", "--format=%P", &first])?;
                    let parents: Vec<_> = parents.split_whitespace().collect();
                    // Phase 1.1 rejects ambiguous merge boundaries instead of overclaiming.
                    if parents.len() != 1 {
                        return Err("VERIFICATION INCONCLUSIVE: merge/root boundary requires a richer predicate model".into());
                    }
                    if self.evaluate(r, parents[0], "boundary_parent_confirmation", flag.clone())?
                        != "good"
                    {
                        return Err(
                            "VERIFICATION INCONCLUSIVE: parent does not pass repeat check".into(),
                        );
                    }
                    r.first_bad = Some(first);
                    r.status = "bisect_confirmed".into();
                    return Ok(());
                }
                let _ = marker;
            }
            Err("VERIFICATION INCONCLUSIVE: bisect evaluation limit reached".into())
        })();
        let reset = self.successful_git(
            r,
            &coordinator,
            &["bisect", "reset"],
            Arc::new(AtomicBool::new(false)),
        );
        let remove = self.successful_git(
            r,
            &repo,
            &["worktree", "remove", "--force", &coord],
            Arc::new(AtomicBool::new(false)),
        );
        if reset.is_err() || remove.is_err() {
            r.cleanup_status = "required".into();
            r.error = Some(format!("WORKTREE CLEANUP REQUIRED: {coord}"));
        }
        result
    }
    fn worker(&self, mut r: VerificationRun, flag: Arc<AtomicBool>) {
        let result = self.experiment(&mut r, flag.clone());
        if flag.load(Ordering::SeqCst) {
            r.status = "cancelled".into();
            r.error = Some("Verification cancelled; completed evidence retained".into());
        } else if let Err(err) = result {
            r.status = if err.contains("INCONCLUSIVE") {
                "inconclusive"
            } else {
                "failed"
            }
            .into();
            r.error = Some(err);
        }
        if let Err(err) = self.cleanup_inner(&mut r, false) {
            r.cleanup_status = "required".into();
            r.error = Some(format!(
                "{}; WORKTREE CLEANUP REQUIRED: {err}",
                r.error.clone().unwrap_or_default()
            ));
        }
        match source_state(Path::new(&r.repository)) {
            Ok(after) => {
                r.source_unchanged = Some(after == r.before);
                r.after = Some(after);
                if r.source_unchanged == Some(false) {
                    r.error=Some("SOURCE REPOSITORY CHANGED DURING VERIFICATION. Before/after evidence retained; changes cannot be attributed to TRACE automatically.".into());
                    if r.status == "bisect_confirmed" {
                        r.status = "inconclusive".into();
                    }
                }
            }
            Err(err) => {
                r.source_unchanged = None;
                r.error = Some(format!("Cannot verify source repository state: {err}"));
                r.status = "inconclusive".into();
            }
        }
        if flag.load(Ordering::SeqCst) {
            r.status = "cancelled".into();
        }
        r.completed_at = Some(now());
        if let Err(err) = self.persist(&mut r) {
            eprintln!("Verification persistence failed: {err}");
        }
    }
    fn owned(&self, r: &VerificationRun) -> Result<PathBuf> {
        let uid = uuid::Uuid::parse_str(&r.id).map_err(|_| "Invalid verification identity")?;
        let dir = self
            .base
            .join(uid.to_string())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !dir.starts_with(&self.base) || dir == self.base {
            return Err("Worktree ownership path invalid".into());
        }
        let owner: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.join("owner.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if owner["id"] != r.id
            || owner["repository"] != r.repository
            || owner["caseId"] != r.case_id
        {
            return Err("Worktree ownership marker mismatch".into());
        }
        if r.worktree != repository::display_path(&dir.join("tree")) {
            return Err("Unexpected owned worktree path".into());
        }
        Ok(dir)
    }
    fn cleanup_inner(&self, r: &mut VerificationRun, unlock: bool) -> Result<()> {
        let owned = self.owned(r)?;
        let repo = PathBuf::from(&r.repository);
        for name in ["tree", "bisect"] {
            let p = owned.join(name);
            if !p.exists() {
                continue;
            }
            if p.is_symlink() {
                return Err("Refusing cleanup of substituted worktree symlink".into());
            }
            let actual = git::git(
                &p,
                &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            );
            if actual.is_err() && unlock && !p.join(".git").exists() {
                let canonical = p.canonicalize().map_err(|e| e.to_string())?;
                if !canonical.starts_with(&owned) {
                    return Err("Partial worktree path escaped ownership directory".into());
                }
                let registrations = self.successful_git(
                    r,
                    &repo,
                    &["worktree", "list", "--porcelain"],
                    Arc::new(AtomicBool::new(false)),
                )?;
                if registrations
                    .lines()
                    .filter_map(|l| l.strip_prefix("worktree "))
                    .any(|registered| {
                        Path::new(registered).canonicalize().ok() == Some(canonical.clone())
                    })
                {
                    return Err(
                        "Registered partial worktree requires Git metadata recovery before removal"
                            .into(),
                    );
                }
                // Explicit cleanup retry only: validated TRACE-owned, unregistered,
                // non-symlink partial creation. Preserve sibling output artifacts.
                std::fs::remove_dir_all(&canonical).map_err(|e| e.to_string())?;
                continue;
            }
            let actual = actual?;
            let expected = git::git(
                &repo,
                &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )?;
            if Path::new(actual.trim())
                .canonicalize()
                .map_err(|e| e.to_string())?
                != Path::new(expected.trim())
                    .canonicalize()
                    .map_err(|e| e.to_string())?
            {
                return Err("Worktree Git ownership mismatch".into());
            }
            let display = repository::display_path(&p);
            if unlock {
                let _ = self.git_op(
                    r,
                    &repo,
                    &["worktree", "unlock", &display],
                    Arc::new(AtomicBool::new(false)),
                );
            }
            self.successful_git(
                r,
                &repo,
                &["worktree", "remove", "--force", &display],
                Arc::new(AtomicBool::new(false)),
            )?;
        }
        r.cleanup_status = "cleaned".into();
        Ok(())
    }
    pub fn cleanup(&self, rid: &str) -> Result<VerificationRun> {
        let mut r = store::load_run(&self.db, rid)?;
        if let Err(err) = validate_run(&r) {
            r.status = "integrity_error".into();
            r.error = Some(err);
        }
        if self.busy(&r.case_id) {
            return Err("Cancel and wait for verification before cleanup".into());
        }
        match self.cleanup_inner(&mut r, true) {
            Ok(()) => {}
            Err(err) => {
                r.cleanup_status = "required".into();
                r.error = Some(format!("WORKTREE CLEANUP REQUIRED: {err}"));
            }
        }
        self.persist(&mut r)?;
        Ok(r)
    }
    pub fn discard(&self, rid: &str) -> Result<VerificationRun> {
        let mut r = store::load_run(&self.db, rid)?;
        if !["interrupted", "failed", "cancelled", "integrity_error"].contains(&r.status.as_str()) {
            return Err("Only interrupted or failed experiments can be discarded".into());
        }
        r = self.cleanup(rid)?;
        if r.cleanup_status != "cleaned" {
            return Err("Clean up the owned worktree before discarding the run".into());
        }
        r.status = "discarded".into();
        r.completed_at = Some(now());
        r.approval_token = None;
        self.persist(&mut r)?;
        Ok(r)
    }
}
fn resolve_revision(root: &Path, revision: &str) -> Result<String> {
    if revision != "HEAD" && !git::valid_sha(revision) {
        return Err("Use HEAD or a full commit SHA from the commit list".into());
    }
    let sha = git::git(
        root,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )?
    .trim()
    .to_string();
    if !git::valid_sha(&sha) {
        return Err("Invalid commit revision".into());
    }
    Ok(sha)
}
pub fn validate_run(r: &VerificationRun) -> Result<()> {
    if r.integrity_hash != run_digest(r) {
        return Err("Modified verification evidence detected (run hash mismatch)".into());
    }
    for e in r.executions.iter().chain(&r.git_operations) {
        if e.integrity_hash != execution::digest(e) {
            return Err("Modified test execution evidence detected".into());
        }
        for (p, h) in [
            (&e.stdout_artifact, &e.stdout_hash),
            (&e.stderr_artifact, &e.stderr_hash),
        ] {
            if std::fs::metadata(p)
                .map_err(|_| "Verification output artifact missing")?
                .len()
                > execution::ARTIFACT_LIMIT as u64
            {
                return Err("Verification output artifact exceeds retained limit".into());
            }
            let bytes = std::fs::read(p).map_err(|_| "Verification output artifact missing")?;
            if hash(&bytes) != *h {
                return Err("Verification output artifact modified".into());
            }
        }
    }
    for step in &r.steps {
        let e = r
            .executions
            .iter()
            .find(|e| e.id == step.test_execution_id)
            .ok_or("Missing actual test execution invalidates verification")?;
        if e.revision != step.commit_sha
            || execution::classification(e).unwrap_or("error") != step.classification
        {
            return Err("Bisect step does not match real execution".into());
        }
    }
    if r.status == "bisect_confirmed" {
        let first = r.first_bad.as_deref().ok_or("Missing first-bad commit")?;
        for (role, sha, class) in [
            (
                "known_good",
                r.known_good.as_deref().ok_or("Missing known good")?,
                "good",
            ),
            ("known_bad", r.known_bad.as_str(), "bad"),
            ("boundary_bad_confirmation", first, "bad"),
        ] {
            if !r
                .steps
                .iter()
                .any(|s| s.role == role && s.commit_sha == sha && s.classification == class)
            {
                return Err("Missing experimental boundary evidence".into());
            }
        }
        if !r
            .steps
            .iter()
            .any(|s| s.role == "boundary_parent_confirmation" && s.classification == "good")
        {
            return Err("Missing parent confirmation".into());
        }
    }
    Ok(())
}
pub fn attach(case: &mut Investigation, r: &VerificationRun) {
    // Replace only this native run's graph nodes; original evidence stays immutable.
    let removed: Vec<_> = case
        .nodes
        .iter()
        .filter(|n| n.data["verificationId"] == r.id)
        .map(|n| n.id.clone())
        .collect();
    case.nodes.retain(|n| !removed.contains(&n.id));
    case.edges
        .retain(|e| !removed.contains(&e.source) && !removed.contains(&e.target));
    let raw = serde_json::to_string(r).unwrap();
    let mut p = Provenance::input("verification_engine", &raw, &r.id);
    p.commit_sha = Some(r.known_bad.clone());
    let valid = validate_run(r).is_ok();
    let rid = format!("verification-{}", r.id);
    let root = case.node(
        "verification_run",
        Status::Derived,
        "Verification run",
        &format!(
            "{} · {} executions · selected test predicate",
            r.status,
            r.executions.len()
        ),
        "test_runner",
        p.clone(),
        json!({"verificationId":r.id,"run":r,"integrityValid":valid}),
    );
    case.nodes.last_mut().unwrap().id = rid.clone();
    if !valid {
        case.nodes.last_mut().unwrap().validity = "invalid".into();
    }
    if let Some(h) = &r.hypothesis_id {
        case.edge(h, &rid, "tested_by", Status::Derived, p.clone());
    }
    if let Some(commit) = case
        .nodes
        .iter()
        .find(|n| n.kind == "commit" && n.data["sha"] == r.suspect.clone().unwrap_or_default())
        .map(|n| n.id.clone())
    {
        case.edge(&commit, &rid, "tested_by", Status::Derived, p.clone());
    }
    let _ = root;
    for (i, e) in r.executions.iter().enumerate() {
        let mut ep = Provenance::input("test_runner", &serde_json::to_string(e).unwrap(), &e.id);
        ep.command = std::iter::once(e.command.executable.clone())
            .chain(e.command.args.clone())
            .collect();
        ep.commit_sha = Some(e.revision.clone());
        let node=case.node("test_execution",Status::Observed,&format!("{} · {}",e.status.to_uppercase(),&e.revision[..12.min(e.revision.len())]),"Executed by TRACE in an isolated worktree","test_runner",ep.clone(),json!({"verificationId":r.id,"execution":e,"role":r.steps.get(i).map(|s|s.role.as_str()),"integrityValid":valid,"independentlyExecuted":true}));
        case.nodes.last_mut().unwrap().id = e.id.clone();
        if !valid {
            case.nodes.last_mut().unwrap().validity = "invalid".into();
        }
        case.edge(&rid, &e.id, "records_execution", Status::Derived, ep);
        let _ = node;
    }
    if r.mode == "bisect" {
        let bid = case.node(
            "bisect_run",
            Status::Derived,
            "Bisect experiment",
            &format!("{} revisions evaluated", r.steps.len()),
            "verification_engine",
            p.clone(),
            json!({"verificationId":r.id,"run":r,"integrityValid":valid}),
        );
        case.edge(&rid, &bid, "narrows_range", Status::Derived, p.clone());
        if r.status == "bisect_confirmed" && valid {
            let boundary=case.node("regression_boundary",Status::Derived,"REGRESSION BOUNDARY VERIFIED",&format!("Selected test first fails at {} within the approved range",r.first_bad.as_deref().unwrap()),"verification_engine",p.clone(),json!({"verificationId":r.id,"run":r,"integrityValid":true,"scope":"selected test predicate","suspectMatches":r.first_bad==r.suspect}));
            case.edge(
                &bid,
                &boundary,
                "confirms_boundary",
                Status::Derived,
                p.clone(),
            );
            for e in &r.executions {
                case.edge(
                    &e.id,
                    &boundary,
                    "supports_boundary",
                    Status::Derived,
                    p.clone(),
                );
            }
        }
    }
}
pub fn audit_case(db: &Path, case: &mut Investigation) -> Result<()> {
    audit_runs(case, &store::runs(db, Some(&case.id))?)
}
pub fn audit_runs(case: &mut Investigation, runs: &[VerificationRun]) -> Result<()> {
    for r in runs {
        let mut error = validate_run(&r).err();
        for n in case
            .nodes
            .iter()
            .filter(|n| n.data["verificationId"] == r.id && n.kind != "test_execution")
        {
            if n.data["run"] != serde_json::to_value(&r).unwrap() {
                error = Some("Modified verification graph result detected".into());
            }
        }
        if r.status == "interrupted" || r.cleanup_status == "required" {
            let warning = format!(
                "{} · temporary worktree: {} · Open Verification Lab to clean up",
                r.status.to_uppercase(),
                r.worktree
            );
            if !case.warnings.contains(&warning) {
                case.warnings.push(warning);
            }
        }
        for e in &r.executions {
            match case.nodes.iter().find(|n| {
                n.id == e.id && n.created_by == "test_runner" && n.kind == "test_execution"
            }) {
                Some(n)
                    if n.data["execution"] == serde_json::to_value(e).unwrap()
                        && n.provenance.source_hash.as_deref()
                            == Some(hash(n.provenance.raw_excerpt.as_bytes()).as_str()) => {}
                _ => error = Some(
                    "Missing or modified graph execution evidence invalidates verification claim"
                        .into(),
                ),
            }
        }
        if let Some(error) = error {
            for n in &mut case.nodes {
                if n.data["verificationId"] == r.id {
                    n.validity = "invalid".into();
                    n.data["integrityValid"] = json!(false);
                }
            }
            let warning = format!("VERIFICATION EVIDENCE INVALID: {error}");
            if !case.warnings.contains(&warning) {
                case.warnings.push(warning);
            }
        }
    }
    Ok(())
}
