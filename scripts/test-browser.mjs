import { chromium, webkit, devices } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
const url=process.env.TEST_URL??'http://127.0.0.1:4173';
let server;
if(!process.env.TEST_URL){server=spawn(process.execPath,['scripts/serve.mjs'],{stdio:'inherit'});await new Promise(r=>setTimeout(r,1000));}
mkdirSync('test-results',{recursive:true});
const results=[];
try {
  for(const scenario of [
    {name:'desktop-chromium',engine:chromium,options:{viewport:{width:1440,height:1100}}},
    {name:'mobile-chromium',engine:chromium,options:{...devices['Pixel 7'],viewport:{width:915,height:412}}},
    {name:'mobile-webkit',engine:webkit,options:{...devices['iPhone 13'],viewport:{width:844,height:390}}},
    {name:'portrait-webkit',engine:webkit,options:{...devices['iPhone 13']}}
  ]) {
    const browser=await scenario.engine.launch({headless:true,...(scenario.engine===chromium&&process.env.CHROMIUM_PATH?{executablePath:process.env.CHROMIUM_PATH}:{})});
    try {
      const context=await browser.newContext(scenario.options);
      const page=await context.newPage();const errors=[];const logs=[];
      page.on('pageerror',e=>{errors.push(e.message);logs.push(e.stack);});
      page.on('console',m=>{logs.push(`${m.type()}: ${m.text()}`);if(m.type()==='error')errors.push(m.text());});
      const response=await page.goto(url);assert.equal(response.status(),200);
      await page.waitForFunction(()=>document.querySelector('#loading').classList.contains('hidden'),{timeout:30000});
      await page.waitForTimeout(400);
      await page.screenshot({path:`test-results/${scenario.name}-title.png`,fullPage:true});
      const before=await page.locator('canvas').screenshot();
      if(scenario.options.hasTouch) {
        const box=await page.locator('canvas').boundingBox();
        // Start button in the Rust-rendered title screen, in the 320x264 virtual viewport.
        await page.touchscreen.tap(box.x+box.width/2,box.y+box.width*(139/320));
      }else{await page.locator('#play').click();}
      await page.waitForTimeout(1100);
      const after=await page.locator('canvas').screenshot();assert.notDeepEqual(before,after,'start must change the game frame');
      await page.screenshot({path:`test-results/${scenario.name}-playing.png`,fullPage:true});
      if(!scenario.options.hasTouch){await page.keyboard.down('ArrowRight');await page.waitForTimeout(1200);await page.keyboard.up('ArrowRight');await page.keyboard.press('Escape');}
      else {
        const box=await page.locator('canvas').boundingBox();
        await page.touchscreen.tap(box.x+box.width*97/320,box.y+box.width*240/320);
      }
      await page.screenshot({path:`test-results/${scenario.name}-controls.png`,fullPage:true});
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false,'no horizontal overflow');
      writeFileSync(`test-results/${scenario.name}-console.json`,JSON.stringify(logs,null,2));
      assert.deepEqual(errors,[],`no browser errors: ${logs.join('\n')}`);
      results.push({scenario:scenario.name,status:'passed'});
      console.log(`PASS ${scenario.name}`);
      await context.close();
    }finally{await browser.close();}
  }
}finally{writeFileSync('test-results/browser-results.json',JSON.stringify(results,null,2));server?.kill();}
