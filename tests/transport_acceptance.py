#!/usr/bin/env python3
"""Actual local TCP and QUIC processes, including repeat pairing attempts."""
import json,os,pathlib,tempfile
from acceptance import ROOT,TestNode

def main():
    checks=[];binary=ROOT/'target/debug'/('porch-node.exe' if os.name=='nt' else 'porch-node')
    def check(name,ok=True):assert ok,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    for transport in ['tcp','quic','both']:
        with tempfile.TemporaryDirectory(prefix='porch-transport-') as tmp:
            nodes=[]
            try:
                a=TestNode('A',pathlib.Path(tmp),binary,model='transport-mock',transport=transport);nodes.append(a)
                b=TestNode('B',pathlib.Path(tmp),binary,transport=transport);nodes.append(b)
                a.call('porch.create',{'name':'Transport regression'});invite=a.call('porch.invite',{'recipient':b.id});b.call('porch.join',{'invite':invite})
                r=b.call('porch.join',{'invite':invite},allow_refusal=True);check(transport+' replay refuses',r['status']=='REFUSED')
                for _ in range(4):assert all(x['updated'] for x in b.call('resources.refresh'))
                check(transport+' discovery stays live after repeated connects')
                grant=a.call('grant.issue',{'recipient':b.id,'capability':'model.inference','resource':'transport-mock','action':'run'});b.call('grant.import',{'grant':grant})
                for i in range(4):assert b.call('job.run',{'id':f'{transport}-{i}','resource':'transport-mock','input':'public transport probe','privacy':'TRUSTED_PEERS','max_output_tokens':16,'timeout_ms':1000})['status']=='COMPLETED'
                check(transport+' repeated signed job round trips')
                p=b.read('network')['peers'][a.id];check(transport+' encrypted authenticated protocol evidence',p['connections'] and all(x['encrypted'] and x['authenticated'] for x in p['connections']) and p['encrypted'] and p['authenticated'] and p['negotiated_protocol']=='/infinite-porch/rpc/1' and (transport=='both' or p['transport']==('QUIC' if transport=='quic' else 'TCP/Noise')))
            finally:
                for n in nodes:n.stop()
    (ROOT/'docs/receipts/qualification/transports.json').write_text(json.dumps({'environment':'LOOPBACK','physical_network':'UNVERIFIED','passed':len(checks),'checks':checks},indent=2)+'\n')
    print(f'Transport suite passed: {len(checks)} checks.')
if __name__=='__main__':main()
