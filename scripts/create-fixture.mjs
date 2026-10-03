import { mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
const root = resolve(process.argv[2] || 'fixtures/showcase');
if (existsSync(join(root, '.git'))) { console.log(`Existing fixture retained: ${root}`); process.exit(0); }
if (existsSync(root)) throw new Error('Fixture destination already exists; choose an empty new directory');
mkdirSync(join(root, 'src'), { recursive: true });
const git = (...args) => {
  const r = spawnSync('git', args, { cwd: root, encoding: 'utf8', windowsHide: true });
  if (r.status !== 0) throw new Error(r.stderr); return r.stdout.trim();
};
git('init', '-b', 'main'); git('config', 'user.name', 'TRACE Fixture'); git('config', 'user.email', 'fixture@trace.local'); git('config', 'core.autocrlf', 'false');
writeFileSync(join(root, 'package.json'), JSON.stringify({ name:'trace-regression-fixture',private:true,type:'module',scripts:{test:'node --test test.mjs'} }, null, 2));
writeFileSync(join(root,'src/parser.ts'), 'export function parseUser(user: { id: string } | null) {\n  if (!user) return null;\n  return user!.id;\n}\n');
writeFileSync(join(root,'test.mjs'), "import { test } from 'node:test';\nimport assert from 'node:assert/strict';\nimport { parseUser } from './src/parser.ts';\ntest('null user is handled safely', () => assert.equal(parseUser(null), null));\ntest('valid user retains id', () => assert.equal(parseUser({id: 'user-41'}), 'user-41'));\n");
writeFileSync(join(root,'crash.mjs'), "import { parseUser } from './src/parser.ts';\nparseUser(null);\n");
git('add','.');git('commit','-m','feat: handle missing users safely');const good=git('rev-parse','HEAD');
const passing=spawnSync(process.execPath,['--test','test.mjs'],{cwd:root,encoding:'utf8'});
if (passing.status !== 0) throw new Error(`Safe fixture test failed: ${passing.stdout}\n${passing.stderr}`);
writeFileSync(join(root,'src/parser.ts'), 'export function parseUser(user: { id: string } | null) {\n  // Normalize the user identifier\n  return user!.id.trim();\n}\n');
git('add','src/parser.ts');git('commit','-m','refactor: simplify user identifier parsing');const regression=git('rev-parse','HEAD');
writeFileSync(join(root,'README.md'),'# User identity service\nSmall real regression repository for TRACE.\n');git('add','README.md');git('commit','-m','docs: describe identity service');const head=git('rev-parse','HEAD');
const crash=spawnSync(process.execPath,['crash.mjs'],{cwd:root,encoding:'utf8',windowsHide:true});
const failing=spawnSync(process.execPath,['--test','test.mjs'],{cwd:root,encoding:'utf8',windowsHide:true});
if (crash.status===0 || failing.status===0) throw new Error('Expected a real regression failure');
mkdirSync(join(root,'evidence'));writeFileSync(join(root,'evidence/stack.txt'),crash.stderr);
writeFileSync(join(root,'evidence/test-output.txt'),`Command: node --test test.mjs\nWorking directory: ${root}\nExit code: ${failing.status}\n${failing.stdout}\n${failing.stderr}`);
writeFileSync(join(root,'evidence/manifest.json'),JSON.stringify({root,good,regression,head,passingExit:passing.status,failingExit:failing.status},null,2));
// Evidence artifacts are intentionally untracked; TRACE must handle a dirty repository.
console.log(JSON.stringify({root,good,regression,head,passingExit:passing.status,failingExit:failing.status},null,2));
