# Two Windows PCs — Mikey's physical procedure

Status: native Windows execution, MSVC build, ACLs, firewall behavior and this
PowerShell flow are UNVERIFIED here. No Windows binaries are claimed or included
by a Linux build. The web desktop UI is served by the native node; there is no
signed Windows installer or Electron/Tauri executable in 0.1.1.

On BOTH PCs install Git, Rust 1.89.0 through rustup, Node 22.12+, and Visual Studio
Build Tools with Desktop development with C++ and a Windows SDK. Confirm with
`rustc --version`, `cargo --version`, `node --version`. Extract the candidate ZIP.
In ordinary PowerShell, from its outer directory:

```powershell
git clone .\InfinitePorch.gitbundle .\InfinitePorch-git
cd .\InfinitePorch-git
rustup toolchain install 1.89.0 --component rustfmt,clippy
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\platform\install-windows.ps1 -BuildFromSource
```

The helper creates user-owned state and removes inherited ACLs from it. Review
`icacls "$env:LOCALAPPDATA\InfinitePorch\state"`; it must not grant access to
other ordinary users. Administrators with host control are outside this protection.
All private keys, tokens, SQLite data and migration backups must stay in that
state directory. Do not place it in a shared Downloads or network folder.

In a control terminal on each PC set these variables (repeat them in a new terminal):

```powershell
$PorchDir = "$env:LOCALAPPDATA\InfinitePorch\0.1.1"
$State = "$env:LOCALAPPDATA\InfinitePorch\state"
$Porch = "$PorchDir\bin\porch.exe"
$Node = "$PorchDir\bin\porch-node.exe"
```

A / RTX 5070 PC:

```powershell
& $Porch --data $State init --alias MIKEY-5070
```

B / RTX 3060 PC:

```powershell
& $Porch --data $State init --alias MIKEY-3060
```

On each PC launch in a separate ordinary terminal and leave it running:

```powershell
& "$env:LOCALAPPDATA\InfinitePorch\0.1.1\bin\porch-node.exe" --data "$env:LOCALAPPDATA\InfinitePorch\state" --ui "$env:LOCALAPPDATA\InfinitePorch\0.1.1\ui"
```

Open http://127.0.0.1:7331 locally. Use its own state/api.token for its own UI.
Never copy tokens or private keys to the other PC. The Qualification page shows
fingerprints, provider observations, paths and evidence results.

Ensure Windows classifies your trusted LAN as Private. If firewall prompts do
not permit the node, an administrator can allow only the peer surface:

```powershell
$Program = "$env:LOCALAPPDATA\InfinitePorch\0.1.1\bin\porch-node.exe"
New-NetFirewallRule -DisplayName 'Infinite Porch TCP private LAN' -Direction Inbound -Program $Program -Protocol TCP -LocalPort 7332 -Profile Private -RemoteAddress LocalSubnet -Action Allow
New-NetFirewallRule -DisplayName 'Infinite Porch UDP private LAN' -Direction Inbound -Program $Program -Protocol UDP -LocalPort 7332,5353 -Profile Private -RemoteAddress LocalSubnet -Action Allow
```

Do not open control port 7331 or Ollama port 11434 on the LAN/WAN. Node execution
stays in an ordinary user terminal. For guest Wi-Fi/client isolation or virtual
adapters, use the real LAN address and the pinned manual connection fallback.

On B, get its public identity and transfer only its peer ID to A:

```powershell
& $Porch --data $State identity show
```

On A, paste B's exact peer ID and generate the recipient-bound invitation:

```powershell
$BPeer = 'PASTE_B_PEER_ID'
& $Porch --data $State qualify host 'Oak Street' --recipient $BPeer --out .\invite.json
& $Porch --data $State identity show
```

Compare A's entire fingerprint on its actual screen. Copy invite.json to B.
On B:

```powershell
$AFingerprint = 'PASTE_A_FINGERPRINT_SHA256'
& $Porch --data $State qualify join .\invite.json --fingerprint $AFingerprint
& $Porch --data $State qualify status
$APeer = 'PASTE_A_PEER_ID'
```

On A, start the local Ollama app if needed. Choose an exact installed model from
scan (qwen3:4b is only an example, not an assumed installation):

```powershell
& $Porch --data $State models scan
$Model = 'qwen3:4b'
& $Porch --data $State share model $Model
& $Porch --data $State models verify $Model
& $Porch --data $State grant issue --recipient $BPeer --capability model.inference --resource $Model --action run --ttl 1800 --max-calls 12 --calls-per-hour 12 --max-output-tokens 64 --out .\model-grant.json
```

Copy only model-grant.json to B. On B:

```powershell
$Model = 'qwen3:4b'
& $Porch --data $State grant import .\model-grant.json
& $Porch --data $State share refresh
& $Porch --data $State models list
& $Porch --data $State qualify run --peer $APeer --model $Model --environment PHYSICAL --separate-machines-confirmed
& $Porch --data $State qualify export .\evidence-baseline-B
& $Porch qualify validate .\evidence-baseline-B
```

Confirm provider=ollama, executor=A, COMPLETED, signed receipt, grant nonce and
input/output digests. Model names, outputs and timings come from actual execution.
If the 30-second operator limit is insufficient, use a smaller installed model;
0.1.1 intentionally does not relax the hard ceiling.

On A revoke, then on B prove refusal:

```powershell
# A:
$Grant = Get-Content .\model-grant.json -Raw | ConvertFrom-Json
& $Porch --data $State grant revoke $Grant.payload.nonce
# B:
& $Porch --data $State qualify run --peer $APeer --model $Model --phase revoked --environment PHYSICAL --separate-machines-confirmed
& $Porch --data $State qualify export .\evidence-revoked-B
```

For messaging, A issues a separate grant with `--capability message.direct
--resource inbox --action send`, B imports it, and B adds `--message` to a run.
To test both directions, B must separately grant A its own inbox.

Stop each daemon with Ctrl+C; rerun the same launch command. Run doctor and
qualify status on each and compare IDs, trust, Porch, revocation and ledger. Make
new exports; do not overwrite baseline evidence. Run OFFLINE_LAN_TEST.md after
A creates a new bounded model grant, because the old one is revoked.

A helper wraps physical run/export/validate from the source tree:

```powershell
.\scripts\platform\qualify-windows.ps1 -Peer $APeer -Model $Model -SeparateMachinesConfirmed -EvidenceDirectory .\evidence-helper-B
```

The helpers are statically reviewed and command contracts checked here; their
native Windows behavior is not claimed. No Authenticode signature or publisher
trust claim is made. Official references: https://learn.microsoft.com/powershell/
and https://learn.microsoft.com/powershell/module/netsecurity/new-netfirewallrule.
