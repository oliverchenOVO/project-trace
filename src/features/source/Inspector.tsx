import { useState, useEffect } from 'react';
import { FileCode2, ShieldCheck, TriangleAlert, Fingerprint } from 'lucide-react';
import type {Evidence,Investigation} from '../../domain/evidence';
import { kindLabels,statusLabels } from '../../domain/evidence';
function Patch({patch}:{patch:string}) {return <pre className="patch">{patch.split('\n').map((l,i)=><div key={i} className={l.startsWith('+')?'added':l.startsWith('-')?'removed':l.startsWith('@@')?'hunk':''}>{l||' '}</div>)}</pre>;}
export function Inspector({evidence,caseFile,onSelect}:{evidence:Evidence|null;caseFile:Investigation|null;onSelect:(id:string)=>void}) {
  const [tab,setTab]=useState('evidence');useEffect(()=>setTab('evidence'),[evidence?.id]);
  if(!evidence)return <aside className="inspector"><div className="panel-caption">CONTEXT INSPECTOR</div><div className="inspector-empty"><Fingerprint size={38} strokeWidth={1}/><h3>Inspect the original.</h3><p>Select any evidence node to view its source, Git output and provenance.</p><div className="principle">TRACE never invents evidence.<br/>TRACE investigates evidence.</div></div></aside>;
  const e=evidence,p=e.provenance;const d=e.data;
  const lines=Array.isArray(d.lines)?d.lines as string[]:null;
  const response=d.response as {supportedBy:string[];contradictedBy:string[];uncertainties:string[];verificationSteps:string[]}|undefined;
  return <aside className={`inspector ${e.epistemicStatus}`}>
    <div className="panel-caption">{kindLabels[e.kind]??e.kind}</div>
    <div className="inspector-title"><FileCode2 size={29}/><div><h2>{e.title}</h2><p>{p.filePath??e.createdBy}</p></div></div>
    <div className="inspector-tabs"><button className={tab==='evidence'?'active':''} onClick={()=>setTab('evidence')}>Evidence</button><button className={tab==='provenance'?'active':''} onClick={()=>setTab('provenance')}>Provenance</button></div>
    <div className="inspector-body">
      <div className={`status-strip ${e.epistemicStatus}`}><ShieldCheck size={14}/>{statusLabels[e.epistemicStatus]}<span>{e.validity==='snapshot'?'SNAPSHOT':e.validity.toUpperCase()}</span></div>
      {e.validity!=='snapshot'?<div className="warning-box">This source is {e.validity}. Retained excerpts describe the captured snapshot, not current repository content.</div>:null}
      {tab==='provenance'?<><h3>ORIGINAL SOURCE</h3><dl><dt>Evidence ID</dt><dd className="mono">{e.id}</dd><dt>Source type</dt><dd>{p.sourceType}</dd><dt>Captured</dt><dd>{new Date(p.capturedAt*1000).toLocaleString()}</dd><dt>File / lines</dt><dd>{p.filePath??'—'} {p.lineStart?`· ${p.lineStart}–${p.lineEnd}`:''}</dd><dt>Commit SHA</dt><dd className="mono">{p.commitSha??'—'}</dd><dt>Artifact ID</dt><dd className="mono">{p.artifactId??'—'}</dd><dt>SHA-256</dt><dd className="mono">{p.sourceHash??'—'}</dd></dl><h3>RECORDED COMMAND ARGUMENTS</h3><pre>{p.command.length?JSON.stringify(p.command,null,2):'No command executed; input or source capture.'}</pre><h3>RAW EXCERPT</h3><pre>{p.rawExcerpt}</pre></>:
      <>
        {lines?<><h3>SOURCE EXCERPT <span>UTF-8 · captured snapshot</span></h3><div className="source-code">{lines.map((line,i)=>{const num=Number(d.start)+i;return <div key={num} className={num===Number(d.line)?'crash-line':''}><span>{num}</span><code>{line||' '}</code></div>;})}</div><p className="crash-label">↳ STACK LOCATION · line {String(d.line)}, column {String(d.column??'unknown')}</p></>:null}
        {typeof d.patch==='string'?<><h3>ACTUAL GIT PATCH</h3><Patch patch={d.patch}/></>:null}
        {e.kind==='blame_result'?<><h3>LINE ATTRIBUTION</h3><dl><dt>Last changed by</dt><dd className="mono">{String(d.sha)}</dd><dt>Author</dt><dd>{String(d.author)}</dd><dt>Author timestamp</dt><dd>{String(d.timestamp)} (Unix seconds)</dd></dl><pre>{String(d.source)}</pre></>:null}
        {e.kind==='commit'?<><h3>COMMIT METADATA</h3><dl><dt>SHA</dt><dd className="mono">{String(d.sha)}</dd><dt>Author</dt><dd>{String(d.author)}</dd><dt>Date</dt><dd>{String(d.timestamp)}</dd></dl></>:null}
        {e.kind==='file_history'&&Array.isArray(d)?d.map((c:{sha:string;subject:string;author:string},i:number)=><div key={i} className="history-row"><b>{c.subject}</b><small>{c.sha.slice(0,12)} · {c.author}</small></div>):null}
        {Array.isArray(d.signals)?<><h3>EXPLICIT HEURISTIC SIGNALS</h3>{(d.signals as string[]).map(s=><p key={s} className="signal">↗ {s}</p>)}<p>No signal establishes causation.</p></>:null}
        {response?<><h3>CONFIDENCE · {String(d.confidence)}</h3><p>{String(d.explanation)}</p><h3>SUPPORTING EVIDENCE</h3>{response.supportedBy.map(id=><button className="evidence-reference" key={id} onClick={()=>onSelect(id)}>{caseFile?.nodes.find(n=>n.id===id)?.title??id}</button>)}<h3>CONTRADICTIONS · {response.contradictedBy.length}</h3>{response.contradictedBy.map(id=><button key={id} onClick={()=>onSelect(id)}>{caseFile?.nodes.find(n=>n.id===id)?.title??id}</button>)}<h3>UNCERTAINTIES</h3><ul>{response.uncertainties.map(s=><li key={s}>{s}</li>)}</ul><h3>VERIFICATION STEPS</h3><ol>{response.verificationSteps.map(s=><li key={s}>{s}</li>)}</ol></>:null}
        {!lines&&!d.patch&&!response?<><h3>ORIGINAL EVIDENCE</h3><pre>{p.rawExcerpt}</pre></>:null}
        <p>{e.summary}</p>
        {['source_line','blame_result','commit','candidate_signal','diff_hunk'].includes(e.kind)?<div className="warning-box"><TriangleAlert size={19}/><div><b>Blame does not prove causation.</b><p>Last modification is a lead. Reproduce the failure and verify the changed behavior before naming a root cause.</p></div></div>:null}
        <h3>CHAIN OF EVIDENCE</h3>{caseFile?.edges.filter(edge=>edge.source===e.id||edge.target===e.id).map(edge=>{const other=edge.source===e.id?edge.target:edge.source;const n=caseFile.nodes.find(n=>n.id===other);return <button key={edge.id} className="evidence-reference" onClick={()=>onSelect(other)}><span>{edge.relation.replaceAll('_',' ')}</span>{n?.title}</button>;})}
      </>}
    </div>
  </aside>;
}
