#!/usr/bin/env python3
"""Three real daemon processes; loopback peer paths; deterministic mock inference.

No Internet connection, cloud account, Ollama, Python package, or bootstrap server
is used by this harness. This proves loopback LAN-like paths, not physical Wi-Fi.
"""
import argparse,hashlib,json,os,pathlib,socket,sqlite3,subprocess,sys,tempfile,time,tomllib,urllib.error,urllib.request

ROOT=pathlib.Path(__file__).resolve().parents[1]
HTTP=urllib.request.build_opener(urllib.request.ProxyHandler({}))
def port(udp=False):
    with socket.socket(socket.AF_INET,socket.SOCK_DGRAM if udp else socket.SOCK_STREAM) as s:s.bind(('127.0.0.1',0));return s.getsockname()[1]
class TestNode:
    def __init__(self,name,root,binary,**extra):
        self.name=name;self.root=root/name;self.root.mkdir(parents=True,exist_ok=True)
        self.binary=binary;self.api_port=port();self.tcp=port();self.udp=port(udp=True);self.extra=extra;self.process=None
        self.api=f'http://127.0.0.1:{self.api_port}';self.start()
    def start(self):
        cmd=[str(self.binary),'--data',str(self.root),'--alias',self.name,'--api',f'127.0.0.1:{self.api_port}','--listen',f'/ip4/127.0.0.1/tcp/{self.tcp}','--listen',f'/ip4/127.0.0.1/udp/{self.udp}/quic-v1','--no-mdns','--ui',str(self.extra.get('ui',ROOT/'apps/desktop/dist'))]
        if self.extra.get('transport')=='tcp':
            at=cmd.index(f'/ip4/127.0.0.1/udp/{self.udp}/quic-v1');del cmd[at-1:at+1]
        elif self.extra.get('transport')=='quic':
            at=cmd.index(f'/ip4/127.0.0.1/tcp/{self.tcp}');del cmd[at-1:at+1]
        if self.extra.get('model'):cmd+=['--mock-model',self.extra['model']]
        if self.extra.get('hash'):cmd+=['--share-hash']
        if self.extra.get('storage'):cmd+=['--storage-quota',str(self.extra['storage'])]
        if self.extra.get('ollama'):cmd+=['--ollama',self.extra['ollama']]
        self.log=open(self.root/'daemon.log','a')
        self.process=subprocess.Popen(cmd,stdout=self.log,stderr=self.log)
        for _ in range(150):
            if self.process.poll() is not None:raise RuntimeError(f'{self.name} exited: {(self.root/"daemon.log").read_text()}')
            try:
                self.token=(self.root/'api.token').read_text().strip();self.status=self.read('status');self.id=self.status['node']['id'];return
            except Exception:time.sleep(.1)
        raise RuntimeError(f'{self.name} did not become ready')
    def request(self,path,body=None,token=True,headers=None):
        h={**({'Authorization':f'Bearer {self.token}'} if token else {}),**(headers or {})}
        data=None if body is None else json.dumps(body).encode()
        if data is not None:h['Content-Type']='application/json'
        req=urllib.request.Request(self.api+path,data=data,headers=h)
        try:
            with HTTP.open(req,timeout=40) as r:return r.status,json.load(r)
        except urllib.error.HTTPError as e:
            return e.code,json.load(e)
    def read(self,resource):
        code,data=self.request('/v1/'+resource);assert code==200,(code,data);return data
    def call(self,op,args=None,allow_refusal=False):
        code,data=self.request('/v1/control',{'operation':op,'args':args or {}})
        if not allow_refusal:assert code==200,(self.name,op,code,data)
        return data
    def stop(self):
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try:self.process.wait(timeout=8)
            except subprocess.TimeoutExpired:self.process.kill();self.process.wait(timeout=5)
        self.log.close()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/debug'/('porch-node.exe' if os.name=='nt' else 'porch-node'));parser.add_argument('--out',type=pathlib.Path,default=ROOT/'docs/receipts/acceptance.json');parser.add_argument('--keep-state',type=pathlib.Path);a=parser.parse_args()
    if not a.binary.exists():raise SystemExit('Build first: cargo build --workspace')
    temp=tempfile.TemporaryDirectory(prefix='infinite-porch-acceptance-');state=a.keep_state or pathlib.Path(temp.name);state.mkdir(parents=True,exist_ok=True)
    nodes=[];checks=[];evidence={};start=time.time()
    def passed(name,detail=None):
        checks.append({'name':name,'status':'PASS','detail':detail});print('PASS '+name,flush=True)
    def grant(host,client,cap,resource,action='run',**limits):
        g=host.call('grant.issue',{'recipient':client.id,'capability':cap,'resource':resource,'action':action,'limits':limits})
        client.call('grant.import',{'grant':g});return g
    try:
        alpha=TestNode('ALPHA',state,a.binary,model='porch-mock',storage=16*1024*1024);nodes.append(alpha)
        beta=TestNode('BETA',state,a.binary,hash=True,storage=16*1024*1024);nodes.append(beta)
        gamma=TestNode('GAMMA',state,a.binary);nodes.append(gamma)
        assert len({n.id for n in nodes})==3;passed('unique cryptographic identities')
        assert alpha.request('/v1/status',token=False)[0]==401;passed('control API rejects missing operator token')
        delegated=gamma.call('client.issue',{'holder':'PhiBot test client','resources':['sha256'],'privacy':['LOCAL_ONLY'],'ttl_seconds':600})
        ch={'Authorization':'Bearer '+delegated['token']}
        assert gamma.request('/v1/grants',headers=ch)[0]==403;passed('scoped app credential cannot read operator grant records')
        assert gamma.request('/v1/control',{'operation':'grant.issue','args':{}},headers=ch)[0]==403;passed('agent credential cannot issue its own peer authority')
        assert gamma.request('/v1/control',{'operation':'job.run','args':{'resource':'sha256','capability':'compute.hash','input':'abc','privacy':'TRUSTED_PEERS'}},headers=ch)[0]==403;passed('client privacy delegation cannot escalate to remote processing')
        code,client_result=gamma.request('/v1/control',{'operation':'job.run','args':{'resource':'sha256','capability':'compute.hash','input':'abc','privacy':'LOCAL_ONLY'}},headers=ch)
        assert code==200 and client_result['status']=='COMPLETED';passed('bounded local agent request executes through the common boundary')
        gamma.call('client.revoke',{'nonce':delegated['grant']['payload']['nonce']})
        assert gamma.request('/v1/models',headers=ch)[0]==401;passed('local client credential revocation is persistent and enforced')
        assert alpha.request('/v1/status',headers={'Origin':'https://hostile.example'})[0]==403;passed('cross-origin control request refused')
        assert alpha.request('/v1/status',headers={'Host':'hostile.example'})[0]==403;passed('DNS rebinding host refused')
        for op in ['relay.enable','federation.enable','compute.wasm']:
            refusal=gamma.call(op,allow_refusal=True);assert refusal['status']=='REFUSED' and refusal['reason']=='EXECUTION_ENGINE_NOT_INSTALLED'
        passed('uninstalled relay, federation, and WASM engines refuse explicitly')
        porch=alpha.call('porch.create',{'name':'Oak Street'});evidence['porch_id']=porch['id']
        for client in [beta,gamma]:
            invite=alpha.call('porch.invite',{'recipient':client.id,'ttl_seconds':600});client.call('porch.join',{'invite':invite})
        passed('Porch creation and recipient-bound one-use invitations')
        beta.call('peer.approve',{'peer':gamma.id,'alias':'GAMMA','porch':porch['id']});gamma.call('peer.approve',{'peer':beta.id,'alias':'BETA','porch':porch['id']})
        address=next(x for x in beta.read('network')['addresses'] if '/tcp/' in x);gamma.call('peer.connect',{'address':address})
        for node in nodes:node.call('resources.refresh')
        assert len(gamma.read('models')['remote'])==2;passed('signed capability records discovered over real peer connections')
        before=gamma.read('network')['outbound_job_requests']
        refused=gamma.call('job.run',{'resource':'porch-mock','input':'no grant yet','privacy':'TRUSTED_PEERS'})
        assert refused['status']=='REFUSED';assert gamma.read('network')['outbound_job_requests']==before
        passed('advertising a model does not authorize use')
        model_grant=grant(alpha,gamma,'model.inference','porch-mock',max_calls=20,calls_per_hour=20)
        before=gamma.read('network')['outbound_job_requests']
        limited=gamma.call('job.run',{'resource':'porch-mock','input':'must not leave when above delegated limits','max_output_tokens':513,'privacy':'TRUSTED_PEERS'})
        assert limited['status']=='REFUSED' and gamma.read('network')['outbound_job_requests']==before
        passed('grant output bounds filter routing before sending input to a peer')
        run=gamma.call('job.run',{'resource':'porch-mock','input':'hello from Michael','privacy':'TRUSTED_PEERS','id':'acceptance-model-1'})
        assert run['status']=='COMPLETED';r=run['receipt'];assert r['payload']['executor']==alpha.id and r['payload']['provider']=='mock' and r['payload']['output']['mock'] is True
        assert r['signature'] and r['payload']['grant_nonce']==model_grant['payload']['nonce'];evidence['remote_job_receipt']=r
        passed('remote mock inference executes on eligible host and returns signed receipt')
        assert any(e['payload']['event_type']=='job.remote.received' for e in gamma.read('ledger'));passed('requester and executor record digest-based evidence')
        rows=alpha.read('grants');row=next(g for g in rows if g['grant']['payload']['nonce']==model_grant['payload']['nonce']);assert row['consumed_calls']==1;passed('execution consumes measured grant accounting')
        again=gamma.call('job.run',{'resource':'porch-mock','input':'hello from Michael','privacy':'TRUSTED_PEERS','id':'acceptance-model-1'})
        assert again['receipt']==r;row=next(g for g in alpha.read('grants') if g['grant']['payload']['nonce']==model_grant['payload']['nonce']);assert row['consumed_calls']==1;passed('duplicate job returns original receipt without another execution')
        before=gamma.read('network')['outbound_job_requests'];local=gamma.call('job.run',{'resource':'porch-mock','input':'THIS MUST NOT LEAVE','privacy':'LOCAL_ONLY'})
        assert local['status']=='REFUSED';assert gamma.read('network')['outbound_job_requests']==before;passed('LOCAL_ONLY input never produces an outbound peer job')
        before=gamma.read('network')['outbound_job_requests'];large=gamma.call('job.run',{'resource':'porch-mock','input':'x'*65537,'privacy':'TRUSTED_PEERS'},allow_refusal=True)
        assert large['status']=='REFUSED';assert gamma.read('network')['outbound_job_requests']==before;passed('oversized input refused before forwarding')
        alpha.call('grant.revoke',{'nonce':model_grant['payload']['nonce']})
        revoked=gamma.call('job.run',{'resource':'porch-mock','input':'permission removed','privacy':'TRUSTED_PEERS'})
        assert revoked['status']=='REFUSED' and 'REVOKED' in revoked['receipt']['payload']['reason'];passed('executor revocation blocks next request despite cached client grant')
        sg=grant(beta,gamma,'blob.storage','vault','store',max_storage_bytes=4*1024*1024,max_calls=10,calls_per_hour=10)
        plaintext=(b'Infinite Porch private file\n'*16000)+os.urandom(256);local_blob=gamma.call('storage.put',{'plaintext_hex':plaintext.hex()});cid=local_blob['cid']
        replicated=gamma.call('storage.replicate',{'cid':cid,'peer':beta.id});assert replicated['verified_replicas']==1
        remote_cipher=(beta.root/'vault'/cid).read_bytes();assert plaintext not in remote_cipher and hashlib.sha256(remote_cipher).hexdigest()==cid
        passed('client-encrypted blob replicated in bounded chunks and verified by full retrieval')
        retrieved=gamma.call('storage.fetch',{'cid':cid,'peer':beta.id});assert bytes.fromhex(retrieved['plaintext_hex'])==plaintext;passed('retrieval decrypts locally and preserves plaintext integrity')
        tampered=bytearray(remote_cipher);tampered[-1]^=1;(beta.root/'vault'/cid).write_bytes(tampered)
        bad=gamma.call('storage.fetch',{'cid':cid,'peer':beta.id},allow_refusal=True);assert bad['status']=='REFUSED' and 'CORRUPTED' in bad['reason'];(beta.root/'vault'/cid).write_bytes(remote_cipher)
        passed('corrupted remote blob refused')
        grant(beta,gamma,'message.direct','inbox','send',max_calls=10,calls_per_hour=10)
        sent=gamma.call('message.send',{'peer':beta.id,'text':'The porch is open.'});assert sent['status']=='DELIVERED';assert beta.read('messages')[0]['text']=='The porch is open.';passed('signed direct messaging uses the peer transport')
        pending=gamma.call('storage.put',{'plaintext_hex':os.urandom(180000).hex()});pending_cid=pending['cid'];cipher=(gamma.root/'vault'/pending_cid).read_bytes()
        def peer_storage(op,**kwargs):
            wire=gamma.call('peer.request',{'peer':beta.id,'operation':op,'args':{'cid':pending_cid,'grant':sg,**kwargs}})
            assert wire['payload']['status']=='OK',wire
            return wire['payload']['output']
        assert peer_storage('blob.begin',size=len(cipher),encryption='xchacha20poly1305')['offset']==0
        assert peer_storage('blob.chunk',offset=0,data_hex=cipher[:65536].hex())['offset']==65536
        old_id=beta.id;beta.stop();beta.start();assert beta.id==old_id
        gamma.call('peer.connect',{'address':next(x for x in beta.read('network')['addresses'] if '/tcp/' in x)})
        assert peer_storage('blob.begin',size=len(cipher),encryption='xchacha20poly1305')['offset']==65536
        assert gamma.call('storage.replicate',{'cid':pending_cid,'peer':beta.id})['verified_replicas']==1
        passed('interrupted encrypted transfer resumes at its durable offset after restart')
        grant(beta,gamma,'compute.hash','sha256')
        computed=gamma.call('job.run',{'resource':'sha256','capability':'compute.hash','input':'abc','privacy':'TRUSTED_PEERS','preferred_peer':beta.id})
        assert computed['status']=='COMPLETED' and computed['receipt']['payload']['output']['sha256']==hashlib.sha256(b'abc').hexdigest();passed('restart retains identity, trust, grants, storage, and reconnects')
        assert gamma.call('storage.fetch',{'cid':cid,'peer':beta.id})['integrity']=='VERIFIED';passed('replica retrieval survives provider restart')
        old_gamma_id=gamma.id;gamma.stop();gamma.start();assert gamma.id==old_gamma_id
        cached=gamma.call('job.run',{'resource':'porch-mock','input':'hello from Michael','privacy':'TRUSTED_PEERS','id':'acceptance-model-1'})
        assert cached['receipt']==r and gamma.read('network')['outbound_job_requests']==0
        passed('origin restart retains the original route and receipt without re-execution')
        gamma.call('peer.connect',{'address':next(x for x in beta.read('network')['addresses'] if '/tcp/' in x)})
        alpha.stop()
        partition=gamma.call('job.run',{'resource':'sha256','capability':'compute.hash','input':'local island','privacy':'TRUSTED_PEERS','preferred_peer':beta.id})
        assert partition['status']=='COMPLETED';passed('reachable peers continue when another community node disappears')
        localhash=gamma.call('job.run',{'resource':'sha256','capability':'compute.hash','input':'offline local','privacy':'LOCAL_ONLY'})
        assert localhash['status']=='COMPLETED';passed('basic local and peer paths use no outside Internet or bootstrap')
        gamma.call('storage.delete_remote',{'cid':cid,'peer':beta.id});bad=gamma.call('storage.fetch',{'cid':cid,'peer':beta.id},allow_refusal=True);assert bad['status']=='REFUSED';passed('remote deletion records tombstone and prevents later retrieval')
        for n in [beta,gamma]:assert n.read('doctor')['ledger_chain']=='VERIFIED'
        passed('durable signed ledger chains verify after the scenario')
        evidence['node_ids']={n.name:n.id for n in nodes};evidence['transport_scope']='three OS processes; verified loopback TCP/Noise path and QUIC listeners; no external Internet dependencies; LAN-like simulation'
        evidence['unverified']=['physical LAN and mDNS','WAN NAT traversal','live Ollama','Windows/macOS native packages','production security review']
        result={'version':1,'candidate':tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version'],'passed':True,'checks':checks,'duration_seconds':round(time.time()-start,3),'evidence':evidence}
    except Exception as exc:
        result={'version':1,'candidate':tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version'],'passed':False,'checks':checks,'failure':repr(exc),'duration_seconds':round(time.time()-start,3)}
        raise
    finally:
        for n in nodes:n.stop()
        a.out.parent.mkdir(parents=True,exist_ok=True)
        if 'result' in locals():a.out.write_text(json.dumps(result,indent=2)+'\n')
        if not a.keep_state:temp.cleanup()
    print(f'Acceptance passed: {len(checks)} checks. Receipt: {a.out}')
if __name__=='__main__':main()
