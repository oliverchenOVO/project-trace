import type {Evidence,Investigation} from '../../domain/evidence';
export function matchesEvidence(e:Evidence,query:string,kind:string,displayText=''){const terms=query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);return (!kind||e.kind===kind)&&terms.every(term=>`${e.title} ${e.summary} ${e.id} ${e.provenance.filePath??''} ${displayText}`.toLocaleLowerCase().includes(term));}
export function selectedPath(c:Investigation,selected:string|null):Set<string>{
 const connected=new Set<string>(selected?[selected]:[]);if(!selected)return connected;
 let changed=true;while(changed){changed=false;for(const e of c.edges)if(connected.has(e.target)&&!connected.has(e.source)){connected.add(e.source);changed=true;}}
 for(const e of c.edges)if(e.source===selected)connected.add(e.target);return connected;
}
