Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
function Protect-PorchState([string]$Path) {
    $item = Get-Item -LiteralPath $Path
    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'State cannot be a reparse point' }
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $acl = Get-Acl -LiteralPath $Path
    if ($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -ne $sid.Value) {
        if (Test-Path (Join-Path $Path 'identity.key')) { throw 'Existing state must be owned by the current user' }
        $acl.SetOwner($sid)
    }
    $acl.SetAccessRuleProtection($true, $false)
    foreach ($rule in @($acl.Access)) {
        if ($rule.AccessControlType -eq 'Allow' -and $rule.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid.Value) { throw 'Existing explicit state access requires operator review' }
    }
    $rule = [Security.AccessControl.FileSystemAccessRule]::new($sid, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')
    $acl.SetAccessRule($rule)
    Set-Acl -LiteralPath $Path -AclObject $acl
    foreach ($file in @('identity.key', 'api.token', 'porch.sqlite', 'porch.sqlite-wal', 'porch.sqlite-shm')) {
        $name = Join-Path $Path $file
        if (Test-Path -LiteralPath $name) {
            $a = Get-Acl -LiteralPath $name
            foreach ($r in @($a.Access)) {
                if ($r.AccessControlType -eq 'Allow' -and $r.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ne $sid.Value) { throw "Unexpected ACL on $file" }
            }
        }
    }
    return @{ result = 'PASS'; strategy = 'current-user directory DACL; inherited credential ACLs inspected'; user_sid = $sid.Value }
}
function Test-PorchPackage([string]$Root) {
    $manifest = Get-Content -LiteralPath (Join-Path $Root 'manifest.json') -Raw | ConvertFrom-Json
    if ($manifest.candidate -ne '0.1.2' -or $manifest.protocol_version -ne 1 -or $manifest.official_release_signature) { throw 'Unexpected package metadata' }
    $prefix = [IO.Path]::GetFullPath($Root).TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
    foreach ($p in $manifest.files.PSObject.Properties) {
        if ([IO.Path]::IsPathRooted($p.Name) -or $p.Name.Contains('..') -or $p.Name.Contains('\')) { throw 'Unsafe manifest path' }
        $path = [IO.Path]::GetFullPath((Join-Path $Root $p.Name))
        if (!$path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'Manifest path leaves package' }
        if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing package file: $($p.Name)" }
        if ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Package reparse point refused' }
        if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $p.Value) { throw "Hash mismatch: $($p.Name)" }
    }
    # These hashes detect changed bytes. The unsigned manifest requires a separately obtained ZIP hash.
    return $manifest
}
function Assert-PorchAdministrator {
    $p = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    if (!$p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Run this firewall action in an elevated PowerShell session' }
}
function Get-PorchFirewallGroup { return "Infinite Porch Field Lab $([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)" }

function Get-PorchFingerprintPhrase([string]$Fingerprint) {
    $hex=($Fingerprint -replace '[ :\-]','').ToLowerInvariant()
    if ($hex -notmatch '^[0-9a-f]{64}$') { throw 'Full SHA-256 fingerprint required' }
    $words=@('amber','birch','cedar','dawn','elm','fern','grove','hearth','island','jade','kite','lake','moss','north','oak','pine')
    $phrase=@();for($i=0;$i -lt 24;$i++){ $phrase+=$words[[Convert]::ToInt32($hex.Substring($i,1),16)] }
    return ($phrase -join ' ')
}
