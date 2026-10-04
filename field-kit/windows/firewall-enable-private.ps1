[CmdletBinding(SupportsShouldProcess)]
param([string]$PackageRoot = (Resolve-Path "$PSScriptRoot\..\..").Path,
      [ValidateRange(1024,65535)][int]$PeerPort = 7332, [switch]$DryRun)
. "$PSScriptRoot\Field-Common.ps1"
if ($PeerPort -in @(7331,11434)) { throw 'Privileged control ports may never be exposed' }
$manifest = Test-PorchPackage $PackageRoot
$program = (Resolve-Path (Join-Path $PackageRoot 'bin\porch-node.exe')).Path
$group = Get-PorchFirewallGroup
$plan = @{ application=$program; port=$PeerPort; protocols=@('TCP','UDP'); direction='Inbound'; profile='Private'; remote_address='LocalSubnet'; group=$group; rollback='firewall-remove.ps1'; unsigned=$true }
$plan | ConvertTo-Json | Write-Output
if ($DryRun -or $WhatIfPreference) { return }
Assert-PorchAdministrator
$created = @()
try {
    foreach ($protocol in @('TCP','UDP')) {
        $name = "$group $protocol $PeerPort"
        if (Get-NetFirewallRule -Name $name -ErrorAction SilentlyContinue) { throw 'Rule already exists; inspect status or remove it first' }
        if ($PSCmdlet.ShouldProcess($name, 'Allow only this application on Private LocalSubnet')) {
            New-NetFirewallRule -Name $name -DisplayName $name -Group $group -Program $program -Direction Inbound -Action Allow -Profile Private -Protocol $protocol -LocalPort $PeerPort -RemoteAddress LocalSubnet -Enabled True | Out-Null
            $created += $name
        }
    }
} catch {
    foreach ($name in $created) { Remove-NetFirewallRule -Name $name }
    throw
}
