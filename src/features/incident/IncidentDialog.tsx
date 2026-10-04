import {useI18n} from '../../i18n/I18n';
import {useState} from 'react';
import {X,ScanLine} from 'lucide-react';
export function IncidentDialog({onClose,onAnalyze,busy}:{onClose:()=>void;onAnalyze:(stack:string,logs:string,tests:string)=>void;busy:boolean}) {
  const {t}=useI18n();
  const [stack,setStack]=useState(''),[logs,setLogs]=useState(''),[tests,setTests]=useState(''),[tab,setTab]=useState('stack');
  return <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="incident-title" className="modal incident-modal">
    <button className="close" onClick={onClose} aria-label={t("Close investigation")} disabled={busy}><X size={20}/></button>
    <ScanLine size={28} className="accent"/><h2 id="incident-title">{t("Open an investigation")}</h2><p>{t("Start with the incident. TRACE resolves real source locations and gathers read-only Git evidence.")}</p>
    <div className="input-tabs">{['stack','logs','tests'].map(inputTab=><button key={inputTab} className={tab===inputTab?'active':''} onClick={()=>setTab(inputTab)}>{t(inputTab==='stack'?'Stack trace *':inputTab==='logs'?'Logs · optional':'Test output · optional')}</button>)}</div>
    {tab==='stack'?<label>{t("JavaScript / TypeScript / Node.js stack trace")}<textarea autoFocus aria-label={t("Stack trace")} value={stack} onChange={e=>setStack(e.target.value)} placeholder={t('Paste the original exception and stack frames here.\n\nExpected format:\n    at functionName (src/file.ts:87:19)')}/></label>:null}
    {tab==='logs'?<label>{t("Original log excerpts")}<textarea aria-label={t("Logs")} value={logs} onChange={e=>setLogs(e.target.value)} placeholder={t("Paste relevant logs. Original text is preserved as an input artifact.")}/></label>:null}
    {tab==='tests'?<label>{t("Original failing-test output")}<textarea aria-label={t("Test output")} value={tests} onChange={e=>setTests(e.target.value)} placeholder={t("Paste the command and its output. Pasted output is recorded as user-supplied evidence, not a TRACE-executed test.")}/></label>:null}
    <div className="modal-footer"><small>{t("Local inspection · no shell access · no automatic AI request")}</small><button className="primary" disabled={busy||!stack.trim()} onClick={()=>onAnalyze(stack,logs,tests)}>{t(busy?'Collecting evidence…':'Build evidence graph')}</button></div>
  </section></div>;
}
