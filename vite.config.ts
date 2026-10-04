import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import type { ServerResponse } from 'node:http';
// Explicit opt-in, loopback-only development harness. Excluded from packaged builds.
function nativeQaBridge(): Plugin {
  return { name: 'trace-native-qa', apply:'serve', configureServer(server) {
    if (process.env.TRACE_QA !== '1') return;
    const child=spawn(resolve('src-tauri/target/debug/trace-probe.exe'),['--serve'],{env:process.env,windowsHide:true,stdio:['pipe','pipe','pipe']});
    let sequence=0;const pending=new Map<number,ServerResponse>();
    createInterface({input:child.stdout}).on('line',line=>{try{const data=JSON.parse(line);const res=pending.get(data.requestId);pending.delete(data.requestId);if(res){res.setHeader('Content-Type','application/json');res.end(JSON.stringify(data));}}catch{}});
    const stopped=(error:string)=>{for(const res of pending.values()){res.statusCode=500;res.end(JSON.stringify({error}));}pending.clear();};
    child.on('error',e=>stopped(e.message));child.on('exit',()=>stopped('Native diagnostic service stopped'));
    child.stderr.on('data',d=>process.stderr.write(d));server.httpServer?.once('close',()=>child.kill());
    server.middlewares.use('/__trace', (req,res) => {
      if (req.method!=='POST'||req.headers.origin!== `http://127.0.0.1:${process.env.TRACE_QA_PORT || '1420'}`) {res.statusCode=403;res.end();return;}
      let body='';req.on('data',chunk=>{body+=chunk;if(body.length>256000)req.destroy();});
      req.on('end',()=> {
        try{const request=JSON.parse(body);if(child.exitCode!==null||child.killed)throw Error('Native diagnostic service stopped');const requestId=++sequence;pending.set(requestId,res);child.stdin.write(JSON.stringify({...request,requestId})+'\n');}
        catch{res.statusCode=400;res.end(JSON.stringify({error:'Invalid or unavailable native diagnostic request'}));}
      });
    });
  }};
}
export default defineConfig({plugins:[react(),nativeQaBridge()],server:{host:'127.0.0.1',port:Number(process.env.TRACE_QA_PORT || 1420),strictPort:true},clearScreen:false});
