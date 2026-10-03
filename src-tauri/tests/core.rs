use serde_json::json;
use std::{path::Path, process::Command};
use tempfile::TempDir;
use trace_core::{
    ai, git, investigation,
    model::*,
    repository::{self, Sessions},
    stack::{NodeParser, StackTraceParser},
    store,
};
fn fixture() -> (TempDir, String, String) {
    let dir = tempfile::Builder::new()
        .prefix("trace fixture 空間 ")
        .tempdir()
        .unwrap();
    let root = dir.path();
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    run(&["init", "-b", "main"]);
    run(&["config", "user.name", "TRACE Test"]);
    run(&["config", "user.email", "test@trace.local"]);
    run(&["config", "core.autocrlf", "false"]);
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/parser.ts"),"export function parseUser(user: { id: string } | null) {\n  if (!user) return null;\n  return user!.id;\n}\n").unwrap();
    run(&["add", "."]);
    run(&["commit", "-m", "safe implementation"]);
    let good = run(&["rev-parse", "HEAD"]);
    // Change the actual crash line too, so blame correctly identifies the regression.
    std::fs::write(root.join("src/parser.ts"),"export function parseUser(user: { id: string } | null) {\n  // guard removed\n  return user!.id.trim();\n}\n").unwrap();
    run(&["add", "."]);
    run(&["commit", "-m", "remove guard"]);
    let bad = run(&["rev-parse", "HEAD"]);
    std::fs::write(root.join("README.md"), "unrelated docs").unwrap();
    run(&["add", "."]);
    run(&["commit", "-m", "documentation"]);
    (dir, good, bad)
}
fn analyze(root: &Path) -> Investigation {
    let s = Sessions::default();
    let r = s.open(&root.to_string_lossy()).unwrap();
    investigation::investigate(
        &s,
        &r.token,
        "TypeError: Cannot read properties of null\n    at parseUser (src/parser.ts:3:10)",
        "2026-10-04 ERROR missing user",
        "not ok 1 - null user",
    )
    .unwrap()
}
#[test]
fn parser_node_relative() {
    let f = NodeParser.parse("TypeError: null\n at parseUser (src/parser.ts:87:19)");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].line, 87);
    assert_eq!(f[0].column, Some(19));
    assert_eq!(f[0].function_name.as_deref(), Some("parseUser"));
}
#[test]
fn parser_windows_drive() {
    let f = NodeParser.parse(" at parseUser (C:\\work space\\src\\parser.ts:3:8)");
    assert_eq!(f[0].file_path, "C:\\work space\\src\\parser.ts");
    assert_eq!(f[0].line, 3);
}
#[test]
fn parser_anonymous_file_url() {
    let f = NodeParser.parse(" at file:///C:/work/src/index.ts:4:9");
    assert_eq!(f[0].line, 4);
    assert_eq!(f[0].file_path, "file:///C:/work/src/index.ts");
    assert_eq!(f[0].function_name, None);
}
#[test]
fn parser_ignores_nonframes() {
    assert!(NodeParser
        .parse("hello\n at node:internal/modules/foo\n nonsense")
        .is_empty());
}
#[test]
fn parser_line_without_column() {
    let f = NodeParser.parse(" at work (src/main.ts:9)");
    assert_eq!(f[0].line, 9);
    assert_eq!(f[0].column, None);
}
#[test]
fn git_commit_parser() {
    let s = format!(
        "{}\x1fTest Author\x1f2026-10-04T01:00:00Z\x1fmessage\0\n",
        "a".repeat(40)
    );
    let c = git::parse_commits(&s);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].subject, "message");
}
#[test]
fn git_blame_parser() {
    let s = format!(
        "{} 3 3 1\nauthor Jane\nauthor-time 1777777\n\treturn user.id;\n",
        "b".repeat(40)
    );
    let b = git::parse_blame(&s).unwrap();
    assert_eq!(b.author, "Jane");
    assert_eq!(b.source, "return user.id;");
    assert!(!b.uncommitted);
}
#[test]
fn git_uncommitted_blame() {
    let b = git::parse_blame(&format!(
        "{} 1 1\nauthor Not Committed Yet\n\tchanged",
        "0".repeat(40)
    ))
    .unwrap();
    assert!(b.uncommitted);
}
#[test]
fn git_rejects_mutating_command() {
    let d = tempfile::tempdir().unwrap();
    assert!(git::run_git(d.path(), &["reset".into(), "--hard".into()])
        .unwrap_err()
        .contains("allowlist"));
}
#[test]
fn git_sha_validation() {
    assert!(git::valid_sha(&"a".repeat(40)));
    assert!(!git::valid_sha("HEAD;whoami"));
    assert!(!git::valid_sha(&"0".repeat(40)));
}
#[test]
fn repository_rejects_non_git() {
    let d = tempfile::tempdir().unwrap();
    assert!(Sessions::default()
        .open(&d.path().to_string_lossy())
        .is_err());
}
#[test]
fn traversal_rejected() {
    let d = tempfile::tempdir().unwrap();
    assert!(repository::resolve(d.path(), "../outside.ts")
        .unwrap_err()
        .contains("traversal"));
}
#[test]
fn absolute_escape_rejected() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    std::fs::write(b.path().join("secret.ts"), "secret").unwrap();
    assert!(repository::resolve(
        &a.path().canonicalize().unwrap(),
        &b.path().join("secret.ts").to_string_lossy()
    )
    .unwrap_err()
    .contains("escapes"));
}
#[test]
fn sensitive_and_generated_excluded() {
    for p in [
        ".env",
        ".env.production",
        "foo/a.pem",
        "secrets.json",
        "node_modules/a.ts",
        "coverage/x.ts",
        "id_rsa",
        "a\\credentials.txt",
    ] {
        assert!(repository::excluded(p), "{p}");
    }
    assert!(!repository::excluded("src/parser.ts"));
}
#[test]
fn source_bounds_and_binary() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    std::fs::write(root.join("a.ts"), "one\ntwo\n").unwrap();
    assert!(repository::source(&root, "a.ts", 3, None).is_err());
    assert!(repository::source(&root, "a.ts", 0, None).is_err());
    std::fs::write(root.join("a.ts"), [0, 1, 2]).unwrap();
    assert!(repository::source(&root, "a.ts", 1, None)
        .unwrap_err()
        .contains("Binary"));
}
#[test]
fn source_reference_exact_line() {
    let (d, _, _) = fixture();
    let root = d.path().canonicalize().unwrap();
    let s = repository::source(&root, "src/parser.ts", 3, Some(10)).unwrap();
    assert_eq!(s.lines[2], "  return user!.id.trim();");
    assert_eq!(s.line, 3);
    assert_eq!(s.path, "src/parser.ts");
}
#[test]
fn real_git_blame_diff_history() {
    let (d, good, bad) = fixture();
    let raw = git::git(
        d.path(),
        &[
            "blame",
            "--line-porcelain",
            "-L",
            "3,3",
            "--",
            "src/parser.ts",
        ],
    )
    .unwrap();
    assert_eq!(git::parse_blame(&raw).unwrap().sha, bad);
    let patch = git::git(
        d.path(),
        &[
            "show",
            "--no-ext-diff",
            "--no-textconv",
            &bad,
            "--",
            "src/parser.ts",
        ],
    )
    .unwrap();
    assert!(patch.contains("-  if (!user) return null;"));
    assert!(patch.contains("+  return user!.id.trim();"));
    let history = git::commits(d.path(), Some("src/parser.ts")).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].sha, bad);
    assert_eq!(history[1].sha, good);
}
#[test]
fn graph_integrity_and_read_only() {
    let (d, _, bad) = fixture();
    let before = git::git(d.path(), &["status", "--porcelain=v1", "-z"]).unwrap();
    let c = analyze(d.path());
    assert!(c.frames[0].resolved);
    assert!(c
        .nodes
        .iter()
        .any(|n| n.kind == "commit" && n.data["sha"] == bad));
    for e in &c.edges {
        assert!(c.nodes.iter().any(|n| n.id == e.source));
        assert!(c.nodes.iter().any(|n| n.id == e.target));
        assert_ne!(e.relation, "causes");
        assert!(!e.provenance.raw_excerpt.is_empty());
    }
    assert!(c.nodes.iter().all(|n| !n.provenance.raw_excerpt.is_empty()));
    assert_eq!(
        git::git(d.path(), &["status", "--porcelain=v1", "-z"]).unwrap(),
        before
    );
}
#[test]
fn dirty_worktree_is_inspectable() {
    let (d, _, _) = fixture();
    std::fs::write(
        d.path().join("src/parser.ts"),
        "function parseUser(user:any) {\n // dirty\n return user.id.toString();\n}\n",
    )
    .unwrap();
    let c = analyze(d.path());
    assert!(c.repository.dirty);
    assert!(c
        .nodes
        .iter()
        .any(|n| n.kind == "blame_result" && n.data["uncommitted"] == true));
    assert!(!c.nodes.iter().any(|n| n.kind == "commit"));
    assert!(c.nodes.iter().any(|n| n.kind == "working_diff"));
}
#[test]
fn empty_repository_source_without_history() {
    let d = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(d.path())
        .status()
        .unwrap()
        .success());
    std::fs::create_dir(d.path().join("src")).unwrap();
    std::fs::write(d.path().join("src/parser.ts"), "one\ntwo\nthree\n").unwrap();
    let c = analyze(d.path());
    assert!(c.repository.head.is_none());
    assert!(c.nodes.iter().any(|n| n.kind == "source_line"));
}
#[test]
fn case_sqlite_roundtrip() {
    let (d, _, _) = fixture();
    let c = analyze(d.path());
    let storage = tempfile::tempdir().unwrap();
    let db = storage.path().join("cases.sqlite");
    store::save(&db, &c).unwrap();
    let loaded = store::load(&db, &c.id).unwrap();
    assert_eq!(loaded.id, c.id);
    assert_eq!(loaded.nodes.len(), c.nodes.len());
    assert_eq!(store::list(&db).unwrap().as_array().unwrap().len(), 1);
}
#[test]
fn source_changes_mark_stale() {
    let (d, _, _) = fixture();
    let mut c = analyze(d.path());
    std::fs::write(d.path().join("src/parser.ts"), "changed").unwrap();
    investigation::check_stale(&mut c, &Sessions::default()).unwrap();
    assert!(c.stale);
    assert!(c
        .nodes
        .iter()
        .any(|n| n.kind == "source_line" && n.validity == "stale"));
}
#[test]
fn missing_source_marks_invalid() {
    let (d, _, _) = fixture();
    let mut c = analyze(d.path());
    std::fs::remove_file(d.path().join("src/parser.ts")).unwrap();
    investigation::check_stale(&mut c, &Sessions::default()).unwrap();
    assert!(c.stale);
    assert!(c
        .nodes
        .iter()
        .any(|n| n.kind == "source_line" && n.validity == "invalid"));
}
fn response(c: &Investigation) -> serde_json::Value {
    json!({"hypothesis":"Guard removal could expose a null value","supportedBy":[c.nodes[0].id],"contradictedBy":[],"uncertainties":["No causal test executed"],"verificationSteps":["Reproduce in isolated fixture"]})
}
#[test]
fn ai_unknown_evidence_rejected() {
    let (d, _, _) = fixture();
    let c = analyze(d.path());
    let mut h = response(&c);
    h["supportedBy"] = json!(["invented-id"]);
    assert!(ai::validate(&h.to_string(), &c).is_err());
}
#[test]
fn ai_verified_evidence_rejected() {
    let (d, _, _) = fixture();
    let c = analyze(d.path());
    let mut h = response(&c);
    h["epistemicStatus"] = json!("verified");
    assert!(ai::validate(&h.to_string(), &c).is_err());
    h.as_object_mut().unwrap().remove("epistemicStatus");
    h["nodes"] = json!([]);
    assert!(ai::validate(&h.to_string(), &c).is_err());
}
#[test]
fn ai_uncertainty_required() {
    let (d, _, _) = fixture();
    let c = analyze(d.path());
    let mut h = response(&c);
    h["uncertainties"] = json!([]);
    assert!(ai::validate(&h.to_string(), &c).is_err());
}
#[test]
fn confidence_blame_alone_low() {
    let (d, _, _) = fixture();
    let mut c = analyze(d.path());
    let mut h = response(&c);
    h["supportedBy"] = json!([c
        .nodes
        .iter()
        .find(|n| n.kind == "blame_result")
        .unwrap()
        .id]);
    let validated = ai::validate(&h.to_string(), &c).unwrap();
    ai::attach(&mut c, validated, "test-model");
    let n = c.nodes.last().unwrap();
    assert_eq!(n.epistemic_status, Status::Hypothesis);
    assert_eq!(n.data["confidence"], "LOW");
    assert_eq!(n.created_by, "ai");
    assert!(c.edges.iter().all(|e| e.relation != "causes"));
}
#[test]
fn secrets_redacted() {
    let redacted=ai::redact("api_key=abcdef-secret\nAuthorization: Bearer abcdef1234\n-----BEGIN PRIVATE KEY-----\nprivate data\n-----END PRIVATE KEY-----");
    assert!(!redacted.contains("abcdef-secret"));
    assert!(!redacted.contains("private data"));
    assert!(!redacted.contains("abcdef1234"));
}
#[test]
fn stale_evidence_excluded_from_ai() {
    let (d, _, _) = fixture();
    let mut c = analyze(d.path());
    let id = c.nodes[0].id.clone();
    c.nodes[0].validity = "stale".into();
    let context = ai::context(&c);
    assert!(!context["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["id"] == id));
    let h = json!({"hypothesis":"possible","supportedBy":[id],"contradictedBy":[],"uncertainties":["missing"],"verificationSteps":["test"]});
    assert!(ai::validate(&h.to_string(), &c).is_err());
}
