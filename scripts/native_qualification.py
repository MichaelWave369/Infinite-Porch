#!/usr/bin/env python3
"""Run and retain each applicable native gate. Never uploads runtime state."""
import argparse,datetime,hashlib,json,os,pathlib,platform,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',type=pathlib.Path,required=True);p.add_argument('--skip-dependency-audit',action='store_true');a=p.parse_args()
    out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ,PORCH_EVIDENCE_ROOT=str(out/'checks'))
    npm='npm.cmd' if os.name=='nt' else 'npm';npx='npx.cmd' if os.name=='nt' else 'npx'
    commands=[('toolchain',[sys.executable,'scripts/check_toolchain.py']),('format',['cargo','fmt','--all','--','--check']),('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),('rust-tests',['cargo','test','--workspace','--locked']),('debug-build',['cargo','build','--workspace','--locked']),('npm-ci',[npm,'ci']),('typescript',[npm,'run','typecheck']),('desktop-sdk-build',[npm,'run','build']),('sdk-tests',[npm,'test']),('cli',[sys.executable,'tests/cli_smoke.py']),('authority-peer-storage',[sys.executable,'tests/acceptance.py']),('qualification',[sys.executable,'tests/qualification_acceptance.py']),('field-correlation',[sys.executable,'tests/field_acceptance.py']),('native-runtime',[sys.executable,'tests/native_runtime.py']),('process-recovery-both',[sys.executable,'tests/process_failures.py']),('process-recovery-quic',[sys.executable,'tests/process_failures.py','--transport','quic']),('transports',[sys.executable,'tests/transport_acceptance.py']),('caps',[sys.executable,'tests/limit_cli.py']),('platform',[sys.executable,'scripts/check_platform.py']),('firewall-structure',[sys.executable,'tests/field_scripts.py']),('browser-install',[npx,'playwright','install',*(['--with-deps'] if platform.system()=='Linux' else []),'chromium']),('ui',[sys.executable,'tests/ui_acceptance.py']),('npm-audit',[npm,'audit','--audit-level=high'])]
    if not a.skip_dependency_audit and platform.system()=='Linux':
        commands.extend([('cargo-audit-install',['cargo','install','cargo-audit','--version','0.22.2','--locked']),('cargo-audit',[sys.executable,'scripts/audit_gate.py'])])
    commands.extend([('portable-build',[sys.executable,'scripts/package.py','--out',str(out/'packages')]),('portable-execution',[sys.executable,'tests/package_acceptance.py','--archive',str(out/'packages'/f"infinite-porch-0.1.2-{platform.system().lower()}-"), '--out',str(out/'checks/receipts/package.json')])])
    results=[]
    def version(cmd):
        try:return subprocess.check_output(cmd,cwd=ROOT,text=True,stderr=subprocess.STDOUT).strip()
        except Exception:return 'UNVERIFIED'
    metadata={'evidence_class':'NATIVE_HOSTED' if env.get('GITHUB_ACTIONS')=='true' else 'LOOPBACK','topology':'NATIVE_HOSTED_LOOPBACK' if env.get('GITHUB_ACTIONS')=='true' else 'LOOPBACK','os':platform.platform(),'architecture':platform.machine(),'runner_os':env.get('RUNNER_OS','managed workspace'),'runner_name':env.get('RUNNER_NAME','UNVERIFIED'),'rust':version(['rustc','--version']), 'node':version(['node','--version']),'python':platform.python_version(),'commit':version(['git','rev-parse','HEAD']),'branch':version(['git','branch','--show-current']),'workflow_commit':env.get('GITHUB_SHA'),'workflow_run_id':env.get('GITHUB_RUN_ID'),'protocol_version':1,'physical_lan':'UNVERIFIED','independent_security_review':'UNVERIFIED','production_qualified':False,'timestamp':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_tree_start':version(['git','write-tree']),'tracked_changes_at_start':version(['git','status','--porcelain','--untracked-files=no'])}
    try:
        for name,cmd in commands:
            if name=='portable-execution':
                archives=list((out/'packages').glob('*.zip'))
                if not archives:results.append({'gate':name,'result':'UNVERIFIED','reason':'No successful package'});continue
                cmd[cmd.index('--archive')+1]=str(archives[0])
            t=time.monotonic();print('RUN '+name,flush=True)
            if name=='browser-install' and env.get('PORCH_BROWSER_PATH'):
                results.append({'gate':name,'result':'PARTIAL','reason':'Existing explicitly selected browser; UI execution gate still required'});continue
            with (out/(name+'.log')).open('w',encoding='utf-8') as log:
                limit=1200 if name in ['clippy','rust-tests','debug-build','cargo-audit-install','portable-build'] else 300
                try:
                    result=subprocess.run(cmd,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,text=True,timeout=limit)
                    exit_code=result.returncode
                except subprocess.TimeoutExpired:
                    log.write(f'\nQUALIFICATION_TIMEOUT: {name} exceeded {limit} seconds; no PASS claim\n')
                    exit_code=124
            record={'gate':name,'result':'PASS' if exit_code==0 else 'FAIL','exit_code':exit_code,'timeout_seconds':limit,'duration_seconds':round(time.monotonic()-t,3),'log':name+'.log'};results.append(record);print(record,flush=True)
            (out/'summary.json').write_text(json.dumps({'metadata':metadata,'gates':results},indent=2)+'\n')
            # Preserve failed gates and collect independent checks; avoid packaging failed builds.
            if name in ['debug-build','npm-ci','desktop-sdk-build'] and exit_code:break
    finally:
        metadata['source_tree_end']=version(['git','write-tree'])
        metadata['tracked_changes_at_end']=version(['git','status','--porcelain','--untracked-files=no'])
        if metadata['source_tree_start']!=metadata['source_tree_end'] or metadata['tracked_changes_at_start'] or metadata['tracked_changes_at_end']:
            results.append({'gate':'source-stability','result':'FAIL','reason':'Qualification requires a clean, immutable tracked checkout'})
        metadata['overall']='FAIL' if any(r['result']=='FAIL' for r in results) else 'PASS'
        (out/'summary.json').write_text(json.dumps({'metadata':metadata,'gates':results},indent=2)+'\n')
        hashes={p.relative_to(out).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file() and p.name!='SHA256SUMS.json'}
        (out/'SHA256SUMS.json').write_text(json.dumps(hashes,indent=2)+'\n')
    return 1 if metadata['overall']=='FAIL' else 0
if __name__=='__main__':sys.exit(main())
