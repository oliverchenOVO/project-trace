use rusqlite::Connection;
use serde_json::json;
use tempfile::TempDir;
use trace_core::{model::Investigation,store};
fn legacy_db() -> (TempDir,std::path::PathBuf,String) {
 let dir=TempDir::new().unwrap();let path=dir.path().join("legacy.sqlite");
 let raw=json!({"id":"legacy-case","title":"Original incident","createdAt":1,"repository":{"token":"token","root":"C:/fixture","head":null,"branch":"main","dirty":false,"status":"","recentCommits":[]},"stack":"ORIGINAL STACK","logs":"log","tests":"output","frames":[],"nodes":[],"edges":[],"warnings":[],"sourceHashes":{},"stale":false}).to_string();
 let c=Connection::open(&path).unwrap();c.execute_batch("CREATE TABLE cases(id TEXT PRIMARY KEY,title TEXT NOT NULL,created INTEGER NOT NULL,snapshot TEXT NOT NULL);").unwrap();c.execute("INSERT INTO cases VALUES('legacy-case','Original incident',1,?1)",[&raw]).unwrap();drop(c);(dir,path,raw)
}
#[test]
fn upgrades_legacy_database_and_keeps_evidence_bytes_unchanged() {
 let (_dir,path,raw)=legacy_db();let initial=store::list(&path).unwrap();assert_eq!(initial[0]["title"],"Original incident");assert_eq!(initial[0]["archived"],false);
 store::update_metadata(&path,"legacy-case","  Portfolio demo  ",vec![" Rust ".into(),"Rust".into(),"".into(),"鑑識".into()],true).unwrap();
 store::touch(&path,"legacy-case").unwrap();let saved=store::list(&path).unwrap();assert_eq!(saved[0]["title"],"Portfolio demo");assert_eq!(saved[0]["tags"],json!(["Rust","鑑識"]));assert_eq!(saved[0]["archived"],true);assert!(saved[0]["lastOpened"].as_u64().unwrap()>0);
 let c=Connection::open(&path).unwrap();let after:String=c.query_row("SELECT snapshot FROM cases WHERE id='legacy-case'",[],|r|r.get(0)).unwrap();assert_eq!(after,raw);
 let mut case:Investigation=serde_json::from_str(&raw).unwrap();case.warnings.push("worker update".into());store::save(&path,&case).unwrap();assert_eq!(store::list(&path).unwrap()[0]["title"],"Portfolio demo");assert_eq!(store::load(&path,"legacy-case").unwrap().stack,"ORIGINAL STACK");
 store::update_metadata(&path,"legacy-case","",vec![],false).unwrap();assert_eq!(store::list(&path).unwrap()[0]["title"],"Original incident");assert_eq!(store::list(&path).unwrap()[0]["archived"],false);
}
#[test]
fn invalid_metadata_is_atomic_and_does_not_create_orphan_cases() {
 let (_dir,path,_)=legacy_db();store::update_metadata(&path,"legacy-case","Valid",vec![],false).unwrap();
 for result in [store::update_metadata(&path,"legacy-case",&"長".repeat(121),vec![],true),store::update_metadata(&path,"legacy-case","line\nbreak",vec![],true),store::update_metadata(&path,"legacy-case","Valid",vec!["x".repeat(25)],true),store::update_metadata(&path,"legacy-case","Valid",(0..9).map(|i|i.to_string()).collect(),true),store::update_metadata(&path,"missing","Name",vec![],false)] {assert!(result.is_err());}
 let list=store::list(&path).unwrap();assert_eq!(list.as_array().unwrap().len(),1);assert_eq!(list[0]["title"],"Valid");assert_eq!(list[0]["archived"],false);
}
#[test]
fn listing_keeps_cases_beyond_fifty_and_orders_recently_opened_first() {
 let (_dir,path,raw)=legacy_db();for i in 0..55 {let mut case:Investigation=serde_json::from_str(&raw).unwrap();case.id=format!("case-{i}");case.created_at=i+2;store::save(&path,&case).unwrap();}
 assert_eq!(store::list(&path).unwrap().as_array().unwrap().len(),56);store::touch(&path,"legacy-case").unwrap();assert_eq!(store::list(&path).unwrap()[0]["id"],"legacy-case");
}
