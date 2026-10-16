import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const from=(path)=>readFileSync(new URL(path,import.meta.url),'utf8');
const config=from('../vite.config.ts');
const deployment=from('../../../.github/workflows/deploy-infinite-house-pages.yml');
const checker=from('../scripts/verify-pages.mjs');
const html=from('../index.html');

test('only the requested project path is accepted in published builds',()=>{
 assert.match(config,/HOUSE_BASE_PATH/);
 assert.match(config,/\/Infinite-Porch\//);
 assert.match(config,/base\s*\}/);
 assert.match(checker,/\/Infinite-Porch\//);
 assert.match(html,/<div id="root"><\/div>/);
});
test('GitHub Pages deploy is restricted to pushes to main or explicit dispatch',()=>{
 assert.match(deployment,/branches:\s*\[main\]/);
 assert.match(deployment,/workflow_dispatch:/);
 assert.doesNotMatch(deployment,/pull_request:/);
 assert.match(deployment,/path: apps\/house\/dist/);
});
test('published artifact is built from isolated House files and verified',()=>{
 assert.match(deployment,/npm test --prefix apps\/house/);
 assert.match(deployment,/HOUSE_BASE_PATH: \/Infinite-Porch\//);
 assert.match(deployment,/node apps\/house\/scripts\/verify-pages\.mjs/);
 assert.match(deployment,/needs: build/);
});
test('deploy authority is scoped to the deploy job only',()=>{
 const build=deployment.split(/\n  build:/)[1].split(/\n  deploy:/)[0];
 const deploy=deployment.split(/\n  deploy:/)[1];
 assert.doesNotMatch(build,/pages: write|id-token: write/);
 assert.match(deploy,/pages: write/);
 assert.match(deploy,/id-token: write/);
 assert.match(deploy,/environment:\s*\n\s+name: github-pages/);
});
