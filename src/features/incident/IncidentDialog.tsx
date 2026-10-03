import {useState} from 'react';
import {X,ScanLine} from 'lucide-react';
export function IncidentDialog({onClose,onAnalyze,busy}:{onClose:()=>void;onAnalyze:(stack:string,logs:string,tests:string)=>void;busy:boolean}) {
  const [stack,setStack]=useState(''),[logs,setLogs]=useState(''),[tests,setTests]=useState(''),[tab,setTab]=useState('stack');
  return <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="incident-title" className="modal incident-modal">
    <button className="close" onClick={onClose} aria-label="Close investigation" disabled={busy}><X size={20}/></button>
    <ScanLine size={28} className="accent"/><h2 id="incident-title">Open an investigation</h2><p>Start with the incident. TRACE resolves real source locations and gathers read-only Git evidence.</p>
    <div className="input-tabs">{['stack','logs','tests'].map(t=><button key={t} className={tab===t?'active':''} onClick={()=>setTab(t)}>{t==='stack'?'Stack trace *':t==='logs'?'Logs · optional':'Test output · optional'}</button>)}</div>
    {tab==='stack'?<label>JavaScript / TypeScript / Node.js stack trace<textarea autoFocus aria-label="Stack trace" value={stack} onChange={e=>setStack(e.target.value)} placeholder={'Paste the original exception and stack frames here.\n\nExpected format:\n    at functionName (src/file.ts:87:19)'}/></label>:null}
    {tab==='logs'?<label>Original log excerpts<textarea aria-label="Logs" value={logs} onChange={e=>setLogs(e.target.value)} placeholder="Paste relevant logs. Original text is preserved as an input artifact."/></label>:null}
    {tab==='tests'?<label>Original failing-test output<textarea aria-label="Test output" value={tests} onChange={e=>setTests(e.target.value)} placeholder="Paste the command and its output. Pasted output is recorded as user-supplied evidence, not a TRACE-executed test."/></label>:null}
    <div className="modal-footer"><small>Local inspection · no shell access · no automatic AI request</small><button className="primary" disabled={busy||!stack.trim()} onClick={()=>onAnalyze(stack,logs,tests)}>{busy?'Collecting evidence…':'Build evidence graph'}</button></div>
  </section></div>;
}
