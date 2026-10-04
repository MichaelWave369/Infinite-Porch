#!/usr/bin/env python3
"""Three OS processes, real loopback TCP/Noise, explicit mock inference.
Never qualifies physical computers or model weights.
"""
import argparse,hashlib,json,os,pathlib,subprocess,tempfile,time
from acceptance import ROOT,TestNode
def main():
    ext='.exe' if os.name=='nt' else '';cli=ROOT/'target/debug'/('porch'+ext);daemon=ROOT/'target/debug'/('porch-node'+ext)
    parser=argparse.ArgumentParser();parser.add_argument('--keep-state',type=pathlib.Path);options=parser.parse_args()
    checks=[];measurements={};nodes=[]
    def check(name,condition=True):
        assert condition,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    with tempfile.TemporaryDirectory(prefix='porch-qualification-') as tmp:
        root=options.keep_state or pathlib.Path(tmp);root.mkdir(parents=True,exist_ok=True)
        def command(node,*args,expected=0):
            p=subprocess.run([str(cli),'--data',str(node.root),'--api',node.api,*args],capture_output=True,text=True,timeout=90)
            assert p.returncode==expected,(args,p.stdout,p.stderr)
            return json.loads(p.stdout) if p.stdout.strip().startswith('{') else p.stdout
        def grant(host,guest,cap,resource,action='run'):
            g=host.call('grant.issue',{'recipient':guest.id,'capability':cap,'resource':resource,'action':action,'limits':{'max_calls':20,'calls_per_hour':20,'max_output_tokens':16}})
            guest.call('grant.import',{'grant':g});return g
        try:
            for name in ['A','B','C']:
                started=time.perf_counter();n=TestNode(name,root,daemon,**({'model':'qual-mock'} if name=='A' else {}));nodes.append(n)
                measurements[f'startup_{name}_ms']=(time.perf_counter()-started)*1000
            a,b,c=nodes;check('three independent persisted identities',len({n.id for n in nodes})==3)
            invite=root/'invite.json';start=time.perf_counter();command(a,'qualify','host','Physical rehearsal','--recipient',b.id,'--out',str(invite))
            fingerprint=a.read('identity')['fingerprint_sha256']
            command(b,'qualify','join',str(invite),'--fingerprint','00'*32,expected=1);check('wrong fingerprint refuses before trust',not b.read('peers'))
            command(c,'qualify','join',str(invite),'--fingerprint',fingerprint,expected=1);check('stolen recipient-bound invitation refused',not c.read('peers'))
            command(b,'qualify','join',str(invite),'--fingerprint',fingerprint);measurements['pairing_ms']=(time.perf_counter()-start)*1000;check('CLI host and fingerprint-confirmed join')
            command(b,'qualify','join',str(invite),'--fingerprint',fingerprint,expected=1);check('invite replay refused')
            start=time.perf_counter();b.call('resources.refresh');measurements['manual_discovery_ms']=(time.perf_counter()-start)*1000
            before=b.read('network')['outbound_job_requests'];r=b.call('job.run',{'resource':'qual-mock','input':'PRIVATE-NO-AUTHORITY-SENTINEL','privacy':'TRUSTED_PEERS'})
            check('advertisement conveys no invocation authority',r['status']=='REFUSED' and b.read('network')['outbound_job_requests']==before)
            g=grant(a,b,'model.inference','qual-mock');grant(a,b,'message.direct','inbox','send')
            start=time.perf_counter();r=b.call('message.send',{'peer':a.id,'text':'public performance probe'});measurements['message_round_trip_ms']=(time.perf_counter()-start)*1000;check('granted messaging returns receipt',r['status']=='DELIVERED')
            start=time.perf_counter();r=b.call('job.run',{'resource':'qual-mock','input':'public performance probe','privacy':'TRUSTED_PEERS','max_output_tokens':16});elapsed=(time.perf_counter()-start)*1000
            measurements['job_round_trip_ms']=elapsed;measurements['dispatch_and_receipt_overhead_ms']=max(0,elapsed-r['receipt']['payload']['duration_ms']);check('remote model mock rehearsal')
            for privacy in ['PORCH_ALLOWED','FEDERATED_ALLOWED']:
                result=b.call('job.run',{'resource':'qual-mock','input':'public privacy probe','privacy':privacy,'max_output_tokens':16})
                check(privacy+' honors joined trusted grant',result['status']=='COMPLETED' and result['receipt']['payload']['executor']==a.id and result['route']['federation']=='NOT_INSTALLED')
            before=c.read('network')['outbound_job_requests'];r=c.call('job.run',{'resource':'qual-mock','input':'PRIVATE-UNTRUSTED-SENTINEL','privacy':'TRUSTED_PEERS','max_output_tokens':16})
            check('untrusted node cannot transmit unauthorized prompt',r['status']=='REFUSED' and before==c.read('network')['outbound_job_requests'])
            report=command(b,'qualify','run','--peer',a.id,'--model','qual-mock','--message','--environment','LOOPBACK')
            statuses={s['id']:s['result'] for s in report['payload']['scenarios']}
            check('mock inference is PARTIAL, never live-provider PASS',statuses['remote_inference']=='PARTIAL')
            check('local privacy qualification PASS',statuses['local_only']=='PASS')
            check('loopback never claims two physical nodes',statuses['separate_physical_nodes']=='UNVERIFIED')
            check('authenticated negotiated network path',report['payload']['network']['peers'][a.id]['encrypted'] and report['payload']['network']['peers'][a.id]['negotiated_protocol']=='/infinite-porch/rpc/1')
            export=root/'evidence';command(b,'qualify','export',str(export));v=command(b,'qualify','validate',str(export));check('CLI sanitized export hashes/signatures/schema valid',v['valid'] and not v['skip_counts_as_pass'])
            text=''.join(p.read_text() for p in export.glob('*.json'));check('evidence excludes unrelated private prompts and operator tokens','PRIVATE-NO-AUTHORITY-SENTINEL' not in text and all(n.token not in text for n in nodes))
            copy=ROOT/'docs/receipts/qualification/example-sanitized-bundle';copy.mkdir(parents=True,exist_ok=True)
            for p in export.glob('*.json'):(copy/p.name).write_bytes(p.read_bytes())
            corrupt=json.loads((export/'results.json').read_text());corrupt[0]['result']='FORGED_PASS';(export/'results.json').write_text(json.dumps(corrupt));command(b,'qualify','validate',str(export),expected=1);check('malformed or modified evidence is rejected')
            for origin in ['null','file://','https://hostile.example','http://127.0.0.1:1']:
                check('cross-origin refused '+origin,b.request('/v1/status',headers={'Origin':origin})[0]==403)
            check('SSE requires operator authority',b.request('/v1/events',token=False)[0]==401)
            check('localhost alias rebinding refused',b.request('/v1/status',headers={'Host':'localhost:'+str(b.api_port)})[0]==403)
            command(b,'qualify','run','--environment','PHYSICAL',expected=1);check('physical label requires explicit attestation')
            before=b.read('network')['outbound_job_requests'];r=b.call('job.run',{'resource':'qual-mock','input':'PRIVATE-LOCAL-SENTINEL','privacy':'LOCAL_ONLY','preferred_peer':a.id});check('LOCAL_ONLY pinned remote candidate cannot receive input',r['status']=='REFUSED' and before==b.read('network')['outbound_job_requests'])
            ledger=json.dumps(b.read('ledger'));check('refused route ledger contains no prompt contents','PRIVATE-LOCAL-SENTINEL' not in ledger and 'PRIVATE-NO-AUTHORITY-SENTINEL' not in ledger)
            a.call('grant.revoke',{'nonce':g['payload']['nonce']});r=command(b,'qualify','run','--peer',a.id,'--model','qual-mock','--phase','revoked');check('revocation qualification records expected refusal',next(x for x in r['payload']['scenarios'] if x['id']=='remote_inference')['result']=='REFUSED_EXPECTED')
            old_id=a.id;old_porch=a.read('status')['porch']['id'];a.stop();a.start();check('restart preserves identity membership and revoked grant',a.id==old_id and a.read('status')['porch']['id']==old_porch and any(x['revoked'] for x in a.read('grants')))
            check('restart preserves entire ledger chain',a.read('doctor')['ledger_chain']=='VERIFIED')
            scan=command(b,'models','scan');check('absent live provider is explicit, no fallback',scan['currently_reachable'] is False)
            # Linux process measurements are honest observations, with explicit
            # unavailable values on other platforms.
            try:
                samples={n.name:[int(x) for x in pathlib.Path(f'/proc/{n.process.pid}/stat').read_text().split()[13:15]] for n in nodes}
                t=time.perf_counter();time.sleep(2);duration=time.perf_counter()-t;hz=os.sysconf('SC_CLK_TCK')
                for n in nodes:
                    after=[int(x) for x in pathlib.Path(f'/proc/{n.process.pid}/stat').read_text().split()[13:15]]
                    measurements[f'idle_cpu_{n.name}_percent']=100*(sum(after)-sum(samples[n.name]))/hz/duration
                    rss=next(x for x in pathlib.Path(f'/proc/{n.process.pid}/status').read_text().splitlines() if x.startswith('VmRSS:'))
                    measurements[f'idle_rss_{n.name}_bytes']=int(rss.split()[1])*1024
            except (OSError,StopIteration,ValueError):measurements['idle_process_metrics']='UNVERIFIED: native /proc metrics missing or incompatible in this managed runtime'
            data=b'x'*(8*1024*1024);start=time.perf_counter();hashlib.sha256(data).digest();measurements['storage_hash_mib_per_second']=8/(time.perf_counter()-start)
            a.stop();start=time.perf_counter();q=command(b,'qualify','run','--peer',a.id,'--message');duration=time.perf_counter()-start
            check('offline message qualification records bounded failure',duration<9 and next(s for s in q['payload']['scenarios'] if s['id']=='messaging')['result']=='FAIL')
        finally:
            for n in reversed(nodes):n.stop()
    receipt={'candidate':'0.1.1','environment':'LOOPBACK','model_provider':'explicit mock','physical_evidence':'UNVERIFIED','passed':len(checks),'checks':checks,'performance':measurements,'performance_limitations':'Single debug-build sample; manual discovery, pairing includes a deliberate failed fingerprint check; dispatch includes API, discovery and receipt transport. No physical LAN or model inference performance claim.'}
    (ROOT/'docs/receipts/qualification/qualification-acceptance.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'Qualification acceptance passed: {len(checks)} checks.')
if __name__=='__main__':main()
