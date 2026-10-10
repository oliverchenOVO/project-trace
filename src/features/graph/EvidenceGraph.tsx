import {useI18n} from '../../i18n/I18n';
import { memo, useMemo, useState, useEffect, useCallback } from 'react';
import { ReactFlow, Background, Controls, ControlButton, Handle, Position, type NodeProps, type Node, type Edge, MarkerType, useReactFlow, useNodesInitialized, useStore, getNodesBounds, getViewportForBounds } from '@xyflow/react';
import { FileCode2, GitCommitHorizontal, TriangleAlert, ScanLine, Sparkles, FlaskConical, GitBranch, FileDiff, ScrollText, Maximize2 } from 'lucide-react';
import type { Evidence, Investigation } from '../../domain/evidence';
import {matchesEvidence,selectedPath} from './graphQuery';
import { kindLabels, statusLabels } from '../../domain/evidence';
const icons:Record<string,typeof FileCode2>={incident:TriangleAlert,stack_frame:ScanLine,source_line:FileCode2,blame_result:GitBranch,commit:GitCommitHorizontal,diff_hunk:FileDiff,test_result:FlaskConical,hypothesis:Sparkles,log_event:ScrollText};
type GraphNode=Node<{evidence:Evidence},'evidence'>;
const EvidenceCard=memo(function EvidenceCard({data,selected}:NodeProps<GraphNode>) {
  const {t,evidenceText}=useI18n();
  const e=data.evidence;const Icon=icons[e.kind]??ScanLine;
  return <div title={t("{0}\n{1}\n{2}",evidenceText(e,e.title),evidenceText(e,e.summary),t(statusLabels[e.epistemicStatus]))} className={`evidence-card ${e.epistemicStatus} ${e.kind==='incident'?'crash':''} ${selected?'selected':''} ${e.validity!=='snapshot'?'invalid':''}`}>
    <Handle type="target" position={Position.Top}/><div className="node-kind"><Icon size={17}/>{t(kindLabels[e.kind]??e.kind)}</div>
    <strong>{e.kind==='regression_boundary'&&e.validity!=='snapshot'?t('INVALIDATED BOUNDARY CLAIM'):evidenceText(e,e.title)}</strong><p>{e.kind==='commit'?e.summary.slice(0,12):['incident','hypothesis'].includes(e.kind)?t(e.summary):evidenceText(e,e.summary)}</p>
    <span className="node-status">{t(e.validity!=='snapshot'?e.validity.toUpperCase():e.kind==='test_execution'?(e.data.independentlyExecuted===false?'EXECUTION ATTEMPT':'ACTUALLY EXECUTED'):e.kind==='regression_boundary'?'BISECT CONFIRMED':statusLabels[e.epistemicStatus])}</span><Handle type="source" position={Position.Bottom}/>
  </div>;
});
const nodeTypes={evidence:EvidenceCard};
const colors={observed:'#4edcf0',derived:'#4edcf0',correlated:'#e6b55c',hypothesis:'#b78aff'};
const positions:Record<string,[number,number]>={incident:[270,0],stack_frame:[270,155],source_line:[270,310],file_history:[560,310],blame_result:[0,310],commit:[0,465],diff_hunk:[290,465],candidate_signal:[570,465],test_result:[570,140],log_event:[0,150],hypothesis:[290,640],working_diff:[0,640],verification_run:[290,850],test_execution:[0,1020],bisect_run:[290,1200],regression_boundary:[290,1380]};
function SelectionSync({selected}:{selected:string|null}){const {setNodes}=useReactFlow();useEffect(()=>{setNodes(current=>current.map(n=>n.selected===(n.id===selected)?n:{...n,selected:n.id===selected}));},[selected,setNodes]);return null;}
function ViewportFitter() {
  const {t}=useI18n();
  const initialized=useNodesInitialized();const {getNodes,getInternalNode,setViewport}=useReactFlow();
  const width=useStore(s=>s.width),height=useStore(s=>s.height);
  // Presentation measurements belong to React Flow; domain evidence is immutable.
  // Fit measured bounds directly without changing any evidence coordinates.
  const refit=useCallback(()=>{if(!width||!height)return;const measured=getNodes().map(n=>getInternalNode(n.id)).filter(n=>n!==undefined);if(!measured.length)return;void setViewport(getViewportForBounds(getNodesBounds(measured),width,height,0.08,1.8,0.12),{duration:0});},[getNodes,getInternalNode,setViewport,width,height]);
  useEffect(()=>{if(initialized)refit();},[initialized,refit]);
  return <Controls showInteractive={false} showFitView={false}><ControlButton onClick={refit} title={t("Fit evidence graph")} aria-label={t("Fit evidence graph")}><Maximize2 size={14}/></ControlButton></Controls>;
}
export function EvidenceGraph({caseFile,selected,onSelect}:{caseFile:Investigation|null;selected:string|null;onSelect:(id:string)=>void}) {
  const {t,language,evidenceText}=useI18n();const [query,setQuery]=useState(''),[kind,setKind]=useState('');const filtering=!!query.trim()||!!kind;const matches=useMemo(()=>caseFile?.nodes.filter(e=>matchesEvidence(e,query,kind,`${evidenceText(e,e.title)} ${evidenceText(e,e.summary)} ${t(kindLabels[e.kind]??e.kind)}`))??[],[caseFile,query,kind,evidenceText,t]);const kinds=useMemo(()=>Array.from(new Set(caseFile?.nodes.map(n=>n.kind)??[])),[caseFile]);
  const [expanded,setExpanded]=useState(false);const [focus,setFocus]=useState(false);const [showFull,setShowFull]=useState(false);
  const hasVerification=caseFile?.nodes.some(n=>n.kind==='verification_run')??false;
  const compact=!filtering&&hasVerification&&!showFull&&!(focus&&selected&&caseFile?.nodes.some(n=>n.id===selected&&!n.data.verificationId&&!['commit','hypothesis'].includes(n.kind)));
  const {nodes,edges}=useMemo(()=> {
    if(!caseFile)return {nodes:[],edges:[]};
    const counts:Record<string,number>={};
    const latest=caseFile.nodes.filter(n=>n.kind==='verification_run').at(-1)?.data.verificationId;
    let visible=caseFile.nodes.filter(n=>{if(n.data.verificationId&&n.data.verificationId!==latest&&compact)return false;if(latest&&compact&&!n.data.verificationId&&!['commit','hypothesis'].includes(n.kind))return false;if(n.kind==='test_execution'&&!expanded)return n.data.role==='known_good'||n.data.role==='known_bad';const count=counts[n.kind]??0;counts[n.kind]=count+1;return expanded||count===0||n.kind==='hypothesis';});
    if(filtering)visible=caseFile.nodes.filter(e=>matchesEvidence(e,query,kind,`${evidenceText(e,e.title)} ${evidenceText(e,e.summary)} ${t(kindLabels[e.kind]??e.kind)}`));
    const path=selectedPath(caseFile,selected);
    if(focus&&selected) {
      const connected=path;
      visible=visible.filter(n=>connected.has(n.id));
    }
    const ids=new Set(visible.map(n=>n.id));const placed:Record<string,number>={};
    const nodes:GraphNode[]=visible.map((e,visibleIndex)=>{const ordinal=placed[e.kind]??0;placed[e.kind]=ordinal+1;let [x,y]=positions[e.kind]??[0,850];if(latest&&compact){[x,y]=({commit:[0,0],hypothesis:[570,0],verification_run:[290,160],test_execution:[0,340],bisect_run:[290,520],regression_boundary:[290,700]} as Record<string,[number,number]>)[e.kind]??[0,0];}if(e.kind==='test_execution'){x=e.data.role==='known_good'?0:570;y=latest&&compact?340:1020;if(expanded){x=(ordinal%3)*290;y=(latest&&compact?340:1020)+Math.floor(ordinal/3)*180;}}if(e.kind==='bisect_run'||e.kind==='regression_boundary')y+=(expanded?Math.ceil((placed.test_execution??0)/3)*180:0);if(filtering){x=(visibleIndex%3)*290;y=Math.floor(visibleIndex/3)*180;}return {id:e.id,type:'evidence',position:{x:x+(filtering||e.kind==='test_execution'?0:ordinal*850),y},data:{evidence:e},selected:e.id===selected};});
    const edges:Edge[]=caseFile.edges.filter(e=>ids.has(e.source)&&ids.has(e.target)).map(e=>({id:e.id,source:e.source,target:e.target,label:t(e.relation.replaceAll('_',' ')),type:'smoothstep',style:{stroke:colors[e.epistemicStatus],strokeWidth:path.has(e.source)&&path.has(e.target)?2.8:1.3,opacity:selected&&!(path.has(e.source)&&path.has(e.target))?0.35:1,strokeDasharray:e.epistemicStatus==='correlated'?'6 5':e.epistemicStatus==='hypothesis'?'2 5':undefined},labelStyle:{fill:'#a5b9cb',fontSize:11},labelBgStyle:{fill:'#0b1520',fillOpacity:0.95},markerEnd:{type:MarkerType.ArrowClosed,color:colors[e.epistemicStatus]}}));
    return {nodes,edges};
  },[caseFile,selected,expanded,focus,showFull,compact,t,filtering,query,kind,evidenceText]);
  return <section className="graph-panel" aria-label={t("Evidence graph")}>
    <div className="canvas-heading"><div><h1>{t("Chain of evidence")}</h1><p>{t(caseFile?'Connect code, changes and runtime evidence to understand what happened.':'An investigation starts with a real source.')}</p></div>
      {caseFile?<div className="graph-actions">{hasVerification?<button onClick={()=>{setShowFull(v=>!v);setExpanded(false);}}>{t(showFull?'Verification path':'Full investigation')}</button>:null}<button onClick={()=>setFocus(v=>!v)} aria-pressed={focus}>{t("Focus path")}</button><button onClick={()=>setExpanded(v=>!v)} aria-pressed={expanded}>{t(expanded?'Primary path':compact?'Expand steps':'Expand frames')}</button></div>:null}
    </div>
    {caseFile?<div className="graph-tools"><input aria-label={t('Search evidence nodes')} placeholder={t('Search evidence nodes')} value={query} onChange={e=>{setQuery(e.target.value);setFocus(false);}}/><select aria-label={t('Filter evidence kind')} value={kind} onChange={e=>{setKind(e.target.value);setFocus(false);}}><option value="">{t('All evidence kinds')}</option>{kinds.map(k=><option value={k} key={k}>{t(kindLabels[k]??k)}</option>)}</select>{filtering?<button onClick={()=>{setQuery('');setKind('');}}>{t('Clear filters')}</button>:null}<span role="status">{t('{0} visible / {1} evidence',nodes.length,caseFile.nodes.length)}</span>{filtering?<div className="graph-search-results">{matches.length?matches.slice(0,6).map(e=><button key={e.id} onClick={()=>onSelect(e.id)} aria-pressed={selected===e.id}>{evidenceText(e,e.title)}</button>):<span>{t('No matching evidence')}</span>}{matches.length>6?<small>{t('Showing first 6 results; narrow your search.')}</small>:null}</div>:null}</div>:null}
    {caseFile?<div className={`flow-stage ${filtering?'has-results':'has-tools'}`}><ReactFlow ariaLabelConfig={{'controls.ariaLabel':t('Graph controls'),'controls.zoomIn.ariaLabel':t('Zoom in'),'controls.zoomOut.ariaLabel':t('Zoom out')}} key={`${query}-${kind}-${language}-${caseFile.id}-${caseFile.nodes.length}-${caseFile.nodes.map(n=>n.validity+'-'+String((n.data.run as {status?:string}|undefined)?.status??'')).join('-')}-${expanded}-${showFull}-${focus}-${focus?selected:''}`} defaultNodes={nodes} edges={edges} nodeTypes={nodeTypes} onNodeClick={(_,n)=>onSelect(n.id)} fitView fitViewOptions={{padding:0.12}} minZoom={0.08} maxZoom={1.8} nodesDraggable={false} nodesConnectable={false} deleteKeyCode={null} colorMode="dark"><SelectionSync selected={selected}/><ViewportFitter/><Background color="#263b4b" gap={22} size={1}/></ReactFlow></div>:
    <div className="empty-scene"><div className="scene-symbol"><ScanLine size={44} strokeWidth={1}/></div><h2>{t("Every bug leaves a trace.")}</h2><p>{t("Follow the stack. Inspect the code.")}<br/>{t("Build a chain of evidence you can trust.")}</p><div className="empty-steps"><span>01 <b>{t("Open a local Git repository")}</b></span><span>02 <b>{t("Paste a Node.js stack trace")}</b></span><span>03 <b>{t("Inspect source, blame and changes")}</b></span></div><small>{t("No repository content is sent automatically.")}<br/>{t("Evidence comes first. Hypotheses come later.")}</small></div>}
    <div className="graph-legend"><span className="legend observed">{t("— Observed / derived")}</span><span className="legend correlated">{t("┄ Correlation")}</span><span className="legend hypothesis">{t("┈ AI hypothesis")}</span></div>
  </section>;
}
