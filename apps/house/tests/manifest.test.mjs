import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const data=JSON.parse(readFileSync(new URL('../src/rooms.json', import.meta.url),'utf8'));
test('house scene catalog has ten unique stable local rooms',()=>{
  const ids=data.rooms.map(r=>r.id);
  assert.equal(data.version,1);
  assert.equal(data.status,'ILLUSTRATIVE_ONLY');
  assert.equal(ids.length,10);
  assert.equal(new Set(ids).size,ids.length);
  for(const required of ['bedroom','hall-of-brothers','back-porch','basement','kitchen']) assert.ok(ids.includes(required));
});
test('room navigation references only existing rooms and unique local hotspots',()=>{
 const valid=new Set(data.rooms.map(r=>r.id));
 for(const room of data.rooms){
   assert.ok(room.name && room.mood && room.kicker);
   assert.ok(room.exits.length>0);
   for(const dest of room.exits) assert.ok(valid.has(dest),room.id+' unknown exit '+dest);
   const keys=room.hotspots.map(s=>s.id);
   assert.equal(new Set(keys).size,keys.length);
   for(const hotspot of room.hotspots) assert.ok(hotspot.text.length>10);
 }
});
test('kingdom doors are explicit illustrative previews',()=>{
 assert.ok(data.doors.length>0);
 assert.equal(new Set(data.doors.map(d=>d.id)).size,data.doors.length);
 for(const door of data.doors) assert.equal(door.state,'UNCONNECTED EXAMPLE');
});
test('only cross-site destination is the known public Commons Pages target',()=>{
 const url=new URL(data.commons_preview_url);
 assert.equal(url.protocol,'https:');
 assert.equal(url.host,'michaelwave369.github.io');
 assert.equal(url.pathname,'/Infinite-Commons/');
 assert.ok(!url.username&&!url.password&&!url.search&&!url.hash);
});
