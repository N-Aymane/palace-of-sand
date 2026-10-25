import { createHash, createHmac, timingSafeEqual, randomBytes, createCipheriv, createDecipheriv } from 'node:crypto';

const cookieName='palace_save_access';
const maxBody=200_000;
function digest(value){return createHash('sha256').update(value).digest();}
function equal(a,b){return timingSafeEqual(digest(a),digest(b));}
function sign(value,key){return createHmac('sha256',key).update('save-session:'+value).digest('base64url');}
function authorized(req,key){
  const cookie=(req.headers.cookie??'').split(';').map(v=>v.trim()).find(v=>v.startsWith(cookieName+'='))?.slice(cookieName.length+1)??'';
  const [expires,nonce,signature]=cookie.split('.');
  return /^\d+$/.test(expires??'')&&Number(expires)>Date.now()&&Number(expires)<Date.now()+3_600_001
    && /^[\w-]{16}$/.test(nonce??'')&&equal(signature??'',sign(expires+'.'+nonce,key));
}
function seal(data,key){
  const iv=randomBytes(12);const cipher=createCipheriv('aes-256-gcm',digest(key),iv);
  cipher.setAAD(Buffer.from('palace-checkpoint-v1'));
  const payload=Buffer.concat([cipher.update(JSON.stringify(data),'utf8'),cipher.final()]);
  return Buffer.concat([iv,cipher.getAuthTag(),payload]).toString('base64url');
}
function open(token,key){
  if(typeof token!=='string'||token.length>maxBody||!/^[\w-]+$/.test(token))throw new Error('Invalid checkpoint');
  const bytes=Buffer.from(token,'base64url');if(bytes.length<29)throw new Error('Invalid checkpoint');
  const decipher=createDecipheriv('aes-256-gcm',digest(key),bytes.subarray(0,12));
  decipher.setAAD(Buffer.from('palace-checkpoint-v1'));decipher.setAuthTag(bytes.subarray(12,28));
  return JSON.parse(Buffer.concat([decipher.update(bytes.subarray(28)),decipher.final()]).toString('utf8'));
}
async function body(req){
  let bytes=0;const chunks=[];
  for await(const chunk of req){bytes+=Buffer.byteLength(chunk);if(bytes>maxBody)throw new Error('Request too large');chunks.push(Buffer.from(chunk));}
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}
export default async function checkpoint(req,res){
  res.setHeader('Content-Type','application/json; charset=utf-8');res.setHeader('Cache-Control','no-store');
  res.setHeader('X-Content-Type-Options','nosniff');
  const reply=(status,data)=>{res.statusCode=status;res.end(JSON.stringify(data));};
  const code=process.env.SAVE_ACCESS_CODE;const key=process.env.SAVE_SIGNING_KEY;
  if(!code||!key||key.length<32){reply(503,{error:'Saving is not configured on this server.'});return;}
  if(req.method==='GET'){reply(200,{unlocked:authorized(req,key)});return;}
  if(req.method!=='POST'){res.setHeader('Allow','GET, POST');reply(405,{error:'Method not allowed.'});return;}
  const origin=req.headers.origin;
  const host=req.headers['x-forwarded-host']??req.headers.host;
  if(origin){try{if(new URL(origin).host!==host){reply(403,{error:'Request origin rejected.'});return;}}
    catch{reply(403,{error:'Request origin rejected.'});return;}}
  if(!(req.headers['content-type']??'').startsWith('application/json')){reply(415,{error:'JSON required.'});return;}
  try{
    const data=await body(req);
    if(data.action==='unlock'){
      if(typeof data.code!=='string'||data.code.length>200||!equal(data.code,code)){
        reply(401,{error:'Incorrect save code.'});return;
      }
      const payload=(Date.now()+3_600_000)+'.'+randomBytes(12).toString('base64url');
      const secure=process.env.VERCEL==='1'||req.headers['x-forwarded-proto']==='https';
      res.setHeader('Set-Cookie',`${cookieName}=${payload}.${sign(payload,key)}; HttpOnly; SameSite=Strict; Path=/api/checkpoint; Max-Age=3600${secure?'; Secure':''}`);
      reply(200,{unlocked:true});return;
    }
    if(data.action==='save'){
      if(!authorized(req,key)){reply(401,{error:'Enter the code to unlock saving.'});return;}
      const state=data.snapshot;
      if(!state||state.version!==1||!Number.isInteger(state.chapter)||state.chapter<0||state.chapter>3
        ||!state.player||!Array.isArray(state.tiles)||!Array.isArray(state.guards)){
        reply(400,{error:'Invalid checkpoint data.'});return;
      }
      const savedAt=new Date().toISOString();
      reply(200,{token:seal({snapshot:state,savedAt},key),chapter:state.chapter+1,savedAt});return;
    }
    if(data.action==='load'){
      const dataSaved=open(data.token,key);reply(200,dataSaved);return;
    }
    reply(400,{error:'Unknown checkpoint action.'});
  }catch{reply(400,{error:'Checkpoint data is invalid, damaged, or too large.'});}
}
