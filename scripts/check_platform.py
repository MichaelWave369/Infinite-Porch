#!/usr/bin/env python3
"""Check shell syntax and CLI contracts; never claims native Windows/macOS runs."""
import json,os,pathlib,shutil,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
(EVIDENCE/'receipts/qualification').mkdir(parents=True,exist_ok=True)
(EVIDENCE/'evidence').mkdir(parents=True,exist_ok=True)
def main():
    cli=ROOT/'target/debug'/('porch.exe' if os.name=='nt' else 'porch');checks=[]
    contracts=[(['qualify','run'],['--environment','--phase','--separate-machines-confirmed','--wan-condition-confirmed','--message']),(['qualify','export'],['<DIRECTORY>']),(['qualify','join'],['--fingerprint']),(['grant','issue'],['--max-duration-ms']),(['configure-limits'],['<FILE>'])]
    for command,expected in contracts:
        result=subprocess.run([str(cli),*command,'--help'],capture_output=True,text=True,check=True)
        assert all(x in result.stdout for x in expected),(command,expected)
        checks.append('CLI contract '+ ' '.join(command))
    if shutil.which('sh'):
        subprocess.run(['sh','-n',str(ROOT/'scripts/platform/install-user.sh')],check=True);checks.append('POSIX install helper syntax')
    ps=shutil.which('pwsh');native_ps='UNVERIFIED: PowerShell unavailable'
    if ps:
        for p in [*(ROOT/'scripts/platform').glob('*.ps1'),*(ROOT/'field-kit/windows').glob('*.ps1')]:
            env=dict(os.environ,PORCH_PS_CHECK=str(p))
            subprocess.run([ps,'-NoProfile','-Command',"$tokens=$null; $errors=$null; [System.Management.Automation.Language.Parser]::ParseFile($env:PORCH_PS_CHECK,[ref]$tokens,[ref]$errors) | Out-Null; if ($errors.Count) { $errors | Out-String | Write-Error; exit 1 }"],env=env,check=True)
        native_ps='PASS: syntax only; native installation/runtime still UNVERIFIED'
    report={'checks':checks,'passed':len(checks),'powershell_syntax':native_ps,'windows_runtime':'UNVERIFIED','macos_runtime':'UNVERIFIED','systemd_execution':'UNVERIFIED'}
    (EVIDENCE/'receipts/qualification/platform-contracts.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
