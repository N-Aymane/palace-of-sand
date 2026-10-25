/* Browser persistence adapter. Rust owns shrine eligibility and all simulation state. */
'use strict';
window.palaceCheckpoint=(()=>{
  const dialog=document.querySelector('#checkpoint-dialog');
  const feedback=document.querySelector('#checkpoint-feedback');
  const saveButton=document.querySelector('#checkpoint-save');
  const loadButton=document.querySelector('#checkpoint-load');
  const slotLabel=document.querySelector('#checkpoint-slot');
  const slotKey='palace-checkpoint-v1';
  let atShrine=false;let unlocked=false;let saving=false;
  function notice(message){feedback.textContent=message;}
  function slot(){try{return JSON.parse(localStorage.getItem(slotKey)??'null');}catch{return null;}}
  function refresh(){
    const saved=slot();loadButton.disabled=!saved?.token;
    saveButton.disabled=!atShrine||!unlocked||saving;
    document.querySelector('#shrine-status').textContent=atShrine?'You are at a safe shrine.':'Find a SAVE shrine to create a checkpoint.';
    slotLabel.textContent=saved?.token?`Chapter ${saved.chapter} · ${new Date(saved.savedAt).toLocaleString()}`:'No saved checkpoint yet.';
  }
  async function api(data){
    const response=await fetch('/api/checkpoint',{method:'POST',credentials:'same-origin',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
    let result;try{result=await response.json();}catch{throw new Error('The save service is unavailable. Try again.');}
    if(!response.ok){if(response.status===401){unlocked=false;refresh();}throw new Error(result.error??'The save request failed.');}
    return result;
  }
  document.querySelector('#checkpoint-open').addEventListener('click',async()=>{
    pauseAndRelease();refresh();dialog.showModal();
    try{const response=await fetch('/api/checkpoint',{credentials:'same-origin',cache:'no-store'});const data=await response.json();
      if(!response.ok)throw new Error(data.error??'Saving is unavailable.');unlocked=data.unlocked;refresh();
      notice(unlocked?'Saving unlocked for this session.': 'Enter the code to unlock saving.');
    }catch(error){notice(error.message);}
  });
  document.querySelector('#checkpoint-close').addEventListener('click',()=>dialog.close());
  document.querySelector('#chapter-start').addEventListener('click',()=>{const code=document.querySelector('#chapter-select').value;dialog.close();tap(code);});
  dialog.addEventListener('keydown',event=>event.stopPropagation());
  document.querySelector('#checkpoint-form').addEventListener('submit',async event=>{
    event.preventDefault();const field=document.querySelector('#save-code');
    try{const code=field.value;field.value='';await api({action:'unlock',code});unlocked=true;refresh();notice('Saving unlocked for one hour.');}
    catch(error){notice(error.message);}
  });
  saveButton.addEventListener('click',()=>{if(!saveButton.disabled){saving=true;refresh();notice('Saving checkpoint…');tap('KeyS');}});
  loadButton.addEventListener('click',()=>{dialog.close();tap('KeyL');});
  async function save(source){
    try{if(!unlocked)throw new Error('Open Save / Load and enter the code first.');
      const saved=await api({action:'save',snapshot:JSON.parse(source)});
      localStorage.setItem(slotKey,JSON.stringify(saved));notice('Checkpoint saved. You can return here later.');
    }catch(error){notice(error.message);if(!dialog.open)dialog.showModal();}
    finally{saving=false;refresh();}
  }
  async function load(){
    const saved=slot();if(!saved?.token)throw new Error('No checkpoint is saved on this browser.');
    const result=await api({action:'load',token:saved.token});return JSON.stringify(result.snapshot);
  }
  function message(text){
    const saveIndex=text.indexOf('PALACE_SAVE:');
    if(saveIndex>=0){void save(text.slice(saveIndex+'PALACE_SAVE:'.length));return;}
    const status=text.match(/PALACE_STATUS:(\d+):(true|false)/);
    if(status){atShrine=status[2]==='true';refresh();document.querySelector('.frame-top>span:last-child').textContent=`CHAPTER ${status[1]} / FOUR CHAPTERS`;}
    const index=text.indexOf('PALACE_NOTICE:');
    if(index>=0){notice(text.slice(index+'PALACE_NOTICE:'.length));saving=false;refresh();}
  }
  function register(imports){
    const original=imports.env.fs_load_file;
    imports.env.fs_load_file=function(ptr,len){
      if(UTF8ToString(ptr,len)!=='/palace/checkpoint')return original(ptr,len);
      const id=FS.unique_id++;
      load().then(source=>{FS.loaded_files[id]=new TextEncoder().encode(source);wasm_exports.file_loaded(id);})
        .catch(error=>{notice(error.message);dialog.showModal();FS.loaded_files[id]=null;wasm_exports.file_loaded(id);});
      return id;
    };
  }
  refresh();return {message,register};
})();
