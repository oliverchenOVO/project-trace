import {useState} from 'react';
import {FolderPen,X} from 'lucide-react';
import {useI18n} from '../i18n/I18n';
import {useDialogFocus} from './useDialogFocus';
import type {Investigation,SavedCase} from '../domain/evidence';
export function CaseManager({caseFile,details,busy,error,onSave,onClose}:{caseFile:Investigation;details?:SavedCase;busy:boolean;error:string;onSave:(name:string,tags:string[],archived:boolean)=>void;onClose:()=>void}){
 const {t}=useI18n();const [name,setName]=useState(details?.title!==caseFile.title?details?.title??'':''),[tags,setTags]=useState(details?.tags?.join(', ')??''),[archived,setArchived]=useState(details?.archived??false);const panel=useDialogFocus(onClose,busy);
 return <div className="modal-backdrop"><section ref={panel} className="modal folder-modal" role="dialog" aria-modal="true" aria-labelledby="manage-title"><button className="close" onClick={onClose} disabled={busy} aria-label={t('Close case manager')}><X size={20}/></button><FolderPen className="accent" size={28}/><h2 id="manage-title">{t('Manage case')}</h2><p>{t('Names, tags and archive state are saved separately. Original evidence and experiment records remain intact.')}</p><form onSubmit={e=>{e.preventDefault();onSave(name,tags.split(/[,，]/),archived);}}>
 <label>{t('Case name')}<input aria-label={t('Case name')} value={name} maxLength={120} placeholder={caseFile.title} onChange={e=>setName(e.target.value)} disabled={busy}/></label><small>{t('Leave blank to use the original incident title.')}</small>
 <label className="manage-tags">{t('Tags')}<input aria-label={t('Tags')} value={tags} onChange={e=>setTags(e.target.value)} disabled={busy} placeholder={t('Separate tags with commas')}/></label><small>{t('Up to 8 tags, 24 characters each.')}</small>
 <label className="report-option"><input type="checkbox" checked={archived} onChange={e=>setArchived(e.target.checked)} disabled={busy}/>{t('Archive this case')}</label><small>{t('Archived cases remain available in the Archived list and can be restored here.')}</small>
 {error?<p className="manager-error" role="alert">{t(error)}</p>:null}<div className="modal-footer"><span className="mono muted">{caseFile.id}</span><button className="primary" disabled={busy}>{t('Save case details')}</button></div></form></section></div>;
}
