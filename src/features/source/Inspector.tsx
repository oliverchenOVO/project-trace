import {useI18n} from '../../i18n/I18n';
import { useState, useEffect } from 'react';
import { FileCode2, ShieldCheck, TriangleAlert, Fingerprint } from 'lucide-react';
import type {Evidence,Investigation} from '../../domain/evidence';
import {VerificationEvidence} from '../verification/VerificationEvidence';
import { kindLabels,evidenceLabel } from '../../domain/evidence';
function Patch({patch}:{patch:string}) {
  const {t,locale,evidenceText}=useI18n();return <pre className="patch">{patch.split('\n').map((l,i)=><div key={i} className={l.startsWith('+')?'added':l.startsWith('-')?'removed':l.startsWith('@@')?'hunk':''}>{t(l||' ')}</div>)}</pre>;}
export function Inspector({evidence,caseFile,onSelect,onVerify}:{evidence:Evidence|null;caseFile:Investigation|null;onSelect:(id:string)=>void;onVerify?:()=>void}) {
  const {t,locale,evidenceText}=useI18n();
  const [tab,setTab]=useState('evidence');useEffect(()=>setTab('evidence'),[evidence?.id]);
  if(!evidence)return <aside className="inspector"><div className="panel-caption">{t("CONTEXT INSPECTOR")}</div><div className="inspector-empty"><Fingerprint size={38} strokeWidth={1}/><h3>{t("Inspect the original.")}</h3><p>{t("Select any evidence node to view its source, Git output and provenance.")}</p><div className="principle">{t("TRACE never invents evidence.")}<br/>{t("TRACE investigates evidence.")}</div></div></aside>;
  const e=evidence,p=e.provenance;const d=e.data;
  const verificationRun=caseFile?.nodes.filter(n=>n.kind==='regression_boundary'&&n.validity==='snapshot'&&n.data.integrityValid===true&&(n.data.run as {firstBad?:string})?.firstBad===d.sha).at(-1);
  const lines=Array.isArray(d.lines)?d.lines as string[]:null;
  const response=d.response as {supportedBy:string[];contradictedBy:string[];uncertainties:string[];verificationSteps:string[]}|undefined;
  return <aside className={`inspector ${e.epistemicStatus}`}>
    <div className="panel-caption">{t(kindLabels[e.kind]??e.kind)}</div>
    <div className="inspector-title"><FileCode2 size={29}/><div><h2>{e.kind==='regression_boundary'&&e.validity!=='snapshot'?t('INVALIDATED BOUNDARY CLAIM'):evidenceText(e,e.title)}</h2><p>{p.filePath??e.createdBy}</p></div></div>
    <div className="inspector-tabs"><button className={tab==='evidence'?'active':''} onClick={()=>setTab('evidence')}>{t("Evidence")}</button><button className={tab==='provenance'?'active':''} onClick={()=>setTab('provenance')}>{t("Provenance")}</button></div>
    <div className="inspector-body">
      <div className={`status-strip ${e.epistemicStatus}`}><ShieldCheck size={14}/>{t(evidenceLabel(e))}<span>{t(e.validity==='snapshot'?'SNAPSHOT':e.validity.toUpperCase())}</span></div>
      {e.validity!=='snapshot'?<div className="warning-box">{t("This source is ")}{t(e.validity)}{t(". Retained excerpts describe the captured snapshot, not current repository content.")}</div>:null}
      {tab==='provenance'?<><h3>{t("ORIGINAL SOURCE")}</h3><dl><dt>{t("Evidence ID")}</dt><dd className="mono">{e.id}</dd><dt>{t("Source type")}</dt><dd>{t(p.sourceType)}</dd><dt>{t("Captured")}</dt><dd>{new Date(p.capturedAt*1000).toLocaleString(locale)}</dd><dt>{t("File / lines")}</dt><dd>{p.filePath??'—'} {t(p.lineStart?`· ${p.lineStart}–${p.lineEnd}`:'')}</dd><dt>{t("Commit SHA")}</dt><dd className="mono">{p.commitSha??'—'}</dd><dt>{t("Artifact ID")}</dt><dd className="mono">{p.artifactId??'—'}</dd><dt>{t("SHA-256")}</dt><dd className="mono">{p.sourceHash??'—'}</dd></dl><h3>{t("RECORDED COMMAND ARGUMENTS")}</h3><pre>{p.command.length?JSON.stringify(p.command,null,2):t('No command executed; input or source capture.')}</pre><h3>{t("RAW EXCERPT")}</h3><pre>{p.rawExcerpt}</pre></>:
      <>
        {['verification_run','test_execution','bisect_run','regression_boundary'].includes(e.kind)?<VerificationEvidence evidence={e} caseFile={caseFile} onSelect={onSelect}/>:null}
        {['hypothesis','commit','candidate_signal'].includes(e.kind)&&onVerify?<button className="primary verify-inspector" onClick={onVerify}>{t("Verify hypothesis")}</button>:null}
        {verificationRun?<div className="confirmed-suspect"><h3>{t("PRIMARY SUSPECT · BISECT CONFIRMED")}</h3><p>{t("The selected test first fails at this commit.")}</p><button onClick={()=>onSelect(verificationRun.id)}>{t("View verification")}</button><small>{t("Verification scope: selected test predicate")}</small></div>:null}
        {lines?<><h3>{t("SOURCE EXCERPT ")}<span>{t("UTF-8 · captured snapshot")}</span></h3><div className="source-code">{lines.map((line,i)=>{const num=Number(d.start)+i;return <div key={num} className={num===Number(d.line)?'crash-line':''}><span>{num}</span><code>{line||' '}</code></div>;})}</div><p className="crash-label">{t("↳ STACK LOCATION · line ")}{String(d.line)}{t(", column ")}{t(String(d.column??'unknown'))}</p></>:null}
        {typeof d.patch==='string'?<><h3>{t("ACTUAL GIT PATCH")}</h3><Patch patch={d.patch}/></>:null}
        {e.kind==='blame_result'?<><h3>{t("LINE ATTRIBUTION")}</h3><dl><dt>{t("Last changed by")}</dt><dd className="mono">{String(d.sha)}</dd><dt>{t("Author")}</dt><dd>{String(d.author)}</dd><dt>{t("Author timestamp")}</dt><dd>{String(d.timestamp)}{t(" (Unix seconds)")}</dd></dl><pre>{String(d.source)}</pre></>:null}
        {e.kind==='commit'?<><h3>{t("COMMIT METADATA")}</h3><dl><dt>{t("SHA")}</dt><dd className="mono">{String(d.sha)}</dd><dt>{t("Author")}</dt><dd>{String(d.author)}</dd><dt>{t("Date")}</dt><dd>{String(d.timestamp)}</dd></dl></>:null}
        {e.kind==='file_history'&&Array.isArray(d)?d.map((c:{sha:string;subject:string;author:string},i:number)=><div key={i} className="history-row"><b>{c.subject}</b><small>{c.sha.slice(0,12)} · {c.author}</small></div>):null}
        {Array.isArray(d.signals)?<><h3>{t("EXPLICIT HEURISTIC SIGNALS")}</h3>{(d.signals as string[]).map(s=><p key={s} className="signal">↗ {t(s)}</p>)}<p>{t("No signal establishes causation.")}</p></>:null}
        {response?<><h3>{t("CONFIDENCE · ")}{t(String(d.confidence))}</h3><p>{String(d.explanation)}</p><h3>{t("SUPPORTING EVIDENCE")}</h3>{response.supportedBy.map(id=><button className="evidence-reference" key={id} onClick={()=>onSelect(id)}>{(()=>{const n=caseFile?.nodes.find(n=>n.id===id);return n?evidenceText(n,n.title):id;})()}</button>)}<h3>{t("CONTRADICTIONS · ")}{response.contradictedBy.length}</h3>{response.contradictedBy.map(id=><button key={id} onClick={()=>onSelect(id)}>{(()=>{const n=caseFile?.nodes.find(n=>n.id===id);return n?evidenceText(n,n.title):id;})()}</button>)}<h3>{t("UNCERTAINTIES")}</h3><ul>{response.uncertainties.map(s=><li key={s}>{s}</li>)}</ul><h3>{t("VERIFICATION STEPS")}</h3><ol>{response.verificationSteps.map(s=><li key={s}>{s}</li>)}</ol></>:null}
        {!lines&&!d.patch&&!response?<><h3>{t("ORIGINAL EVIDENCE")}</h3><pre>{p.rawExcerpt}</pre></>:null}
        {e.kind==='stack_frame'?<><h3>{t("PARSED STACK · ")}{caseFile?.frames.length}{t(" FRAMES")}</h3>{caseFile?.frames.map(f=>{const target=caseFile.nodes.find(n=>n.kind==='stack_frame'&&n.data.index===f.index);return <button key={f.index} className="evidence-reference" disabled={!target} onClick={()=>target&&onSelect(target.id)}><span>#{f.index} · {t(f.resolved?'SOURCE RESOLVED':'UNRESOLVED')}</span><code>{f.raw}</code></button>;})}</>:null}
        <p>{['incident','hypothesis'].includes(e.kind)?t(e.summary):evidenceText(e,e.summary)}</p>
        {['source_line','blame_result','commit','candidate_signal','diff_hunk'].includes(e.kind)?<div className="warning-box"><TriangleAlert size={19}/><div><b>{t("Blame does not prove causation.")}</b><p>{t("Last modification is a lead. Reproduce the failure and verify the changed behavior before naming a root cause.")}</p></div></div>:null}
        <h3>{t("CHAIN OF EVIDENCE")}</h3>{caseFile?.edges.filter(edge=>edge.source===e.id||edge.target===e.id).map(edge=>{const other=edge.source===e.id?edge.target:edge.source;const n=caseFile.nodes.find(n=>n.id===other);return <button key={edge.id} className="evidence-reference" onClick={()=>onSelect(other)}><span>{t(edge.relation.replaceAll('_',' '))}</span>{n?evidenceText(n,n.title):''}</button>;})}
      </>}
    </div>
  </aside>;
}
