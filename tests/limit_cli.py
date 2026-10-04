#!/usr/bin/env python3
import json,os,pathlib,subprocess,tempfile,urllib.request,urllib.error
from acceptance import ROOT,TestNode,HTTP
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
(EVIDENCE/'receipts/qualification').mkdir(parents=True,exist_ok=True)
(EVIDENCE/'evidence').mkdir(parents=True,exist_ok=True)

def main():
    checks=[];ext='.exe' if os.name=='nt' else '';cli=ROOT/'target/debug'/('porch'+ext)
    with tempfile.TemporaryDirectory(prefix='porch-caps-') as tmp:
        root=pathlib.Path(tmp);node=TestNode('N',root,ROOT/'target/debug'/('porch-node'+ext))
        def run(*args):return subprocess.run([str(cli),'--data',str(node.root),*args],capture_output=True,text=True,timeout=30)
        try:
            caps=root/'limits.json';caps.write_text(json.dumps({'input_bytes':8,'message_bytes':8,'request_body_bytes':4096}))
            assert run('configure-limits',str(caps)).returncode==1;checks.append('active daemon refuses offline configuration')
            node.stop();r=run('configure-limits',str(caps));assert r.returncode==0,(r.stdout,r.stderr);checks.append('offline caps persisted')
            node.start();r=node.call('job.run',{'resource':'sha256','capability':'compute.hash','input':'123456789','privacy':'LOCAL_ONLY'},allow_refusal=True);assert r['reason']=='INPUT_TOO_LARGE';checks.append('restart applies input cap')
            req=urllib.request.Request(node.api+'/v1/control',data=json.dumps({'operation':'job.run','args':{'input':'x'*5000}}).encode(),headers={'Authorization':'Bearer '+node.token,'Content-Type':'application/json'})
            try:
                HTTP.open(req,timeout=10);raise AssertionError('oversized request accepted')
            except urllib.error.HTTPError as err:assert err.code==413
            checks.append('HTTP request byte cap enforced')
            cap=json.loads(caps.read_text());cap['concurrent_jobs']=17;caps.write_text(json.dumps(cap));node.stop();assert run('configure-limits',str(caps)).returncode==1;checks.append('unsafe caps refuse without overwriting last safe config')
            assert json.loads((node.root/'config.json').read_text())['limits']['concurrent_jobs']==2
        finally:
            node.stop()
    (EVIDENCE/'receipts/qualification/limit-cli.json').write_text(json.dumps({'passed':len(checks),'checks':checks},indent=2)+'\n');print(f'Limit CLI passed: {len(checks)} checks.')
if __name__=='__main__':main()
