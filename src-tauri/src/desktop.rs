use crate::{ai, investigation, model::*, repository::Sessions, store};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::Manager;
struct Backend {
    sessions: Sessions,
    db: PathBuf,
    cases: Mutex<HashMap<String, Investigation>>,
}
type State = Arc<Backend>;
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn open_repository(state: tauri::State<'_, State>, path: String) -> Result<Repository> {
    let s = state.inner().clone();
    blocking(move || s.sessions.open(&path)).await
}
#[tauri::command]
async fn analyze(
    state: tauri::State<'_, State>,
    token: String,
    stack: String,
    logs: String,
    tests: String,
) -> Result<Investigation> {
    let s = state.inner().clone();
    blocking(move || {
        let c = investigation::investigate(&s.sessions, &token, &stack, &logs, &tests)?;
        store::save(&s.db, &c)?;
        s.cases
            .lock()
            .map_err(|_| "Case lock failed")?
            .insert(c.id.clone(), c.clone());
        Ok(c)
    })
    .await
}
#[tauri::command]
async fn list_cases(state: tauri::State<'_, State>) -> Result<serde_json::Value> {
    let s = state.inner().clone();
    blocking(move || store::list(&s.db)).await
}
#[tauri::command]
async fn load_case(state: tauri::State<'_, State>, id: String) -> Result<Investigation> {
    let s = state.inner().clone();
    blocking(move || {
        let mut c = store::load(&s.db, &id)?;
        investigation::check_stale(&mut c, &s.sessions)?;
        s.cases
            .lock()
            .map_err(|_| "Case lock failed")?
            .insert(c.id.clone(), c.clone());
        Ok(c)
    })
    .await
}
#[tauri::command]
async fn ai_context(state: tauri::State<'_, State>, id: String) -> Result<serde_json::Value> {
    let s = state.inner().clone();
    blocking(move || {
        let cases = s.cases.lock().map_err(|_| "Case lock failed")?;
        let c = cases.get(&id).ok_or("Case is not open")?;
        Ok(ai::context(c))
    })
    .await
}
#[tauri::command]
async fn generate_hypothesis(
    state: tauri::State<'_, State>,
    id: String,
    model: String,
) -> Result<Investigation> {
    let s = state.inner().clone();
    blocking(move || {
        let mut c = s
            .cases
            .lock()
            .map_err(|_| "Case lock failed")?
            .get(&id)
            .cloned()
            .ok_or("Case is not open")?;
        investigation::check_stale(&mut c, &s.sessions)?;
        ai::generate(&mut c, &model)?;
        investigation::check_stale(&mut c,&s.sessions)?;
        if c.stale {return Err("Repository changed while AI was running. Refresh the case before generating a hypothesis.".into());}
        store::save(&s.db, &c)?;
        s.cases
            .lock()
            .map_err(|_| "Case lock failed")?
            .insert(id, c.clone());
        Ok(c)
    })
    .await
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = app.path().app_local_data_dir()?.join("cases.sqlite");
            app.manage(Arc::new(Backend {
                sessions: Sessions::default(),
                db,
                cases: Mutex::new(HashMap::new()),
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_repository,
            analyze,
            list_cases,
            load_case,
            ai_context,
            generate_hypothesis
        ])
        .run(tauri::generate_context!())
        .expect("TRACE desktop startup failed");
}
