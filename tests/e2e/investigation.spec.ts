import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
const root=resolve('fixtures/showcase');
const stack=readFileSync(join(root,'evidence/stack.txt'),'utf8');
const output=readFileSync(join(root,'evidence/test-output.txt'),'utf8');
const manifest=JSON.parse(readFileSync(join(root,'evidence/manifest.json'),'utf8'));
test('real native golden path: source → blame → commit → patch → saved case',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto('/');await expect(page).toHaveTitle('TRACE — AI Bug Crime Scene');await expect(page.getByText('Every bug leaves a trace.')).toBeVisible();
  await page.screenshot({path:join(tmpdir(),'trace-welcome.png')});
  await page.getByRole('button',{name:'Enter repository path'}).click();await page.getByRole('textbox',{name:'Repository path'}).fill(root);await page.getByRole('button',{name:'Open repository',exact:true}).click();
  await expect(page.getByRole('dialog',{name:'Open an investigation'})).toBeVisible({timeout:90000});
  await page.getByRole('textbox',{name:'Stack trace'}).fill(stack);
  await page.getByRole('button',{name:'Test output · optional'}).click();await page.getByRole('textbox',{name:'Test output'}).fill(output);
  await page.getByRole('button',{name:'Build evidence graph'}).click();
  await expect(page.locator('.source-code')).toBeVisible({timeout:120000});await expect(page.locator('.crash-line')).toContainText('return user!.id.trim();');
  await expect(page.getByRole('alert')).toHaveCount(0);await expect(page.locator('.unverified')).toContainText('Root cause unverified');
  await page.screenshot({path:join(tmpdir(),'trace-investigation.png')});
  await page.locator('.react-flow__node').filter({hasText:'Last changed line'}).click();await expect(page.locator('.inspector-body')).toContainText(manifest.regression);await expect(page.locator('.warning-box')).toContainText('Blame does not prove causation.');
  await page.locator('.react-flow__node').filter({hasText:'refactor: simplify user identifier parsing'}).click();await expect(page.locator('.inspector-body')).toContainText(manifest.regression);
  await page.locator('.react-flow__node').filter({hasText:'Actual commit patch'}).click();await expect(page.locator('.patch')).toContainText('-  if (!user) return null;');await expect(page.locator('.patch')).toContainText('+  return user!.id.trim();');
  await page.getByRole('button',{name:'Provenance',exact:true}).click();await expect(page.locator('.inspector-body')).toContainText('"--no-ext-diff"');
  await page.locator('.react-flow__node').filter({hasText:'Attached test output'}).click();await expect(page.locator('.inspector-body')).toContainText('TRACE did not execute this command');
  await page.getByRole('button',{name:'AI context',exact:true}).click();await expect(page.getByRole('dialog',{name:'Review AI context'})).toBeVisible({timeout:30000});await expect(page.locator('.context-preview')).toContainText('Actual commit patch');await page.getByRole('button',{name:'Close AI context'}).click();
  await page.reload();await page.locator('.saved-case').first().click();await expect(page.locator('.source-code')).toBeVisible({timeout:90000});await expect(page.getByRole('alert')).toHaveCount(0);expect(errors).toEqual([]);
  await page.setViewportSize({width:1024,height:768});
  await expect.poll(()=>page.evaluate(()=>{const pane=document.querySelector('.flow-stage')!.getBoundingClientRect();return [...document.querySelectorAll('.react-flow__node')].every(n=>{const r=n.getBoundingClientRect();return r.left>=pane.left-2&&r.right<=pane.right+2&&r.top>=pane.top-2&&r.bottom<=pane.bottom+2;});})).toBe(true);
  await page.screenshot({path:join(tmpdir(),'trace-1024.png')});await expect(page.locator('.inspector')).toBeVisible();
  await page.setViewportSize({width:768,height:900});await page.screenshot({path:join(tmpdir(),'trace-768.png')});await expect(page.locator('.case-rail')).toBeVisible();
});
test('invalid repository returns an actionable error without evidence',async({page})=>{
  await page.goto('/');await page.getByRole('button',{name:'Enter repository path'}).click();await page.getByRole('textbox',{name:'Repository path'}).fill(resolve('src'));await page.getByRole('button',{name:'Open repository',exact:true}).click();
  // A subfolder of a repository resolves to its actual Git root, so use a missing folder instead.
  await expect(page.getByRole('dialog',{name:'Open an investigation'})).toBeVisible({timeout:90000});await page.getByRole('button',{name:'Close investigation'}).click();
  await page.getByRole('button',{name:'Enter repository path'}).click();await page.getByRole('textbox',{name:'Repository path'}).fill('Z:\\trace-nonexistent-folder');await page.getByRole('button',{name:'Open repository',exact:true}).click();await expect(page.getByRole('alert')).toContainText('Repository folder unavailable');
});
test('real local AI cites existing evidence and remains an unverified hypothesis',async({page})=>{
  test.skip(!process.env.TRACE_AI_MODEL,'Set TRACE_AI_MODEL to an installed local model to exercise actual inference');
  test.setTimeout(300000);
  await page.goto('/');await page.locator('.saved-case').first().click();await expect(page.locator('.source-code')).toBeVisible({timeout:90000});
  await page.getByRole('button',{name:'AI context',exact:true}).click();await page.getByRole('textbox',{name:'Ollama model'}).fill(process.env.TRACE_AI_MODEL!);await page.getByRole('button',{name:'Generate hypothesis',exact:true}).click();
  await expect(page.locator('.evidence-card.hypothesis').first()).toBeVisible({timeout:150000});
  await page.locator('.evidence-card.hypothesis').first().click();await expect(page.locator('.inspector-body')).toContainText('AI HYPOTHESIS');await expect(page.locator('.inspector-body')).toContainText('VERIFICATION STEPS');await expect(page.locator('.unverified')).toContainText('Root cause unverified');await expect(page.getByRole('alert')).toHaveCount(0);
  await page.screenshot({path:join(tmpdir(),'trace-hypothesis.png')});
});
