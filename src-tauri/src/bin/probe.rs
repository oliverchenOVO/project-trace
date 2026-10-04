//! Development/test-only JSON adapter to the exact native core. No shell or AI mock.
use serde_json::{json, Value};
use std::{
    io::{BufRead, Read},
    path::PathBuf,
    sync::Arc,
};
use trace_core::{ai, investigation, model::*, repository::Sessions, store, verification};
fn process(r: Value, service: &Arc<verification::Service>) -> Result<Value> {
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
            verification::audit_case(&db, &mut c)?;
            Ok(json!(c))
        }
        "ai_context" => Ok(ai::context(&store::load(&db, string("id"))?)),
        "generate_hypothesis" => {
            if service.busy(string("id")) {
                return Err("Wait for verification before generating AI".into());
            }
            let mut c = store::load(&db, string("id"))?;
            investigation::check_stale(&mut c, &sessions)?;
            ai::generate(&mut c, string("model"))?;
            investigation::check_stale(&mut c, &sessions)?;
            if c.stale {
                return Err("Repository changed while AI was running".into());
            }
            store::save(&db, &c)?;
            Ok(json!(c))
        }
        "prepare_verification" => Ok(json!(service.prepare(
            serde_json::from_value(args["request"].clone()).map_err(|e| e.to_string())?
        )?)),
        "approve_verification" => Ok(json!(service.approve(string("runId"), string("token"))?)),
        "verification_status" => {
            let (mut c, mut runs) = store::load_bundle(&db, string("id"))?;
            verification::audit_runs(&mut c, &runs)?;
            for r in &mut runs {
                if let Err(err) = verification::validate_run(r) {
                    r.status = "integrity_error".into();
                    r.error = Some(err);
                }
            }
            Ok(json!({"runs":runs,"caseFile":c}))
        }
        "cancel_verification" => {
            service.cancel(string("runId"))?;
            Ok(json!(null))
        }
        "discard_verification" => Ok(json!(service.discard(string("runId"))?)),
        "cleanup_verification" => Ok(json!(service.cleanup(string("runId"))?)),
        _ => Err("Unknown diagnostic command".into()),
    }
}
fn main() {
    let db = PathBuf::from(std::env::var_os("TRACE_QA_DB").expect("TRACE_QA_DB required"));
    let base = db.parent().unwrap().join(format!(
        "{}-verification",
        db.file_stem().unwrap().to_string_lossy()
    ));
    let service = verification::Service::new(db, base).expect("Diagnostic verification service");
    if std::env::args().any(|a| a == "--serve") {
        for line in std::io::stdin().lock().lines() {
            let raw = match line {
                Ok(s) => s,
                Err(_) => break,
            };
            let result = serde_json::from_str::<Value>(&raw).map_err(|e| e.to_string());
            let request_id = result
                .as_ref()
                .ok()
                .and_then(|r| r["requestId"].as_u64())
                .unwrap_or(0);
            let response = match result.and_then(|r| process(r, &service)) {
                Ok(v) => json!({"requestId":request_id,"ok":v}),
                Err(e) => json!({"requestId":request_id,"error":e}),
            };
            println!("{response}");
        }
        return;
    }
    let mut raw = String::new();
    std::io::stdin()
        .take(256_000)
        .read_to_string(&mut raw)
        .unwrap();
    let result = serde_json::from_str(&raw)
        .map_err(|e| e.to_string())
        .and_then(|r| process(r, &service));
    println!(
        "{}",
        match result {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e}),
        }
    );
}
