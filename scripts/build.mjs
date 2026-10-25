import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, copyFileSync, cpSync, readFileSync, writeFileSync } from 'node:fs';
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
const metadata=JSON.parse(execFileSync(cargo,['metadata','--format-version','1','--locked','--features','renderer'],{cwd:root,env,encoding:'utf8',maxBuffer:8*1024*1024}));
const macroquad=metadata.packages.find(p=>p.name==='macroquad'&&p.version==='0.4.14');
const miniquad=metadata.packages.find(p=>p.name==='miniquad');
const sound=metadata.packages.find(p=>p.name==='quad-snd');
if(!macroquad||!miniquad||!sound) throw new Error('matching renderer packages are missing');
const dist=join(root,'dist');mkdirSync(dist,{recursive:true});
copyFileSync(join(root,'target/wasm32-unknown-unknown/release/palace-of-sand.wasm'),join(dist,'game.wasm'));
// Macroquad 0.4.14's prebundled loader predates framebuffer functions in
// its resolved Miniquad dependency. Compose the exact locked crate runtimes.
const gl=readFileSync(join(dirname(miniquad.manifest_path),'js/gl.js'),'utf8');
const audio=readFileSync(join(dirname(sound.manifest_path),'js/audio.js'),'utf8');
writeFileSync(join(dist,'mq.js'),gl+'\n;(function(){\n'+audio+'\n})();\n');
for(const file of ['index.html','style.css','shell.js','favicon.svg']) copyFileSync(join(root,'web',file),join(dist,file));
cpSync(join(root,'assets/art'),join(dist,'art'),{recursive:true});
writeFileSync(join(dist,'build.json'),JSON.stringify({version:'0.5.2',graphics:'cinematic-raster',macroquad:'0.4.14',wasmSha256:createHash('sha256').update(readFileSync(join(dist,'game.wasm'))).digest('hex')},null,2));
console.log('Static WebAssembly bundle ready in dist/');
