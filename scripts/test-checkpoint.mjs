import {test,after} from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import checkpoint from '../server/checkpoint.mjs';
process.env.SAVE_ACCESS_CODE='test-only-save-code';process.env.SAVE_SIGNING_KEY='test-only-signing-key-never-used-in-production';
const server=http.createServer(checkpoint);await new Promise(r=>server.listen(0,'127.0.0.1',r));
const url=`http://127.0.0.1:${server.address().port}/api/checkpoint`;
after(()=>server.close());
async function post(body,cookie='',origin){return fetch(url,{method:'POST',headers:{'Content-Type':'application/json',cookie,...(origin?{origin}:{})},body:JSON.stringify(body)});}
const snapshot={version:1,chapter:2,player:{room:1},tiles:[],guards:[]};
test('unauthorized save and incorrect code are rejected',async()=>{
  assert.equal((await post({action:'save',snapshot})).status,401);
  assert.equal((await post({action:'unlock',code:'wrong'})).status,401);
});
test('authorized save seals data and can load without exposing code',async()=>{
  const unlock=await post({action:'unlock',code:process.env.SAVE_ACCESS_CODE});assert.equal(unlock.status,200);
  const cookie=unlock.headers.get('set-cookie').split(';')[0];assert.match(unlock.headers.get('set-cookie'),/HttpOnly/);
  const save=await post({action:'save',snapshot},cookie);assert.equal(save.status,200);
  const data=await save.json();assert.equal(data.chapter,3);assert.ok(data.token);
  const loaded=await post({action:'load',token:data.token});assert.deepEqual((await loaded.json()).snapshot,snapshot);
  const changed=data.token.slice(0,40)+(data.token[40]==='a'?'b':'a')+data.token.slice(41);
  assert.equal((await post({action:'load',token:changed})).status,400);
  assert.equal((await post({action:'save',snapshot},cookie+'forged')).status,401);
});
test('cross-origin writes and malformed saves fail safely',async()=>{
  assert.equal((await post({action:'unlock',code:process.env.SAVE_ACCESS_CODE},'','https://foreign.example')).status,403);
  assert.equal((await post({action:'load',token:'not-valid'})).status,400);
});
