// Connect to a TRACE process launched with WebView2 remote debugging enabled.
import {chromium} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
import assert from 'node:assert/strict';
const root=resolve('fixtures/showcase'),m=JSON.parse(readFileSync(join(root,'evidence/manifest.json'),'utf8'));
const deadline=Date.now()+60000;
while(true){try{const r=await fetch('http://127.0.0.1:24918/json/version',{signal:AbortSignal.timeout(2000)});if(r.ok)break;}catch{}if(Date.now()>deadline)throw Error('TRACE WebView2 debug endpoint did not become ready within 60 seconds');await new Promise(r=>setTimeout(r,500));}
const browser=await chromium.connectOverCDP('http://127.0.0.1:24918',{timeout:60000});
const page=browser.contexts()[0].pages()[0];assert(page,'TRACE WebView2 page must exist');
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.waitForSelector('.app-shell',{timeout:60000});assert.equal(await page.title(),'TRACE — AI Bug Crime Scene');
assert.equal(await page.evaluate(()=>globalThis.isTauri),true);
await page.getByRole('button',{name:'Enter repository path'}).click();await page.getByRole('textbox',{name:'Repository path'}).fill(root);await page.getByRole('button',{name:'Open repository',exact:true}).click();
await page.getByRole('dialog',{name:'Open an investigation'}).waitFor({timeout:90000});
await page.getByRole('textbox',{name:'Stack trace'}).fill(readFileSync(join(root,'evidence/stack.txt'),'utf8'));
await page.getByRole('button',{name:'Test output · optional'}).click();await page.getByRole('textbox',{name:'Test output'}).fill(readFileSync(join(root,'evidence/test-output.txt'),'utf8'));await page.getByRole('button',{name:'Build evidence graph'}).click();
await page.locator('.source-code').waitFor({timeout:180000});assert((await page.locator('.crash-line').textContent()).includes('user!.id.trim()'));
await page.screenshot({path:join(tmpdir(),'trace-desktop-source.png')});
await page.locator('.react-flow__node').filter({hasText:'Last changed line'}).click();assert((await page.locator('.inspector-body').textContent()).includes(m.regression));
await page.locator('.react-flow__node').filter({hasText:'Actual commit patch'}).click();assert((await page.locator('.patch').textContent()).includes('-  if (!user) return null;'));
await page.getByRole('button',{name:'Provenance',exact:true}).click();assert((await page.locator('.inspector-body').textContent()).includes('--no-ext-diff'));
if(process.env.TRACE_AI_MODEL){
 await page.getByRole('button',{name:'AI context',exact:true}).click();await page.getByRole('textbox',{name:'Ollama model'}).fill(process.env.TRACE_AI_MODEL);await page.getByRole('button',{name:'Generate hypothesis',exact:true}).click();
 await page.locator('.evidence-card.hypothesis').first().waitFor({timeout:150000});await page.locator('.evidence-card.hypothesis').first().click();assert((await page.locator('.inspector-body').textContent()).includes('VERIFICATION STEPS'));
 await page.screenshot({path:join(tmpdir(),'trace-desktop-hypothesis.png')});
}
assert((await page.locator('.unverified').textContent()).includes('Root cause unverified'));assert.equal(await page.getByRole('alert').count(),0);assert.deepEqual(errors,[]);
await page.reload();await page.locator('.saved-case').first().click();await page.locator('.source-code').waitFor({timeout:90000});
assert.equal(await page.getByRole('alert').count(),0);
console.log(JSON.stringify({desktop:'passed real WebView2 / Tauri IPC workflow',url:page.url(),regression:m.regression,savedCaseReopened:true,aiModel:process.env.TRACE_AI_MODEL||null,pageErrors:errors,screenshot:join(tmpdir(),'trace-desktop-source.png')},null,2));
await browser.close();
