use trace_core::repository;
#[test]
fn encoded_file_url_resolves_spaces_and_unicode() {
    let d = tempfile::Builder::new()
        .prefix("trace 空間 ")
        .tempdir()
        .unwrap();
    let p = d.path().join("source file.ts");
    std::fs::write(&p, "source\n").unwrap();
    let root = d.path().canonicalize().unwrap();
    let url = url::Url::from_file_path(&p).unwrap();
    let src = repository::source(&root, url.as_str(), 1, None).unwrap();
    assert_eq!(src.path, "source file.ts");
}
