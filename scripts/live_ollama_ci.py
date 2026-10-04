#!/usr/bin/env python3
"""Optional authentic weights; environmental failures are retained, never mocked."""
import argparse,hashlib,json,os,pathlib,platform,subprocess,sys,tempfile,time,urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tests'))
from acceptance import TestNode,HTTP,port
MODEL='smollm2:135m-instruct-q4_0'
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
    summary={'evidence_class':'NATIVE_HOSTED','topology':'NATIVE_HOSTED_LOOPBACK','result':'UNVERIFIED','model':MODEL,'selection':'Small 135M CPU probe; registry tag revalidated by real pull and inventory. Not a claim of global smallest model.','distribution_source':'https://docs.ollama.com/linux','model_source':'https://ollama.com/library/smollm2:135m-instruct-q4_0','commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'physical_lan':'UNVERIFIED','gpu':'UNVERIFIED','production_qualified':False,'checks':[]}
    nodes=[];server=None;stage='distribution';failed=False
    with tempfile.TemporaryDirectory(prefix='porch-live-') as tmp:
        work=pathlib.Path(tmp);archive=work/'ollama.tar.zst';runtime=work/'runtime';runtime.mkdir()
        try:
            if platform.system()!='Linux':raise RuntimeError('Optional supported native Linux distribution job only')
            arch={'x86_64':'amd64','aarch64':'arm64'}.get(platform.machine().lower())
            if not arch:raise RuntimeError('Unsupported distribution architecture')
            url=f'https://ollama.com/download/ollama-linux-{arch}.tar.zst';summary['distribution_url']=url
            with (out/'distribution.log').open('w') as log:
                subprocess.run(['curl','--fail','--location','--max-time','480','--output',str(archive),url],check=True,stdout=log,stderr=log,timeout=490)
                summary['distribution_sha256']=hashlib.sha256(archive.read_bytes()).hexdigest()
                subprocess.run(['tar','--zstd','-xf',str(archive),'-C',str(runtime)],check=True,stdout=log,stderr=log)
            ollama=runtime/'bin/ollama';env=dict(os.environ,OLLAMA_HOST=f'127.0.0.1:{port()}',OLLAMA_MODELS=str(work/'models'),OLLAMA_NO_CLOUD='1',CUDA_VISIBLE_DEVICES='',HIP_VISIBLE_DEVICES='')
            summary['version']=subprocess.check_output([str(ollama),'--version'],env=env,text=True,stderr=subprocess.STDOUT).strip()
            server_log=(out/'server.log').open('w');server=subprocess.Popen([str(ollama),'serve'],env=env,stdout=server_log,stderr=server_log)
            base='http://'+env['OLLAMA_HOST'];stage='startup'
            for _ in range(150):
                try:
                    with HTTP.open(base+'/api/tags',timeout=2) as response:json.load(response)
                    break
                except Exception:time.sleep(.2)
            else:raise RuntimeError('Authentic runtime did not become ready')
            stage='model-pull'
            with (out/'pull.log').open('w') as log:subprocess.run([str(ollama),'pull',MODEL],env=env,check=True,stdout=log,stderr=log,timeout=600)
            with HTTP.open(base+'/api/tags',timeout=10) as response:inventory=json.load(response)
            (out/'inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
            assert any(m['name']==MODEL for m in inventory['models']),'Pulled model missing from real inventory'
            summary['checks'].append({'name':'authentic runtime pulled installed weights','result':'PASS'})
            stage='porch-integration'
            alpha=TestNode('ALPHA',work,ROOT/'target/debug/porch-node',ollama=base);nodes.append(alpha)
            beta=TestNode('BETA',work,ROOT/'target/debug/porch-node');nodes.append(beta)
            local=alpha.call('model.verify',{'model':MODEL})
            # Configure exact installed model before local verification if no share yet.
            if local['status']!='COMPLETED':
                alpha.call('share.configure',{'model':{'name':MODEL,'provider':'ollama','exposed':False}})
                local=alpha.call('model.verify',{'model':MODEL})
            assert local['status']=='COMPLETED' and local['receipt']['payload']['provider']=='ollama','Actual local inference failed'
            alpha.call('porch.create',{'name':'Native live Ollama'})
            beta.call('porch.join',{'invite':alpha.call('porch.invite',{'recipient':beta.id})})
            alpha.call('share.configure',{'model':{'name':MODEL,'provider':'ollama','exposed':True}})
            grant=alpha.call('grant.issue',{'recipient':beta.id,'capability':'model.inference','resource':MODEL,'limits':{'max_calls':3,'calls_per_hour':3,'max_output_tokens':16,'max_duration_ms':30000}})
            beta.call('grant.import',{'grant':grant});beta.call('resources.refresh')
            args={'peer':alpha.id,'model':MODEL,'session':'hosted-live','environment':'NATIVE_HOSTED'}
            run=beta.call('qualification.run',args)
            assert next(s for s in run['payload']['scenarios'] if s['id']=='remote_inference')['result']=='PASS','Governed remote live inference failed'
            summary['checks'].append({'name':'governed remote authentic Ollama inference','result':'PASS'})
            alpha.call('grant.revoke',{'nonce':grant['payload']['nonce']})
            refused=beta.call('qualification.run',{**args,'phase':'revoked'})
            assert next(s for s in refused['payload']['scenarios'] if s['id']=='remote_inference')['result']=='REFUSED_EXPECTED','Revoked inference not refused'
            summary['checks'].append({'name':'signed refusal after revoke','result':'PASS'})
            snapshots=[]
            for n in nodes:
                v=n.call('field.snapshot',{'session':'hosted-live','environment':'NATIVE_HOSTED'});snapshots.append(v);(out/(n.name+'-participant.json')).write_text(json.dumps(v,indent=2)+'\n')
            subprocess.run([str(ROOT/'target/debug/porch'),'qualify','correlate',str(out/'ALPHA-participant.json'),str(out/'BETA-participant.json'),'--out',str(out/'correlation')],check=True,stdout=subprocess.DEVNULL)
            (out/'local-receipt.json').write_text(json.dumps(local['receipt'],indent=2)+'\n');summary['result']='PASS'
        except Exception as exc:
            summary['failed_stage']=stage;summary['failure']=str(exc);summary['result']='FAIL' if stage=='porch-integration' else 'SKIPPED_ENVIRONMENT';failed=stage=='porch-integration'
        finally:
            for n in nodes:n.stop()
            if server:
                server.terminate()
                try:server.wait(timeout=15)
                except subprocess.TimeoutExpired:server.kill();server.wait()
                server_log.close()
            (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
    return 1 if failed else 0
if __name__=='__main__':sys.exit(main())
