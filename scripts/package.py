#!/usr/bin/env python3
"""Build a portable, foreground-only node bundle for the current OS."""
import argparse,hashlib,json,os,pathlib,platform,shutil,subprocess,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser();p.add_argument('--skip-build',action='store_true');p.add_argument('--out',type=pathlib.Path,default=ROOT/'dist');a=p.parse_args()
    if not a.skip_build:
        subprocess.run(['cargo','build','--release','--workspace','--locked'],cwd=ROOT,check=True)
        subprocess.run(['npm.cmd' if os.name=='nt' else 'npm','run','build'],cwd=ROOT,check=True)
    label=f'infinite-porch-0.1.0-{platform.system().lower()}-{platform.machine().lower()}'
    destination=a.out/label;destination.mkdir(parents=True,exist_ok=True)
    (destination/'bin').mkdir(exist_ok=True);extension='.exe' if os.name=='nt' else ''
    for name in ['porch','porch-node']:shutil.copy2(ROOT/'target/release'/(name+extension),destination/'bin'/(name+extension))
    shutil.copytree(ROOT/'apps/desktop/dist',destination/'ui',dirs_exist_ok=True)
    for name in ['README.md','LICENSE']:shutil.copy2(ROOT/name,destination/name)
    shutil.copytree(ROOT/'docs',destination/'docs',dirs_exist_ok=True)
    shutil.copy2(ROOT/'config.example.json',destination/'config.example.json')
    (destination/'start.sh').write_text('#!/bin/sh\nset -eu\nPORCH_BUNDLE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nexec "$PORCH_BUNDLE_DIR/bin/porch-node" --data "$PORCH_BUNDLE_DIR/state" --ui "$PORCH_BUNDLE_DIR/ui" "$@"\n')
    (destination/'start.sh').chmod(0o755)
    (destination/'start.ps1').write_text('& "$PSScriptRoot/bin/porch-node.exe" --data "$PSScriptRoot/state" --ui "$PSScriptRoot/ui" @args\n')
    manifest={'candidate':'0.1.0','platform':platform.platform(),'architecture':platform.machine(),'files':{}}
    for file in sorted(destination.rglob('*')):
        if file.is_file():manifest['files'][str(file.relative_to(destination))]=hashlib.sha256(file.read_bytes()).hexdigest()
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    archive=a.out/(label+'.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for file in sorted(destination.rglob('*')):
            if file.is_file():z.write(file,str(file.relative_to(a.out)))
    print(archive)
if __name__=='__main__':main()
