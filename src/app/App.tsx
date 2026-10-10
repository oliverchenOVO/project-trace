import {useI18n} from '../i18n/I18n';
import {useState,useEffect,useCallback,lazy,Suspense} from 'react';
import {FolderOpen,Plus,ScanLine,GitBranch,TriangleAlert,Database,Sparkles,X,RefreshCw,FileText} from 'lucide-react';
import {backend} from '../lib/backend';
import type {Repository,Investigation,SavedCase,AiContext} from '../domain/evidence';
import {CaseRail} from '../components/CaseRail';
import {EvidenceGraph} from '../features/graph/EvidenceGraph';
import {Inspector} from '../features/source/Inspector';
import {IncidentDialog} from '../features/incident/IncidentDialog';
import {VerificationLab} from '../features/verification/VerificationLab';
import {ContextDialog} from '../features/hypothesis/ContextDialog';
const CaseManager=lazy(()=>import('../components/CaseManager').then(m=>({default:m.CaseManager})));
const ReportDialog=lazy(()=>import('../features/report/ReportDialog').then(m=>({default:m.ReportDialog})));
export function App() {
  const {t,locale,language,setLanguage}=useI18n();
  const [repository,setRepository]=useState<Repository|null>(null),[caseFile,setCase]=useState<Investigation|null>(null),[saved,setSaved]=useState<SavedCase[]>([]),[selected,setSelected]=useState<string|null>(null);
  const [verificationOpen,setVerificationOpen]=useState(false);
  const [reportOpen,setReportOpen]=useState(false);const [managerOpen,setManagerOpen]=useState(false);
  const [busy,setBusy]=useState(false),[error,setError]=useState(''),[incident,setIncident]=useState(false),[folderDialog,setFolderDialog]=useState(false),[path,setPath]=useState(''),[context,setContext]=useState<AiContext|null>(null);
  const refreshCases=useCallback(()=>backend.listCases().then(setSaved),[]);
  useEffect(()=>{refreshCases().catch(()=>{});},[refreshCases]);
  const perform=async(action:()=>Promise<void>)=>{setBusy(true);setError('');try{await action();}catch(e){setError(String(e instanceof Error?e.message:e));}finally{setBusy(false);}};
  const openPath=(p:string)=>perform(async()=>{const repo=await backend.openRepository(p);setRepository(repo);setCase(null);setSelected(null);setFolderDialog(false);setIncident(true);});
  const chooseFolder=()=>perform(async()=>{if(backend.desktop){const p=await backend.chooseFolder(t('Select a local Git repository'));if(p){const r=await backend.openRepository(p);setRepository(r);setCase(null);setSelected(null);setIncident(true);}}else{setFolderDialog(true);}});
  const applyCase=(c:Investigation)=>{setCase(c);setRepository(c.repository);setSelected(c.nodes.find(n=>n.kind==='source_line')?.id??c.nodes[0]?.id??null);};
  const analyze=(stack:string,logs:string,tests:string)=>perform(async()=>{if(!repository)throw Error('Open a repository first');const c=await backend.analyze(repository.token,stack,logs,tests);applyCase(c);setIncident(false);await refreshCases();});
  const load=(id:string)=>perform(async()=>{applyCase(await backend.loadCase(id));await refreshCases();});
  const caseDetails=saved.find(c=>c.id===caseFile?.id);
  const saveDetails=(name:string,tags:string[],archived:boolean)=>perform(async()=>{if(!caseFile)return;await backend.updateCaseMetadata(caseFile.id,name,tags,archived);await refreshCases();setManagerOpen(false);});
  const inspect=()=>perform(async()=>{if(caseFile)setContext(await backend.context(caseFile.id));});
  const generate=(model:string)=>perform(async()=>{if(caseFile){applyCase(await backend.hypothesis(caseFile.id,model));setContext(null);await refreshCases();}});
  const evidence=caseFile?.nodes.find(n=>n.id===selected)??null;
  return <div className="app-shell" aria-busy={busy}>
    <header className="app-header"><div className="brand"><ScanLine size={28}/><strong>{t("TRACE")}</strong><span>{t("AI Bug Crime Scene")}</span></div><div className="case-title">{t(caseFile?`Case: ${caseDetails?.title??caseFile.title}`:'Local software forensics')}</div><div className="header-actions"><select className="language-select" aria-label="Interface language / 介面語言" value={language} onChange={e=>setLanguage(e.target.value==='zh-TW'?'zh-TW':'en')}><option value="zh-TW">繁體中文</option><option value="en">English</option></select><button onClick={chooseFolder} disabled={busy}><FolderOpen size={16}/>{t("Open repository…")}</button><button onClick={()=>setFolderDialog(true)} disabled={busy} aria-label={t("Enter repository path")}>{t("Path…")}</button><button className="primary" onClick={()=>setIncident(true)} disabled={!repository||busy}><Plus size={17}/>{t("New investigation")}</button></div></header>
    {error?<div className="error-banner" role="alert"><TriangleAlert size={18}/><span>{t(error)}</span><button onClick={()=>setError('')} aria-label={t("Dismiss error")}><X size={17}/></button></div>:null}
    {caseFile?.stale?<div className="stale-banner" role="status"><TriangleAlert size={16}/>{t("STALE SNAPSHOT · Repository or source changed. Retained evidence is historical.")}<button onClick={()=>setIncident(true)}>{t("Start fresh investigation")}</button></div>:null}
    <main className="workspace"><CaseRail repository={repository} caseFile={caseFile} saved={saved} selected={selected} busy={busy} onLoad={load} onSelect={setSelected} onNew={()=>setIncident(true)}/><EvidenceGraph key={caseFile?.id??'empty'} caseFile={caseFile} selected={selected} onSelect={setSelected}/><Inspector evidence={evidence} caseFile={caseFile} onSelect={setSelected} onVerify={()=>setVerificationOpen(true)}/></main>
    <footer className="statusbar"><div><GitBranch size={15}/>{repository?.branch??t('No repository')}<span className="divider"/>{t(caseFile?`${caseFile.nodes.length} evidence items · ${caseFile.edges.length} connections`:'Read-only repository inspection')}</div><div>{caseFile?<><button onClick={()=>load(caseFile.id)} disabled={busy}><RefreshCw size={13}/>{t("Check snapshot")}</button><button onClick={()=>{setError('');setManagerOpen(true);}} disabled={busy}>{t('Manage case')}</button><button onClick={()=>setVerificationOpen(true)} disabled={busy}>{t("Verify hypothesis")}</button><button onClick={()=>setReportOpen(true)} disabled={busy}><FileText size={14}/>{t("Export report")}</button><button onClick={inspect} disabled={busy||caseFile.stale}><Sparkles size={14}/>{t("AI context")}</button><span className="divider"/><Database size={14}/>{t("Snapshot ")}{new Date(caseFile.createdAt*1000).toLocaleTimeString(locale)}</>:<span>{t("Source · History · Provenance")}</span>}<span className="divider"/><span className="unverified"><TriangleAlert size={14}/>{t("Root cause unverified")}</span></div></footer>
    {caseFile?.warnings.length?<details className="warnings-drawer"><summary>{caseFile.warnings.length}{t(" investigation notices")}</summary>{caseFile.warnings.map((w,i)=><p key={i}>{t(w)}</p>)}</details>:null}
    {busy?<div className="progress-bar" role="status" aria-label={t("Processing evidence")}/>:null}
    {folderDialog?<div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="folder-title" className="modal folder-modal"><button className="close" aria-label={t("Close folder selector")} onClick={()=>setFolderDialog(false)}><X size={20}/></button><FolderOpen className="accent" size={28}/><h2 id="folder-title">{t("Select a repository")}</h2><p>{t("Enter a local Git repository folder. TRACE validates the path and keeps repository inspection read-only.")}</p><form onSubmit={e=>{e.preventDefault();openPath(path);}}><label>{t("Local Git repository path")}<input autoFocus aria-label={t("Repository path")} value={path} onChange={e=>setPath(e.target.value)} placeholder={t("C:\\\\projects\\\\my-repository")}/></label><div className="modal-footer"><small>{t("Spaces and Windows paths supported")}</small><button className="primary" disabled={busy||!path.trim()}>{t("Open repository")}</button></div></form></section></div>:null}
    {incident?<IncidentDialog onClose={()=>setIncident(false)} onAnalyze={analyze} busy={busy}/>:null}
    {verificationOpen&&caseFile?<VerificationLab caseFile={caseFile} onCase={c=>setCase(previous=>previous?.id===c.id?({...c,stale:c.stale||previous.stale}):previous)} onClose={()=>setVerificationOpen(false)}/>:null}
    <Suspense fallback={<div className="modal-backdrop" role="status">{t('Loading panel…')}</div>}>{reportOpen&&caseFile?<ReportDialog caseFile={caseFile} details={caseDetails} onClose={()=>setReportOpen(false)}/>:null}
    {managerOpen&&caseFile?<CaseManager key={caseFile.id} caseFile={caseFile} details={caseDetails} busy={busy} error={error} onSave={saveDetails} onClose={()=>setManagerOpen(false)}/>:null}</Suspense>
    {context?<ContextDialog context={context} onClose={()=>setContext(null)} onGenerate={generate} busy={busy}/>:null}
  </div>;
}
