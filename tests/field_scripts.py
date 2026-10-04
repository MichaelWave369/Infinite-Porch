#!/usr/bin/env python3
"""Safety contracts plus real PowerShell parser execution when available."""
import json,os,pathlib,re,shutil,subprocess
from acceptance import ROOT
EVIDENCE=pathlib.Path(os.environ.get('PORCH_EVIDENCE_ROOT',str(ROOT/'docs')))
def main():
    kit=ROOT/'field-kit/windows';checks=[]
    def check(name,ok):assert ok,name;checks.append({'name':name,'result':'PASS'});print('PASS '+name)
    enable=(kit/'firewall-enable-private.ps1').read_text();remove=(kit/'firewall-remove.ps1').read_text();field=(kit/'Porch-FieldLab.ps1').read_text()
    check('firewall Private application LocalSubnet peer-only contract',all(s in enable for s in ['-Program $program','-Profile Private','-RemoteAddress LocalSubnet','@(7331,11434)','-LocalPort $PeerPort','Assert-PorchAdministrator','SupportsShouldProcess','$DryRun','rollback']))
    check('firewall rollback limited to per-user Porch group',all(s in remove for s in ['Get-PorchFirewallGroup','Unrelated firewall rule refused','ShouldProcess','Assert-PorchAdministrator']))
    check('WAN phase cannot change adapters routers or unrelated services',not re.search(r'\b(Disable-NetAdapter|Set-NetIPInterface|Restart-Service|Stop-Service|netsh|Remove-NetRoute)\b',field,re.I))
    check('field process stop checks exact owned PID path and start time',all(s in field for s in ['$p.Path -ne $owned.path','$p.StartTime','Stop-Process -Id $p.Id']))
    check('explicit pairing confirmation and fingerprint pinning',all(s in field for s in ['CONFIRM','-Fingerprint','qualify','pair-join','pair-host']))
    check('hosted field setup refuses physical class',"$env:GITHUB_ACTIONS -eq 'true'" in field)
    ps=shutil.which('pwsh');syntax='UNVERIFIED'
    if ps:
        for file in kit.glob('*.ps1'):
            env=dict(os.environ,PORCH_PS_FILE=str(file))
            subprocess.run([ps,'-NoProfile','-Command',"$t=$null; $e=$null; [System.Management.Automation.Language.Parser]::ParseFile($env:PORCH_PS_FILE,[ref]$t,[ref]$e)|Out-Null; if($e.Count){$e|Out-String|Write-Error;exit 1}"],check=True,env=env)
        syntax='PASS';checks.append({'name':'native PowerShell parses all field scripts','result':'PASS'})
    out=EVIDENCE/'receipts/field-scripts.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'passed':len(checks),'checks':checks,'powershell_syntax':syntax,'privileged_firewall_changes':'NOT_RUN'},indent=2)+'\n')
if __name__=='__main__':main()
