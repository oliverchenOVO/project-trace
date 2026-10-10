import {useState,useMemo} from 'react';
import {Download,X,FileText,TriangleAlert} from 'lucide-react';
import {useI18n} from '../../i18n/I18n';
import {useDialogFocus} from '../../components/useDialogFocus';
import type {Investigation,SavedCase} from '../../domain/evidence';
import {createReport,reportSections,downloadReport,evidenceDiagram} from './report';
export function ReportDialog({caseFile,details,onClose}:{caseFile:Investigation;details?:SavedCase;onClose:()=>void}){
 const {t,language}=useI18n();const [includeExcerpts,setIncludeExcerpts]=useState(false);const [notice,setNotice]=useState('');
 const panel=useDialogFocus(onClose);
 const options=useMemo(()=>({language,includeExcerpts,details}),[language,includeExcerpts,details]);
 const sections=useMemo(()=>reportSections(caseFile,options),[caseFile,options]);
 const diagram=useMemo(()=>`data:image/svg+xml,${encodeURIComponent(evidenceDiagram(caseFile,language))}`,[caseFile,language]);
 const exportFile=(format:'html'|'md')=>{try{downloadReport(createReport(caseFile,options,format),caseFile.id,format);setNotice(t('Download started'));}catch(e){setNotice(String(e));}};
 return <div className="modal-backdrop"><section ref={panel} className="modal report-modal" role="dialog" aria-modal="true" aria-labelledby="report-title"><button className="close" onClick={onClose} aria-label={t('Close report')}><X size={20}/></button><FileText className="accent" size={28}/><h2 id="report-title">{t('Export investigation report')}</h2><p>{t('Review the report before sharing. Export runs locally and does not execute tests or send data to AI.')}</p>
 <label className="report-option"><input type="checkbox" checked={includeExcerpts} onChange={e=>setIncludeExcerpts(e.target.checked)}/>{t('Include original evidence excerpts')}</label>
 <div className="warning-box"><TriangleAlert size={18}/><span>{t(includeExcerpts?'Original excerpts may include source code, logs, model replies and local paths. Review sensitive content before sharing.':'The summary includes the case title, evidence IDs and retained results. Original excerpts are excluded by default. Review sensitive content before sharing.')}</span></div>
 <div className="report-preview" tabIndex={0} aria-label={t('Report preview')}>{sections.map((s,i)=><section key={i}>{s.heading===t('Evidence inventory')?<figure className="report-diagram"><img src={diagram} alt={t('Evidence relationship diagram')}/><figcaption>{t('HTML includes an offline diagram and clickable evidence references. Markdown retains the full ID mapping.')}</figcaption></figure>:null}<h3>{s.heading}</h3><pre>{s.text}</pre></section>)}</div>
 <div className="report-actions"><span role="status">{notice}</span><button onClick={()=>exportFile('md')}><Download size={15}/>{t('Download Markdown')}</button><button className="primary" onClick={()=>exportFile('html')}><Download size={15}/>{t('Download HTML')}</button></div></section></div>;
}
