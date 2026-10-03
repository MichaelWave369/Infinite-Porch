#!/usr/bin/env python3
"""An interactive three-node mock demo. No resource claims about real hardware."""
import argparse,pathlib,sys,time
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]/'tests'))
from acceptance import ROOT,TestNode
def main():
    p=argparse.ArgumentParser();p.add_argument('--state',type=pathlib.Path,default=ROOT/'state/demo');p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/debug'/('porch-node.exe' if sys.platform=='win32' else 'porch-node'));a=p.parse_args()
    if a.state.exists():raise SystemExit('Choose a fresh --state directory for a new demo.')
    nodes=[]
    try:
        alpha=TestNode('ALPHA',a.state,a.binary,model='porch-mock',storage=16*1024*1024);nodes.append(alpha)
        beta=TestNode('BETA',a.state,a.binary,hash=True,storage=16*1024*1024);nodes.append(beta)
        gamma=TestNode('GAMMA',a.state,a.binary);nodes.append(gamma)
        porch=alpha.call('porch.create',{'name':'Oak Street'})
        for node in [beta,gamma]:node.call('porch.join',{'invite':alpha.call('porch.invite',{'recipient':node.id})})
        beta.call('peer.approve',{'peer':gamma.id,'alias':'GAMMA','porch':porch['id']});gamma.call('peer.approve',{'peer':beta.id,'alias':'BETA','porch':porch['id']})
        gamma.call('peer.connect',{'address':next(x for x in beta.read('network')['addresses'] if '/tcp/' in x)})
        for host,cap,resource,action in [(alpha,'model.inference','porch-mock','run'),(beta,'compute.hash','sha256','run'),(beta,'blob.storage','vault','store'),(beta,'message.direct','inbox','send')]:
            grant=host.call('grant.issue',{'recipient':gamma.id,'capability':cap,'resource':resource,'action':action,'limits':{'max_storage_bytes':4*1024*1024 if cap=='blob.storage' else 0}});gamma.call('grant.import',{'grant':grant})
        for node in nodes:node.call('resources.refresh')
        print('Three real peer daemons; explicitly MOCK inference; loopback paths. Ctrl+C stops them. Keys stay in the selected state directory.',flush=True)
        for node in nodes:print(f'{node.name}: {node.api} | local operator token file: {node.root/"api.token"}',flush=True)
        while True:time.sleep(1)
    except KeyboardInterrupt:pass
    finally:
        for node in nodes:node.stop()
if __name__=='__main__':main()
