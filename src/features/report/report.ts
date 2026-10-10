import type {Investigation} from '../../domain/evidence';
import {kindLabels,evidenceLabel} from '../../domain/evidence';
import {translate,translateEvidence,type Language} from '../../i18n/translate';
import {runLabels,boundaryWording,type VerificationRun,type Execution} from '../verification/types';
export interface ReportOptions {language:Language;includeExcerpts:boolean}
interface Section {heading:string;text:string}
export function reportSections(c:Investigation,options:ReportOptions):Section[] {
  const t=(s:string)=>translate(options.language,s);
  const sections:Section[]=[
    {heading:t('Case overview'),text:[c.title,`${t('Case ID')}: ${c.id}`,`${t('REPOSITORY')}: ${c.repository.root.split(/[\\/]/).pop()}`,`${t('HEAD')}: ${c.repository.head??'—'}`,`${t('Snapshot')}: ${new Date(c.createdAt*1000).toISOString()}`,`${t('Snapshot state')}: ${t(c.stale?'STALE':'SNAPSHOT')}`,t('Root cause unverified')].join('\n')},
    {heading:t('Report scope'),text:t('This report describes retained evidence, not a fresh execution. A bisect result applies only to the selected test and revision range.')},
  ];
  for(const e of c.nodes.filter(n=>n.kind==='verification_run')){
    const r=e.data.run as unknown as VerificationRun|undefined;
    if(!r)continue;
    const valid=!c.stale&&e.validity==='snapshot'&&e.data.integrityValid===true&&r.status==='bisect_confirmed';
    const lines=[`${t('Evidence ID')}: ${e.id}`,`${t('RESULT')}: ${t(runLabels[r.status]??r.status)}`,`${t('Evidence validity')}: ${t(e.validity.toUpperCase())}`,`${t('KNOWN GOOD')}: ${r.knownGood??'—'}`,`${t('KNOWN BAD')}: ${r.knownBad}`,`${t('FIRST FAILING REVISION')}: ${valid?r.firstBad??t('Not established'):t('No current verified boundary')}`,`${t('CLEANUP')}: ${t(r.cleanupStatus)}`,`${t('ACTIVE REPOSITORY')}: ${t(r.sourceUnchanged===true?'UNCHANGED':r.sourceUnchanged===false?'CHANGED DURING VERIFICATION':'Not yet checked')}`];
    if(valid&&r.firstBad)lines.push(t(boundaryWording(r.firstBad)));
    lines.push(`${t('SELECTED TEST')}: ${r.command.requested.executable} ${JSON.stringify(r.command.requested.args)}`);
    sections.push({heading:t('Retained experiment'),text:lines.join('\n')});
  }
  sections.push({heading:t('Evidence inventory'),text:c.nodes.map(e=>`${e.id} · ${t(kindLabels[e.kind]??e.kind)} · ${t(evidenceLabel(e))} · ${t(e.validity.toUpperCase())}`).join('\n')});
  sections.push({heading:t('Evidence connections'),text:c.edges.map(e=>`${e.source} → ${e.target} · ${t(e.relation.replaceAll('_',' '))}`).join('\n')});
  if(c.warnings.length)sections.push({heading:t('Investigation notices'),text:c.warnings.map(t).join('\n')});
  if(options.includeExcerpts){
    for(const e of c.nodes){
      // Native provenance for these nodes contains a serialized run, including
      // approval and environment internals. Export only explicit report fields.
      if(['verification_run','bisect_run','regression_boundary'].includes(e.kind))continue;
      if(e.kind==='test_execution'){
        const x=e.data.execution as unknown as Execution|undefined;if(!x)continue;
        sections.push({heading:`${t('TEST EXECUTION')} · ${e.id}`,text:[`${t('REVISION')}: ${x.revision}`,`${t('RESULT')}: ${t(x.status)}`,`${t('EXIT CODE')}: ${x.exitCode??'—'}`,`${t('STDOUT')}\n${x.stdout}`,`${t('STDERR')}\n${x.stderr}`].join('\n\n')});
      }else sections.push({heading:`${t(kindLabels[e.kind]??e.kind)} · ${e.id}`,text:[translateEvidence(options.language,e,e.title),e.summary,e.provenance.rawExcerpt].join('\n\n')});
    }
  }
  return sections;
}
export function escapeHtml(text:string){return text.replace(/[&<>"']/g,ch=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]!));}
function fence(text:string){const longest=Math.max(0,...(text.match(/`+/g)??[]).map(s=>s.length));const mark='`'.repeat(Math.max(3,longest+1));return `${mark}\n${text}\n${mark}`;}
export function createReport(c:Investigation,options:ReportOptions,format:'html'|'md'):string {
  const sections=reportSections(c,options);const title=translate(options.language,'Investigation report');
  if(format==='md')return `# TRACE — ${title}\n\n`+sections.map(s=>`## ${s.heading}\n\n${fence(s.text)}\n`).join('\n');
  return `<!doctype html><html lang="${options.language==='en'?'en':'zh-Hant'}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><title>TRACE — ${escapeHtml(title)}</title><style>body{margin:0;background:#f3f6f8;color:#203446;font:16px/1.65 system-ui,'Microsoft JhengHei',sans-serif}main{max-width:1000px;margin:32px auto;padding:32px;background:white;border-radius:12px}h1{font-size:30px}h2{font-size:20px;border-bottom:1px solid #d5dfe6;padding-bottom:8px}section{margin:28px 0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:14px/1.7 Consolas,monospace;background:#f4f7f9;padding:18px;border-radius:6px}@media print{body{background:white}main{margin:0;padding:0}section{break-inside:avoid}pre{background:white}}@media(max-width:600px){main{margin:0;padding:20px}}</style></head><body><main><h1>TRACE — ${escapeHtml(title)}</h1>${sections.map(s=>`<section><h2>${escapeHtml(s.heading)}</h2><pre>${escapeHtml(s.text)}</pre></section>`).join('')}</main></body></html>`;
}
export function downloadReport(content:string,id:string,format:'html'|'md'){
  const url=URL.createObjectURL(new Blob([content],{type:format==='html'?'text/html;charset=utf-8':'text/markdown;charset=utf-8'}));
  const a=document.createElement('a');a.href=url;a.download=`TRACE-${id.replace(/[^a-zA-Z0-9-]/g,'').slice(0,50)||'case'}.${format}`;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
