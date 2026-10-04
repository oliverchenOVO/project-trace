import { invoke, isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {Repository,Investigation,SavedCase,AiContext} from '../domain/evidence';
import type {VerificationRun,VerificationRequest} from '../features/verification/types';
let qaRoot='';
async function call<T>(command:string,args:Record<string,unknown>={}):Promise<T> {
  if(isTauri()) return invoke<T>(command,args);
  if(import.meta.env.DEV) {
    const response=await fetch('/__trace',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({command,args:{...args,path:args.path??qaRoot}})});
    if(!response.ok) throw new Error('Repository access requires the TRACE desktop app. Browser mode is visual preview only.');
    const data=await response.json();if(data.error)throw new Error(data.error);return data.ok as T;
  }
  throw new Error('Open TRACE desktop to inspect local repositories.');
}
export const backend={
  desktop:isTauri(),
  async chooseFolder(title='Select a local Git repository'):Promise<string|null> {if(!isTauri())return null;const result=await open({directory:true,multiple:false,title});return typeof result==='string'?result:null;},
  async openRepository(path:string) {const repo=await call<Repository>('open_repository',{path});qaRoot=repo.root;return repo;},
  analyze:(token:string,stack:string,logs:string,tests:string)=>call<Investigation>('analyze',{token,stack,logs,tests}),
  listCases:()=>call<SavedCase[]>('list_cases'),
  loadCase:async(id:string)=>{const c=await call<Investigation>('load_case',{id});qaRoot=c.repository.root;return c;},
  context:(id:string)=>call<AiContext>('ai_context',{id}),
  hypothesis:(id:string,model:string)=>call<Investigation>('generate_hypothesis',{id,model}),
  prepareVerification:(request:VerificationRequest)=>call<VerificationRun>('prepare_verification',{request}),
  approveVerification:(runId:string,token:string)=>call<VerificationRun>('approve_verification',{runId,token}),
  verificationStatus:(id:string)=>call<{runs:VerificationRun[];caseFile:Investigation}>('verification_status',{id}),
  cancelVerification:(runId:string)=>call<void>('cancel_verification',{runId}),
  discardVerification:(runId:string)=>call<VerificationRun>('discard_verification',{runId}),
  cleanupVerification:(runId:string)=>call<VerificationRun>('cleanup_verification',{runId})
};
