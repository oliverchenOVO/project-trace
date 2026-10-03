import { defineConfig } from '@playwright/test';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
export default defineConfig({testDir:'./tests/e2e',timeout:180000,workers:1,reporter:'list',outputDir:join(tmpdir(),'trace-playwright-results'),use:{baseURL:'http://127.0.0.1:1420',viewport:{width:1536,height:1000}},webServer:{command:'npm run dev',url:'http://127.0.0.1:1420',reuseExistingServer:!process.env.CI,timeout:120000,env:{TRACE_QA:'1',TRACE_QA_DB:join(tmpdir(),'trace-showcase-qa.sqlite')}}});
