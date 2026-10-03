use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, String>;
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn hash(bytes: &[u8]) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(bytes))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Observed,
    Derived,
    Correlated,
    Hypothesis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub source_type: String,
    pub file_path: Option<String>,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub commit_sha: Option<String>,
    pub command: Vec<String>,
    pub raw_excerpt: String,
    pub artifact_id: Option<String>,
    pub source_hash: Option<String>,
    pub captured_at: u64,
}
impl Provenance {
    pub fn input(kind: &str, raw: &str, artifact: &str) -> Self {
        Self {
            source_type: kind.into(),
            raw_excerpt: raw.into(),
            artifact_id: Some(artifact.into()),
            file_path: None,
            line_start: None,
            line_end: None,
            commit_sha: None,
            command: vec![],
            source_hash: Some(hash(raw.as_bytes())),
            captured_at: now(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNode {
    pub id: String,
    pub kind: String,
    pub epistemic_status: Status,
    pub title: String,
    pub summary: String,
    pub created_by: String,
    pub strength: String,
    pub provenance: Provenance,
    pub data: serde_json::Value,
    pub validity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: String,
    pub epistemic_status: Status,
    pub provenance: Provenance,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub sha: String,
    pub author: String,
    pub timestamp: String,
    pub subject: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub token: String,
    pub root: String,
    pub head: Option<String>,
    pub branch: String,
    pub dirty: bool,
    pub status: String,
    pub recent_commits: Vec<Commit>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub index: usize,
    pub raw: String,
    pub function_name: Option<String>,
    pub file_path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub language: String,
    pub resolved: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExcerpt {
    pub path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub start: usize,
    pub lines: Vec<String>,
    pub hash: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Investigation {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub repository: Repository,
    pub stack: String,
    pub logs: String,
    pub tests: String,
    pub frames: Vec<StackFrame>,
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
    pub warnings: Vec<String>,
    pub source_hashes: BTreeMap<String, String>,
    pub stale: bool,
}
impl Investigation {
    pub fn node(
        &mut self,
        kind: &str,
        status: Status,
        title: &str,
        summary: &str,
        creator: &str,
        provenance: Provenance,
        data: serde_json::Value,
    ) -> String {
        let n = id();
        self.nodes.push(EvidenceNode {
            id: n.clone(),
            kind: kind.into(),
            epistemic_status: status.clone(),
            title: title.into(),
            summary: summary.into(),
            created_by: creator.into(),
            strength: if status == Status::Observed {
                "DIRECT"
            } else if status == Status::Derived {
                "VERIFIED"
            } else {
                "INDIRECT"
            }
            .into(),
            provenance,
            data,
            validity: "snapshot".into(),
        });
        n
    }
    pub fn edge(
        &mut self,
        source: &str,
        target: &str,
        relation: &str,
        status: Status,
        p: Provenance,
    ) {
        self.edges.push(EvidenceEdge {
            id: id(),
            source: source.into(),
            target: target.into(),
            relation: relation.into(),
            epistemic_status: status,
            provenance: p,
        });
    }
}
