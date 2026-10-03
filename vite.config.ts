import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
// Explicit opt-in, loopback-only development harness. Excluded from packaged builds.
function nativeQaBridge(): Plugin {
  return { name: 'trace-native-qa', apply:'serve', configureServer(server) {
    if (process.env.TRACE_QA !== '1') return;
    server.middlewares.use('/__trace', (req,res) => {
      if (req.method!=='POST'||req.headers.origin!== `http://127.0.0.1:${process.env.TRACE_QA_PORT || '1420'}`) {res.statusCode=403;res.end();return;}
      let body='';req.on('data',chunk=>{body+=chunk;if(body.length>256000)req.destroy();});
      req.on('end',()=> {
        const child=spawn(resolve('src-tauri/target/debug/trace-probe.exe'),[],{env:process.env,windowsHide:true,stdio:['pipe','pipe','pipe']});
        let out='';child.stdout.on('data',d=>{out+=d;});child.on('error',e=>{res.statusCode=500;res.end(JSON.stringify({error:e.message}));});
        child.on('close',()=>{res.setHeader('Content-Type','application/json');res.end(out||JSON.stringify({error:'Native diagnostic process failed'}));});child.stdin.end(body);
      });
    });
  }};
}
export default defineConfig({plugins:[react(),nativeQaBridge()],server:{host:'127.0.0.1',port:Number(process.env.TRACE_QA_PORT || 1420),strictPort:true},clearScreen:false});
