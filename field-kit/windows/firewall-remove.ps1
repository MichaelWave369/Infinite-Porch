[CmdletBinding(SupportsShouldProcess)]
param([switch]$DryRun)
. "$PSScriptRoot\Field-Common.ps1"
$group = Get-PorchFirewallGroup
$rules = @(Get-NetFirewallRule -Group $group -ErrorAction SilentlyContinue)
$rules | Select-Object Name, Group, Profile | Format-Table | Out-String | Write-Output
if ($DryRun -or $WhatIfPreference) { return }
Assert-PorchAdministrator
foreach ($rule in $rules) {
    if ($rule.Group -ne $group -or !$rule.Name.StartsWith("$group ")) { throw 'Unrelated firewall rule refused' }
    if ($PSCmdlet.ShouldProcess($rule.Name, 'Remove Infinite Porch field rule')) { Remove-NetFirewallRule -Name $rule.Name }
}
