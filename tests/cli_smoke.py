#!/usr/bin/env python3
import hashlib,json,os,pathlib,subprocess,tempfile
from acceptance import ROOT,TestNode
def main():
    extension='.exe' if os.name=='nt' else '';binary=ROOT/'target/debug'/('porch'+extension);node_binary=ROOT/'target/debug'/('porch-node'+extension)
    with tempfile.TemporaryDirectory(prefix='porch-cli-') as tmp:
        root=pathlib.Path(tmp);data=root/'NODE'
        def run(*args,expect=0,env=None):
            result=subprocess.run([str(binary),'--data',str(data),*args],env=env,capture_output=True,text=True)
            assert result.returncode==expect,(args,result.stderr,result.stdout)
            return result.stdout
        init=json.loads(run('init','--alias','CLI smoke'));assert init['initialized']
        backup=root/'protected.backup';environment=dict(os.environ,PORCH_TEST_PASSWORD='test-only-long-protected-password')
        run('identity','backup','--out',str(backup),'--password-env','PORCH_TEST_PASSWORD',env=environment)
        restored=root/'restored';subprocess.run([str(binary),'--data',str(restored),'identity','restore',str(backup),'--password-env','PORCH_TEST_PASSWORD'],env=environment,capture_output=True,check=True)
        assert (restored/'identity.key').read_bytes()==(data/'identity.key').read_bytes()
        node=TestNode('NODE',root,node_binary)
        try:
            def live(*args,expect=0):return run('--api',node.api,*args,expect=expect)
            assert json.loads(live('identity','show'))['id']==init['peer']
            assert json.loads(live('doctor'))['ledger_chain']=='VERIFIED'
            assert json.loads(live('create','CLI Porch'))['name']=='CLI Porch'
            request={'resource':'sha256','capability':'compute.hash','input':'abc','privacy':'LOCAL_ONLY'}
            result=json.loads(live('call','job.run','--json',json.dumps(request)));assert result['receipt']['payload']['output']['sha256']==hashlib.sha256(b'abc').hexdigest()
            live('model','run','missing-model','abc',expect=2)
            proof=root/'rotation.json';new=root/'rotated';run('identity','rotate','--new-data',str(new),'--out',str(proof));assert json.loads(proof.read_text())['old']['payload']['authority_carried'] is False
        finally:node.stop()
    out=ROOT/'docs/receipts/cli-smoke.json';out.write_text(json.dumps({'passed':True,'checks':['init','protected identity backup/restore','identity API','doctor','community create','local hash receipt','refusal exit code','explicit rotation proof']},indent=2)+'\n');print('CLI smoke passed: 8 checks.')
if __name__=='__main__':main()
