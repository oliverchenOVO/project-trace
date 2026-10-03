import {useState} from 'react';
import {X,Sparkles} from 'lucide-react';
import type {AiContext} from '../../domain/evidence';
export function ContextDialog({context,onClose,onGenerate,busy}:{context:AiContext;onClose:()=>void;onGenerate:(model:string)=>void;busy:boolean}) {
  const [model,setModel]=useState('');
  return <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="ai-title" className="modal ai-modal"><button className="close" aria-label="Close AI context" onClick={onClose} disabled={busy}><X size={20}/></button><Sparkles className="violet" size={28}/><h2 id="ai-title">Review AI context</h2><p>Only this evidence package will be sent to your local Ollama service. The response stays a hypothesis and every cited ID must exist.</p>
    <div className="context-stats"><span><b>{context.evidence.length}</b> evidence items</span><span><b>{context.fileCount}</b> files</span><span><b>{context.lineCount}</b> lines</span><span><b>{(context.bytes/1024).toFixed(1)}</b> KB</span></div><small>{context.excludedCount} evidence items excluded · common secrets redacted</small>
    <div className="context-preview">{context.evidence.map(e=><details key={e.id}><summary>{e.title}<span>{e.status}</span></summary><code>{e.id}</code><pre>{e.excerpt}</pre>{e.excerptTruncated?<small>Excerpt truncated to context limit.</small>:null}</details>)}</div>
    <label>Installed Ollama model<input aria-label="Ollama model" value={model} onChange={e=>setModel(e.target.value)} placeholder="Enter an installed model name"/></label>
    <div className="modal-footer"><small>127.0.0.1:11434 · model cannot execute tools</small><button className="primary violet-button" disabled={busy||!model.trim()} onClick={()=>onGenerate(model)}>{busy?'Investigating evidence…':'Generate hypothesis'}</button></div>
  </section></div>;
}
