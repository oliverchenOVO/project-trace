export type EpistemicStatus = 'observed'|'derived'|'correlated'|'hypothesis';
export interface Provenance {sourceType:string;filePath:string|null;lineStart:number|null;lineEnd:number|null;commitSha:string|null;command:string[];rawExcerpt:string;artifactId:string|null;sourceHash:string|null;capturedAt:number}
export interface Evidence {id:string;kind:string;epistemicStatus:EpistemicStatus;title:string;summary:string;createdBy:string;strength:string;provenance:Provenance;data:Record<string,unknown>;validity:string}
export interface EvidenceEdge {id:string;source:string;target:string;relation:string;epistemicStatus:EpistemicStatus;provenance:Provenance}
export interface Commit {sha:string;author:string;timestamp:string;subject:string}
export interface Repository {token:string;root:string;head:string|null;branch:string;dirty:boolean;status:string;recentCommits:Commit[]}
export interface StackFrame {index:number;raw:string;functionName:string|null;filePath:string;line:number;column:number|null;language:string;resolved:boolean}
export interface Investigation {id:string;title:string;createdAt:number;repository:Repository;stack:string;logs:string;tests:string;frames:StackFrame[];nodes:Evidence[];edges:EvidenceEdge[];warnings:string[];sourceHashes:Record<string,string>;stale:boolean}
export interface SavedCase {id:string;title:string;createdAt:number}
export interface AiContext {evidence:Array<{id:string;kind:string;status:string;title:string;summary:string;excerpt:string;excerptTruncated:boolean}>;fileCount:number;lineCount:number;bytes:number;excludedCount:number;policy:string}
export const statusLabels:Record<EpistemicStatus,string>={observed:'OBSERVED',derived:'DERIVED · SOURCE VERIFIED',correlated:'CORRELATION',hypothesis:'AI HYPOTHESIS'};
export const kindLabels:Record<string,string>={incident:'INCIDENT',stack_frame:'STACK FRAME',source_line:'SOURCE LOCATION',blame_result:'BLAME',commit:'COMMIT',diff_hunk:'DIFF',candidate_signal:'CANDIDATE',file_history:'FILE HISTORY',test_result:'TEST OUTPUT',log_event:'LOGS',hypothesis:'AI HYPOTHESIS',working_diff:'WORKING TREE'};
Object.assign(kindLabels,{test_execution:'TEST EXECUTION',verification_run:'VERIFICATION RUN',bisect_run:'BISECT RUN',regression_boundary:'REGRESSION BOUNDARY'});

export function evidenceLabel(e:Evidence):string {if(e.kind==='test_execution')return e.data.independentlyExecuted===true?'OBSERVED · ACTUALLY EXECUTED':'OBSERVED · EXECUTION ATTEMPT';if(e.kind==='regression_boundary')return 'DERIVED · TEST BOUNDARY';if(['verification_run','bisect_run'].includes(e.kind))return 'DERIVED · EXPERIMENT RECORD';return statusLabels[e.epistemicStatus];}
