use serde_json::json;
use trace_core::{ai, model::*};
fn case() -> Investigation {
    let mut c = Investigation {
        id: id(),
        title: "Incident".into(),
        created_at: now(),
        repository: Repository {
            token: "test".into(),
            root: "test".into(),
            head: None,
            branch: "main".into(),
            dirty: false,
            status: "".into(),
            recent_commits: vec![],
        },
        stack: "error".into(),
        logs: "".into(),
        tests: "".into(),
        frames: vec![],
        nodes: vec![],
        edges: vec![],
        warnings: vec![],
        source_hashes: Default::default(),
        stale: false,
    };
    let p = Provenance::input("source", &"x".repeat(500_000), "artifact");
    for _ in 0..25 {
        c.node(
            "source_line",
            Status::Derived,
            &"字".repeat(1500),
            &"字".repeat(100_000),
            "static_analysis",
            p.clone(),
            json!({}),
        );
    }
    c
}
#[test]
fn context_is_bounded_even_for_minified_unicode_sources() {
    let c = case();
    let p = ai::context(&c);
    assert!(p["bytes"].as_u64().unwrap() <= 32_002);
    assert_eq!(
        p["bytes"].as_u64().unwrap() as usize,
        serde_json::to_vec(&p["evidence"]).unwrap().len()
    );
    assert!(!p["evidence"].as_array().unwrap().is_empty());
}
#[test]
fn duplicate_citations_rejected() {
    let c = case();
    let id = &c.nodes[0].id;
    let h = json!({"hypothesis":"Maybe","supportedBy":[id,id],"contradictedBy":[],"uncertainties":["unknown"],"verificationSteps":["reproduce"]});
    assert!(ai::validate(&h.to_string(), &c)
        .unwrap_err()
        .contains("Duplicate"));
}
#[test]
fn contradictory_citations_rejected() {
    let c = case();
    let id = &c.nodes[0].id;
    let h = json!({"hypothesis":"Maybe","supportedBy":[id],"contradictedBy":[id],"uncertainties":["unknown"],"verificationSteps":["reproduce"]});
    assert!(ai::validate(&h.to_string(), &c)
        .unwrap_err()
        .contains("both support"));
}
#[test]
fn sensitive_provenance_never_in_context() {
    let mut c = case();
    for n in &mut c.nodes {
        n.provenance.file_path = Some(".env.production".into());
    }
    let p = ai::context(&c);
    assert!(p["evidence"].as_array().unwrap().is_empty());
    assert_eq!(p["excludedCount"], 25);
}
