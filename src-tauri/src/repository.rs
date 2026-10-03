use crate::{git, model::*};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Default)]
pub struct Sessions(pub Mutex<HashMap<String, PathBuf>>);
impl Sessions {
    pub fn open(&self, path: &str) -> Result<Repository> {
        let root = Path::new(path)
            .canonicalize()
            .map_err(|e| format!("Repository folder unavailable: {e}"))?;
        if !root.is_dir() {
            return Err("Select a repository folder".into());
        }
        let top = git::git(&root, &["rev-parse", "--show-toplevel"])?;
        let canonical = Path::new(top.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let token = id();
        let repo = metadata(&canonical, &token)?;
        self.0
            .lock()
            .map_err(|_| "Repository session lock failed")?
            .insert(token, canonical);
        Ok(repo)
    }
    pub fn root(&self, token: &str) -> Result<PathBuf> {
        self.0
            .lock()
            .map_err(|_| "Repository session lock failed")?
            .get(token)
            .cloned()
            .ok_or("Repository session expired; open the folder again".into())
    }
}
pub fn display_path(p: &Path) -> String {
    p.to_string_lossy()
        .trim_start_matches("\\\\?\\")
        .to_string()
}
pub fn metadata(root: &Path, token: &str) -> Result<Repository> {
    let head = match git::git(root, &["rev-parse", "--verify", "HEAD"]) {
        Ok(h) => Some(h.trim().to_string()),
        Err(e) => {
            if e.contains("Needed a single revision") || e.contains("unknown revision") {
                None
            } else {
                return Err(e);
            }
        }
    };
    let branch = git::git(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "unborn".into())
        .trim()
        .to_string();
    let status = git::git(
        root,
        &["status", "--porcelain=v1", "-z", "--untracked-files=normal"],
    )?;
    let recent_commits = if head.is_some() {
        git::commits(root, None)?
    } else {
        vec![]
    };
    Ok(Repository {
        token: token.into(),
        root: display_path(root),
        head,
        branch,
        dirty: !status.is_empty(),
        status,
        recent_commits,
    })
}
pub fn excluded(path: &str) -> bool {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    path.split('/').any(|p| {
        p == ".git"
            || p == ".env"
            || p.starts_with(".env.")
            || p.ends_with(".pem")
            || p.ends_with(".key")
            || p == "id_rsa"
            || p == "id_ed25519"
            || p.starts_with("credentials")
            || p.starts_with("secrets")
            || ["node_modules", "dist", "build", "coverage"].contains(&p)
    })
}
pub fn resolve(root: &Path, input: &str) -> Result<PathBuf> {
    let url_path;
    let input = if input.trim().starts_with("file://") {
        url_path = url::Url::parse(input.trim())
            .map_err(|_| "Invalid file URL")?
            .to_file_path()
            .map_err(|_| "File URL is not a local path")?;
        url_path.to_str().ok_or("Source URL is not UTF-8")?
    } else {
        input.trim()
    };
    let input = input.strip_prefix("webpack:///").unwrap_or(input);
    if input.contains('\0') || input.split(['/', '\\']).any(|p| p == "..") {
        return Err("Path traversal rejected".into());
    }
    let p = Path::new(input);
    let path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    let canonical = path
        .canonicalize()
        .map_err(|_| format!("Stack source does not exist: {input}"))?;
    if !canonical.starts_with(root) {
        return Err("Source path escapes the selected repository".into());
    }
    let rel = canonical
        .strip_prefix(root)
        .map_err(|_| "Invalid source reference")?;
    if excluded(&rel.to_string_lossy()) {
        return Err("Source excluded by the sensitive/generated-file policy".into());
    }
    if !canonical.is_file() {
        return Err("Source is not a regular file".into());
    }
    Ok(canonical)
}
pub fn source(
    root: &Path,
    input: &str,
    line: usize,
    column: Option<usize>,
) -> Result<SourceExcerpt> {
    let p = resolve(root, input)?;
    let bytes = read_source(&p)?;
    let hash = hash(&bytes);
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| "Source encoding unsupported; UTF-8 is required")?;
    let lines: Vec<_> = text.lines().map(str::to_string).collect();
    if line == 0 || line > lines.len() {
        return Err(format!(
            "Stack line {line} outside file bounds ({} lines)",
            lines.len()
        ));
    }
    let start = line.saturating_sub(6).max(1);
    let end = (line + 6).min(lines.len());
    Ok(SourceExcerpt {
        path: p
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/"),
        line,
        column,
        start,
        lines: lines[start - 1..end].to_vec(),
        hash,
    })
}
pub fn read_source(p: &Path) -> Result<Vec<u8>> {
    if p.metadata().map_err(|e| e.to_string())?.len() > 512_000 {
        return Err("Source exceeds the 512 KB limit".into());
    }
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    if bytes.contains(&0) {
        return Err("Binary source cannot be inspected".into());
    }
    Ok(bytes)
}
