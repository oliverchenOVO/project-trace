use trace_core::git::installed_git;
#[test]
fn missing_git_is_actionable() {
    let d = tempfile::tempdir().unwrap();
    assert!(installed_git(d.path(), None)
        .unwrap_err()
        .contains("Install Git for Windows"));
}
#[test]
fn repository_local_executable_rejected() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    let name = if cfg!(windows) { "git.exe" } else { "git" };
    std::fs::write(root.join(name), b"untrusted executable").unwrap();
    let path = std::env::join_paths([&root]).unwrap();
    assert!(installed_git(&root, Some(&path)).is_err());
}
#[test]
fn pins_installed_absolute_path_and_ignores_relative_search() {
    let repo = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let name = if cfg!(windows) { "git.exe" } else { "git" };
    let exe = tools.path().join(name);
    std::fs::write(&exe, b"test executable reference; never executed").unwrap();
    let path = std::env::join_paths([std::path::Path::new("."), tools.path()]).unwrap();
    assert_eq!(
        installed_git(&repo.path().canonicalize().unwrap(), Some(&path)).unwrap(),
        exe.canonicalize().unwrap()
    );
}
