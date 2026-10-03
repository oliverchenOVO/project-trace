use crate::model::*;
use rusqlite::{params, Connection};
use std::path::Path;

fn connect(path: &Path) -> Result<Connection> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS cases(id TEXT PRIMARY KEY, title TEXT NOT NULL, created INTEGER NOT NULL, snapshot TEXT NOT NULL);").map_err(|e|e.to_string())?;
    Ok(c)
}
pub fn save(path: &Path, case: &Investigation) -> Result<()> {
    let c = connect(path)?;
    c.execute("INSERT INTO cases(id,title,created,snapshot) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET snapshot=excluded.snapshot,title=excluded.title",params![case.id,case.title,case.created_at,serde_json::to_string(case).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn load(path: &Path, id: &str) -> Result<Investigation> {
    let c = connect(path)?;
    let s: String = c
        .query_row("SELECT snapshot FROM cases WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| format!("Invalid saved case: {e}"))
}
pub fn list(path: &Path) -> Result<serde_json::Value> {
    let c = connect(path)?;
    let mut stmt = c
        .prepare("SELECT id,title,created FROM cases ORDER BY created DESC LIMIT 50")
        .map_err(|e| e.to_string())?;
    let rows=stmt.query_map([],|r|Ok(serde_json::json!({"id":r.get::<_,String>(0)?,"title":r.get::<_,String>(1)?,"createdAt":r.get::<_,u64>(2)?}))).map_err(|e|e.to_string())?;
    Ok(serde_json::Value::Array(
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    ))
}
