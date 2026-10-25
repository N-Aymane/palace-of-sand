import http from 'node:http';
import { createReadStream, statSync } from 'node:fs';
import { resolve, extname, sep } from 'node:path';
import checkpoint from '../server/checkpoint.mjs';
try {process.loadEnvFile('.env.save.local');}catch{}
const root=resolve('dist');
const mime={'.html':'text/html; charset=utf-8','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.svg':'image/svg+xml'};
http.createServer((req,res)=>{
  if(new URL(req.url,'http://localhost').pathname==='/api/checkpoint'){
    checkpoint(req,res).catch(()=>{res.statusCode=500;res.end('{"error":"Save service unavailable."}');});return;
  }
  try {
    const path=resolve(root,'.'+decodeURIComponent(new URL(req.url,'http://localhost').pathname));
    const file=path===root?resolve(root,'index.html'):path;
    if(!file.startsWith(root+sep)||!statSync(file).isFile()) {res.writeHead(404);res.end('Not found');return;}
    res.setHeader('Content-Type',mime[extname(file)]??'application/octet-stream');res.setHeader('Cache-Control','no-store');
    createReadStream(file).pipe(res);
  }catch {res.writeHead(404);res.end('Not found');}
}).listen(4173,'127.0.0.1',()=>console.log('Palace of Sand: http://127.0.0.1:4173'));
