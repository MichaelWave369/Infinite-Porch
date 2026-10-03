import {test} from 'node:test';
import assert from 'node:assert/strict';
import {PorchClient,PhiVesselPorchAdapter,PhiBotPorchAdapter} from '../packages/sdk/dist/index.js';
test('remote endpoints and path injection cannot become control APIs',()=>{
  assert.throws(()=>new PorchClient('test','https://remote.example'),/loopback/);
  assert.throws(()=>new PorchClient('test','http://127.0.0.1.evil.example'),/loopback/);
  assert.throws(()=>new PorchClient('test').read('../grants'),/Invalid/);
});
test('applications and model-routing adapters default to LOCAL_ONLY',async()=>{
  const requests=[];const previous=globalThis.fetch;
  globalThis.fetch=async(url,options)=>{requests.push({url,options,body:JSON.parse(options.body)});return new Response(JSON.stringify({status:'REFUSED',reason:'NO_ELIGIBLE_EXECUTOR',route:{}}),{headers:{'content-type':'application/json'}})};
  try{
    const client=new PorchClient('operator');
    const proposal=new PhiVesselPorchAdapter(client).propose({resource:'model',input:'private'});
    assert.equal(proposal.execution_authority,false);assert.equal(requests.length,0);
    await new PhiVesselPorchAdapter(client).execute(proposal.request);
    await new PhiBotPorchAdapter(client).execute({resource:'model',input:'private'});
    for(const r of requests){assert.equal(r.body.args.privacy,'LOCAL_ONLY');assert.equal(r.body.operation,'job.run');assert.equal(r.options.redirect,'error');}
    await client.run({resource:'model',input:'approved',privacy:'TRUSTED_PEERS'});
    assert.equal(requests.at(-1).body.args.privacy,'TRUSTED_PEERS');
  }finally{globalThis.fetch=previous}
});
