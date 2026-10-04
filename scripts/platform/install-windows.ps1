[CmdletBinding()]
param([string]$Destination = "$env:LOCALAPPDATA\InfinitePorch\0.1.1", [switch]$BuildFromSource)
$ErrorActionPreference = 'Stop'
if (Test-Path $Destination) { throw 'Destination already exists; choose a new -Destination or review the existing installation.' }
$PorchSource = (Resolve-Path "$PSScriptRoot\..\..").Path
if ($BuildFromSource) {
    Push-Location $PorchSource
    try {
        & cargo build --release --workspace --locked
        if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
        & npm.cmd ci
        if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
        & npm.cmd run build
        if ($LASTEXITCODE -ne 0) { throw 'Desktop build failed' }
        New-Item -ItemType Directory -Force "$Destination\bin" | Out-Null
        Copy-Item 'target\release\porch.exe','target\release\porch-node.exe' "$Destination\bin"
        Copy-Item 'apps\desktop\dist' "$Destination\ui" -Recurse
    } finally { Pop-Location }
} else {
    if (!(Test-Path "$PorchSource\bin\porch-node.exe")) { throw 'This package contains no Windows binaries. From the source tree run this helper with -BuildFromSource after installing Rust 1.89.0, Node 22.12+ and MSVC build tools.' }
    New-Item -ItemType Directory -Force $Destination | Out-Null
    Copy-Item "$PorchSource\bin" "$Destination\bin" -Recurse
    Copy-Item "$PorchSource\ui" "$Destination\ui" -Recurse
}
$PorchState = "$env:LOCALAPPDATA\InfinitePorch\state"
New-Item -ItemType Directory -Force $PorchState | Out-Null
$PorchUser = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
& icacls.exe $PorchState /inheritance:r /grant:r "${PorchUser}:(OI)(CI)F" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'State ACL setup failed; inspect before creating keys' }
Write-Host "Installed to $Destination"
Write-Host "State: $PorchState (current-user ACL; physical Windows verification remains required)"
Write-Host "Initialize: & '$Destination\bin\porch.exe' --data '$PorchState' init --alias 'MIKEY-PC'"
Write-Host "Launch: & '$Destination\bin\porch-node.exe' --data '$PorchState' --ui '$Destination\ui'"
