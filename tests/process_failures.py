#!/usr/bin/env python3
"""Terminate only child test daemons; the provider is an owned HTTP fixture.
No host network, real Ollama service, firewall or unrelated process is changed.
"""
import argparse,concurrent.futures,http.server,http.client,urllib.error,json,os,pathlib,sqlite3,tempfile,threading,time
from acceptance import EVIDENCE_CLASS,EVIDENCE_TOPOLOGY,ROOT,TestNode
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
(EVIDENCE/'receipts/qualification').mkdir(parents=True,exist_ok=True)
class Provider(http.server.BaseHTTPRequestHandler):
    calls=0
    lock=threading.Lock()
    def log_message(self,*args):pass
    def send(self,v):
        b=json.dumps(v).encode()
        try:self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(b)));self.end_headers();self.wfile.write(b)
        except (BrokenPipeError,ConnectionResetError):pass
    def do_GET(self):
        self.send({'models':[{'name':'slow-fixture','digest':'fixture'}]} if self.path=='/api/tags' else {'models':[]} if self.path=='/api/ps' else {'version':'HTTP-FIXTURE'})
    def do_POST(self):
        self.rfile.read(int(self.headers['Content-Length']))
        with self.lock:type(self).calls+=1
        time.sleep(1)
        self.send({'done':True,'response':'public fixture output','eval_count':3})
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--transport',choices=['both','tcp','quic'],default='both');options=parser.parse_args()
    checks=[];nodes=[];reconnection=[];ext='.exe' if os.name=='nt' else '';binary=ROOT/'target/debug'/('porch-node'+ext)
    server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Provider);server.daemon_threads=True
    threading.Thread(target=server.serve_forever,daemon=True).start()
    def check(name,condition=True):assert condition,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    def reconcile(node,args,phase):
        for attempt in range(1,4):
            r=node.call('job.run',args,allow_refusal=True)
            code=r.get('receipt',{}).get('payload',{}).get('reason') or r.get('reason')
            reconnection.append({'phase':phase,'attempt':attempt,'status':r.get('status'),'reason':code})
            if r.get('receipt') or 'UNCERTAIN' in (code or ''):return r
            assert code and ('PEER_JOB_DEADLINE' in code or 'PEER_TRANSPORT_FAILURE' in code),(phase,r)
            time.sleep(.05)
        raise AssertionError(('bounded reconnection failed',phase,reconnection))
    def wait(predicate):
        for _ in range(100):
            if predicate():return
            time.sleep(.03)
        raise AssertionError('bounded state wait failed')
    with tempfile.TemporaryDirectory(prefix='porch-process-fault-') as tmp:
        root=pathlib.Path(tmp)
        try:
            a=TestNode('A',root,binary,ollama=f'http://127.0.0.1:{server.server_port}',transport=options.transport);nodes.append(a)
            b=TestNode('B',root,binary,transport=options.transport);nodes.append(b)
            a.call('porch.create',{'name':'Failure fixture'});i=a.call('porch.invite',{'recipient':b.id});b.call('porch.join',{'invite':i})
            a.call('share.configure',{'model':{'name':'slow-fixture','provider':'ollama','exposed':True}})
            g=a.call('grant.issue',{'recipient':b.id,'capability':'model.inference','resource':'slow-fixture','action':'run'});b.call('grant.import',{'grant':g})
            args={'id':'executor-death','resource':'slow-fixture','input':'public crash fixture','privacy':'TRUSTED_PEERS','timeout_ms':3000,'max_output_tokens':16}
            with concurrent.futures.ThreadPoolExecutor() as pool:
                pending=pool.submit(b.call,'job.run',args,True);wait(lambda:Provider.calls>=1);old=a.id;a.stop();r=pending.result(timeout=40)
            check('executor death yields bounded failure',r['status']=='REFUSED')
            a.start();check('executor identity persists after forced process stop',a.id==old)
            check('running job becomes visible UNCERTAIN',any(j['id']=='executor-death' and j['status']=='UNCERTAIN' for j in a.read('active-jobs')))
            r=reconcile(b,args,'executor_restart');check('uncertain job retry does not reexecute',r['status']=='REFUSED' and 'UNCERTAIN' in r['reason'] and Provider.calls==1)
            args={**args,'id':'requester-death'};before=Provider.calls
            with concurrent.futures.ThreadPoolExecutor() as pool:
                pending=pool.submit(b.call,'job.run',args,True);wait(lambda:Provider.calls>before);old=b.id;b.stop()
                try:pending.result(timeout=40)
                except (ConnectionResetError,ConnectionAbortedError,http.client.RemoteDisconnected,urllib.error.URLError,json.JSONDecodeError):pass
            wait(lambda:any(j['payload']['job_id']=='requester-death' for j in a.read('jobs')))
            check('executor records terminal outcome after requester disappears',any(j['payload']['job_id']=='requester-death' for j in a.read('jobs')))
            b.start();check('requester identity persists',b.id==old)
            r=reconcile(b,args,'requester_restart');check('requester restart reconciles original signed receipt once',r['status']=='COMPLETED' and Provider.calls==before+1)
            b.call('resources.refresh');ad=b.read('models')['remote'][0];check('resource advertisement was refreshed after restart',ad['payload']['expires_at']>int(time.time()))
            a.call('grant.revoke',{'nonce':g['payload']['nonce']});a.stop();a.start()
            retry_args={**args,'id':'new-after-revocation'};calls_before=Provider.calls
            r=reconcile(b,retry_args,'revoked_restart')
            check('revocation remains effective after executor restart',r['status']=='REFUSED' and 'REVOKED' in r.get('receipt',{}).get('payload',{}).get('reason','') and Provider.calls==calls_before)
            check('signed ledger continuity survives both process stops',a.read('doctor')['ledger_chain']=='VERIFIED' and b.read('doctor')['ledger_chain']=='VERIFIED')
        finally:
            for n in reversed(nodes):n.stop()
            server.shutdown();server.server_close()
    receipt={'candidate':'0.1.2','environment':EVIDENCE_CLASS,'topology':EVIDENCE_TOPOLOGY,'provider':'owned HTTP contract fixture; no weights','passed':len(checks),'checks':checks,'physical_qualification':'UNVERIFIED','transport':options.transport,'bounded_reconnection_observations':reconnection}
    (EVIDENCE/('receipts/qualification/process-failures'+('' if options.transport=='both' else '-'+options.transport)+'.json')).write_text(json.dumps(receipt,indent=2)+'\n');print(f'Process failure suite passed: {len(checks)} checks.')
if __name__=='__main__':main()
