import {useState,useMemo,useEffect,useRef} from 'react';
import {Download,X,FileText,TriangleAlert} from 'lucide-react';
import {useI18n} from '../../i18n/I18n';
import type {Investigation} from '../../domain/evidence';
import {createReport,reportSections,downloadReport} from './report';
export function ReportDialog({caseFile,onClose}:{caseFile:Investigation;onClose:()=>void}){
 const {t,language}=useI18n();const [includeExcerpts,setIncludeExcerpts]=useState(false);const [notice,setNotice]=useState('');
 const panel=useRef<HTMLElement>(null);const closeRef=useRef(onClose);closeRef.current=onClose;
 useEffect(()=>{
  const previous=document.activeElement as HTMLElement|null;
  panel.current?.querySelector<HTMLButtonElement>('button')?.focus();
  const handle=(event:KeyboardEvent)=>{
   if(event.key==='Escape'){event.preventDefault();closeRef.current();}
   if(event.key!=='Tab')return;
   const controls=Array.from(panel.current?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),[tabindex="0"]')??[]);
   const first=controls[0],last=controls.at(-1);
   if(event.shiftKey&&document.activeElement===first){event.preventDefault();last?.focus();}
   else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first?.focus();}
  };
  document.addEventListener('keydown',handle);return()=>{document.removeEventListener('keydown',handle);if(previous?.isConnected)previous.focus();};
 },[]);
 const options=useMemo(()=>({language,includeExcerpts}),[language,includeExcerpts]);
 const sections=useMemo(()=>reportSections(caseFile,options),[caseFile,options]);
 const exportFile=(format:'html'|'md')=>{try{downloadReport(createReport(caseFile,options,format),caseFile.id,format);setNotice(t('Download started'));}catch(e){setNotice(String(e));}};
 return <div className="modal-backdrop"><section ref={panel} className="modal report-modal" role="dialog" aria-modal="true" aria-labelledby="report-title"><button className="close" onClick={onClose} aria-label={t('Close report')}><X size={20}/></button><FileText className="accent" size={28}/><h2 id="report-title">{t('Export investigation report')}</h2><p>{t('Review the report before sharing. Export runs locally and does not execute tests or send data to AI.')}</p>
 <label className="report-option"><input type="checkbox" checked={includeExcerpts} onChange={e=>setIncludeExcerpts(e.target.checked)}/>{t('Include original evidence excerpts')}</label>
 <div className="warning-box"><TriangleAlert size={18}/><span>{t(includeExcerpts?'Original excerpts may include source code, logs, model replies and local paths. Review sensitive content before sharing.':'The summary includes the case title, evidence IDs and retained results. Original excerpts are excluded by default. Review sensitive content before sharing.')}</span></div>
 <div className="report-preview" tabIndex={0} aria-label={t('Report preview')}>{sections.map((s,i)=><section key={i}><h3>{s.heading}</h3><pre>{s.text}</pre></section>)}</div>
 <div className="report-actions"><span role="status">{notice}</span><button onClick={()=>exportFile('md')}><Download size={15}/>{t('Download Markdown')}</button><button className="primary" onClick={()=>exportFile('html')}><Download size={15}/>{t('Download HTML')}</button></div></section></div>;
}
