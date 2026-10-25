/* Browser adapter only: loading, focus, fullscreen, accessible control shortcuts. */
'use strict';
const gameCanvas=document.querySelector('#glcanvas');
const loading=document.querySelector('#loading');
const engineState=document.querySelector('#engine-state');
const keys=new Set();
// Long presses on game controls must remain held input, not open a browser menu.
gameCanvas.addEventListener('contextmenu',event=>event.preventDefault());
const touchInputs=new Map();
const keyCodes={KeyZ:90,KeyX:88,KeyT:84,Enter:13,Escape:27,KeyR:82,KeyM:77,F12:123,ArrowLeft:37,ArrowUp:38,ArrowRight:39,ArrowDown:40,ShiftLeft:16,ShiftRight:16};
function sendKey(code,type='keydown') {
  const event=new KeyboardEvent(type,{code,key:code.startsWith('Key')?code.slice(3):code,bubbles:true,cancelable:true,keyCode:keyCodes[code],which:keyCodes[code]});
  gameCanvas.dispatchEvent(event);
}
function tap(code) {sendKey(code);requestAnimationFrame(()=>requestAnimationFrame(()=>sendKey(code,'keyup')));gameCanvas.focus({preventScroll:true});}
function releaseTouchInputs(){
  const codes=new Set([...touchInputs.values()].map(entry=>entry.code));
  for(const {button} of touchInputs.values())button.classList.remove('is-held');
  touchInputs.clear();
  for(const code of codes)sendKey(code,'keyup');
}
for(const button of document.querySelectorAll('[data-key]')){
  button.addEventListener('contextmenu',event=>event.preventDefault());
  button.addEventListener('pointerdown',event=>{
    event.preventDefault();
    if(touchInputs.has(event.pointerId))return;
    const code=button.dataset.key;
    const alreadyHeld=[...touchInputs.values()].some(entry=>entry.code===code);
    touchInputs.set(event.pointerId,{button,code});
    button.setPointerCapture(event.pointerId);button.classList.add('is-held');
    gameCanvas.focus({preventScroll:true});
    if(!alreadyHeld)sendKey(code);
  });
  const release=event=>{
    const entry=touchInputs.get(event.pointerId);if(!entry)return;
    touchInputs.delete(event.pointerId);
    if(![...touchInputs.values()].some(other=>other.code===entry.code)){
      entry.button.classList.remove('is-held');sendKey(entry.code,'keyup');
    }
  };
  for(const type of ['pointerup','pointercancel','lostpointercapture'])button.addEventListener(type,release);
}
window.addEventListener('orientationchange',releaseTouchInputs);
function pauseAndRelease() {
  releaseTouchInputs();
  for(const code of keys)sendKey(code,'keyup');keys.clear();
  sendKey('F12');requestAnimationFrame(()=>sendKey('F12','keyup'));
}
window.addEventListener('keydown',event=>{keys.add(event.code);if(['ArrowLeft','ArrowUp','ArrowRight','ArrowDown','Space'].includes(event.code)&&document.activeElement===gameCanvas)event.preventDefault();});
window.addEventListener('keyup',event=>keys.delete(event.code));
window.addEventListener('blur',pauseAndRelease);
document.addEventListener('visibilitychange',()=>{if(document.hidden)pauseAndRelease();});
document.querySelector('#play').addEventListener('click',()=>tap('Enter'));
document.querySelector('#duel').addEventListener('click',()=>tap('KeyT'));
document.querySelector('#restart').addEventListener('click',()=>tap('KeyR'));
let muted=false;
try{muted=localStorage.getItem('palace-muted')==='true';}catch{}
function soundLabel(){document.querySelector('#sound').textContent=muted?'Sound off':'Sound on';document.querySelector('#sound').setAttribute('aria-pressed',String(muted));}
soundLabel();
document.querySelector('#sound').addEventListener('click',()=>{muted=!muted;soundLabel();try{localStorage.setItem('palace-muted',String(muted));}catch{}tap('KeyM');});
window.addEventListener('keydown',event=>{if(event.code==='KeyM'&&event.isTrusted){muted=!muted;soundLabel();try{localStorage.setItem('palace-muted',String(muted));}catch{}}});
document.querySelector('#fullscreen').addEventListener('click',async()=>{
  try{if(document.fullscreenElement)await document.exitFullscreen();else await document.querySelector('#game').requestFullscreen();gameCanvas.focus({preventScroll:true});}
  catch{engineState.textContent='FULLSCREEN UNAVAILABLE — PLAY IN WINDOW';}
});
function fail(message){loading.classList.remove('hidden');document.querySelector('#loading-text').textContent=message;loading.querySelector('small').textContent='Reload to retry. WebAssembly and WebGL are required.';engineState.textContent='ENGINE COULD NOT START';}
window.addEventListener('error',event=>fail(event.message||'The game could not start.'));
window.addEventListener('unhandledrejection',event=>fail(String(event.reason?.message??event.reason)));
for(const method of ['log','info']){
const originalLog=console[method].bind(console);
console[method]=(...args)=>{
  originalLog(...args);
  if(args.some(arg=>String(arg).includes('PALACE_GRAPHICS_READY'))){
    loading.classList.add('hidden');engineState.textContent='CINEMATIC EDITION / READY';
    requestAnimationFrame(()=>{gameCanvas.focus({preventScroll:true});if(muted)tap('KeyM');});
  }
};
}
const readyPlugin={name:'palace_shell',version:1,register_plugin(){},on_init(){
  engineState.textContent='LOADING CINEMATIC ART';
  document.querySelector('#loading-text').textContent='Lighting the dungeon…';
  loading.querySelector('small').textContent='Loading detailed environments and character animation';
}};
try{miniquad_add_plugin(readyPlugin);load('/game.wasm');}catch(error){fail(error.message);}
