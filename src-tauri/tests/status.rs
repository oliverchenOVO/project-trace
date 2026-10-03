use trace_core::repository::parse_status;
#[test]
fn status_clean() {
    let raw = format!("# branch.oid {}\0# branch.head main\0", "a".repeat(40));
    let (h, b, d) = parse_status(&raw).unwrap();
    assert_eq!(h, Some("a".repeat(40)));
    assert_eq!(b, "main");
    assert!(!d);
}
#[test]
fn status_unborn_dirty() {
    let (h, b, d) =
        parse_status("# branch.oid (initial)\0# branch.head main\0? source file.ts\0").unwrap();
    assert_eq!(h, None);
    assert_eq!(b, "main");
    assert!(d);
}
#[test]
fn status_detached() {
    let raw = format!(
        "# branch.oid {}\0# branch.head (detached)\0",
        "b".repeat(40)
    );
    assert_eq!(parse_status(&raw).unwrap().1, "(detached)");
}
#[test]
fn status_malformed_rejected() {
    assert!(parse_status("# branch.head main\0").is_err());
    assert!(parse_status("# branch.oid injected\0# branch.head main\0").is_err());
}
