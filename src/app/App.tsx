import {useState,useEffect,useCallback} from 'react';
import {FolderOpen,Plus,ScanLine,GitBranch,TriangleAlert,Database,Sparkles,X,RefreshCw} from 'lucide-react';
import {backend} from '../lib/backend';
import type {Repository,Investigation,SavedCase,AiContext} from '../domain/evidence';
import {CaseRail} from '../components/CaseRail';
import {EvidenceGraph} from '../features/graph/EvidenceGraph';
import {Inspector} from '../features/source/Inspector';
import {IncidentDialog} from '../features/incident/IncidentDialog';
import {ContextDialog} from '../features/hypothesis/ContextDialog';
export function App() {
  const [repository,setRepository]=useState<Repository|null>(null),[caseFile,setCase]=useState<Investigation|null>(null),[saved,setSaved]=useState<SavedCase[]>([]),[selected,setSelected]=useState<string|null>(null);
  const [busy,setBusy]=useState(false),[error,setError]=useState(''),[incident,setIncident]=useState(false),[folderDialog,setFolderDialog]=useState(false),[path,setPath]=useState(''),[context,setContext]=useState<AiContext|null>(null);
  const refreshCases=useCallback(()=>backend.listCases().then(setSaved),[]);
  useEffect(()=>{refreshCases().catch(()=>{});},[refreshCases]);
  const perform=async(action:()=>Promise<void>)=>{setBusy(true);setError('');try{await action();}catch(e){setError(String(e instanceof Error?e.message:e));}finally{setBusy(false);}};
  const openPath=(p:string)=>perform(async()=>{const repo=await backend.openRepository(p);setRepository(repo);setCase(null);setSelected(null);setFolderDialog(false);setIncident(true);});
  const chooseFolder=()=>perform(async()=>{if(backend.desktop){const p=await backend.chooseFolder();if(p){const r=await backend.openRepository(p);setRepository(r);setCase(null);setSelected(null);setIncident(true);}}else{setFolderDialog(true);}});
  const applyCase=(c:Investigation)=>{setCase(c);setRepository(c.repository);setSelected(c.nodes.find(n=>n.kind==='source_line')?.id??c.nodes[0]?.id??null);};
  const analyze=(stack:string,logs:string,tests:string)=>perform(async()=>{if(!repository)throw Error('Open a repository first');const c=await backend.analyze(repository.token,stack,logs,tests);applyCase(c);setIncident(false);await refreshCases();});
  const load=(id:string)=>perform(async()=>applyCase(await backend.loadCase(id)));
  const inspect=()=>perform(async()=>{if(caseFile)setContext(await backend.context(caseFile.id));});
  const generate=(model:string)=>perform(async()=>{if(caseFile){applyCase(await backend.hypothesis(caseFile.id,model));setContext(null);await refreshCases();}});
  const evidence=caseFile?.nodes.find(n=>n.id===selected)??null;
  return <div className="app-shell" aria-busy={busy}>
    <header className="app-header"><div className="brand"><ScanLine size={28}/><strong>TRACE</strong><span>AI Bug Crime Scene</span></div><div className="case-title">{caseFile?`Case: ${caseFile.title}`:'Local software forensics'}</div><div className="header-actions"><button onClick={chooseFolder} disabled={busy}><FolderOpen size={16}/>Open repository…</button><button onClick={()=>setFolderDialog(true)} disabled={busy} aria-label="Enter repository path">Path…</button><button className="primary" onClick={()=>setIncident(true)} disabled={!repository||busy}><Plus size={17}/>New investigation</button></div></header>
    {error?<div className="error-banner" role="alert"><TriangleAlert size={18}/><span>{error}</span><button onClick={()=>setError('')} aria-label="Dismiss error"><X size={17}/></button></div>:null}
    {caseFile?.stale?<div className="stale-banner" role="status"><TriangleAlert size={16}/>STALE SNAPSHOT · Repository or source changed. Retained evidence is historical.<button onClick={()=>setIncident(true)}>Start fresh investigation</button></div>:null}
    <main className="workspace"><CaseRail repository={repository} caseFile={caseFile} saved={saved} onLoad={load} onSelect={setSelected} onNew={()=>setIncident(true)}/><EvidenceGraph caseFile={caseFile} selected={selected} onSelect={setSelected}/><Inspector evidence={evidence} caseFile={caseFile} onSelect={setSelected}/></main>
    <footer className="statusbar"><div><GitBranch size={15}/>{repository?.branch??'No repository'}<span className="divider"/>{caseFile?`${caseFile.nodes.length} evidence items · ${caseFile.edges.length} connections`:'Read-only repository inspection'}</div><div>{caseFile?<><button onClick={()=>load(caseFile.id)} disabled={busy}><RefreshCw size={13}/>Check snapshot</button><button onClick={inspect} disabled={busy||caseFile.stale}><Sparkles size={14}/>AI context</button><span className="divider"/><Database size={14}/>Snapshot {new Date(caseFile.createdAt*1000).toLocaleTimeString()}</>:<span>Source · History · Provenance</span>}<span className="divider"/><span className="unverified"><TriangleAlert size={14}/>Root cause unverified</span></div></footer>
    {caseFile?.warnings.length?<details className="warnings-drawer"><summary>{caseFile.warnings.length} investigation notices</summary>{caseFile.warnings.map((w,i)=><p key={i}>{w}</p>)}</details>:null}
    {busy?<div className="progress-bar" role="status" aria-label="Processing evidence"/>:null}
    {folderDialog?<div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="folder-title" className="modal folder-modal"><button className="close" aria-label="Close folder selector" onClick={()=>setFolderDialog(false)}><X size={20}/></button><FolderOpen className="accent" size={28}/><h2 id="folder-title">Select a repository</h2><p>Enter a local Git repository folder. TRACE validates the path and keeps repository inspection read-only.</p><form onSubmit={e=>{e.preventDefault();openPath(path);}}><label>Local Git repository path<input autoFocus aria-label="Repository path" value={path} onChange={e=>setPath(e.target.value)} placeholder="C:\\projects\\my-repository"/></label><div className="modal-footer"><small>Spaces and Windows paths supported</small><button className="primary" disabled={busy||!path.trim()}>Open repository</button></div></form></section></div>:null}
    {incident?<IncidentDialog onClose={()=>setIncident(false)} onAnalyze={analyze} busy={busy}/>:null}
    {context?<ContextDialog context={context} onClose={()=>setContext(null)} onGenerate={generate} busy={busy}/>:null}
  </div>;
}
