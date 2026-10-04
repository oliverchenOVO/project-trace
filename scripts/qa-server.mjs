// Test-only transport adapter. Serves the actual production bundle and forwards
// its Tauri calls to the same Rust core. No evidence is fabricated here.
import http from 'node:http';
import {readFileSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
const port=24917,dist=resolve('dist'),origin=`http://127.0.0.1:${port}`;
const native=spawn(resolve('src-tauri/target/debug/trace-probe.exe'),['--serve'],{windowsHide:true,env:process.env});
let sequence=0;const pending=new Map();
createInterface({input:native.stdout}).on('line',line=>{try{const data=JSON.parse(line);const res=pending.get(data.requestId);pending.delete(data.requestId);if(res)res.writeHead(200,{'Content-Type':'application/json'}).end(JSON.stringify(data));}catch{}});
native.stderr.on('data',d=>process.stderr.write(d));
native.on('exit',()=>{for(const res of pending.values())res.writeHead(500).end(JSON.stringify({error:'Native diagnostic service stopped'}));pending.clear();});
process.on('exit',()=>native.kill());
const transport=`<script>globalThis.isTauri=true;let traceRoot='';window.__TAURI_INTERNALS__={invoke:async(command,args={})=>{const r=await fetch('/__trace',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({command,args:{...args,path:args.path||traceRoot}})});const d=await r.json();if(d.error)throw Error(d.error);if(command==='open_repository')traceRoot=d.ok.root;if(command==='load_case')traceRoot=d.ok.repository.root;return d.ok;}};</script>`;
http.createServer((req,res)=>{
  if(req.url==='/__trace'){
    if(req.method!=='POST'||req.headers.origin!==origin){res.writeHead(403).end();return;}
    let raw='';req.on('data',d=>{raw+=d;if(raw.length>256000)req.destroy();});
    req.on('end',()=>{try{const r=JSON.parse(raw);const requestId=++sequence;pending.set(requestId,res);native.stdin.write(JSON.stringify({...r,requestId})+'\n');}catch{res.writeHead(400).end(JSON.stringify({error:'Invalid diagnostic request'}));}});return;
  }
  if(req.url==='/favicon.ico'){res.writeHead(204).end();return;}
  try{
    const pathname=decodeURIComponent(new URL(req.url,origin).pathname);const file=resolve(dist,'.'+(pathname==='/'?'/index.html':pathname));
    if(!file.startsWith(dist+sep)){res.writeHead(403).end();return;}
    const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.png':'image/png'}[extname(file)]||'application/octet-stream';
    let data=readFileSync(file);if(extname(file)==='.html')data=Buffer.from(data.toString().replace('<script type="module"',transport+'<script type="module"'));
    res.writeHead(200,{'Content-Type':mime});res.end(data);
  }catch{res.writeHead(404).end();}
}).listen(port,'127.0.0.1',()=>console.log(`TRACE production-bundle QA at ${origin}`));
