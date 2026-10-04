#!/usr/bin/env python3
"""Optional real UI acceptance. Install Chromium: npx playwright install chromium."""
import argparse,os,pathlib,subprocess,tempfile
from acceptance import ROOT,TestNode

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/debug'/('porch-node.exe' if os.name=='nt' else 'porch-node'));a=p.parse_args()
    with tempfile.TemporaryDirectory(prefix='porch-ui-') as temp:
        state=pathlib.Path(temp);nodes=[]
        try:
            alpha=TestNode('ALPHA',state,a.binary,model='porch-mock');nodes.append(alpha)
            gamma=TestNode('GAMMA',state,a.binary);nodes.append(gamma)
            alpha.call('porch.create',{'name':'Oak Street'})
            invite=alpha.call('porch.invite',{'recipient':gamma.id,'ttl_seconds':600});gamma.call('porch.join',{'invite':invite})
            xss=TestNode('XSS',state,a.binary);nodes.append(xss)
            gamma.call('peer.approve',{'peer':xss.id,'alias':'<img src=x onerror=alert(1)>','porch':alpha.read('status')['porch']['id']})
            grant=alpha.call('grant.issue',{'recipient':gamma.id,'capability':'model.inference','resource':'porch-mock','action':'run','limits':{'max_calls':5,'calls_per_hour':5}});gamma.call('grant.import',{'grant':grant});gamma.call('resources.refresh')
            subprocess.run(['node',str(ROOT/'scripts/ui-smoke.mjs'),gamma.api,str(gamma.root/'api.token'),str(ROOT/'docs/evidence')],cwd=ROOT,check=True)
        finally:
            for n in nodes:n.stop()
if __name__=='__main__':main()
