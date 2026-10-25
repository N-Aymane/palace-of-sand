import { chromium, webkit, devices } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
const url=process.env.TEST_URL??'http://127.0.0.1:4173';
const targetOrigin=new URL(url).origin;
const protectedDeployment=process.env.TEST_PROTECTED==='1';
if(protectedDeployment&&new URL(url).hostname.endsWith('.vercel.app')&&!process.env.VERCEL_OIDC_TOKEN){
  try{process.loadEnvFile('.env.local');}catch{}
}
let server;
if(!process.env.TEST_URL){server=spawn(process.execPath,['scripts/serve.mjs'],{stdio:'inherit',env:{...process.env,SAVE_ACCESS_CODE:'browser-test-save-code',SAVE_SIGNING_KEY:'browser-test-signing-key-not-for-production'}});await new Promise(r=>setTimeout(r,1000));}
mkdirSync('test-results',{recursive:true});
const results=[];
try {
  for(const scenario of [
    {name:'desktop-chromium',engine:chromium,options:{viewport:{width:1440,height:1100}}},
    {name:'portrait-chromium',engine:chromium,options:{...devices['Pixel 7'],viewport:{width:393,height:852}}},
    {name:'mobile-chromium',engine:chromium,options:{...devices['Pixel 7'],viewport:{width:915,height:412}}},
    {name:'mobile-webkit',engine:webkit,options:{...devices['iPhone 13'],viewport:{width:844,height:390}}},
    {name:'portrait-webkit',engine:webkit,options:{...devices['iPhone 13']}}
  ]) {
    if(process.env.TEST_SCENARIOS&&!process.env.TEST_SCENARIOS.split(',').includes(scenario.name))continue;
    const browser=await scenario.engine.launch({headless:true,...(scenario.engine===chromium&&process.env.CHROMIUM_PATH?{executablePath:process.env.CHROMIUM_PATH}:{})});
    try {
      const context=await browser.newContext(scenario.options);
      if(protectedDeployment&&new URL(url).hostname.endsWith('.vercel.app')&&process.env.VERCEL_OIDC_TOKEN){
        await context.route(`${targetOrigin}/**`,route=>route.continue({headers:{...route.request().headers(),'x-vercel-trusted-oidc-idp-token':process.env.VERCEL_OIDC_TOKEN}}));
      }
      const page=await context.newPage();page.setDefaultTimeout(60000);const errors=[];const logs=[];
      // Capture the live canvas without waiting for its animation loop to become stable.
      const captureCanvas=async()=>page.screenshot({clip:await page.locator('canvas').boundingBox(),timeout:60000});
      page.on('pageerror',e=>{errors.push(e.message);logs.push(e.stack);});
      page.on('console',m=>{logs.push(`${m.type()}: ${m.text()}`);
        // A wrong unlock code deliberately returns 401 and is asserted in the UI below.
        const expectedDenial=m.location().url.includes('/api/checkpoint')&&m.text().includes('401');
        if(m.type()==='error'&&!expectedDenial)errors.push(m.text());});
      const response=await page.goto(url);assert.equal(response.status(),200);
      try { await page.waitForFunction(()=>document.querySelector('#loading').classList.contains('hidden'),{timeout:30000}); }
      catch(error){
        writeFileSync(`test-results/${scenario.name}-startup-console.json`,JSON.stringify(logs,null,2));
        await page.screenshot({path:`test-results/${scenario.name}-startup-failure.png`,fullPage:true}).catch(()=>{});
        throw error;
      }
      await page.waitForTimeout(400);
      if(scenario.options.hasTouch){
        const layout=await page.evaluate(()=>{
          const game=document.querySelector('#game').getBoundingClientRect();
          const canvas=document.querySelector('canvas').getBoundingClientRect();
          const deck=document.querySelector('.handheld-controls').getBoundingClientRect();
          return {game:{x:game.x,y:game.y,width:game.width,height:game.height},canvas:{width:canvas.width,height:canvas.height,bottom:canvas.bottom},deck:{top:deck.top,bottom:deck.bottom},width:innerWidth,height:innerHeight,overflow:document.documentElement.scrollHeight>innerHeight+1};
        });
        assert.ok(Math.abs(layout.game.height-layout.height)<2,'handheld fills webview height');
        assert.ok(Math.abs(layout.game.width-layout.width)<2,'handheld fills webview width');
        assert.ok(Math.abs(layout.canvas.width/layout.canvas.height-1.6)<.02,'game keeps aspect ratio');
        assert.ok(layout.deck.top>=layout.canvas.bottom,'controls stay below gameplay');
        assert.ok(layout.deck.bottom>=layout.height-2&&layout.deck.bottom<=layout.height+1,'controls reach bottom of viewport');
        assert.equal(layout.overflow,false,'no vertical page scroll in handheld mode');
        if(scenario.engine===chromium){
          const session=await context.newCDPSession(page);
          const points=await Promise.all(['ArrowRight','KeyZ'].map(async key=>{
            const b=await page.locator(`[data-key=${key}]`).boundingBox();return {x:b.x+b.width/2,y:b.y+b.height/2};
          }));
          await session.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:points});
          assert.equal(await page.locator('[data-key].is-held').count(),2,'simultaneous direction and action');
          await session.send('Input.dispatchTouchEvent',{type:'touchCancel',touchPoints:[]});
          assert.equal(await page.locator('[data-key].is-held').count(),0,'cancel releases every control');
          await session.detach();
        }

      }

      await page.screenshot({path:`test-results/${scenario.name}-title.png`,fullPage:true});
      const before=await captureCanvas();
      if(scenario.options.hasTouch) {
        const box=await page.locator('canvas').boundingBox();
        // Start button in the Rust-rendered title screen, in the 320x200 game viewport.
        await page.touchscreen.tap(box.x+box.width/2,box.y+box.width*(139/320));
      }else{await page.locator('#play').click();}
      await page.waitForTimeout(1100);
      const after=await captureCanvas();assert.notDeepEqual(before,after,'start must change the game frame');
      await page.screenshot({path:`test-results/${scenario.name}-playing.png`,fullPage:true});
      if(!scenario.options.hasTouch){await page.keyboard.down('ArrowRight');await page.waitForTimeout(1200);await page.keyboard.up('ArrowRight');await page.keyboard.press('Escape');}
      else {
        await page.locator('[data-key=ArrowRight]').tap();
      }
      await page.screenshot({path:`test-results/${scenario.name}-controls.png`,fullPage:true});
      if(scenario.engine===chromium&&scenario.name!=='portrait-chromium') {
        const combatStart=logs.length;
        const waitCombat=async(predicate,message)=>{
          for(let i=0;i<600;i++) {
            if(predicate(logs.slice(combatStart)))return;
            await page.waitForTimeout(100);
          }
          writeFileSync(`test-results/${scenario.name}-console.json`,JSON.stringify(logs,null,2));
          await page.screenshot({path:`test-results/${scenario.name}-combat-failure.png`,fullPage:true});
          assert.fail(message+'\n'+logs.slice(combatStart).join('\n'));
        };
        await page.locator('#duel').click();
        await page.waitForTimeout(300);
        let touch;
        const touchButton=async(x,pressed)=>{
          const box=await page.locator(x===217?'[data-key=KeyX]':'[data-key=KeyZ]').boundingBox();
          touch??=await context.newCDPSession(page);
          await touch.send('Input.dispatchTouchEvent',{type:pressed?'touchStart':'touchEnd',
            touchPoints:pressed?[{x:box.x+box.width/2,y:box.y+box.height/2}]:[]});
        };
        if(scenario.options.hasTouch)await touchButton(217,true);
        else await page.keyboard.down('x');
        await waitCombat(lines=>lines.some(s=>s.includes('COMBAT_BLOCK')||s.includes('COMBAT_PARRY')),'defence must stop a real guard strike');
        await page.screenshot({path:`test-results/${scenario.name}-parry.png`,fullPage:true});
        if(scenario.options.hasTouch){await touchButton(217,false);await touchButton(282,true);}
        else{await page.keyboard.up('x');await page.keyboard.down('z');}
        await waitCombat(lines=>lines.filter(s=>s.includes('COMBAT_HIT')).length>=3,'holding attack must hit and defeat the guard');
        if(scenario.options.hasTouch)await touchButton(282,false);else await page.keyboard.up('z');
        await page.screenshot({path:`test-results/${scenario.name}-combat.png`,fullPage:true});
      }
      if(scenario.options.hasTouch){
        await page.locator('#checkpoint-open').tap();
        const bounds=await page.locator('#checkpoint-dialog').boundingBox();
        assert.ok(bounds.x>=0&&bounds.y>=0&&bounds.x+bounds.width<=scenario.options.viewport.width+1,'mobile save menu fits viewport width');
        assert.ok(bounds.height<=scenario.options.viewport.height*.91,'mobile save menu scrolls within the viewport');
        await page.screenshot({path:`test-results/${scenario.name}-save-menu.png`,fullPage:true});
        await page.locator('#checkpoint-close').tap();
      }
      if(scenario.name==='desktop-chromium'){
        await page.locator('#checkpoint-open').click();
        await page.locator('#chapter-select').selectOption('Digit2');
        await page.locator('#chapter-start').click();
        for(let wait=0;wait<100&&!logs.some(line=>line.includes('PALACE_STATUS:2:true'));wait++)await page.waitForTimeout(100);
        assert.ok(logs.some(line=>line.includes('PALACE_STATUS:2:true')),'chapter selector starts at a real Rust save shrine');
        await page.locator('#checkpoint-open').click();
        await page.screenshot({path:'test-results/checkpoint-menu.png',fullPage:true});
        if(!process.env.TEST_URL||process.env.TEST_SAVE_CODE){
          await page.locator('#save-code').fill('incorrect-code');
          await page.locator('#checkpoint-form button[type=submit]').click();
          await page.waitForFunction(()=>document.querySelector('#checkpoint-feedback').textContent.includes('Incorrect'));
          assert.equal(await page.locator('#checkpoint-save').isEnabled(),false);
          await page.locator('#save-code').fill(process.env.TEST_SAVE_CODE??'browser-test-save-code');
          await page.locator('#checkpoint-form button[type=submit]').click();
          await page.waitForFunction(()=>!document.querySelector('#checkpoint-save').disabled);
          await page.locator('#checkpoint-save').click();
          await page.waitForFunction(()=>document.querySelector('#checkpoint-feedback').textContent.includes('Checkpoint saved'),{timeout:30000});
          await page.screenshot({path:'test-results/checkpoint-saved.png',fullPage:true});
          await page.locator('#checkpoint-load').click();
          for(let wait=0;wait<150&&!logs.some(line=>line.includes('PALACE_NOTICE:Checkpoint restored'));wait++)await page.waitForTimeout(100);
          assert.ok(logs.some(line=>line.includes('PALACE_NOTICE:Checkpoint restored')),'sealed browser slot loads back into the Rust simulation');
          await page.reload();await page.waitForFunction(()=>document.querySelector('#loading').classList.contains('hidden'));
          await page.locator('#checkpoint-open').click();await page.locator('#checkpoint-load').click();
          for(let wait=0;wait<150&&logs.filter(line=>line.includes('PALACE_NOTICE:Checkpoint restored')).length<2;wait++)await page.waitForTimeout(100);
          assert.ok(logs.filter(line=>line.includes('PALACE_NOTICE:Checkpoint restored')).length>=2,'checkpoint survives page reload');
        }else await page.locator('#checkpoint-close').click();
      }
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false,'no horizontal overflow');
      writeFileSync(`test-results/${scenario.name}-console.json`,JSON.stringify(logs,null,2));
      assert.deepEqual(errors,[],`no browser errors: ${logs.join('\n')}`);
      results.push({scenario:scenario.name,status:'passed'});
      console.log(`PASS ${scenario.name}`);
      await context.close();
    }finally{await browser.close();}
  }
}finally{writeFileSync('test-results/browser-results.json',JSON.stringify(results,null,2));server?.kill();}
