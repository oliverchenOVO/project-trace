import {useI18n} from '../../i18n/I18n';
import {useState} from 'react';
import {X,Sparkles} from 'lucide-react';
import type {AiContext} from '../../domain/evidence';
export function ContextDialog({context,onClose,onGenerate,busy}:{context:AiContext;onClose:()=>void;onGenerate:(model:string)=>void;busy:boolean}) {
  const {t,evidenceText}=useI18n();
  const [model,setModel]=useState('');
  return <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="ai-title" className="modal ai-modal"><button className="close" aria-label={t("Close AI context")} onClick={onClose} disabled={busy}><X size={20}/></button><Sparkles className="violet" size={28}/><h2 id="ai-title">{t("Review AI context")}</h2><p>{t("Only this evidence package will be sent to your local Ollama service. The response stays a hypothesis and every cited ID must exist.")}</p>
    <div className="context-stats"><span><b>{context.evidence.length}</b>{t(" evidence items")}</span><span><b>{context.fileCount}</b>{t(" files")}</span><span><b>{context.lineCount}</b>{t(" lines")}</span><span><b>{t((context.bytes/1024).toFixed(1))}</b>{t(" KB")}</span></div><small>{context.excludedCount}{t(" evidence items excluded · common secrets redacted")}</small>
    <div className="context-preview">{context.evidence.map(e=><details key={e.id}><summary>{evidenceText(e,e.title)}<span>{t(e.status)}</span></summary><code>{e.id}</code><pre>{e.excerpt}</pre>{e.excerptTruncated?<small>{t("Excerpt truncated to context limit.")}</small>:null}</details>)}</div>
    <label>{t("Installed Ollama model")}<input aria-label={t("Ollama model")} value={model} onChange={e=>setModel(e.target.value)} placeholder={t("Enter an installed model name")}/></label>
    <div className="modal-footer"><small>{t("127.0.0.1:11434 · model cannot execute tools")}</small><button className="primary violet-button" disabled={busy||!model.trim()} onClick={()=>onGenerate(model)}>{t(busy?'Investigating evidence…':'Generate hypothesis')}</button></div>
  </section></div>;
}
