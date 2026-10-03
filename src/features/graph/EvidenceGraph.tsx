import { memo, useMemo, useState, useEffect, useRef, type RefObject } from 'react';
import { ReactFlow, Background, Controls, Handle, Position, type NodeProps, type Node, type Edge, MarkerType, useReactFlow, useNodesInitialized } from '@xyflow/react';
import { FileCode2, GitCommitHorizontal, TriangleAlert, ScanLine, Sparkles, FlaskConical, GitBranch, FileDiff, ScrollText } from 'lucide-react';
import type { Evidence, Investigation } from '../../domain/evidence';
import { kindLabels, statusLabels } from '../../domain/evidence';
const icons:Record<string,typeof FileCode2>={incident:TriangleAlert,stack_frame:ScanLine,source_line:FileCode2,blame_result:GitBranch,commit:GitCommitHorizontal,diff_hunk:FileDiff,test_result:FlaskConical,hypothesis:Sparkles,log_event:ScrollText};
type GraphNode=Node<{evidence:Evidence},'evidence'>;
const EvidenceCard=memo(function EvidenceCard({data,selected}:NodeProps<GraphNode>) {
  const e=data.evidence;const Icon=icons[e.kind]??ScanLine;
  return <div title={`${e.title}\n${e.summary}\n${statusLabels[e.epistemicStatus]}`} className={`evidence-card ${e.epistemicStatus} ${e.kind==='incident'?'crash':''} ${selected?'selected':''} ${e.validity!=='snapshot'?'invalid':''}`}>
    <Handle type="target" position={Position.Top}/><div className="node-kind"><Icon size={17}/>{kindLabels[e.kind]??e.kind}</div>
    <strong>{e.title}</strong><p>{e.kind==='commit'?e.summary.slice(0,12):e.summary}</p>
    <span className="node-status">{e.validity==='snapshot'?statusLabels[e.epistemicStatus]:e.validity.toUpperCase()}</span><Handle type="source" position={Position.Bottom}/>
  </div>;
});
const nodeTypes={evidence:EvidenceCard};
const colors={observed:'#4edcf0',derived:'#4edcf0',correlated:'#e6b55c',hypothesis:'#b78aff'};
const positions:Record<string,[number,number]>={incident:[270,0],stack_frame:[270,155],source_line:[270,310],file_history:[560,310],blame_result:[0,310],commit:[0,465],diff_hunk:[290,465],candidate_signal:[570,465],test_result:[570,140],log_event:[0,150],hypothesis:[290,640],working_diff:[0,640]};
function ViewportFitter({container}:{container:RefObject<HTMLDivElement|null>}) {
  const initialized=useNodesInitialized();const {fitView}=useReactFlow();
  useEffect(()=>{if(!initialized||!container.current)return;const observer=new ResizeObserver(()=>{void fitView({padding:0.12,duration:0});});observer.observe(container.current);return()=>observer.disconnect();},[initialized,fitView,container]);
  return null;
}
export function EvidenceGraph({caseFile,selected,onSelect}:{caseFile:Investigation|null;selected:string|null;onSelect:(id:string)=>void}) {
  const [expanded,setExpanded]=useState(false);const [focus,setFocus]=useState(false);
  const stage=useRef<HTMLDivElement>(null);
  const {nodes,edges}=useMemo(()=> {
    if(!caseFile)return {nodes:[],edges:[]};
    const counts:Record<string,number>={};
    let visible=caseFile.nodes.filter(n=>{const count=counts[n.kind]??0;counts[n.kind]=count+1;return expanded||count===0||n.kind==='hypothesis';});
    if(focus&&selected) {
      const connected=new Set([selected]);let changed=true;
      while(changed){changed=false;for(const e of caseFile.edges){if(connected.has(e.target)&&!connected.has(e.source)){connected.add(e.source);changed=true;}}}
      for(const e of caseFile.edges)if(e.source===selected)connected.add(e.target);
      visible=visible.filter(n=>connected.has(n.id));
    }
    const ids=new Set(visible.map(n=>n.id));const placed:Record<string,number>={};
    const nodes:GraphNode[]=visible.map(e=>{const ordinal=placed[e.kind]??0;placed[e.kind]=ordinal+1;const [x,y]=positions[e.kind]??[0,850];return {id:e.id,type:'evidence',position:{x:x+ordinal*850,y},data:{evidence:e},selected:e.id===selected};});
    const edges:Edge[]=caseFile.edges.filter(e=>ids.has(e.source)&&ids.has(e.target)).map(e=>({id:e.id,source:e.source,target:e.target,label:e.relation.replaceAll('_',' '),type:'smoothstep',style:{stroke:colors[e.epistemicStatus],strokeWidth:1.3,strokeDasharray:e.epistemicStatus==='correlated'?'6 5':e.epistemicStatus==='hypothesis'?'2 5':undefined},labelStyle:{fill:'#a5b9cb',fontSize:11},labelBgStyle:{fill:'#0b1520',fillOpacity:0.95},markerEnd:{type:MarkerType.ArrowClosed,color:colors[e.epistemicStatus]}}));
    return {nodes,edges};
  },[caseFile,selected,expanded,focus]);
  return <section className="graph-panel" aria-label="Evidence graph">
    <div className="canvas-heading"><div><h1>Chain of evidence</h1><p>{caseFile?'Connect code, changes and runtime evidence to understand what happened.':'An investigation starts with a real source.'}</p></div>
      {caseFile?<div className="graph-actions"><button onClick={()=>setFocus(v=>!v)} aria-pressed={focus}>Focus path</button><button onClick={()=>setExpanded(v=>!v)} aria-pressed={expanded}>{expanded?'Primary path':'Expand frames'}</button></div>:null}
    </div>
    {caseFile?<div ref={stage} className="flow-stage"><ReactFlow key={`${caseFile.id}-${caseFile.nodes.length}-${expanded}-${focus}`} nodes={nodes} edges={edges} nodeTypes={nodeTypes} onNodeClick={(_,n)=>onSelect(n.id)} fitView fitViewOptions={{padding:0.12}} minZoom={0.2} maxZoom={1.8} nodesDraggable={false} nodesConnectable={false} colorMode="dark"><ViewportFitter container={stage}/><Background color="#263b4b" gap={22} size={1}/><Controls showInteractive={false}/></ReactFlow></div>:
    <div className="empty-scene"><div className="scene-symbol"><ScanLine size={44} strokeWidth={1}/></div><h2>Every bug leaves a trace.</h2><p>Follow the stack. Inspect the code.<br/>Build a chain of evidence you can trust.</p><div className="empty-steps"><span>01 <b>Open a local Git repository</b></span><span>02 <b>Paste a Node.js stack trace</b></span><span>03 <b>Inspect source, blame and changes</b></span></div><small>No repository content is sent automatically.<br/>Evidence comes first. Hypotheses come later.</small></div>}
    <div className="graph-legend"><span className="legend observed">— Observed / derived</span><span className="legend correlated">┄ Correlation</span><span className="legend hypothesis">┈ AI hypothesis</span></div>
  </section>;
}
