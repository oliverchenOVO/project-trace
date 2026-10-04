use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tempfile::TempDir;
#[cfg(windows)]
#[test]
fn bisect_launch_error_retains_attempt_without_bad_classification() {
    let f = Fixture::new(3, 1);
    let exe = f._temp.path().join("not-a-program.exe");
    std::fs::write(&exe, b"not a Windows PE executable").unwrap();
    let mut cmd = fixture_command();
    cmd.executable = exe.to_str().unwrap().into();
    let r = f.run(f.request("bisect", cmd, 0));
    assert_eq!(r.status, "inconclusive");
    assert_eq!(r.executions[0].status, "launch_error");
    assert_eq!(r.steps[0].classification, "error");
    assert!(r.first_bad.is_none());
    assert_eq!(r.cleanup_status, "cleaned");
    let case = store::load(&f.db, &f.case_id).unwrap();
    assert!(case
        .nodes
        .iter()
        .any(|n| n.kind == "test_execution" && n.data["independentlyExecuted"] == false));
}
#[test]
fn real_started_run_recovers_as_interrupted_with_cleanup() {
    let f = Fixture::new(3, 1);
    let mut r = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    git(
        &f.repo,
        &["worktree", "add", "--detach", &r.worktree, &r.known_bad],
    );
    r.status = "running".into();
    r.started_at = Some(trace_core::model::now());
    r.integrity_hash = verification::run_digest(&r);
    store::save_run(&f.db, &r).unwrap();
    let recovered = Service::new(f.db.clone(), f.base.clone()).unwrap();
    let r = recovered.list(&f.case_id).unwrap().pop().unwrap();
    assert_eq!(r.status, "interrupted");
    assert!(!recovered.busy(&f.case_id));
    assert_eq!(recovered.discard(&r.id).unwrap().status, "discarded");
    assert!(!Path::new(&r.worktree).exists());
}
#[test]
fn explicit_cleanup_removes_owned_unregistered_partial_creation() {
    let f = Fixture::new(3, 1);
    let r = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    std::fs::create_dir(&r.worktree).unwrap();
    std::fs::write(Path::new(&r.worktree).join("partial.txt"), "owned partial").unwrap();
    let clean = f.service.cleanup(&r.id).unwrap();
    assert_eq!(clean.cleanup_status, "cleaned", "{:?}", clean.error);
    assert!(!Path::new(&r.worktree).exists());
    assert!(f.repo.exists());
}
#[test]
fn modified_execution_provenance_is_detected() {
    let f = Fixture::new(3, 1);
    f.run(f.request("test_bad", fixture_command(), 0));
    let mut c = store::load(&f.db, &f.case_id).unwrap();
    let e = c
        .nodes
        .iter_mut()
        .find(|n| n.kind == "test_execution")
        .unwrap();
    e.provenance.command = vec!["fake command".into()];
    verification::audit_case(&f.db, &mut c).unwrap();
    assert!(c
        .nodes
        .iter()
        .filter(|n| n.kind == "test_execution")
        .all(|n| n.validity == "invalid"));
}
use trace_core::{
    ai,
    execution::{self, TestCommand},
    investigation,
    repository::Sessions,
    store,
    verification::{self, Request, Service},
};
fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn command(script: &str, timeout: u64) -> TestCommand {
    TestCommand {
        executable: "node".into(),
        args: vec!["-e".into(), script.into()],
        working_directory: ".".into(),
        timeout_ms: timeout,
    }
}
struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    db: PathBuf,
    base: PathBuf,
    case_id: String,
    shas: Vec<String>,
    service: Arc<Service>,
}
impl Fixture {
    fn new(count: usize, bad_at: usize) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("來源 repository 中文 spaces");
        std::fs::create_dir(&repo).unwrap();
        git(&repo, &["init", "-b", "main"]);
        git(&repo, &["config", "user.name", "TRACE Test"]);
        git(&repo, &["config", "user.email", "test@trace.local"]);
        git(&repo, &["config", "core.autocrlf", "false"]);
        let mut shas = vec![];
        for i in 0..count {
            std::fs::write(
                repo.join("value.txt"),
                if i >= bad_at { "bad" } else { "good" },
            )
            .unwrap();
            std::fs::write(repo.join("history.txt"), i.to_string()).unwrap();
            std::fs::write(
                repo.join("package.json"),
                r#"{"scripts":{"test":"node test.cjs"}}"#,
            )
            .unwrap();
            std::fs::write(
                repo.join("test.cjs"),
                "process.exit(require('fs').readFileSync('value.txt','utf8')==='good'?0:1)",
            )
            .unwrap();
            git(&repo, &["add", "."]);
            git(&repo, &["commit", "-m", &format!("revision {i}")]);
            shas.push(git(&repo, &["rev-parse", "HEAD"]));
        }
        std::fs::write(repo.join("untracked note.txt"), "keep me").unwrap();
        let sessions = Sessions::default();
        let opened = sessions.open(repo.to_str().unwrap()).unwrap();
        let case = investigation::investigate(
            &sessions,
            &opened.token,
            "TypeError: fixture\n at parse (test.cjs:1:1)",
            "",
            "fake pasted PASS",
        )
        .unwrap();
        let db = temp.path().join("cases.sqlite");
        store::save(&db, &case).unwrap();
        let base = temp.path().join("TRACE 暫存 worktrees");
        let service = Service::new(db.clone(), base.clone()).unwrap();
        Self {
            _temp: temp,
            repo,
            db,
            base,
            case_id: case.id,
            shas,
            service,
        }
    }
    fn request(&self, mode: &str, cmd: TestCommand, good: usize) -> Request {
        Request {
            case_id: self.case_id.clone(),
            command: cmd,
            known_good: Some(self.shas[good].clone()),
            known_bad: self.shas.last().unwrap().clone(),
            mode: mode.into(),
            hypothesis_id: None,
        }
    }
    fn run(&self, request: Request) -> verification::VerificationRun {
        let prepared = self.service.prepare(request).unwrap();
        self.service
            .approve(&prepared.id, prepared.approval_token.as_deref().unwrap())
            .unwrap();
        self.wait(&prepared.id)
    }
    fn wait(&self, id: &str) -> verification::VerificationRun {
        let deadline = Instant::now() + Duration::from_secs(180);
        loop {
            let r = store::load_run(&self.db, id).unwrap();
            if r.completed_at.is_some() {
                return r;
            }
            assert!(
                Instant::now() < deadline,
                "run did not finish: {:?}",
                r.error
            );
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}
fn fixture_command() -> TestCommand {
    TestCommand {
        executable: "node".into(),
        args: vec!["test.cjs".into()],
        working_directory: ".".into(),
        timeout_ms: 10000,
    }
}
#[test]
fn structured_command_rejects_shell() {
    let t = tempfile::tempdir().unwrap();
    let mut c = command("", 1000);
    c.executable = "cmd.exe".into();
    assert!(execution::resolve_command(&c, t.path())
        .unwrap_err()
        .contains("Shell"));
}
#[test]
fn command_directory_traversal_rejected() {
    let t = tempfile::tempdir().unwrap();
    let mut c = command("", 1000);
    c.working_directory = "..\\outside".into();
    assert!(execution::resolve_command(&c, t.path()).is_err());
    c.working_directory = "C:\\outside".into();
    assert!(execution::resolve_command(&c, t.path()).is_err());
}
#[test]
fn invalid_timeout_and_nul_rejected() {
    let t = tempfile::tempdir().unwrap();
    assert!(execution::resolve_command(&command("", 0), t.path()).is_err());
    assert!(execution::resolve_command(&command("a\0b", 1000), t.path()).is_err());
}
#[test]
fn executable_missing_is_actionable() {
    let t = tempfile::tempdir().unwrap();
    let mut c = command("", 1000);
    c.executable = "trace-no-such-executable".into();
    assert!(execution::resolve_command(&c, t.path())
        .unwrap_err()
        .contains("unavailable"));
}
#[test]
fn missing_cwd_is_launch_error() {
    let t = tempfile::tempdir().unwrap();
    let mut c = command("process.exit(0)", 1000);
    c.working_directory = "absent".into();
    let rc = execution::resolve_command(&c, t.path()).unwrap();
    let e = execution::execute(
        &rc,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(e.status, "launch_error");
    assert!(execution::classification(&e).is_none());
}
#[test]
fn actual_process_captures_stdout_stderr_exit() {
    let t = tempfile::tempdir().unwrap();
    let c = execution::resolve_command(
        &command(
            "console.log('actual stdout');console.error('actual stderr');process.exit(7)",
            10000,
        ),
        t.path(),
    )
    .unwrap();
    let e = execution::execute(
        &c,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(e.exit_code, Some(7));
    assert_eq!(e.status, "failed");
    assert!(e.stdout.contains("actual stdout"));
    assert!(e.stderr.contains("actual stderr"));
    assert_eq!(e.integrity_hash, execution::digest(&e));
}
#[test]
fn timeout_not_bad() {
    let t = tempfile::tempdir().unwrap();
    let c = execution::resolve_command(
        &command("console.log('started');setInterval(()=>{},1000)", 300),
        t.path(),
    )
    .unwrap();
    let e = execution::execute(
        &c,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(e.status, "timeout");
    assert!(e.timed_out);
    assert!(execution::classification(&e).is_none());
}
#[test]
fn cancellation_not_failure() {
    let t = tempfile::tempdir().unwrap();
    let flag = Arc::new(AtomicBool::new(false));
    let cancel = flag.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        cancel.store(true, Ordering::SeqCst);
    });
    let c =
        execution::resolve_command(&command("setInterval(()=>{},1000)", 10000), t.path()).unwrap();
    let e =
        execution::execute(&c, t.path(), "revision", &t.path().join("artifacts"), flag).unwrap();
    assert_eq!(e.status, "cancelled");
    assert!(execution::classification(&e).is_none());
}
#[test]
fn descendant_process_is_terminated_on_timeout() {
    let t = tempfile::tempdir().unwrap();
    let marker = t.path().join("child ticks.txt");
    let path = serde_json::to_string(marker.to_str().unwrap()).unwrap();
    let child_code = format!("setInterval(()=>require('fs').appendFileSync({path},'tick'),20)");
    let script=format!("require('child_process').spawn(process.execPath,['-e',{}],{{stdio:'ignore'}});setInterval(()=>{{}},1000)",serde_json::to_string(&child_code).unwrap());
    let c = execution::resolve_command(&command(&script, 5000), t.path()).unwrap();
    let e = execution::execute(
        &c,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert!(e.timed_out);
    let before = std::fs::metadata(&marker).unwrap().len();
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(std::fs::metadata(marker).unwrap().len(), before);
}
#[test]
fn bounded_large_output_saved_without_memory_growth() {
    let t = tempfile::tempdir().unwrap();
    let c = execution::resolve_command(
        &command("process.stdout.write('x'.repeat(100000));", 10000),
        t.path(),
    )
    .unwrap();
    let e = execution::execute(
        &c,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(e.status, "passed");
    assert_eq!(e.stdout.len(), 32000);
    assert_eq!(std::fs::metadata(e.stdout_artifact).unwrap().len(), 100000);
    assert!(e.output_truncated);
}
#[test]
fn runaway_output_is_inconclusive() {
    let t = tempfile::tempdir().unwrap();
    let c = execution::resolve_command(
        &command("process.stdout.write('x'.repeat(20*1024*1024));", 10000),
        t.path(),
    )
    .unwrap();
    let e = execution::execute(
        &c,
        t.path(),
        "revision",
        &t.path().join("artifacts"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(e.status, "output_limit");
    assert!(execution::classification(&e).is_none());
    assert!(
        std::fs::metadata(e.stdout_artifact).unwrap().len() <= execution::ARTIFACT_LIMIT as u64
    );
}
#[test]
fn explicit_approval_required_and_token_single_use() {
    let f = Fixture::new(3, 1);
    let p = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    assert!(!Path::new(&p.worktree).exists());
    assert!(f.service.approve(&p.id, "fake approval").is_err());
    f.service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap();
    assert!(f
        .service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .is_err());
    assert_eq!(f.wait(&p.id).status, "bad_confirmed");
}
#[test]
fn real_bisect_five_revisions_finds_c_preserves_dirty_checkout() {
    let f = Fixture::new(5, 2);
    let before = verification::source_state(&f.repo.canonicalize().unwrap()).unwrap();
    let r = f.run(f.request("bisect", fixture_command(), 0));
    assert_eq!(r.status, "bisect_confirmed", "{:?}", r.error);
    assert_eq!(r.first_bad, Some(f.shas[2].clone()));
    assert_eq!(r.source_unchanged, Some(true));
    assert_eq!(r.cleanup_status, "cleaned");
    assert!(!Path::new(&r.worktree).exists());
    assert_eq!(
        before,
        verification::source_state(&f.repo.canonicalize().unwrap()).unwrap()
    );
    assert_eq!(
        git(&f.repo, &["worktree", "list", "--porcelain"])
            .matches("worktree ")
            .count(),
        1
    );
    verification::validate_run(&r).unwrap();
    assert!(r.steps.len() >= 5);
    assert!(r
        .steps
        .iter()
        .all(|s| r.executions.iter().any(|e| e.id == s.test_execution_id)));
}
#[test]
fn real_bisect_seven_revisions_non_power_of_two() {
    let f = Fixture::new(7, 4);
    let r = f.run(f.request("bisect", fixture_command(), 0));
    assert_eq!(r.status, "bisect_confirmed", "{:?}", r.error);
    assert_eq!(r.first_bad, Some(f.shas[4].clone()));
}
#[test]
fn adjacent_boundary_supported() {
    let f = Fixture::new(2, 1);
    let r = f.run(f.request("bisect", fixture_command(), 0));
    assert_eq!(r.status, "bisect_confirmed", "{:?}", r.error);
    assert_eq!(r.first_bad, Some(f.shas[1].clone()));
}
#[test]
fn invalid_good_aborts_without_boundary() {
    let f = Fixture::new(4, 1);
    let r = f.run(f.request("bisect", fixture_command(), 1));
    assert_eq!(r.status, "failed");
    assert!(r.error.unwrap().contains("INVALID GOOD BOUNDARY"));
    assert!(r.first_bad.is_none());
    assert_eq!(r.cleanup_status, "cleaned");
}
#[test]
fn invalid_bad_aborts_without_boundary() {
    let f = Fixture::new(3, 2);
    let r = f.run(f.request("bisect", command("process.exit(0)", 10000), 0));
    assert_eq!(r.status, "failed");
    assert!(r.error.unwrap().contains("INVALID BAD BOUNDARY"));
    assert!(r.first_bad.is_none());
}
#[test]
fn equal_or_unrelated_boundary_rejected() {
    let f = Fixture::new(3, 1);
    let mut request = f.request("bisect", fixture_command(), 0);
    request.known_good = Some(request.known_bad.clone());
    assert!(f.service.prepare(request).is_err());
    let mut request = f.request("bisect", fixture_command(), 0);
    request.known_good = Some("f".repeat(40));
    assert!(f.service.prepare(request).is_err());
}
#[test]
fn source_change_before_approval_blocks_execution() {
    let f = Fixture::new(3, 1);
    let p = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    std::fs::write(f.repo.join("untracked note.txt"), "external change").unwrap();
    assert!(f
        .service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap_err()
        .contains("CHANGED BEFORE APPROVAL"));
}
#[test]
fn timeout_worktree_cleanup_and_no_bad_classification() {
    let f = Fixture::new(3, 1);
    let r = f.run(f.request("bisect", command("setInterval(()=>{},1000)", 300), 0));
    assert_eq!(r.status, "inconclusive");
    assert_eq!(r.steps[0].classification, "error");
    assert_eq!(r.cleanup_status, "cleaned");
    assert!(r.first_bad.is_none());
    assert_eq!(r.source_unchanged, Some(true));
}
#[test]
fn cancel_running_experiment_cleanup() {
    let f = Fixture::new(3, 1);
    let p = f
        .service
        .prepare(f.request("bisect", command("setInterval(()=>{},1000)", 10000), 0))
        .unwrap();
    f.service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap();
    std::thread::sleep(Duration::from_millis(500));
    f.service.cancel(&p.id).unwrap();
    let r = f.wait(&p.id);
    assert_eq!(r.status, "cancelled");
    assert_eq!(r.cleanup_status, "cleaned");
    assert!(r.first_bad.is_none());
}
#[test]
fn missing_evidence_and_tamper_invalidate_boundary() {
    let f = Fixture::new(3, 1);
    let r = f.run(f.request("bisect", fixture_command(), 0));
    assert_eq!(r.status, "bisect_confirmed", "{:?}", r.error);
    let mut tampered = r.clone();
    tampered.executions[0].exit_code = Some(0);
    assert!(verification::validate_run(&tampered).is_err());
    let mut missing = r.clone();
    missing.executions.remove(0);
    missing.integrity_hash = verification::run_digest(&missing);
    assert!(verification::validate_run(&missing)
        .unwrap_err()
        .contains("Missing actual"));
    std::fs::write(&r.executions[0].stdout_artifact, "fake").unwrap();
    assert!(verification::validate_run(&r)
        .unwrap_err()
        .contains("modified"));
}
#[test]
fn saved_case_retains_verification_but_missing_graph_proof_invalidates() {
    let f = Fixture::new(3, 1);
    let r = f.run(f.request("bisect", fixture_command(), 0));
    let mut case = store::load(&f.db, &f.case_id).unwrap();
    assert!(case.nodes.iter().any(|n| n.kind == "regression_boundary"));
    assert!(case
        .nodes
        .iter()
        .any(|n| n.kind == "test_result" && n.data["independentlyExecuted"] == false));
    assert!(case
        .nodes
        .iter()
        .any(|n| n.kind == "test_execution" && n.data["independentlyExecuted"] == true));
    assert!(case.edges.iter().all(|e| e.relation != "causes"));
    case.nodes.retain(|n| n.id != r.executions[0].id);
    verification::audit_case(&f.db, &mut case).unwrap();
    assert!(case
        .nodes
        .iter()
        .filter(|n| n.kind == "regression_boundary")
        .all(|n| n.validity == "invalid"));
}
#[test]
fn interrupted_run_recovery_does_not_execute() {
    let f = Fixture::new(3, 1);
    let mut r = f
        .service
        .prepare(f.request("bisect", fixture_command(), 0))
        .unwrap();
    r.status = "running".into();
    r.approval_token = None;
    r.integrity_hash = verification::run_digest(&r);
    store::save_run(&f.db, &r).unwrap();
    let recovered = Service::new(f.db.clone(), f.base.clone()).unwrap();
    let r = recovered.list(&f.case_id).unwrap().pop().unwrap();
    assert_eq!(r.status, "interrupted");
    assert!(r.executions.is_empty());
    assert!(!Path::new(&r.worktree).exists());
    assert_eq!(recovered.cleanup(&r.id).unwrap().cleanup_status, "cleaned");
}
#[test]
fn ownership_mismatch_cannot_delete_other_paths() {
    let f = Fixture::new(3, 1);
    let mut r = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    r.worktree = f.repo.to_str().unwrap().into();
    r.integrity_hash = verification::run_digest(&r);
    store::save_run(&f.db, &r).unwrap();
    let r = f.service.cleanup(&r.id).unwrap();
    assert_eq!(r.cleanup_status, "required");
    assert!(f.repo.join("value.txt").exists());
}
#[test]
fn ai_cannot_create_executed_test_or_bisect_proof() {
    let f = Fixture::new(3, 1);
    let case = store::load(&f.db, &f.case_id).unwrap();
    for kind in ["test_execution", "regression_boundary", "bisect_run"] {
        let raw=serde_json::json!({"hypothesis":"fake","supportedBy":[case.nodes[0].id],"contradictedBy":[],"uncertainties":["none"],"verificationSteps":["none"],"kind":kind}).to_string();
        assert!(ai::validate(&raw, &case).is_err());
    }
}
#[cfg(windows)]
#[test]
fn npm_cmd_resolves_without_shell_and_runs_real_package_test() {
    let f = Fixture::new(3, 1);
    let c = TestCommand {
        executable: "npm.cmd".into(),
        args: vec!["test".into()],
        working_directory: ".".into(),
        timeout_ms: 20000,
    };
    let p = f.service.prepare(f.request("test_bad", c, 0)).unwrap();
    assert!(p.command.executable.to_lowercase().ends_with("node.exe"));
    assert!(p.command.args[0].ends_with("npm-cli.js"));
    f.service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap();
    let r = f.wait(&p.id);
    assert_eq!(r.status, "bad_confirmed", "{:?}", r.error);
    assert!(r.executions[0].stdout.contains("node test.cjs"));
}
#[test]
fn locked_worktree_failure_is_visible_and_owned_cleanup_retry_unlocks() {
    let f = Fixture::new(3, 1);
    let p = f
        .service
        .prepare(f.request("test_bad", command("setInterval(()=>{},1000)", 10000), 0))
        .unwrap();
    f.service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !Path::new(&p.worktree).join("test.cjs").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    git(
        &f.repo,
        &[
            "worktree",
            "lock",
            "--reason",
            "TRACE test locked cleanup",
            &p.worktree,
        ],
    );
    f.service.cancel(&p.id).unwrap();
    let r = f.wait(&p.id);
    assert_eq!(r.cleanup_status, "required");
    assert!(r.error.unwrap().contains("CLEANUP REQUIRED"));
    assert!(Path::new(&r.worktree).exists());
    let retried = f.service.cleanup(&r.id).unwrap();
    assert_eq!(retried.cleanup_status, "cleaned", "{:?}", retried.error);
    assert!(!Path::new(&retried.worktree).exists());
}
#[test]
fn actual_orphan_recovery_removes_only_owned_worktree() {
    let f = Fixture::new(3, 1);
    let mut r = f
        .service
        .prepare(f.request("test_bad", fixture_command(), 0))
        .unwrap();
    git(
        &f.repo,
        &["worktree", "add", "--detach", &r.worktree, &r.known_bad],
    );
    r.status = "running".into();
    r.integrity_hash = verification::run_digest(&r);
    store::save_run(&f.db, &r).unwrap();
    let recovered = Service::new(f.db.clone(), f.base.clone()).unwrap();
    let r = recovered.cleanup(&r.id).unwrap();
    assert_eq!(r.status, "interrupted");
    assert_eq!(r.cleanup_status, "cleaned");
    assert_eq!(
        git(&f.repo, &["worktree", "list", "--porcelain"])
            .matches("worktree ")
            .count(),
        1
    );
}
#[test]
fn external_source_change_during_execution_is_reported() {
    let f = Fixture::new(3, 1);
    let p = f
        .service
        .prepare(f.request(
            "test_bad",
            command("setTimeout(()=>process.exit(1),1000)", 10000),
            0,
        ))
        .unwrap();
    f.service
        .approve(&p.id, p.approval_token.as_deref().unwrap())
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    std::fs::write(f.repo.join("untracked note.txt"), "external modification").unwrap();
    let r = f.wait(&p.id);
    assert_eq!(r.source_unchanged, Some(false));
    assert!(r
        .error
        .unwrap()
        .contains("SOURCE REPOSITORY CHANGED DURING"));
    assert_ne!(r.before.files_hash, r.after.unwrap().files_hash);
}
#[test]
fn checkout_filters_cannot_execute_repository_configuration() {
    let f = Fixture::new(3, 1);
    let marker = f._temp.path().join("forbidden-filter.txt");
    let js = format!(
        "require('fs').writeFileSync({},'executed');process.stdin.pipe(process.stdout)",
        serde_json::to_string(marker.to_str().unwrap()).unwrap()
    );
    let cmd = format!("node -e {}", serde_json::to_string(&js).unwrap());
    git(&f.repo, &["config", "filter.evil.smudge", &cmd]);
    git(&f.repo, &["config", "filter.evil.required", "true"]);
    std::fs::write(
        f.repo.join(".git/info/attributes"),
        "value.txt filter=evil\n",
    )
    .unwrap();
    let r = f.run(f.request("test_bad", fixture_command(), 0));
    assert_eq!(r.status, "bad_confirmed", "{:?}", r.error);
    assert!(!marker.exists());
    assert!(r
        .git_operations
        .iter()
        .any(|e| e.command.args.contains(&"filter.evil.smudge=".to_string())));
}
#[test]
fn modified_boundary_graph_data_is_invalidated() {
    let f = Fixture::new(3, 1);
    let r = f.run(f.request("bisect", fixture_command(), 0));
    assert_eq!(r.status, "bisect_confirmed", "{:?}", r.error);
    let mut c = store::load(&f.db, &f.case_id).unwrap();
    let node = c
        .nodes
        .iter_mut()
        .find(|n| n.kind == "regression_boundary")
        .unwrap();
    node.data["run"]["firstBad"] = serde_json::json!("f".repeat(40));
    verification::audit_case(&f.db, &mut c).unwrap();
    assert_eq!(
        c.nodes
            .iter()
            .find(|n| n.kind == "regression_boundary")
            .unwrap()
            .validity,
        "invalid"
    );
}

#[test]
fn delayed_ai_merge_preserves_concurrent_verification_evidence() {
    let f = Fixture::new(3, 1);
    let mut outdated = store::load(&f.db, &f.case_id).unwrap();
    let source = outdated.nodes[0].id.clone();
    let hypothesis = ai::attach(
        &mut outdated,
        ai::Hypothesis {
            hypothesis: "A pending model response".into(),
            supported_by: vec![source],
            contradicted_by: vec![],
            uncertainties: vec!["Scope remains unverified".into()],
            verification_steps: vec!["Execute the selected test".into()],
        },
        "test-response",
    );
    let planned = f
        .service
        .prepare(f.request(
            "test_bad",
            command("setTimeout(()=>process.exit(1),1000)", 10000),
            0,
        ))
        .unwrap();
    f.service
        .approve(&planned.id, planned.approval_token.as_deref().unwrap())
        .unwrap();
    store::merge_ai(&f.db, &outdated).unwrap();
    let done = f.wait(&planned.id);
    assert_eq!(done.status, "bad_confirmed", "{:?}", done.error);
    // A response based on an older snapshot must not erase already completed proof either.
    let merged = store::merge_ai(&f.db, &outdated).unwrap();
    assert_eq!(
        merged.nodes.iter().filter(|n| n.id == hypothesis).count(),
        1
    );
    assert!(merged.nodes.iter().any(|n| n.kind == "test_execution"));
    let (mut case, runs) = store::load_bundle(&f.db, &f.case_id).unwrap();
    verification::audit_runs(&mut case, &runs).unwrap();
    assert!(case
        .nodes
        .iter()
        .filter(|n| n.data["verificationId"] == done.id)
        .all(|n| n.validity == "snapshot"));
}
