use crate::model::*;
use rusqlite::{params, Connection, TransactionBehavior};
use std::path::Path;

fn connect(path: &Path) -> Result<Connection> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    c.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS cases(id TEXT PRIMARY KEY, title TEXT NOT NULL, created INTEGER NOT NULL, snapshot TEXT NOT NULL); CREATE TABLE IF NOT EXISTS verification_runs(id TEXT PRIMARY KEY, case_id TEXT NOT NULL, snapshot TEXT NOT NULL); CREATE TABLE IF NOT EXISTS case_metadata(case_id TEXT PRIMARY KEY, name TEXT NOT NULL DEFAULT '', tags TEXT NOT NULL DEFAULT '[]', archived INTEGER NOT NULL DEFAULT 0, last_opened INTEGER NOT NULL DEFAULT 0);").map_err(|e|e.to_string())?;
    Ok(c)
}
pub fn save_run(path: &Path, run: &crate::verification::VerificationRun) -> Result<()> {
    let c = connect(path)?;
    c.execute("INSERT INTO verification_runs VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET snapshot=excluded.snapshot",params![run.id,run.case_id,serde_json::to_string(run).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn load_run(path: &Path, id: &str) -> Result<crate::verification::VerificationRun> {
    let c = connect(path)?;
    let raw: String = c
        .query_row(
            "SELECT snapshot FROM verification_runs WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}
pub fn runs(
    path: &Path,
    case_id: Option<&str>,
) -> Result<Vec<crate::verification::VerificationRun>> {
    let c = connect(path)?;
    let mut s=c.prepare("SELECT snapshot FROM verification_runs WHERE (?1 IS NULL OR case_id=?1) ORDER BY rowid").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([case_id], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.map(|r| serde_json::from_str(&r.map_err(|e| e.to_string())?).map_err(|e| e.to_string()))
        .collect()
}
pub fn save_bundle(path: &Path, run: &crate::verification::VerificationRun) -> Result<()> {
    let mut c = connect(path)?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let raw: String = tx
        .query_row(
            "SELECT snapshot FROM cases WHERE id=?1",
            [&run.case_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut case: Investigation = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if run.source_unchanged == Some(false) {
        case.stale = true;
    }
    crate::verification::attach(&mut case, run);
    tx.execute("INSERT INTO verification_runs VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET snapshot=excluded.snapshot",params![run.id,run.case_id,serde_json::to_string(run).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    tx.execute(
        "UPDATE cases SET snapshot=?1 WHERE id=?2",
        params![
            serde_json::to_string(&case).map_err(|e| e.to_string())?,
            case.id
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}
pub fn load_bundle(
    path: &Path,
    id: &str,
) -> Result<(Investigation, Vec<crate::verification::VerificationRun>)> {
    let mut c = connect(path)?;
    let tx = c.transaction().map_err(|e| e.to_string())?;
    let raw: String = tx
        .query_row("SELECT snapshot FROM cases WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let case = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let runs = {
        let mut stmt = tx
            .prepare("SELECT snapshot FROM verification_runs WHERE case_id=?1 ORDER BY rowid")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([id], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.map(|r| {
            serde_json::from_str(&r.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>>>()?
    };
    tx.commit().map_err(|e| e.to_string())?;
    Ok((case, runs))
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
        .prepare("SELECT c.id,COALESCE(NULLIF(m.name,''),c.title),c.created,COALESCE(m.tags,'[]'),COALESCE(m.archived,0),COALESCE(m.last_opened,0),c.title FROM cases c LEFT JOIN case_metadata m ON m.case_id=c.id ORDER BY COALESCE(NULLIF(m.last_opened,0),c.created) DESC,c.created DESC,c.rowid DESC")
        .map_err(|e| e.to_string())?;
    let rows=stmt.query_map([],|r|Ok(serde_json::json!({"id":r.get::<_,String>(0)?,"title":r.get::<_,String>(1)?,"createdAt":r.get::<_,u64>(2)?,"tags":serde_json::from_str::<Vec<String>>(&r.get::<_,String>(3)?).unwrap_or_default(),"archived":r.get::<_,bool>(4)?,"lastOpened":r.get::<_,u64>(5)?,"originalTitle":r.get::<_,String>(6)?}))).map_err(|e|e.to_string())?;
    Ok(serde_json::Value::Array(
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    ))
}

/// Presentation metadata is separate from immutable evidence snapshots and runs.
pub fn update_metadata(path: &Path, id: &str, name: &str, tags: Vec<String>, archived: bool) -> Result<()> {
    let name = name.trim();
    if name.chars().count() > 120 || name.chars().any(char::is_control) {
        return Err("Case name must be at most 120 characters without control characters".into());
    }
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() { continue; }
        if tag.chars().count() > 24 || tag.chars().any(char::is_control) {
            return Err("Tags must be at most 24 characters without control characters".into());
        }
        if !normalized.contains(&tag) { normalized.push(tag); }
    }
    if normalized.len() > 8 { return Err("Use at most 8 tags".into()); }
    let mut c = connect(path)?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM cases WHERE id=?1)",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if !exists { return Err("Saved case not found".into()); }
    tx.execute("INSERT INTO case_metadata(case_id,name,tags,archived) VALUES(?1,?2,?3,?4) ON CONFLICT(case_id) DO UPDATE SET name=excluded.name,tags=excluded.tags,archived=excluded.archived",params![id,name,serde_json::to_string(&normalized).map_err(|e|e.to_string())?,archived]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}

pub fn touch(path: &Path, id: &str) -> Result<()> {
    let c = connect(path)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_secs();
    c.execute("INSERT INTO case_metadata(case_id,last_opened) SELECT id,?2 FROM cases WHERE id=?1 ON CONFLICT(case_id) DO UPDATE SET last_opened=excluded.last_opened",params![id,now]).map_err(|e|e.to_string())?;
    Ok(())
}

/// Merge only validated AI hypotheses into the latest snapshot under the writer lock.
/// Verification workers can advance while an Ollama response is in flight.
pub fn merge_ai(path: &Path, incoming: &Investigation) -> Result<Investigation> {
    let mut c = connect(path)?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let raw: String = tx
        .query_row(
            "SELECT snapshot FROM cases WHERE id=?1",
            [&incoming.id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut case: Investigation = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if case.stale || incoming.stale {
        return Err("Refresh the stale case before retaining an AI hypothesis".into());
    }
    let new_ids: std::collections::HashSet<_> = incoming
        .nodes
        .iter()
        .filter(|n| {
            n.created_by == "ai"
                && n.kind == "hypothesis"
                && n.epistemic_status == Status::Hypothesis
                && !case.nodes.iter().any(|old| old.id == n.id)
        })
        .map(|n| n.id.clone())
        .collect();
    case.nodes.extend(
        incoming
            .nodes
            .iter()
            .filter(|n| new_ids.contains(&n.id))
            .cloned(),
    );
    for edge in &incoming.edges {
        if new_ids.contains(&edge.target)
            && edge.epistemic_status == Status::Hypothesis
            && ["supports", "contradicts"].contains(&edge.relation.as_str())
            && case.nodes.iter().any(|n| n.id == edge.source)
        {
            case.edges.push(edge.clone());
        }
    }
    tx.execute(
        "UPDATE cases SET snapshot=?1 WHERE id=?2",
        params![
            serde_json::to_string(&case).map_err(|e| e.to_string())?,
            case.id
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(case)
}
