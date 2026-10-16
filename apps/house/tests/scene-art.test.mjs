import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';

const path = (p) => new URL(p,import.meta.url);
const manifest=JSON.parse(readFileSync(path('../src/rooms.json'),'utf8'));
const art=readFileSync(path('../src/RoomScene.tsx'),'utf8');
const motion=readFileSync(path('../src/scene-art.css'),'utf8');
const main=readFileSync(path('../src/main.tsx'),'utf8');

test('every house room has an independently designed original SVG scene',()=>{
 for(const room of manifest.rooms){
   assert.ok(art.includes(`case '${room.id}':`),`No illustrated composition for ${room.id}`);
 }
 const count=(art.match(/case '[a-z-]+':/g)||[]).length;
 assert.equal(count,manifest.rooms.length,'Do not silently reuse a generic room for a missing scene');
});
test('scene artwork cannot initiate an API action or fetch remote assets',()=>{
 assert.doesNotMatch(art,/fetch\s*\(|XMLHttpRequest|WebSocket|https?:\/\/|localStorage|Authorization|PorchClient/);
 assert.doesNotMatch(motion,/url\(\s*['"]?https?:|@import/);
 assert.match(art,/Decorative only/);
});
test('ambient motion is user controllable with reduced-motion fallback',()=>{
 assert.match(main,/aria-pressed=\{motionOn\}/);
 assert.match(main,/onClick=\{\(\) => setMotionOn/);
 assert.match(art,/data-motion=\{motionOn\?'on':'off'\}/);
 assert.match(motion,/\[data-motion="off"\]/);
 assert.match(motion,/@media\(prefers-reduced-motion:reduce\)/);
});
test('Hall doors remain illustrative and Commons remains an external public preview',()=>{
 assert.match(main,/UNCONNECTED EXAMPLE|manifest\.doors/);
 assert.match(main,/rel="noopener noreferrer"/);
 assert.match(main,/no shared session/);
});
