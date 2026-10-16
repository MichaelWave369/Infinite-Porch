#!/usr/bin/env node
/**
 * Fail CI before uploading if the built public site cannot resolve its assets
 * beneath the case-sensitive Infinite-Porch project path.
 */
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root=resolve(fileURLToPath(new URL('..',import.meta.url)));
const dist=join(root,'dist');
const indexPath=join(dist,'index.html');
const prefix='/Infinite-Porch/';
assert.ok(existsSync(indexPath),'Missing dist/index.html');
const html=readFileSync(indexPath,'utf8');
assert.match(html,/<div id="root"><\/div>/);
assert.doesNotMatch(html,/(?:src|href)=["']\/(?:src|assets)\//);
const references=[...html.matchAll(/\b(?:src|href)=["']([^"']+)["']/g)].map(match=>match[1]);
const assets=references.filter(ref=>/\.(?:js|css)(?:\?|$)/.test(ref));
assert.ok(assets.some(ref=>ref.endsWith('.js')),'Bundled JavaScript is missing');
assert.ok(assets.some(ref=>ref.endsWith('.css')),'Bundled stylesheet is missing');
for(const ref of references) {
  if(ref.startsWith('data:'))continue;
  assert.ok(ref.startsWith(prefix),'Asset outside project path: '+ref);
  const asset=ref.slice(prefix.length).split('?')[0];
  assert.ok(!asset.includes('..') && !asset.startsWith('/'),'Unsafe asset path: '+ref);
  assert.ok(existsSync(join(dist,asset)),'Missing generated asset '+asset);
}
const publicFiles=[];
function walk(directory){
 for(const entry of readdirSync(directory)){
  const full=join(directory,entry);
  if(statSync(full).isDirectory()) walk(full);
  else publicFiles.push(relative(dist,full).replaceAll('\\','/'));
 }
}
walk(dist);
for(const file of publicFiles){
 assert.ok(file==='index.html'||/^assets\/[a-zA-Z0-9_.-]+\.(js|css)$/.test(file),'Unexpected public file '+file);
}
const jsFiles=publicFiles.filter(file=>file.endsWith('.js'));
for(const file of jsFiles){
 const source=readFileSync(join(dist,file),'utf8');
 assert.doesNotMatch(source,/\/v1\/control|\/v1\/grants|api\.token|PRIVATE-NO-AUTHORITY-SENTINEL/,'Forbidden Porch control code in public bundle: '+file);
}
console.log(JSON.stringify({
  verdict:'PASS',
  base:prefix,
  files:publicFiles,
  assetCount:assets.length,
  note:'Public static illustration and local browser drafts only; no Porch identity/auth/session integration.'
},null,2));
