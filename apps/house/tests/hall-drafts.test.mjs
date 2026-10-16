import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {
  HALL_STORAGE_KEY,HALL_MAX_DOORS,HALL_MAX_BYTES, defaultHall,
  validateHall,parseHall,readHall,writeHall,upsertDoor,removeDoor,
} from '../src/hallStore.mjs';
const manifest=JSON.parse(readFileSync(new URL('../src/rooms.json',import.meta.url),'utf8'));

function storage(initial=null) {
 const values=new Map(initial===null?[]:[[HALL_STORAGE_KEY,initial]]);
 return {getItem:key=>values.get(key)??null,setItem:(key,value)=>{values.set(key,value)},removeItem:key=>values.delete(key),raw:()=>values.get(HALL_STORAGE_KEY)};
}
const make={companion:'Test Companion',kingdom:'The Luminous Garden',description:'A peaceful imaginary world.',theme:'moss'};
test('starter Hall reuses the P0 sample kingdom names without claiming real AI presence',()=>{
 const hall=defaultHall();
 assert.equal(hall.doors.length,manifest.doors.length);
 assert.deepEqual(hall.doors.map(d=>d.kingdom),manifest.doors.map(d=>d.label));
 assert.ok(hall.doors.every(d=>d.connection==='UNCONNECTED'&&d.source==='example'));
});
test('doors can be added, redesigned, removed, and exported losslessly as strict local-only drafts',()=>{
 const original=defaultHall(),id='door-test-123';
 const added=upsertDoor(original,make,id);
 assert.equal(added.doors.length,original.doors.length+1);
 assert.equal(original.doors.length,3,'immutable updates cannot modify canonical starting examples');
 const changed=upsertDoor(added,{...make,kingdom:'Garden of Brass'},id);
 assert.equal(changed.doors.length,added.doors.length);
 assert.equal(changed.doors.find(d=>d.id===id).kingdom,'Garden of Brass');
 assert.equal(changed.doors.find(d=>d.id===id).connection,'UNCONNECTED');
 assert.deepEqual(parseHall(JSON.stringify(changed)),changed);
 assert.equal(removeDoor(changed,id).doors.length,original.doors.length);
});
test('malformed localStorage is not replaced automatically or treated as user consent',()=>{
 const fake=storage('{"schema":999}'),result=readHall(fake);
 assert.equal(result.blocked,true);assert.equal(result.persisted,false);
 assert.equal(fake.raw(),'{"schema":999}');
 assert.match(result.error,/original value has been left untouched/);
});
test('well-formed local-only drafts persist on one browser profile, not a remote Porch',()=>{
 const fake=storage();
 assert.equal(readHall(fake).persisted,false);
 const hall=upsertDoor(defaultHall(),make,'door-test-123');
 writeHall(fake,hall);
 assert.equal(readHall(fake).persisted,true);
 assert.deepEqual(readHall(fake).hall,hall);
});
test('imports reject extra fields, authority claims, external endpoints, and duplicate IDs',()=>{
 const hall=defaultHall();
 assert.throws(()=>validateHall({...hall,operatorToken:'secret'}),/unexpected/);
 assert.throws(()=>validateHall({...hall,doors:[{...hall.doors[0],apiKey:'secret'}]}),/unexpected/);
 assert.throws(()=>validateHall({...hall,doors:[{...hall.doors[0],connection:'LIVE'}]}),/live connection/);
 assert.throws(()=>validateHall({...hall,doors:[...hall.doors,hall.doors[0]]}),/Duplicate/);
 assert.throws(()=>parseHall('not json'));
 assert.throws(()=>parseHall('x'.repeat(HALL_MAX_BYTES+1)),/limit/);
});
test('hall bounds, palette allowlist, and 24-door cap cannot be bypassed by the UI',()=>{
 let hall=defaultHall();
 assert.throws(()=>upsertDoor(hall,{...make,companion:'x'.repeat(250)},'door-fail-1'),/Invalid/);
 assert.throws(()=>upsertDoor(hall,{...make,theme:'javascript:bad'},'door-fail-2'),/palette/);
 assert.throws(()=>upsertDoor(hall,make,'../../secrets'),/ID/);
 for(let i=hall.doors.length;i<HALL_MAX_DOORS;i++)hall=upsertDoor(hall,make,'door-bounded-'+i);
 assert.equal(hall.doors.length,HALL_MAX_DOORS);
 assert.throws(()=>upsertDoor(hall,make,'door-overflow'),/limit/);
});
test('Hall editor is presentation-only and has no Porch credential or remote API transport',()=>{
 const files=['../src/HallOfBrothers.tsx','../src/hallStore.mjs'];
 for(const file of files) {
  const code=readFileSync(new URL(file,import.meta.url),'utf8');
  assert.doesNotMatch(code,/\bfetch\s*\(|XMLHttpRequest|WebSocket|PorchClient|client\.issue|model\.inference/);
 }
 const editor=readFileSync(new URL('../src/HallOfBrothers.tsx',import.meta.url),'utf8');
 assert.match(editor,/local storage/i);
 assert.match(editor,/UNCONNECTED/);
 assert.match(editor,/accept="application\/json,\.json"/);
});
