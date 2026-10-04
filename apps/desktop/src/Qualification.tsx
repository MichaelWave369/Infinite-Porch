import React from 'react';
type Value = Record<string, any>;
type Props = { value:Value; peers:Value[]; busy:boolean; action:(op:string,args:Value)=>Promise<unknown> };
const shown=(v:unknown)=>v===true?'YES':v===false?'NO':'UNVERIFIED';
export function Qualification({value:q,peers,busy,action}:Props){
  const last=q.results?.[0]?.payload, counts:Record<string,number>={};
  for(const s of last?.scenarios??[])counts[s.result]=(counts[s.result]??0)+1;
  async function exportBundle(){
    const bundle=await action('qualification.export',{});if(!bundle)return;
    const url=URL.createObjectURL(new Blob([JSON.stringify(bundle,null,2)],{type:'application/json'}));
    const a=document.createElement('a');a.href=url;a.download='porch-qualification-bundle.json';a.click();URL.revokeObjectURL(url);
  }
  return <>
    <section className="boundary"><div><strong>CANDIDATE · PLATFORM PARTIAL · SECURITY REVIEW PENDING</strong><p>Physical runs require operator confirmation and external evidence. Skipped and unverified scenarios count separately.</p></div></section>
    <div className="two-columns">
      <section className="card"><h2>Node and build</h2><dl><dt>Software / protocol</dt><dd>{q.build.software_version} / {q.build.protocol_version}</dd><dt>Build commit</dt><dd><code className="address">{q.build.commit}</code></dd><dt>Peer identity</dt><dd><code className="address">{q.node.peer_id}</code></dd><dt>Compare this SHA-256 fingerprint on the intended PC</dt><dd><code className="address">{q.node.fingerprint_sha256}</code></dd><dt>Ledger verifies</dt><dd>{shown(q.ledger_verified)}</dd><dt>Last physical qualification</dt><dd>{q.last_physical_qualification?new Date(q.last_physical_qualification.payload.timestamp*1000).toLocaleString():'UNVERIFIED'}</dd></dl></section>
      <section className="card"><h2>Provider readiness</h2><p>Installed, reachable, advertised and permitted are separate observations. An unloaded model can be loaded by a granted invocation.</p><button disabled={busy} onClick={()=>void action('model.scan',{})}>Scan local Ollama</button><p>Currently reachable: {shown(q.models.scan?.currently_reachable)}</p>{q.models.states?.map((m:Value)=><div className="model-row" key={m.model}><div><strong>{m.model}</strong><small>{m.provider} · advertised {shown(m.advertised)} · reachable {shown(m.currently_reachable)} · installed {shown(m.installed_now)} · loaded {shown(m.loaded_now)}</small><small>Grant exists: {shown(m.remote_invocation_permitted_for_at_least_one_peer)} · last invocation: {shown(m.last_verified?.invocation_successful)}</small></div><button disabled={busy} onClick={()=>void action('model.verify',{model:m.model})}>Verify invocation</button></div>)}</section>
      <section className="card"><h2>Run qualification</h2><p>Uses a fixed public diagnostic prompt. A model run consumes its bounded grant. Compare separate exports from both machines.</p><form onSubmit={e=>{e.preventDefault();const d=new FormData(e.currentTarget);void action('qualification.run',{peer:d.get('peer')||null,model:d.get('model')||null,environment:d.get('environment'),phase:d.get('phase'),separate_machines_confirmed:d.get('separate')==='on',wan_condition_confirmed:d.get('wan')==='on',message:d.get('message')==='on'})}}>
        <label>Qualification peer<select name="peer"><option value="">Local checks only</option>{peers.filter(p=>p.active).map(p=><option key={p.peer} value={p.peer}>{p.alias}</option>)}</select></label>
        <label>Qualification model<input name="model" placeholder="Exact shared model, optional"/></label>
        <label>Evidence environment<select name="environment"><option>LOOPBACK</option><option>SIMULATED</option><option>PHYSICAL</option></select></label>
        <label>Qualification phase<select name="phase"><option>baseline</option><option>revoked</option><option>restart</option><option>offline</option><option>restored</option></select></label>
        <label className="qualification-check"><input type="checkbox" name="separate"/>I confirm these are separate physical machines.</label>
        <label className="qualification-check"><input type="checkbox" name="wan"/>I confirm the stated upstream WAN condition, with LAN preserved.</label>
        <label className="qualification-check"><input type="checkbox" name="message"/>Send the public diagnostic message using a separate inbox grant.</label>
        <button className="primary" disabled={busy}>Run qualification</button>
      </form></section>
      <section className="card"><h2>Scenario evidence</h2><p>{last?`${last.environment} · ${last.phase} · ${new Date(last.timestamp*1000).toLocaleString()}`:'No qualification run yet.'}</p><div className="qualification-counts">{['PASS','FAIL','REFUSED_EXPECTED','SKIPPED_ENVIRONMENT','UNVERIFIED','PARTIAL'].map(s=><span className="badge gray" key={s}>{s} {counts[s]??0}</span>)}</div><button disabled={busy||!last} onClick={()=>void exportBundle()}>Export qualification evidence</button>{last?.scenarios.map((s:Value)=><details className="ledger-detail" key={s.id}><summary><strong>{s.id}</strong><span className={`badge ${s.result==='FAIL'?'amber':'gray'}`}>{s.result}</span></summary><p>{s.expectation}</p><pre>{JSON.stringify(s.observation,null,2)}</pre></details>)}<p className="small">Validate this download with <code>porch qualify validate porch-qualification-bundle.json</code>. Node signatures establish provenance, not independent review.</p></section>
    </div>
    <section className="card"><h2>Physical peer paths</h2>{Object.entries(q.network.peers??{}).map(([id,v])=>{const p=v as Value;return <details className="ledger-detail" key={id}><summary><strong>{peers.find(x=>x.peer===id)?.alias??id}</strong><span className="badge gray">{p.connected?p.path:'OFFLINE'}</span></summary><p>{p.transport} · encryption {p.encryption} · authenticated {shown(p.authenticated)} · trust {p.trust_state} · RTT {p.latency_ms??'unmeasured'} ms · age {p.connection_age_seconds??'unmeasured'} seconds</p><p className="small">IP classification does not prove LAN topology or Internet disconnection.</p><details><summary>Advanced socket and protocol observations</summary><pre>{JSON.stringify(p,null,2)}</pre></details></details>})}<h3>Active or uncertain jobs</h3><pre>{JSON.stringify(q.active_jobs,null,2)}</pre></section>
  </>;
}
