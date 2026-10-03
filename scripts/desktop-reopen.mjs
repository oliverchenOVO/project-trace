// Run after fully stopping and relaunching the packaged TRACE executable.
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
const deadline=Date.now()+60000;
while(true){try{const r=await fetch('http://127.0.0.1:24918/json/version',{signal:AbortSignal.timeout(2000)});if(r.ok)break;}catch{}if(Date.now()>deadline)throw Error('TRACE WebView2 endpoint unavailable');await new Promise(r=>setTimeout(r,500));}
const browser=await chromium.connectOverCDP('http://127.0.0.1:24918');
const page=browser.contexts()[0].pages()[0];
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.waitForSelector('.app-shell',{timeout:60000});
assert.equal(await page.evaluate(()=>globalThis.isTauri),true);
await page.locator('.saved-case').first().click();
await page.locator('.source-code').waitFor({timeout:90000});
assert((await page.locator('.crash-line').textContent()).includes('user!.id.trim()'));
assert(await page.locator('.evidence-card.hypothesis').count()>0,'AI hypothesis must survive a process restart');
await page.getByRole('button',{name:'Focus path',exact:true}).click();
await page.getByRole('button',{name:'Focus path',exact:true}).click();
await page.getByRole('button',{name:'Expand frames',exact:true}).click();
assert(await page.locator('.react-flow__node').count()>9);
await page.getByRole('button',{name:'Primary path',exact:true}).click();
await page.locator('.evidence-card.hypothesis').first().click();
assert((await page.locator('.inspector-body').textContent()).includes('VERIFICATION STEPS'));
assert.equal(await page.getByRole('alert').count(),0);assert.deepEqual(errors,[]);
await page.screenshot({path:join(tmpdir(),'trace-desktop-reopened.png')});
console.log(JSON.stringify({actualProcessRestart:'passed',sourceSnapshotRetained:true,hypothesisRetained:true,focusAndExpand:'passed',pageErrors:errors},null,2));
await browser.close();
