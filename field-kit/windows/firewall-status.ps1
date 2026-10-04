[CmdletBinding()]
param()
. "$PSScriptRoot\Field-Common.ps1"
Get-NetConnectionProfile | Select-Object Name, NetworkCategory, IPv4Connectivity, IPv6Connectivity
$group = Get-PorchFirewallGroup
foreach ($rule in @(Get-NetFirewallRule -Group $group -ErrorAction SilentlyContinue)) {
    $rule | Select-Object Name, Enabled, Direction, Action, Profile
    $rule | Get-NetFirewallPortFilter | Select-Object Protocol, LocalPort
    $rule | Get-NetFirewallApplicationFilter | Select-Object Program
    $rule | Get-NetFirewallAddressFilter | Select-Object RemoteAddress
}
Write-Output 'Control API 7331 and Ollama 11434 must remain loopback-only. No firewall settings changed.'
