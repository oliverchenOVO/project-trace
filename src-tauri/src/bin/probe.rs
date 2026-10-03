//! Development/test-only JSON adapter to the exact native core. No shell or AI mock.
use serde_json::{json, Value};
use std::{io::Read, path::PathBuf};
use trace_core::{ai, investigation, model::*, repository::Sessions, store};
fn process(r: Value) -> Result<Value> {
    let sessions = Sessions::default();
    let args = &r["args"];
    let db = std::env::var_os("TRACE_QA_DB")
        .map(PathBuf::from)
        .ok_or("TRACE_QA_DB must be configured for diagnostics")?;
    let string = |key: &str| args[key].as_str().unwrap_or("");
    match r["command"].as_str().unwrap_or("") {
        "open_repository" => Ok(json!(sessions.open(string("path"))?)),
        "analyze" => {
            let repo = sessions.open(string("path"))?;
            let c = investigation::investigate(
                &sessions,
                &repo.token,
                string("stack"),
                string("logs"),
                string("tests"),
            )?;
            store::save(&db, &c)?;
            Ok(json!(c))
        }
        "list_cases" => store::list(&db),
        "load_case" => {
            let mut c = store::load(&db, string("id"))?;
            investigation::check_stale(&mut c, &sessions)?;
            Ok(json!(c))
        }
        "ai_context" => Ok(ai::context(&store::load(&db, string("id"))?)),
        "generate_hypothesis" => {
            let mut c = store::load(&db, string("id"))?;
            investigation::check_stale(&mut c, &sessions)?;
            ai::generate(&mut c, string("model"))?;
            store::save(&db, &c)?;
            Ok(json!(c))
        }
        _ => Err("Unknown diagnostic command".into()),
    }
}
fn main() {
    let mut raw = String::new();
    std::io::stdin()
        .take(256_000)
        .read_to_string(&mut raw)
        .unwrap();
    let result = serde_json::from_str(&raw)
        .map_err(|e| e.to_string())
        .and_then(process);
    println!(
        "{}",
        match result {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e}),
        }
    );
}
