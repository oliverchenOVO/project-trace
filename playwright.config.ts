import { defineConfig } from '@playwright/test';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
export default defineConfig({testDir:'./tests/e2e',timeout:240000,workers:1,reporter:'list',outputDir:join(tmpdir(),'trace-playwright-results'),use:{baseURL:'http://127.0.0.1:24917',viewport:{width:1536,height:1000},launchOptions:{args:['--disable-gpu']}},webServer:{command:'node scripts/qa-server.mjs',url:'http://127.0.0.1:24917',reuseExistingServer:false,timeout:120000,env:{TRACE_QA_DB:join(tmpdir(),'trace-showcase-qa.sqlite')}}});
