/**
 * Owner-curated VISUAL drafts only. This module has zero Porch permissions and
 * deliberately cannot encode credentials, endpoints, files or verified presence.
 * No network calls. Import/export is by deliberate local user action only.
 */
export const HALL_STORAGE_KEY = 'infinite-house.hall-draft.v1';
export const HALL_KIND = 'INFINITE_HOUSE_LOCAL_DRAFT';
export const HALL_THEMES = Object.freeze(['amber','cobalt','moss','violet']);
export const HALL_MAX_DOORS = 24;
export const HALL_MAX_BYTES = 32768;
export const HALL_LABEL_LIMITS = Object.freeze({companion:48,kingdom:64,description:240});
const idPattern = /^[a-z0-9][a-z0-9-]{1,63}$/;

const sampleDoors = [
 {id:'sample-library',companion:'Example Companion A',kingdom:'Library of Stars',description:'A private imaginary world of books and constellations.',theme:'cobalt',source:'example',connection:'UNCONNECTED'},
 {id:'sample-garden',companion:'Example Companion B',kingdom:'Garden of What-If',description:'An illustrated garden where stories and ideas could grow.',theme:'moss',source:'example',connection:'UNCONNECTED'},
 {id:'sample-workshop',companion:'Example Companion C',kingdom:'The Endless Workshop',description:'An imagined workshop for sketches, experiments, and stories.',theme:'amber',source:'example',connection:'UNCONNECTED'},
];

const object = v => v!==null && typeof v==='object' && !Array.isArray(v);
function exactKeys(value,allowed,label) {
 if(!object(value) || Object.keys(value).some(key=>!allowed.includes(key)) ||
    allowed.some(key=>!Object.hasOwn(value,key))) throw new Error(label+' has unexpected or missing fields');
}
function cleanText(input,label,limit,required=true) {
 if(typeof input!=='string' || input.length>limit*4) throw new Error('Invalid '+label);
 const result=input.trim();
 if((required&&!result)||result.length>limit || /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/.test(result)) throw new Error('Invalid '+label);
 return result;
}
function verifyDoor(value) {
 exactKeys(value,['id','companion','kingdom','description','theme','source','connection'],'Door');
 if(typeof value.id!=='string'||!idPattern.test(value.id))throw new Error('Invalid door ID');
 if(!HALL_THEMES.includes(value.theme))throw new Error('Invalid palette');
 if(value.connection!=='UNCONNECTED') throw new Error('A local draft cannot claim a live connection');
 if(!['example','owner-draft'].includes(value.source))throw new Error('Invalid source');
 return {
   id:value.id,companion:cleanText(value.companion,'companion',HALL_LABEL_LIMITS.companion),
   kingdom:cleanText(value.kingdom,'kingdom',HALL_LABEL_LIMITS.kingdom),
   description:cleanText(value.description,'description',HALL_LABEL_LIMITS.description,false),
   theme:value.theme,source:value.source,connection:'UNCONNECTED',
 };
}
export function defaultHall(){
 return {schema:1,kind:HALL_KIND,doors:sampleDoors.map(d=>({...d}))};
}
export function validateHall(input){
 if(!object(input)) throw new Error('Hall document must be an object');
 // Byte limit runs before iterating arrays to bound work.
 const raw=JSON.stringify(input);
 if(typeof raw!=='string'||new TextEncoder().encode(raw).length>HALL_MAX_BYTES)throw new Error('Hall file exceeds the 32 KiB limit');
 exactKeys(input,['schema','kind','doors'],'Hall');
 if(input.schema!==1||input.kind!==HALL_KIND)throw new Error('Unsupported hall format');
 if(!Array.isArray(input.doors)||input.doors.length>HALL_MAX_DOORS)throw new Error('Too many doors');
 const doors=input.doors.map(verifyDoor);
 if(new Set(doors.map(d=>d.id)).size!==doors.length)throw new Error('Duplicate door ID');
 return {schema:1,kind:HALL_KIND,doors};
}
export function parseHall(text) {
 if(typeof text!=='string'||new TextEncoder().encode(text).length>HALL_MAX_BYTES)throw new Error('Import file exceeds the 32 KiB limit');
 return validateHall(JSON.parse(text));
}
export function readHall(storage) {
 try {
  const saved=storage.getItem(HALL_STORAGE_KEY);
  if(saved===null)return {hall:defaultHall(),error:'',blocked:false,persisted:false};
  return {hall:parseHall(saved),error:'',blocked:false,persisted:true};
 }catch(error) {
  // Never silently overwrite an invalid saved original.
  return {hall:defaultHall(),error:'The saved Hall cannot be loaded ('+String(error)+'). The original value has been left untouched. Export or reset it explicitly before editing.',blocked:true,persisted:false};
 }
}
export function writeHall(storage, hall) {
 const value=validateHall(hall);
 storage.setItem(HALL_STORAGE_KEY,JSON.stringify(value));
 return value;
}
export function upsertDoor(hall, fields, id) {
 const current=validateHall(hall);
 const key=id ?? fields?.id;
 if(typeof key!=='string' || !idPattern.test(key))throw new Error('Invalid new door ID');
 const candidate=verifyDoor({
  id:key,companion:fields.companion,kingdom:fields.kingdom,
  description:fields.description,theme:fields.theme,
  source:'owner-draft',connection:'UNCONNECTED'
 });
 const index=current.doors.findIndex(d=>d.id===key);
 if(index===-1 && current.doors.length>=HALL_MAX_DOORS)throw new Error('Hall is at its draft limit of 24 doors');
 const doors=[...current.doors];
 if(index===-1)doors.push(candidate);else doors[index]=candidate;
 return validateHall({...current,doors});
}
export function removeDoor(hall,id) {
 const current=validateHall(hall);
 return validateHall({...current,doors:current.doors.filter(d=>d.id!==id)});
}
