//! Explicit approved process execution. No command-string shell path.
use crate::{model::*, repository::display_path};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
pub const ARTIFACT_LIMIT: usize = 16 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub timeout_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCommand {
    pub requested: TestCommand,
    pub executable: String,
    pub args: Vec<String>,
    pub environment: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub id: String,
    pub command: ResolvedCommand,
    pub cwd: String,
    pub revision: String,
    pub started_at: u64,
    pub finished_at: u64,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_artifact: String,
    pub stderr_artifact: String,
    pub stdout_hash: String,
    pub stderr_hash: String,
    pub output_truncated: bool,
    pub timed_out: bool,
    pub cancelled: bool,
    pub status: String,
    pub error: Option<String>,
    pub integrity_hash: String,
}
pub fn digest(e: &Execution) -> String {
    let mut v = e.clone();
    v.integrity_hash.clear();
    hash(&serde_json::to_vec(&v).unwrap())
}
pub fn classification(e: &Execution) -> Option<&'static str> {
    match e.status.as_str() {
        "passed" => Some("good"),
        "failed" => Some("bad"),
        _ => None,
    }
}
pub fn resolve_command(c: &TestCommand, repo: &Path) -> Result<ResolvedCommand> {
    if c.executable.trim().is_empty()
        || c.executable.len() > 1024
        || c.args.len() > 128
        || c.args.iter().any(|a| a.len() > 8192 || a.contains('\0'))
        || c.executable.contains('\0')
    {
        return Err("Invalid structured test command".into());
    }
    if !(100..=3_600_000).contains(&c.timeout_ms) {
        return Err("Timeout must be between 0.1 seconds and 1 hour".into());
    }
    let relative = Path::new(&c.working_directory);
    if relative.is_absolute()
        || c.working_directory.contains(':')
        || c.working_directory
            .replace('\\', "/")
            .split('/')
            .any(|s| s == "..")
    {
        return Err("Working directory must be relative inside the temporary worktree".into());
    }
    let name = Path::new(&c.executable)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if [
        "cmd",
        "cmd.exe",
        "powershell",
        "powershell.exe",
        "pwsh",
        "pwsh.exe",
        "sh",
        "bash",
        "wscript.exe",
        "cscript.exe",
    ]
    .contains(&name.as_str())
    {
        return Err(
            "Shell interpreters are not supported. Enter a direct executable and argument array."
                .into(),
        );
    }
    let find = |name: &str| -> Result<PathBuf> {
        let candidate = Path::new(name);
        let mut choices = vec![];
        if candidate.is_absolute() {
            choices.push(candidate.to_path_buf());
        } else if candidate.components().count() == 1 {
            if let Some(p) = std::env::var_os("PATH") {
                for dir in std::env::split_paths(&p).filter(|p| p.is_absolute()) {
                    choices.push(dir.join(name));
                    if cfg!(windows) && Path::new(name).extension().is_none() {
                        choices.push(dir.join(format!("{name}.exe")));
                        choices.push(dir.join(format!("{name}.cmd")));
                    }
                }
            }
        }
        for p in choices {
            if p.is_file() {
                if let Ok(p) = p.canonicalize() {
                    if !p.starts_with(repo) {
                        return Ok(p);
                    }
                }
            }
        }
        Err(format!("Test executable unavailable outside the source repository: {name}. Install it or select its absolute path; no dependencies are installed automatically."))
    };
    let mut program = find(&c.executable)?;
    let mut args = c.args.clone();
    if program
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd"))
    {
        let stem = program
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_ascii_lowercase();
        let suffix = match stem.as_str() {
            "npm" => "node_modules/npm/bin/npm-cli.js",
            "pnpm" => "node_modules/pnpm/bin/pnpm.cjs",
            "yarn" => "node_modules/yarn/bin/yarn.js",
            _ => return Err("Batch scripts are unsupported; select a direct executable".into()),
        };
        let cli=program.parent().unwrap().join(suffix).canonicalize().map_err(|_|format!("Cannot resolve {stem}.cmd to a trusted JavaScript CLI. Select node.exe with the installed CLI file explicitly."))?;
        if cli.starts_with(repo) {
            return Err("Repository-local package manager rejected".into());
        }
        program = find("node")?;
        args.insert(0, display_path(&cli));
    }
    if cfg!(windows)
        && !program
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
    {
        return Err("Select a direct .exe program or supported npm/pnpm/yarn launcher".into());
    }
    Ok(ResolvedCommand {
        requested: c.clone(),
        executable: display_path(&program),
        args,
        environment: serde_json::json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"executableResolution":"canonical absolute PATH entry outside source repository","policy":"Inherited system environment except Git overrides, NODE_OPTIONS and package-manager shell overrides; normal network/filesystem permissions. Not a sandbox."}),
    })
}
pub fn working_directory(root: &Path, relative: &str) -> Result<PathBuf> {
    let p = root
        .join(relative)
        .canonicalize()
        .map_err(|e| format!("Test working directory unavailable: {e}"))?;
    if !p.starts_with(&root.canonicalize().map_err(|e| e.to_string())?) || !p.is_dir() {
        return Err("Test working directory escapes worktree".into());
    }
    Ok(p)
}

#[cfg(windows)]
struct ProcessTree(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl ProcessTree {
    fn attach(child: &std::process::Child) -> Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{
            Foundation::*,
            System::{Diagnostics::ToolHelp::*, JobObjects::*, Threading::*},
        };
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err("Cannot create Windows process Job Object".into());
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            ) == 0
                || AssignProcessToJobObject(job, child.as_raw_handle() as _) == 0
            {
                CloseHandle(job);
                return Err(format!(
                    "Cannot isolate process tree: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                CloseHandle(job);
                return Err("Cannot enumerate suspended process thread".into());
            }
            let mut entry: THREADENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of_val(&entry) as u32;
            let mut ok = Thread32First(snapshot, &mut entry);
            let mut resumed = false;
            while ok != 0 {
                if entry.th32OwnerProcessID == child.id() {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                    if !thread.is_null() {
                        resumed = ResumeThread(thread) != u32::MAX;
                        CloseHandle(thread);
                        if resumed {
                            break;
                        }
                    }
                }
                ok = Thread32Next(snapshot, &mut entry);
            }
            CloseHandle(snapshot);
            if !resumed {
                CloseHandle(job);
                return Err("Cannot resume isolated test process".into());
            }
            Ok(Self(job))
        }
    }
}
#[cfg(windows)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

pub fn execute(
    command: &ResolvedCommand,
    worktree: &Path,
    revision: &str,
    artifacts: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<Execution> {
    std::fs::create_dir_all(artifacts).map_err(|e| e.to_string())?;
    let eid = id();
    let outpath = artifacts.join(format!("{eid}.stdout"));
    let errpath = artifacts.join(format!("{eid}.stderr"));
    let mut e = Execution {
        id: eid,
        command: command.clone(),
        cwd: display_path(&worktree.join(&command.requested.working_directory)),
        revision: revision.into(),
        started_at: now(),
        finished_at: now(),
        duration_ms: 0,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        stdout_artifact: display_path(&outpath),
        stderr_artifact: display_path(&errpath),
        stdout_hash: hash(b""),
        stderr_hash: hash(b""),
        output_truncated: false,
        timed_out: false,
        cancelled: false,
        status: "launch_error".into(),
        error: None,
        integrity_hash: String::new(),
    };
    let start = Instant::now();
    let cwd = working_directory(worktree, &command.requested.working_directory);
    let mut cmd = Command::new(&command.executable);
    cmd.args(&command.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, _) in std::env::vars() {
        if key.starts_with("GIT_")
            || key.eq_ignore_ascii_case("NODE_OPTIONS")
            || key.eq_ignore_ascii_case("npm_config_script_shell")
        {
            cmd.env_remove(key);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000 | 0x00000004);
    }
    let spawned = cwd.and_then(|cwd| {
        e.cwd = display_path(&cwd);
        cmd.current_dir(cwd)
            .spawn()
            .map_err(|err| format!("Test launch failed: {err}"))
    });
    match spawned {
        Err(err) => {
            e.error = Some(err);
            std::fs::write(&outpath, []).map_err(|e| e.to_string())?;
            std::fs::write(&errpath, []).map_err(|e| e.to_string())?;
        }
        Ok(mut child) => {
            #[cfg(windows)]
            let tree = match ProcessTree::attach(&child) {
                Ok(tree) => tree,
                Err(err) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(err);
                }
            };
            let overflow = Arc::new(AtomicBool::new(false));
            fn capture<R: Read + Send + 'static>(
                mut stream: R,
                path: PathBuf,
                overflow: Arc<AtomicBool>,
            ) -> std::thread::JoinHandle<Result<(Vec<u8>, String)>> {
                std::thread::spawn(move || {
                    use sha2::{Digest, Sha256};
                    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
                    let mut digest = Sha256::new();
                    let mut preview = vec![];
                    let mut count = 0;
                    let mut b = [0u8; 8192];
                    loop {
                        let n = stream.read(&mut b).map_err(|e| e.to_string())?;
                        if n == 0 {
                            break;
                        }
                        let keep = n.min(ARTIFACT_LIMIT.saturating_sub(count));
                        if keep > 0 {
                            file.write_all(&b[..keep]).map_err(|e| e.to_string())?;
                            digest.update(&b[..keep]);
                            if preview.len() < 32000 {
                                let p = keep.min(32000 - preview.len());
                                preview.extend_from_slice(&b[..p]);
                            }
                            count += keep;
                        }
                        if keep < n {
                            overflow.store(true, Ordering::SeqCst);
                        }
                    }
                    file.sync_all().map_err(|e| e.to_string())?;
                    Ok((preview, format!("{:x}", digest.finalize())))
                })
            }
            let out = capture(child.stdout.take().unwrap(), outpath, overflow.clone());
            let err = capture(child.stderr.take().unwrap(), errpath, overflow.clone());
            loop {
                if cancel.load(Ordering::SeqCst) {
                    e.status = "cancelled".into();
                    e.cancelled = true;
                    break;
                }
                if overflow.load(Ordering::SeqCst) {
                    e.status = "output_limit".into();
                    e.error=Some("Output exceeded 16 MiB per stream; retained bounded artifacts. Verification inconclusive.".into());
                    break;
                }
                if start.elapsed() > Duration::from_millis(command.requested.timeout_ms) {
                    e.status = "timeout".into();
                    e.timed_out = true;
                    break;
                }
                match child.try_wait() {
                    Ok(Some(s)) => {
                        e.exit_code = s.code();
                        e.status = if s.success() { "passed" } else { "failed" }.into();
                        break;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        e.error = Some(err.to_string());
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            #[cfg(windows)]
            drop(tree);
            let _ = child.kill();
            let _ = child.wait();
            let (stdout, sh) = out.join().map_err(|_| "Output reader failed")??;
            let (stderr, eh) = err.join().map_err(|_| "Output reader failed")??;
            e.output_truncated = std::fs::metadata(&e.stdout_artifact)
                .map(|m| m.len() > stdout.len() as u64)
                .unwrap_or(false)
                || std::fs::metadata(&e.stderr_artifact)
                    .map(|m| m.len() > stderr.len() as u64)
                    .unwrap_or(false);
            e.stdout = String::from_utf8_lossy(&stdout).into_owned();
            e.stderr = String::from_utf8_lossy(&stderr).into_owned();
            e.stdout_hash = sh;
            e.stderr_hash = eh;
            if overflow.load(Ordering::SeqCst) {
                e.status = "output_limit".into();
                e.error = Some(
                    "Output exceeded the bounded artifact limit; verification inconclusive".into(),
                );
            }
        }
    }
    e.finished_at = now();
    e.duration_ms = start.elapsed().as_millis() as u64;
    e.integrity_hash = digest(&e);
    Ok(e)
}
