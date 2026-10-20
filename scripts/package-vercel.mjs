// Package an already-built and verified bundle. This does not run a remote build.
import { cpSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
if(!existsSync('dist/game.wasm')||!existsSync('dist/build.json'))throw new Error('build or download the verified dist/ artifact first');
mkdirSync('.vercel/output/static',{recursive:true});
cpSync('dist/','.vercel/output/static/',{recursive:true});
writeFileSync('.vercel/output/config.json',JSON.stringify({version:3,routes:[
  {src:'/(.*)',headers:{'X-Content-Type-Options':'nosniff','Referrer-Policy':'strict-origin-when-cross-origin'},continue:true},
  {src:'/game.wasm',headers:{'Content-Type':'application/wasm','Cache-Control':'public, max-age=0, must-revalidate'},continue:true},
  {handle:'filesystem'}
]},null,2));
console.log('Vercel static output packaged:',JSON.parse(readFileSync('dist/build.json','utf8')));
