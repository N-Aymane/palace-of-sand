import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, copyFileSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { createHash } from 'node:crypto';

const root=resolve(import.meta.dirname,'..');
const local=join(root,'.tools','cargo');
const env={...process.env};
let cargo='cargo';
if(existsSync(join(local,'bin',process.platform==='win32'?'cargo.exe':'cargo'))) {
  env.CARGO_HOME=local;env.RUSTUP_HOME=join(root,'.tools','rustup');
  env.PATH=join(local,'bin')+(process.platform==='win32'?';':':')+env.PATH;
  cargo=join(local,'bin',process.platform==='win32'?'cargo.exe':'cargo');
}
execFileSync(cargo,['build','--locked','--release','--target','wasm32-unknown-unknown','--features','renderer'],{cwd:root,env,stdio:'inherit'});
const metadata=JSON.parse(execFileSync(cargo,['metadata','--format-version','1','--locked'],{cwd:root,env,encoding:'utf8',maxBuffer:8*1024*1024}));
const macroquad=metadata.packages.find(p=>p.name==='macroquad'&&p.version==='0.4.14');
if(!macroquad) throw new Error('matching Macroquad package is missing');
const dist=join(root,'dist');mkdirSync(dist,{recursive:true});
copyFileSync(join(root,'target/wasm32-unknown-unknown/release/palace-of-sand.wasm'),join(dist,'game.wasm'));
copyFileSync(join(dirname(macroquad.manifest_path),'js/mq_js_bundle.js'),join(dist,'mq.js'));
for(const file of ['index.html','style.css','shell.js','favicon.svg']) copyFileSync(join(root,'web',file),join(dist,file));
writeFileSync(join(dist,'build.json'),JSON.stringify({version:'0.1.0',macroquad:'0.4.14',wasmSha256:createHash('sha256').update(readFileSync(join(dist,'game.wasm'))).digest('hex')},null,2));
console.log('Static WebAssembly bundle ready in dist/');
