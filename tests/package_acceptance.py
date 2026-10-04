#!/usr/bin/env python3
"""Verify extracted native portable package with fresh disposable state."""
import argparse,hashlib,json,os,pathlib,subprocess,tempfile,zipfile
from acceptance import ROOT,TestNode,HTTP
import sys
sys.path.insert(0,str(ROOT/'scripts'))
from verify_package import verify

def main():
    p=argparse.ArgumentParser();p.add_argument('--archive',type=pathlib.Path);p.add_argument('--out',type=pathlib.Path,default=ROOT/'docs/receipts/qualification/package.json');a=p.parse_args()
    archive=a.archive or next((ROOT/'dist').glob('infinite-porch-0.1.1-*.zip'))
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
        check('native CLI version',subprocess.check_output([str(cli),'--version'],text=True).strip()=='porch 0.1.1')
        state=temp/'state';initial=json.loads(subprocess.check_output([str(cli),'--data',str(state),'init','--alias','Package check'],text=True));check('fresh persisted identity initialized',bool(initial['fingerprint_sha256']))
        if os.name!='nt':check('private Unix key and token modes',state.stat().st_mode & 0o777==0o700 and (state/'identity.key').stat().st_mode & 0o777==0o600 and (state/'api.token').stat().st_mode & 0o777==0o600)
        if os.name!='nt':
            install=temp/'user-install';install_state=temp/'user-state'
            subprocess.run(['sh',str(package/'scripts/platform/install-user.sh'),str(install),str(install_state)],capture_output=True,text=True,check=True)
            check('user install helper supports isolated user paths',(install/'bin/porch').exists() and install_state.stat().st_mode & 0o777==0o700)
        node=TestNode('state',temp,binary,ui=str(package/'ui'))
        try:
            q=node.read('qualification');check('bundled node metadata matches package commit',q['build']['software_version']=='0.1.1' and q['build']['protocol_version']==1 and q['build']['commit']==m['commit'] and q['build']['branch']==m['branch'] and q['build']['source_tree_dirty_at_build']==m['source_tree_dirty_at_packaging'])
            with HTTP.open(node.api,timeout=10) as response:html=response.read().decode()
            asset=next((package/'ui/assets').glob('*.js'));check('bundled desktop served', '<div id="root">' in html and 'qualification.run' in asset.read_text())
            r=node.call('job.run',{'capability':'compute.hash','resource':'sha256','input':'package probe','privacy':'LOCAL_ONLY'});check('native builtin job and signed receipt',r['status']=='COMPLETED' and r['receipt']['payload']['output']['sha256']==hashlib.sha256(b'package probe').hexdigest())
            command=[str(cli),'--data',str(node.root),'--api',node.api]
            subprocess.run([*command,'qualify','run','--environment','LOOPBACK'],capture_output=True,text=True,check=True)
            export=temp/'evidence';subprocess.run([*command,'qualify','export',str(export)],capture_output=True,text=True,check=True)
            v=json.loads(subprocess.check_output([str(cli),'qualify','validate',str(export)],text=True));check('packaged evidence export and offline validator',v['valid'] and not v['skip_counts_as_pass'])
            check('packaged doctor verifies ledger',node.read('doctor')['ledger_chain']=='VERIFIED')
        finally:node.stop()
    a.out.parent.mkdir(parents=True,exist_ok=True);a.out.write_text(json.dumps({'passed':len(checks),'checks':checks,'integrity':integrity,'native_platform':os.name,'physical_qualification':'UNVERIFIED'},indent=2)+'\n')
    print(f'Package acceptance passed: {len(checks)} checks.')
if __name__=='__main__':main()
