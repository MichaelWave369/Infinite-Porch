#!/usr/bin/env python3
"""Build a portable, foreground-only node bundle for the current OS."""
import argparse,datetime,hashlib,json,os,pathlib,platform,shutil,subprocess,tomllib,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
(EVIDENCE/'receipts/qualification').mkdir(parents=True,exist_ok=True)
(EVIDENCE/'evidence').mkdir(parents=True,exist_ok=True)
def main():
    p=argparse.ArgumentParser();p.add_argument('--skip-build',action='store_true');p.add_argument('--out',type=pathlib.Path,default=ROOT/'dist');a=p.parse_args()
    if not a.skip_build:
        subprocess.run(['cargo','build','--release','--workspace','--locked'],cwd=ROOT,check=True)
        subprocess.run(['npm.cmd' if os.name=='nt' else 'npm','run','build'],cwd=ROOT,check=True)
    version=tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version']
    arch={'amd64':'x86_64','aarch64':'arm64'}.get(platform.machine().lower(),platform.machine().lower())
    label=f'infinite-porch-{version}-{platform.system().lower()}-{arch}'
    destination=a.out/label
    if destination.exists():raise SystemExit('Package destination already exists; choose a fresh --out directory')
    destination.mkdir(parents=True)
    (destination/'bin').mkdir(exist_ok=True);extension='.exe' if os.name=='nt' else ''
    for name in ['porch','porch-node']:shutil.copy2(ROOT/'target/release'/(name+extension),destination/'bin'/(name+extension))
    shutil.copytree(ROOT/'apps/desktop/dist',destination/'ui',dirs_exist_ok=True)
    shutil.copytree(ROOT/'packages/sdk/dist',destination/'sdk/dist')
    shutil.copy2(ROOT/'packages/sdk/package.json',destination/'sdk/package.json')
    shutil.copytree(ROOT/'field-kit',destination/'field-kit',ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
    if os.environ.get('PORCH_GATE_REPORT'):shutil.copy2(os.environ['PORCH_GATE_REPORT'],destination/'qualification-gates.json')
    for name in ['README.md','LICENSE','START_HERE.txt','Cargo.lock','package-lock.json','rust-toolchain.toml']:shutil.copy2(ROOT/name,destination/name)
    shutil.copytree(ROOT/'docs',destination/'docs',dirs_exist_ok=True)
    shutil.copy2(ROOT/'config.example.json',destination/'config.example.json')
    shutil.copytree(ROOT/'scripts',destination/'scripts',dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
    (destination/'start.sh').write_text('#!/bin/sh\nset -eu\nPORCH_BUNDLE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nexec "$PORCH_BUNDLE_DIR/bin/porch-node" --data "$PORCH_BUNDLE_DIR/state" --ui "$PORCH_BUNDLE_DIR/ui" "$@"\n')
    (destination/'start.sh').chmod(0o755)
    (destination/'start.ps1').write_text('& "$PSScriptRoot/bin/porch-node.exe" --data "$PSScriptRoot/state" --ui "$PSScriptRoot/ui" @args\n')
    def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
    timestamp=datetime.datetime.fromtimestamp(int(os.environ.get('SOURCE_DATE_EPOCH',int(datetime.datetime.now(datetime.timezone.utc).timestamp()))),datetime.timezone.utc)
    manifest={'candidate':version,'protocol_version':1,'commit':git('rev-parse','HEAD'),'branch':git('branch','--show-current'),'source_tree_dirty_at_packaging':bool(git('status','--porcelain','--untracked-files=no')),'platform':platform.platform(),'architecture':platform.machine(),'build_timestamp':timestamp.isoformat(),'rust_toolchain':tomllib.loads((ROOT/'rust-toolchain.toml').read_text())['toolchain']['channel'],'official_release_signature':False,'physical_qualification':'UNVERIFIED','reproducibility':'SOURCE_DATE_EPOCH normalizes zip times. Compiler/linker, target libraries and browser asset builds can still vary. No bitwise reproducibility claim.','files':{}}
    for file in sorted(destination.rglob('*')):
        if file.is_file():manifest['files'][file.relative_to(destination).as_posix()]=hashlib.sha256(file.read_bytes()).hexdigest()
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    archive=a.out/(label+'.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for file in sorted(destination.rglob('*')):
            if file.is_file():
                info=zipfile.ZipInfo(file.relative_to(a.out).as_posix(),timestamp.timetuple()[:6]);info.external_attr=(file.stat().st_mode & 0xffff)<<16;info.compress_type=zipfile.ZIP_DEFLATED
                z.writestr(info,file.read_bytes())
    archive.with_suffix('.zip.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n')
    print(archive)
if __name__=='__main__':main()
