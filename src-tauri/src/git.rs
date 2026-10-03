use crate::model::*;
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub const OUTPUT_LIMIT: usize = 2_000_000;
pub fn run_git(root: &Path, args: &[String]) -> Result<String> {
    let allowed = [
        "rev-parse",
        "status",
        "log",
        "show",
        "diff",
        "blame",
        "ls-files",
    ];
    if !args
        .first()
        .map(|s| allowed.contains(&s.as_str()))
        .unwrap_or(false)
    {
        return Err("Git operation is not on the read-only allowlist".into());
    }
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args([
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.pager=cat",
            "-c",
            "log.showSignature=false",
        ])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat");
    for (key, _) in std::env::vars() {
        if key.starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| {
        format!("Cannot launch Git. Install Git for Windows and restart TRACE. {e}")
    })?;
    let out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    fn bounded<R: Read>(r: R) -> std::io::Result<Vec<u8>> {
        let mut b = vec![];
        r.take((OUTPUT_LIMIT + 1) as u64).read_to_end(&mut b)?;
        Ok(b)
    }
    let a = std::thread::spawn(move || bounded(out));
    let b = std::thread::spawn(move || bounded(err));
    let start = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(s) => break s,
            None => {
                if start.elapsed() > Duration::from_secs(20) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("Git inspection timed out after 20 seconds".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    };
    let stdout = a
        .join()
        .map_err(|_| "Git reader failed")?
        .map_err(|e| e.to_string())?;
    let stderr = b
        .join()
        .map_err(|_| "Git reader failed")?
        .map_err(|e| e.to_string())?;
    if stdout.len() > OUTPUT_LIMIT || stderr.len() > OUTPUT_LIMIT {
        return Err(
            "Git output exceeds the 2 MB inspection limit; narrow the investigation".into(),
        );
    }
    if !status.success() {
        return Err(format!(
            "Git {} failed: {}",
            args[0],
            String::from_utf8_lossy(&stderr)
        ));
    }
    String::from_utf8(stdout)
        .map_err(|_| "Git output contains unsupported non-UTF-8 paths or content".into())
}
pub fn git(root: &Path, args: &[&str]) -> Result<String> {
    run_git(
        root,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}
pub fn parse_commits(raw: &str) -> Vec<Commit> {
    raw.split('\0')
        .filter_map(|r| {
            let parts: Vec<_> = r.trim_start_matches('\n').split('\x1f').collect();
            if parts.len() != 4 {
                return None;
            }
            Some(Commit {
                sha: parts[0].into(),
                author: parts[1].into(),
                timestamp: parts[2].into(),
                subject: parts[3].trim_end().into(),
            })
        })
        .collect()
}
pub fn commits(root: &Path, file: Option<&str>) -> Result<Vec<Commit>> {
    let mut args = vec![
        "log".into(),
        "-12".into(),
        "--format=%H%x1f%an%x1f%aI%x1f%s%x00".into(),
    ];
    if let Some(f) = file {
        args.push("--".into());
        args.push(f.into());
    }
    parse_history(root, &args)
}
fn parse_history(root: &Path, args: &[String]) -> Result<Vec<Commit>> {
    Ok(parse_commits(&run_git(root, args)?))
}
pub fn valid_sha(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64)
        && s.bytes().all(|c| c.is_ascii_hexdigit())
        && !s.bytes().all(|c| c == b'0')
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Blame {
    pub sha: String,
    pub author: String,
    pub timestamp: String,
    pub source: String,
    pub uncommitted: bool,
}
pub fn parse_blame(raw: &str) -> Result<Blame> {
    let sha = raw
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().next())
        .ok_or("Empty blame result")?
        .trim_start_matches('^')
        .to_string();
    if !(sha.len() == 40 || sha.len() == 64) || !sha.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid blame SHA".into());
    }
    let value = |prefix: &str| {
        raw.lines()
            .find_map(|l| l.strip_prefix(prefix))
            .unwrap_or("")
            .to_string()
    };
    Ok(Blame {
        uncommitted: sha.bytes().all(|c| c == b'0'),
        sha,
        author: value("author "),
        timestamp: value("author-time "),
        source: value("\t"),
    })
}
pub fn provenance(
    args: Vec<String>,
    raw: &str,
    file: Option<&str>,
    line: Option<usize>,
    sha: Option<&str>,
) -> Provenance {
    Provenance {
        source_type: "git".into(),
        file_path: file.map(str::to_string),
        line_start: line,
        line_end: line,
        commit_sha: sha.map(str::to_string),
        command: std::iter::once("git".into()).chain(args).collect(),
        raw_excerpt: raw.into(),
        artifact_id: None,
        source_hash: Some(hash(raw.as_bytes())),
        captured_at: now(),
    }
}
