import {describe,it,expect} from 'vitest';
import type {Investigation,Evidence} from '../../domain/evidence';
import {createReport,reportSections,evidenceDiagram} from './report';
const node:Evidence={id:'ev-1',kind:'source_line',title:'file.ts:1',summary:'private-source-summary',epistemicStatus:'derived',createdBy:'core',strength:'strong',validity:'snapshot',data:{privateField:'DO_NOT_EXPORT'},provenance:{sourceType:'file',filePath:'file.ts',lineStart:1,lineEnd:1,commitSha:null,command:[],rawExcerpt:'SECRET_CODE',artifactId:null,sourceHash:null,capturedAt:1}};
function caseFile():Investigation{return {id:'case-1',title:'Sample incident',createdAt:1,repository:{token:'SESSION_TOKEN',root:'C:/private/project',head:'abc',branch:'main',dirty:false,status:'',recentCommits:[]},stack:'PRIVATE_STACK',logs:'PRIVATE_LOG',tests:'PRIVATE_TEST',frames:[],nodes:[structuredClone(node)],edges:[],warnings:[],sourceHashes:{},stale:false};}
describe('shareable reports',()=>{
 it('uses readable references and local links while retaining full IDs and managed names',()=>{
  const c=caseFile();c.nodes.push({...structuredClone(node),id:'ev-2',title:'Second evidence'});c.edges=[{id:'edge',source:'ev-1',target:'ev-2',relation:'supports',epistemicStatus:'derived',provenance:node.provenance}];
  const options={language:'en' as const,includeExcerpts:false,details:{id:c.id,title:'Named case',createdAt:1,tags:['demo']}};
  const html=createReport(c,options,'html');expect(html).toContain('Named case');expect(html).toContain('E01 · file.ts:1');expect(html).toContain('href="#evidence-2"');expect(html).toContain('id="evidence-2"');expect(html).toContain('<svg');expect(html).not.toContain('SECRET_CODE');
  expect(createReport(c,options,'md')).toContain('E02 · ev-2');expect(JSON.stringify(c)).not.toContain('Named case');
 });
 it('escapes SVG titles, limits diagram size and labels stale boundary records as historical',()=>{
  const c=caseFile();c.nodes[0].title='<script>alert(1)</script>';c.nodes=Array.from({length:61},(_,i)=>({...structuredClone(c.nodes[0]),id:`ev-${i}`}));
  expect(evidenceDiagram(c,'en')).not.toContain('<script>');expect(evidenceDiagram(c,'en').match(/<rect /g)).toHaveLength(60);c.nodes[0].kind='regression_boundary';c.stale=true;expect(evidenceDiagram(c,'en')).toContain('Historical boundary record');
 });
 it('omits raw evidence and native internals by default without mutating the case',()=>{
  const c=caseFile(),before=JSON.stringify(c);const html=createReport(c,{language:'en',includeExcerpts:false},'html');
  for(const secret of ['SESSION_TOKEN','DO_NOT_EXPORT','SECRET_CODE','PRIVATE_STACK','PRIVATE_LOG','PRIVATE_TEST','private-source-summary','C:/private'])expect(html).not.toContain(secret);
  expect(html).toContain('ev-1');expect(JSON.stringify(c)).toBe(before);
 });
 it('includes original excerpts only when selected and never serializes hidden fields',()=>{
  const html=createReport(caseFile(),{language:'zh-TW',includeExcerpts:true},'html');expect(html).toContain('SECRET_CODE');expect(html).toContain('案件概覽');expect(html).not.toContain('SESSION_TOKEN');expect(html).not.toContain('DO_NOT_EXPORT');
 });
 it('escapes executable HTML in titles and excerpts',()=>{
  const c=caseFile();c.title='<img src=x onerror="alert(1)">';c.nodes[0].provenance.rawExcerpt='</pre><script>alert(1)</script>';
  const html=createReport(c,{language:'en',includeExcerpts:true},'html');expect(html).not.toContain('<script>');expect(html).not.toContain('<img');expect(html).toContain('&lt;script&gt;');expect(html).toContain("default-src 'none'");
 });
 it('keeps hostile Markdown inside a fence longer than all source backticks',()=>{
  const c=caseFile();c.title='```\n![external](https://example.invalid/x)\n````';const md=createReport(c,{language:'en',includeExcerpts:false},'md');expect(md).toContain('`````\n'+c.title);expect(md).toContain('Root cause unverified');
 });
 it('does not promote stale or invalid experiment records to a current boundary',()=>{
  const c=caseFile();c.nodes=[{...node,kind:'verification_run',data:{integrityValid:true,run:{status:'bisect_confirmed',knownGood:'good',knownBad:'bad',firstBad:'FIRST_BAD',cleanupStatus:'cleaned',sourceUnchanged:true,approvalToken:'APPROVAL_SECRET',command:{requested:{executable:'node',args:['test']},environment:{SECRET:'ENV_SECRET'}}}}}];
  c.nodes[0].provenance={...node.provenance,rawExcerpt:JSON.stringify(c.nodes[0].data.run)};
  for(const includeExcerpts of [false,true]){const output=createReport(c,{language:'en',includeExcerpts},'html');expect(output).not.toContain('APPROVAL_SECRET');expect(output).not.toContain('ENV_SECRET');}
  let text=reportSections(c,{language:'en',includeExcerpts:false}).map(s=>s.text).join('\n');expect(text).toContain('FIRST_BAD');expect(text).not.toContain('APPROVAL_SECRET');expect(text).not.toContain('ENV_SECRET');
  c.stale=true;text=reportSections(c,{language:'en',includeExcerpts:false}).map(s=>s.text).join('\n');expect(text).toContain('No current verified boundary');expect(text).not.toContain('FIRST_BAD');
  c.stale=false;c.nodes[0].data.integrityValid=false;text=reportSections(c,{language:'en',includeExcerpts:false}).map(s=>s.text).join('\n');expect(text).not.toContain('FIRST_BAD');
 });
});
