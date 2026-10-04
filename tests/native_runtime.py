#!/usr/bin/env python3
"""Fresh native executable smoke, authentication and durable restart continuity."""
import json,os,pathlib,tempfile
from acceptance import ROOT,TestNode
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
def main():
    checks=[]
    def check(name,ok=True):assert ok,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    with tempfile.TemporaryDirectory(prefix='porch-native smoke ') as tmp:
        n=TestNode('ALPHA spaced path',pathlib.Path(tmp),ROOT/'target/debug'/('porch-node.exe' if os.name=='nt' else 'porch-node'))
        try:
            check('native node rejects unauthenticated control',n.request('/v1/status',token=False)[0]==401)
            status=n.read('status');identity=n.read('identity');check('native identity and alias',status['node']['alias']=='ALPHA spaced path' and identity['id']==n.id)
            check('native doctor and ledger',n.read('doctor')['ledger_chain']=='VERIFIED')
            before=n.read('ledger');old_id=n.id
            n.stop();n.start()
            check('same native identity after restart',old_id==n.id)
            after=n.read('ledger');old_hashes={x['signature'] for x in before}
            check('same signed ledger prefix after restart',old_hashes.issubset({x['signature'] for x in after}) and n.read('doctor')['ledger_chain']=='VERIFIED')
            environment='NATIVE_HOSTED' if os.environ.get('GITHUB_ACTIONS')=='true' else 'LOOPBACK'
            n.call('qualification.run',{'environment':environment})
            bundle=n.call('qualification.export');check('native sanitized evidence export',bundle['manifest']['payload']['environment']==environment)
            out=EVIDENCE/'receipts/native-smoke';out.mkdir(parents=True,exist_ok=True)
            (out/'bundle.json').write_text(json.dumps(bundle,indent=2)+'\n')
            (out/'runtime.json').write_text(json.dumps({'checks':checks,'passed':len(checks),'peer_id':n.id,'build':n.read('qualification')['build'],'environment':environment,'physical_lan':'UNVERIFIED'},indent=2)+'\n')
        finally:n.stop()
if __name__=='__main__':main()
