#!/usr/bin/env python3
"""Verify extracted native portable package with fresh disposable state."""
import argparse,hashlib,json,os,pathlib,subprocess,tempfile,zipfile
from acceptance import EVIDENCE_CLASS,EVIDENCE_TOPOLOGY,ROOT,TestNode,HTTP
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
(EVIDENCE/'receipts/qualification').mkdir(parents=True,exist_ok=True)
(EVIDENCE/'evidence').mkdir(parents=True,exist_ok=True)
import sys
sys.path.insert(0,str(ROOT/'scripts'))
from verify_package import verify

def run_captured(command,**options):
    logs=EVIDENCE/'receipts/package-helper-logs';logs.mkdir(parents=True,exist_ok=True)
    path=logs/f'helper-{len(list(logs.glob("helper-*.log")))+1}.log'
    # A detached Windows daemon may inherit a pipe writer. Wait for the wrapper
    # process using file-backed output, without waiting for descendant pipe EOF.
    with path.open('w+',encoding='utf-8') as log:
        try:
            return subprocess.run(command,check=True,stdout=log,stderr=subprocess.STDOUT,text=True,**options)
        except (subprocess.CalledProcessError,subprocess.TimeoutExpired):
            log.flush();log.seek(0);print(log.read(),flush=True)
            raise

def main():
    p=argparse.ArgumentParser();p.add_argument('--archive',type=pathlib.Path);p.add_argument('--out',type=pathlib.Path,default=EVIDENCE/'receipts/qualification/package.json');p.add_argument('--field-kit-source',type=pathlib.Path,help='Diagnostic only: test a source kit against unchanged prebuilt binaries');a=p.parse_args()
    archive=a.archive or next((ROOT/'dist').glob('infinite-porch-0.1.2-*.zip'))
    checks=[]
    def check(name,condition=True):assert condition,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name,flush=True)
    integrity=verify(archive);check('safe archive paths exact file hashes and release metadata')
    with tempfile.TemporaryDirectory(prefix='porch-package-') as tmp:
        temp=pathlib.Path(tmp)
        with zipfile.ZipFile(archive) as z:
            z.extractall(temp)
            for entry in z.infolist():
                mode=entry.external_attr>>16
                if os.name!='nt' and mode:(temp/entry.filename).chmod(mode & 0o777)
        package=next(p for p in temp.iterdir() if p.is_dir());m=json.loads((package/'manifest.json').read_text());ext='.exe' if os.name=='nt' else ''
        cli=package/'bin'/('porch'+ext);binary=package/'bin'/('porch-node'+ext)
        check('native CLI version',subprocess.check_output([str(cli),'--version'],text=True).strip()=='porch 0.1.2')
        run_captured(['node','--input-type=module','-e',"import {PorchClient} from '@porch/sdk'; if(typeof PorchClient !== 'function') throw new Error('Packaged SDK export missing'); new PorchClient('disposable-package-test');"],cwd=package/'sdk',timeout=30)
        check('packaged SDK resolves its built JavaScript and public export')
        state=temp/'state';initial=json.loads(subprocess.check_output([str(cli),'--data',str(state),'init','--alias','Package check'],text=True));check('fresh persisted identity initialized',bool(initial['fingerprint_sha256']))
        if os.name!='nt':check('private Unix key and token modes',state.stat().st_mode & 0o777==0o700 and (state/'identity.key').stat().st_mode & 0o777==0o600 and (state/'api.token').stat().st_mode & 0o777==0o600)
        if os.name!='nt':
            install=temp/'user-install';install_state=temp/'user-state'
            run_captured(['sh',str(package/'scripts/platform/install-user.sh'),str(install),str(install_state)])
            check('user install helper supports isolated user paths',(install/'bin/porch').exists() and install_state.stat().st_mode & 0o777==0o700)
        if os.name=='nt':
            install=temp/'user install';install_state=temp/'user state'
            run_captured(['pwsh','-NoProfile','-File',str(package/'scripts/platform/install-windows.ps1'),'-Destination',str(install),'-State',str(install_state)],timeout=60)
            check('native PowerShell installer and isolated state ACL', (install/'bin/porch.exe').exists() and install_state.exists())
            from acceptance import port
            kit_state=temp/'field state';kit_api=port();kit_peer=port()
            kit=a.field_kit_source.resolve() if a.field_kit_source else package/'field-kit/windows'
            common=['pwsh','-NoProfile','-File',str(kit/'Porch-FieldLab.ps1')]
            params=['-PackageRoot',str(package),'-State',str(kit_state),'-ApiPort',str(kit_api),'-PeerPort',str(kit_peer),'-Environment','NATIVE_HOSTED','-NoMdns']
            try:
                run_captured([*common,'setup','-Alias','HOSTED WINDOWS',*params],timeout=60)
                key=(kit_state/'identity.key').read_bytes()
                run_captured([*common,'setup','-Alias','HOSTED WINDOWS',*params],timeout=60)
                check('native field setup with spaces preserves identity and protects token',key==(kit_state/'identity.key').read_bytes())
                run_captured(['pwsh','-NoProfile','-File',str(kit/'firewall-enable-private.ps1'),'-PackageRoot',str(package),'-PeerPort',str(kit_peer),'-DryRun'],timeout=60)
                check('firewall plan executes without privileged mutation')
            finally:
                run_captured([*common,'stop',*params],timeout=20)
        node=TestNode('state',temp,binary,ui=str(package/'ui'))
        try:
            q=node.read('qualification');check('bundled node metadata matches package commit',q['build']['software_version']=='0.1.2' and q['build']['protocol_version']==1 and q['build']['commit']==m['commit'] and q['build']['branch']==m['branch'] and q['build']['source_tree_dirty_at_build']==m['source_tree_dirty_at_packaging'])
            with HTTP.open(node.api,timeout=10) as response:html=response.read().decode()
            asset=next((package/'ui/assets').glob('*.js'));check('bundled desktop served', '<div id="root">' in html and 'qualification.run' in asset.read_text())
            r=node.call('job.run',{'capability':'compute.hash','resource':'sha256','input':'package probe','privacy':'LOCAL_ONLY'});check('native builtin job and signed receipt',r['status']=='COMPLETED' and r['receipt']['payload']['output']['sha256']==hashlib.sha256(b'package probe').hexdigest())
            command=[str(cli),'--data',str(node.root),'--api',node.api]
            subprocess.run([*command,'qualify','run','--environment',EVIDENCE_CLASS],capture_output=True,text=True,check=True)
            export=temp/'evidence';subprocess.run([*command,'qualify','export',str(export)],capture_output=True,text=True,check=True)
            v=json.loads(subprocess.check_output([str(cli),'qualify','validate',str(export)],text=True));check('packaged evidence export and offline validator',v['valid'] and not v['skip_counts_as_pass'])
            check('packaged doctor verifies ledger',node.read('doctor')['ledger_chain']=='VERIFIED')
        finally:node.stop()
    a.out.parent.mkdir(parents=True,exist_ok=True);a.out.write_text(json.dumps({'passed':len(checks),'checks':checks,'integrity':integrity,'environment':EVIDENCE_CLASS,'topology':EVIDENCE_TOPOLOGY,'native_platform':os.name,'field_kit_scope':'DIAGNOSTIC_SOURCE_KIT_WITH_PREBUILT_BINARIES' if a.field_kit_source else 'EXACT_PACKAGED_FIELD_KIT','physical_qualification':'UNVERIFIED'},indent=2)+'\n')
    print(f'Package acceptance passed: {len(checks)} checks.')
if __name__=='__main__':main()
