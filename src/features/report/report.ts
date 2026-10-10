import type {Investigation,SavedCase,Evidence} from '../../domain/evidence';
import {kindLabels,evidenceLabel,statusLabels} from '../../domain/evidence';
import {translate,translateEvidence,type Language} from '../../i18n/translate';
import {runLabels,boundaryWording,type VerificationRun,type Execution} from '../verification/types';
export interface ReportOptions {language:Language;includeExcerpts:boolean;details?:SavedCase}
interface Section {heading:string;text:string}
export function evidenceReference(c:Investigation,id:string){const index=c.nodes.findIndex(e=>e.id===id);return index<0?'?':`E${String(index+1).padStart(2,'0')}`;}
function evidenceTitle(c:Investigation,e:Evidence,language:Language){return e.kind==='regression_boundary'&&(c.stale||e.validity!=='snapshot')?translate(language,'Historical boundary record'):translateEvidence(language,e,e.title);}
export function reportSections(c:Investigation,options:ReportOptions):Section[] {
  const t=(s:string)=>translate(options.language,s);
  const sections:Section[]=[
    {heading:t('Case overview'),text:[options.details?.title??c.title,`${t('Case ID')}: ${c.id}`,`${t('Tags')}: ${options.details?.tags?.join(' · ')||'—'}`,`${t('REPOSITORY')}: ${c.repository.root.split(/[\\/]/).pop()}`,`${t('HEAD')}: ${c.repository.head??'—'}`,`${t('Snapshot')}: ${new Date(c.createdAt*1000).toISOString()}`,`${t('Snapshot state')}: ${t(c.stale?'STALE':'SNAPSHOT')}`,t('Root cause unverified')].join('\n')},
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
  sections.push({heading:t('Evidence inventory'),text:c.nodes.map(e=>`${evidenceReference(c,e.id)} · ${evidenceTitle(c,e,options.language)}\n${e.id.slice(0,8)} · ${t(kindLabels[e.kind]??e.kind)} · ${t(evidenceLabel(e))} · ${t(e.validity.toUpperCase())}`).join('\n\n')});
  sections.push({heading:t('Evidence connections'),text:c.edges.map(e=>`${evidenceReference(c,e.source)} → ${evidenceReference(c,e.target)} · ${t(e.relation.replaceAll('_',' '))}`).join('\n')});
  if(c.warnings.length)sections.push({heading:t('Investigation notices'),text:c.warnings.map(t).join('\n')});
  if(options.includeExcerpts){
    for(const e of c.nodes){
      // Native provenance for these nodes contains a serialized run, including
      // approval and environment internals. Export only explicit report fields.
      if(['verification_run','bisect_run','regression_boundary'].includes(e.kind))continue;
      if(e.kind==='test_execution'){
        const x=e.data.execution as unknown as Execution|undefined;if(!x)continue;
        sections.push({heading:`${t('TEST EXECUTION')} · ${e.id}`,text:[`${t('REVISION')}: ${x.revision}`,`${t('RESULT')}: ${t(x.status)}`,`${t('EXIT CODE')}: ${x.exitCode??'—'}`,`${t('STDOUT')}\n${x.stdout}`,`${t('STDERR')}\n${x.stderr}`].join('\n\n')});
      }else sections.push({heading:`${t(kindLabels[e.kind]??e.kind)} · ${evidenceReference(c,e.id)}`,text:[translateEvidence(options.language,e,e.title),e.summary,e.provenance.rawExcerpt].join('\n\n')});
    }
  }
  return sections;
}
export function escapeHtml(text:string){return text.replace(/[&<>"']/g,ch=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]!));}
export function evidenceDiagram(c:Investigation,language:Language):string {
 const nodes=c.nodes.slice(0,60),width=1000,height=Math.max(140,Math.ceil(nodes.length/3)*140+30),colors={observed:'#087d91',derived:'#087d91',correlated:'#997014',hypothesis:'#7557a3'};
 const point=(id:string)=>{const i=nodes.findIndex(e=>e.id===id);return i<0?null:{x:20+(i%3)*330,y:20+Math.floor(i/3)*140};};
 const edges=c.edges.map(e=>{const a=point(e.source),b=point(e.target);return a&&b?`<path d="M${a.x+145} ${a.y+100} L${b.x+145} ${b.y}" fill="none" stroke="${colors[e.epistemicStatus]}" stroke-width="2" ${['correlated','hypothesis'].includes(e.epistemicStatus)?'stroke-dasharray="5 4"':''} marker-end="url(#report-arrow)"/>`:'';}).join('');
 const cards=nodes.map(e=>{const p=point(e.id)!;const index=c.nodes.indexOf(e);const title=evidenceTitle(c,e,language);const color=e.validity!=='snapshot'?'#ae3345':colors[e.epistemicStatus];return `<a href="#evidence-${index+1}"><title>${escapeHtml(title)} · ${escapeHtml(e.id)}</title><rect x="${p.x}" y="${p.y}" width="290" height="100" rx="8" fill="#f8fbfd" stroke="${color}" stroke-width="2"/><text x="${p.x+12}" y="${p.y+24}" fill="${color}" font-size="13">${evidenceReference(c,e.id)} · ${escapeHtml(translate(language,kindLabels[e.kind]??e.kind))}</text><text x="${p.x+12}" y="${p.y+50}" fill="#203446" font-size="12">${escapeHtml(title.length>32?title.slice(0,32)+'…':title)}</text><text x="${p.x+12}" y="${p.y+78}" fill="#61758a" font-size="11">${escapeHtml(translate(language,e.validity.toUpperCase()))} · ${escapeHtml(translate(language,statusLabels[e.epistemicStatus]))}</text></a>`;}).join('');
 return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="${escapeHtml(translate(language,'Evidence relationship diagram'))}" style="width:100%;height:auto;font-family:system-ui,'Microsoft JhengHei',sans-serif"><defs><marker id="report-arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto"><path d="M0 0 L6 3 L0 6Z" fill="#61758a"/></marker></defs>${edges}${cards}</svg>`;
}
function fence(text:string){const longest=Math.max(0,...(text.match(/`+/g)??[]).map(s=>s.length));const mark='`'.repeat(Math.max(3,longest+1));return `${mark}\n${text}\n${mark}`;}
export function createReport(c:Investigation,options:ReportOptions,format:'html'|'md'):string {
  const sections=reportSections(c,options);const title=translate(options.language,'Investigation report');
  if(format==='md')return `# TRACE — ${title}\n\n`+sections.map(s=>`## ${s.heading}\n\n${fence(s.text)}\n`).join('\n')+`\n## ${translate(options.language,'Full evidence IDs')}\n\n${fence(c.nodes.map(e=>`${evidenceReference(c,e.id)} · ${e.id}`).join('\n'))}\n`;
  const t=(text:string)=>translate(options.language,text);
  const link=(id:string)=>{const index=c.nodes.findIndex(e=>e.id===id);return index<0?escapeHtml(id):`<a href="#evidence-${index+1}">${evidenceReference(c,id)}</a>`;};
  const inventory=`<table><thead><tr><th>${t('Evidence')}</th><th>${t('RESULT')}</th><th>${t('Evidence validity')}</th></tr></thead><tbody>${c.nodes.map((e,i)=>`<tr id="evidence-${i+1}"><td><b>${evidenceReference(c,e.id)} · ${escapeHtml(evidenceTitle(c,e,options.language))}</b><br><small>${escapeHtml(t(kindLabels[e.kind]??e.kind))} · ${escapeHtml(e.id.slice(0,8))}</small><details><summary>${t('Full evidence IDs')}</summary><code>${escapeHtml(e.id)}</code></details></td><td>${escapeHtml(t(evidenceLabel(e)))}</td><td>${escapeHtml(t(e.validity.toUpperCase()))}</td></tr>`).join('')}</tbody></table>`;
  const connections=`<ul>${c.edges.map(e=>`<li>${link(e.source)} → ${link(e.target)} · ${escapeHtml(t(e.relation.replaceAll('_',' ')))}</li>`).join('')}</ul>`;
  const diagram=`<section><h2>${t('Evidence relationship diagram')}</h2><p>${t('A schematic of retained evidence; the inventory includes every item. Diagram shows up to 60 nodes. Select a node to jump to its record.')}</p>${evidenceDiagram(c,options.language)}</section>`;
  const body=sections.map(section=>`${section.heading===t('Evidence inventory')?diagram:''}<section><h2>${escapeHtml(section.heading)}</h2>${section.heading===t('Evidence inventory')?inventory:section.heading===t('Evidence connections')?connections:`<pre>${escapeHtml(section.text)}</pre>`}</section>`).join('');
  return `<!doctype html><html lang="${options.language==='en'?'en':'zh-Hant'}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><title>TRACE — ${escapeHtml(title)}</title><style>*{box-sizing:border-box}body{margin:0;background:#f3f6f8;color:#203446;font:16px/1.65 system-ui,'Microsoft JhengHei',sans-serif}main{max-width:1064px;margin:32px auto;padding:32px;background:white;border-radius:12px}h1{font-size:30px}h2{font-size:20px;border-bottom:1px solid #d5dfe6;padding-bottom:8px}section{margin:28px 0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:14px/1.7 Consolas,monospace;background:#f4f7f9;padding:18px;border-radius:6px}table{width:100%;border-collapse:collapse;font-size:13px;table-layout:fixed}td,th{text-align:left;vertical-align:top;padding:12px;border-bottom:1px solid #d5dfe6;overflow-wrap:anywhere}th:first-child{width:52%}small,p,summary{color:#61758a}details{font-size:11px}summary{cursor:pointer}a{color:#087d91}tr:target{background:#e5f7fb}li{margin:8px 0}code{overflow-wrap:anywhere}@media print{body{background:white}main{margin:0;padding:0}pre{background:white}tr{break-inside:avoid}details code{display:block}}@media(max-width:600px){main{margin:0;padding:20px}table{font-size:11px}td,th{padding:6px}}</style></head><body><main><h1>TRACE — ${escapeHtml(title)}</h1>${body}</main></body></html>`;

}
export function downloadReport(content:string,id:string,format:'html'|'md'){
  const url=URL.createObjectURL(new Blob([content],{type:format==='html'?'text/html;charset=utf-8':'text/markdown;charset=utf-8'}));
  const a=document.createElement('a');a.href=url;a.download=`TRACE-${id.replace(/[^a-zA-Z0-9-]/g,'').slice(0,50)||'case'}.${format}`;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
