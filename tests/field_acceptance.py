#!/usr/bin/env python3
"""Real loopback participant signatures, mismatches, storage and restart checks."""
import hashlib,json,os,pathlib,tempfile
from acceptance import ROOT,TestNode
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
def main():
    checks=[];nodes=[];session='native-field';environment='NATIVE_HOSTED' if os.environ.get('GITHUB_ACTIONS')=='true' else 'LOOPBACK'
    def check(name,ok=True):assert ok,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    with tempfile.TemporaryDirectory(prefix='porch-field-') as tmp:
        root=pathlib.Path(tmp);binary=ROOT/'target/debug'/('porch-node.exe' if os.name=='nt' else 'porch-node')
        try:
            a=TestNode('ALPHA',root,binary,model='field-mock',storage=2*1024*1024);nodes.append(a)
            b=TestNode('BETA',root,binary);nodes.append(b)
            c=TestNode('GAMMA',root,binary);nodes.append(c)
            porch=a.call('porch.create',{'name':'Native field'})
            for n in [b,c]:n.call('porch.join',{'invite':a.call('porch.invite',{'recipient':n.id})})
            for cap,res,action in [('model.inference','field-mock','run'),('message.direct','inbox','send'),('blob.storage','vault','store')]:
                g=a.call('grant.issue',{'recipient':b.id,'capability':cap,'resource':res,'action':action,'limits':{'max_calls':20,'calls_per_hour':20,'max_output_tokens':16,'max_storage_bytes':1024*1024}});b.call('grant.import',{'grant':g})
                if cap=='model.inference':mg=g
            b.call('resources.refresh')
            params={'session':session,'environment':environment}
            a_before=a.call('field.snapshot',params)
            run=b.call('qualification.run',{**params,'peer':a.id,'model':'field-mock','message':True,'storage':True})
            results={s['id']:s['result'] for s in run['payload']['scenarios']}
            check('fixed public field transfer and integrity check',results['encrypted_storage']=='PASS' and results['storage_integrity']=='PASS')
            check('bounded authority and independently signed discovery',results['bounded_authorization']=='PASS' and results['signed_discovery']=='PASS')
            b_after=b.call('field.snapshot',params);a_after=a.call('field.snapshot',params)
            def correlate(av,bv,name):
                import subprocess
                left=root/(name+'-a.json');right=root/(name+'-b.json');left.write_text(json.dumps(av));right.write_text(json.dumps(bv))
                cli=ROOT/'target/debug'/('porch.exe' if os.name=='nt' else 'porch')
                p=subprocess.run([str(cli),'qualify','correlate',str(left),str(right),'--out',str(root/(name+'-report'))],capture_output=True,text=True)
                assert p.returncode==0,p.stderr
                return json.loads(p.stdout)
            bad=correlate(a_before,b_after,'missing')
            check('completed origin without executor record is EVIDENCE_MISMATCH',bad['state']=='FAILED' and any('EVIDENCE_MISMATCH' in p for p in bad['problems']))
            report=correlate(a_after,b_after,'matched')
            check('bilateral job request grant input output route and signature match',len(report['job_correlations'])==1 and report['job_correlations'][0]['result']=='MATCH')
            check('hosted loopback and mock never earn physical certificate',report['state']=='PARTIAL' and not report['bilateral_lan_path'] and not report['live_ollama_match'] and not report['production_qualified'])
            a.call('grant.revoke',{'nonce':mg['payload']['nonce']})
            revoked=b.call('qualification.run',{**params,'peer':a.id,'model':'field-mock','phase':'revoked'})
            check('revoked public field job produces signed expected refusal',next(s for s in revoked['payload']['scenarios'] if s['id']=='remote_inference')['result']=='REFUSED_EXPECTED')
            for n in [a,b]:
                before=n.call('field.snapshot',params)
                refusal=n.call('field.restart',{'before':before},allow_refusal=True)
                check(n.name+' restart gate refuses same process',refusal['status']=='REFUSED')
                n.stop();n.start()
                restarted=n.call('field.restart',{'before':before})
                check(n.name+' fresh process verifies identity porch trust revocation accounting and ledger prefix',restarted['payload']['scenarios'][0]['result']=='PASS')
            b.call('resources.refresh')
            exported=[n.call('field.snapshot',params) for n in [a,b]]
            final=correlate(*exported,'final')
            check('bilateral refused receipt correlates after restart',final['revoked_refusal_match'] and all(j['result']=='MATCH' for j in final['job_correlations']))
            blob=next(s for s in run['payload']['scenarios'] if s['id']=='encrypted_storage')['observation']['cid']
            check('owner and executor independently record matching ciphertext',any(s['cid']==blob for s in exported[0]['payload']['storage']) and any(s['cid']==blob for s in exported[1]['payload']['storage']))
            b.call('job.run',{'resource':'sha256','capability':'compute.hash','privacy':'LOCAL_ONLY','input':'PRIVATE-UNSELECTED-FIELD-SENTINEL'})
            snapshot=b.call('field.snapshot',params);raw=json.dumps(snapshot)
            check('field evidence excludes unrelated prompts and credentials','PRIVATE-UNSELECTED-FIELD-SENTINEL' not in raw and all(n.token not in raw for n in nodes))
            forged=json.loads(json.dumps(exported[0]));forged['payload']['environment']='PHYSICAL_LAN'
            import subprocess
            left=root/'forged.json';left.write_text(json.dumps(forged));right=root/'right.json';right.write_text(json.dumps(exported[1]));cli=ROOT/'target/debug'/('porch.exe' if os.name=='nt' else 'porch')
            p=subprocess.run([str(cli),'qualify','correlate',str(left),str(right),'--out',str(root/'forged-report')],capture_output=True,text=True)
            check('evidence-class promotion invalidates participant signature',p.returncode!=0)
            out=EVIDENCE/'receipts/native-field';out.mkdir(parents=True,exist_ok=True)
            for n,v in zip([a,b],exported):(out/(n.name+'-participant.json')).write_text(json.dumps(v,indent=2)+'\n')
            (out/'correlation.json').write_text(json.dumps(final,indent=2)+'\n')
        finally:
            for n in nodes:n.stop()
    out=EVIDENCE/'receipts/field.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'passed':len(checks),'environment':environment,'checks':checks},indent=2)+'\n')
    print(f'Field acceptance passed: {len(checks)} checks.')
if __name__=='__main__':main()
