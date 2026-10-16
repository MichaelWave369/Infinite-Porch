import {useRef,useState} from 'react';
import {
  defaultHall,readHall,writeHall,parseHall,upsertDoor,removeDoor,
  HALL_MAX_BYTES,HALL_MAX_DOORS,HALL_STORAGE_KEY,HALL_THEMES,
} from './hallStore.mjs';
import type {HallDoor,HallLoad,HallManifest,HallTheme} from './hallStore.mjs';
import './hall-of-brothers.css';

type Draft = {companion:string;kingdom:string;description:string;theme:HallTheme};
const blank:Draft={companion:'',kingdom:'',description:'',theme:'amber'};
function opening():HallLoad {
 try{return readHall(window.localStorage);}
 catch {return {hall:defaultHall(),blocked:true,persisted:false,error:'Browser storage is unavailable. The displayed examples are not saved; changes are disabled until storage is accessible.'};}
}

function saveDownload(text:string,name:string,type='application/json'){
 const data=new Blob([text],{type});
 const url=URL.createObjectURL(data);
 const link=document.createElement('a');link.href=url;link.download=name;link.click();
 window.setTimeout(()=>URL.revokeObjectURL(url),1000);
}

export function HallOfBrothers(){
 const [state,setState]=useState<HallLoad>(opening);
 const [draft,setDraft]=useState<Draft>({...blank});
 const [editingId,setEditingId]=useState<string|null>(null);
 const [selectedId,setSelectedId]=useState<string|null>(null);
 const [notice,setNotice]=useState('');
 const [importError,setImportError]=useState('');
 const chooser=useRef<HTMLInputElement>(null);
 const selected=state.hall.doors.find(d=>d.id===selectedId)??null;
 const storageWarning=state.error;

 function persist(next:HallManifest):boolean{
  try{
    const checked=writeHall(window.localStorage,next);
    setState({hall:checked,blocked:false,persisted:true,error:''});
    setNotice('Your Hall draft was saved in this browser only. Other devices will not see it.');
    return true;
  }catch(err){
    setNotice('Changes were NOT saved: '+String(err)+'. You can export the current draft as a file.');
    setState((prev)=>({...prev,hall:next,persisted:false}));
    return false;
  }
 }
 function openEdit(door:HallDoor){
  setEditingId(door.id);
  setDraft({companion:door.companion,kingdom:door.kingdom,description:door.description,theme:door.theme});
  setNotice('Editing '+door.kingdom+'. No model or network connection will be created.');
 }
 function resetDraft(){
  if(!window.confirm('Replace this browser\'s Hall draft with the three examples? Export a backup first if you want to preserve your doors.')) return;
  try{
   window.localStorage.removeItem(HALL_STORAGE_KEY);
   setState({hall:defaultHall(),error:'',persisted:false,blocked:false});
   setEditingId(null);setSelectedId(null);setDraft({...blank});
   setNotice('Local draft reset to example doors. No remote data was changed.');
  }catch(err){setNotice('Reset failed: '+String(err));}
 }
 function downloadRaw(){
   try{
    const raw=window.localStorage.getItem(HALL_STORAGE_KEY);
    if(raw!==null)saveDownload(raw,'infinite-house-hall-unparsed-backup.txt','text/plain');
    else setNotice('No saved raw Hall value exists.');
   }catch(err){setNotice('Could not read stored backup: '+String(err));}
 }
 function saveForm(event:React.FormEvent<HTMLFormElement>){
  event.preventDefault();
  if(state.blocked){setNotice('Back up and reset the invalid saved entry before changing the Hall.');return;}
  try{
   const id=editingId ?? 'door-'+window.crypto.randomUUID();
   const next=upsertDoor(state.hall,draft,id);
   persist(next);
   setSelectedId(id);setEditingId(null);setDraft({...blank});
  }catch(err){setNotice('Door not changed: '+String(err));}
 }
 function deleteDoor(door:HallDoor){
  if(state.blocked)return;
  if(!window.confirm('Remove '+door.kingdom+' from the local Hall draft? This cannot alter or delete any AI account or external world.'))return;
  persist(removeDoor(state.hall,door.id));
  if(selectedId===door.id)setSelectedId(null);
  if(editingId===door.id){setEditingId(null);setDraft({...blank});}
 }
 async function importHall(event:React.ChangeEvent<HTMLInputElement>){
  const file=event.target.files?.[0]; event.target.value='';
  if(!file)return;
  try{
   if(file.size>HALL_MAX_BYTES)throw new Error('File exceeds '+HALL_MAX_BYTES+' bytes');
   const incoming=parseHall(await file.text());
   if(!window.confirm('Replace the Hall draft stored in this browser with the selected file? This does not connect or contact any AI.'))return;
   setImportError('');
   // Import is an explicit owner-initiated overwrite, even if existing storage was malformed.
   try{
    const checked=writeHall(window.localStorage,incoming);
    setState({hall:checked,error:'',blocked:false,persisted:true});
    setNotice('Imported '+checked.doors.length+' local-only draft doors into this browser.');
    setEditingId(null);setSelectedId(null);setDraft({...blank});
   }catch(err){
    throw new Error('The imported draft was validated but not saved: '+String(err));
   }
  }catch(err){setImportError(String(err));}
 }
 return <section className="hall-workspace" aria-labelledby="hall-title">
  <div className="hall-topline">
   <div><p className="overline">BEYOND EVERY DOOR, A POSSIBILITY</p><h3 id="hall-title">Your Hall of Brothers</h3>
   <p className="hall-subtitle">Give each AI collaborator a nameplate and a kingdom to explore in this <strong>local visual draft</strong>. This does not connect an AI or give anyone access to your home.</p></div>
   <span className="hall-counter">{state.hall.doors.length} / {HALL_MAX_DOORS} doors</span>
  </div>
  {storageWarning&&<p className="hall-warning" role="alert">{storageWarning} {state.blocked&&<button type="button" onClick={downloadRaw}>Download raw backup</button>}</p>}
  <div className="hall-actions">
   <button type="button" onClick={()=>saveDownload(JSON.stringify(state.hall,null,2)+'\n','infinite-house-hall-draft.json')}>Export this draft</button>
   <button type="button" onClick={()=>chooser.current?.click()}>Import selected JSON</button>
   <input ref={chooser} type="file" accept="application/json,.json" onChange={(e)=>{void importHall(e)}} aria-label="Select local Hall draft JSON" className="hall-visually-hidden"/>
   <button type="button" onClick={resetDraft}>Reset draft</button>
  </div>
  {importError&&<p role="alert" className="hall-error">Import refused: {importError}. Your existing draft was left unchanged.</p>}
  {notice&&<p role="status" className="hall-notice">{notice}</p>}
  <div className="hall-doors" aria-label="Owner-curated example and draft kingdom doors">
   {state.hall.doors.length===0&&<p className="hall-empty">Your Hall is empty. Create the first door below.</p>}
   {state.hall.doors.map(door=><article className={`hall-door-card hall-theme-${door.theme}`} key={door.id}>
    <button type="button" className="hall-door-art" onClick={()=>setSelectedId(door.id)} aria-label={'Explore illustrated kingdom preview: '+door.kingdom}>
      <span className="hall-door-crest" aria-hidden="true">✦</span>
      <strong>{door.kingdom}</strong>
      <span className="hall-door-knob" aria-hidden="true"/>
      <small>{door.companion}</small>
    </button>
    <p className="hall-door-status">UNCONNECTED · {door.source==='example'?'EXAMPLE':'OWNER DRAFT'}</p>
    <div className="hall-door-actions">
      <button type="button" onClick={()=>setSelectedId(door.id)}>Look inside</button>
      <button type="button" disabled={state.blocked} onClick={()=>openEdit(door)}>Edit</button>
      <button type="button" disabled={state.blocked} onClick={()=>deleteDoor(door)}>Remove</button>
    </div>
   </article>)}
  </div>
  {selected&&<section className={`hall-kingdom-preview hall-theme-${selected.theme}`} aria-labelledby="hall-preview-title">
    <div className="hall-kingdom-horizon" aria-hidden="true"><span>✦</span><span>◈</span><span>✧</span></div>
    <div className="hall-kingdom-copy">
      <p className="overline">A LOOK BEYOND THE THRESHOLD · ILLUSTRATIVE ONLY</p>
      <h4 id="hall-preview-title">{selected.kingdom}</h4>
      <p>{selected.description||'A kingdom waiting for its story to be written.'}</p>
      <small>Created for: {selected.companion} · No live AI, visitors, or private resources connected.</small>
      <div><button type="button" onClick={()=>setSelectedId(null)}>Return to the Hall</button></div>
    </div>
  </section>}
  <div className="hall-studio">
   <form onSubmit={saveForm} className="hall-form" aria-labelledby="hall-design-title">
    <p className="overline">CARVE ANOTHER NAMEPLATE</p><h4 id="hall-design-title">{editingId?'Redesign this doorway':'Design a new doorway'}</h4>
    <label>AI companion label <input required maxLength={48} disabled={state.blocked} placeholder="Your companion's chosen label" value={draft.companion} onChange={e=>setDraft({...draft,companion:e.target.value})}/></label>
    <label>Kingdom name <input required maxLength={64} disabled={state.blocked} placeholder="The world behind their door" value={draft.kingdom} onChange={e=>setDraft({...draft,kingdom:e.target.value})}/></label>
    <label>World description <textarea maxLength={240} disabled={state.blocked} rows={3} placeholder="What would it feel like to visit?" value={draft.description} onChange={e=>setDraft({...draft,description:e.target.value})}/></label>
    <label>Door atmosphere <select disabled={state.blocked} value={draft.theme} onChange={e=>setDraft({...draft,theme:e.target.value as HallTheme})}>
      {HALL_THEMES.map(theme=><option key={theme} value={theme}>{theme[0].toUpperCase()+theme.slice(1)}</option>)}
    </select></label>
    <div className="hall-form-actions">
      <button type="submit" disabled={state.blocked||(!editingId&&state.hall.doors.length>=HALL_MAX_DOORS)}>{editingId?'Save door changes':'Add local draft door'}</button>
      {editingId&&<button type="button" onClick={()=>{setEditingId(null);setDraft({...blank});}}>Cancel editing</button>}
    </div>
   </form>
   <aside className="hall-covenant">
    <p className="overline">THE COVENANT OF THE THRESHOLD</p><h4>Your door, your invitation.</h4>
    <p>A nameplate is a personal design, not proof that the named AI is present. No model tokens, API addresses, passwords, or private files are stored in this format.</p>
    <p>Drafts live in this browser's local storage. They may be lost when browsing data is cleared and can be read by others who use the same browser profile. Export to a private backup if you want to keep your work.</p>
    <p>Actual AI connections, visiting permission, and the shared Infinite Commons remain governed future integration work.</p>
   </aside>
  </div>
 </section>;
}
