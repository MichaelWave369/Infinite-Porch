# Two-PC Windows field lab

Use the **native Windows 0.1.2 portable ZIP**. Compare its SHA-256 with the separately obtained delivery report, extract it into a directory you own, and run PowerShell 7 from `field-kit/windows`. The binaries and manifest are unsigned. Hashes detect altered bytes; an unsigned manifest alone does not establish publisher authenticity.

If Windows blocks downloaded scripts after you verify the ZIP hash, review the scripts and remove their download marker with `Get-ChildItem -LiteralPath . -Filter *.ps1 | Unblock-File`. Do not change a managed or machine-wide execution policy.

Both PCs use a Private Windows network profile. Inspect `firewall-status.ps1`. Preview changes with `firewall-enable-private.ps1 -DryRun`; if needed run it in an elevated shell. It permits only the package's porch-node executable, TCP/UDP peer port 7332, Private profile and LocalSubnet. It never exposes control port 7331 or Ollama 11434. Remove only the kit's rules with `firewall-remove.ps1`. Do not change a Public network to Private without understanding the network.

Run this on each PC, using PC-A / PC-B respectively:

```powershell
.\Porch-FieldLab.ps1 setup -Alias PC-A -Session field-001
```

Setup preserves identity, checks credential ACLs, starts only its owned node, scans models and runs doctor. The default state is `%LOCALAPPDATA%\InfinitePorch\field-state`; `-State` supports a separate directory. Reuse the same `-Session`, state and any custom ports on all later commands. Repeated setup does not overwrite keys. A missing key next to an existing database refuses and requires recovery. Installation is per-user; no automatic login, OS service or router changes occur.

On both PCs display peer identity and full SHA-256 fingerprint, and compare through a channel you already trust. The pairing wizard also displays a 96-bit word phrase as a comparison aid; joining still requires the complete SHA-256 fingerprint. A host invitation displays its recipient and Porch; joining displays the host, Porch and intended local recipient. The CLI fingerprint pinning refuses a different host before trust. Invitations expire in ten minutes and are one-use.

```powershell
# PC-A: paste the actual PC-B peer ID, then type CONFIRM after comparison.
.\Porch-FieldLab.ps1 pair-host -Peer '<PC-B-ID>' -File .\invite.json -Session field-001
# Transfer invitation to PC-B. PC-B compares host fingerprint on PC-A.
.\Porch-FieldLab.ps1 pair-join -File .\invite.json -Fingerprint '<PC-A-SHA256>' -Session field-001
```

Peer addresses come from the signed invitation; no fixed household IP is assumed. If the host address changes, obtain another pinned invitation/address from the intended host; do not approve a nearby discovery automatically.

On PC-A, start your already installed local Ollama and install a model whose license and hardware requirements you accept. The kit does not install Ollama or buy services on physical PCs. Select an **exact installed tag**, verify real local inference, expose it, and issue a bounded one-hour grant (12 calls, 16 output tokens, 30 seconds each). It also exports an inbox grant. Resource visibility does not grant permission.

```powershell
.\Porch-FieldLab.ps1 model-host -Peer '<PC-B-ID>' -Model '<exact-installed-tag>' -Out .\model-grant.json -Session field-001
.\Porch-FieldLab.ps1 storage-host -Peer '<PC-B-ID>' -Out .\storage-grant.json -Session field-001
# PC-B imports model-grant.json, message-grant.json and storage-grant.json separately.
.\Porch-FieldLab.ps1 model-join -File .\model-grant.json -Session field-001
.\Porch-FieldLab.ps1 model-join -File .\message-grant.json -Session field-001
.\Porch-FieldLab.ps1 model-join -File .\storage-grant.json -Session field-001
.\Porch-FieldLab.ps1 qualify -Peer '<PC-A-ID>' -Model '<exact-installed-tag>' -SeparateMachinesConfirmed -Out .\baseline-B -Session field-001
```

For a single guided requester sequence, add `-Guided` to `qualify`. It runs the baseline, pauses for the owner to revoke, verifies the refusal, checks requester restart, and asks for the owner's independently exported post-restart snapshot before correlation. Owner actions remain on PC-A because requester authority cannot revoke owner grants. Failed phases are retained while independent checks continue safely. The manual commands below provide the same phases individually.

The public diagnostic checks identity, path, signed discovery, trust, message delivery, bounded authorization, actual remote inference, signed receipt, encrypted storage transfer, retrieval integrity and ledger. A mock invocation is PARTIAL. Failed steps remain FAIL even when independent steps succeed. Finish owner-mediated revocation and restart before the final report:

```powershell
# PC-A
.\Porch-FieldLab.ps1 revoke -File .\model-grant.json -Session field-001
# PC-B
.\Porch-FieldLab.ps1 refused-check -Peer '<PC-A-ID>' -Model '<exact-installed-tag>' -SeparateMachinesConfirmed -Out .\revoked-B -Session field-001
# BOTH PCs, after revocation; distinct fresh output directories
.\Porch-FieldLab.ps1 restart-check -SeparateMachinesConfirmed -Out .\restart-A -Session field-001
.\Porch-FieldLab.ps1 snapshot -SeparateMachinesConfirmed -Out .\PC-A-participant.json -Session field-001
# PC-B uses restart-B and PC-B-participant.json
```

The restart command stops only a PID whose executable path and start time match the kit's own launch record. It checks a new process nonce, unchanged identity/membership/trust/revocation, unchanged public jobs and grant accounting, a verified ledger prefix, and fresh provider scanning. It refuses to terminate a manually launched or unrelated process.

Collect final participant JSON from each PC, then on either PC:

```powershell
.\Porch-FieldLab.ps1 correlate -File .\PC-A-participant.json -EvidenceB .\PC-B-participant.json -Out .\physical-report -Session field-001
```

Correlation verifies signatures and binds session, Porch, peer identities, job IDs, canonical request digests, grants, input/output digests, timestamps, routes and matching receipt bytes from independently queried databases. A completed receipt without the corresponding origin/executor record is EVIDENCE_MISMATCH and FAILED. It also requires both LAN path observations for physical qualification. The report can remain NOT_RUN, PARTIAL or FAILED; successful tests never generate a production badge. Independent review remains UNVERIFIED.

Before revocation, run the optional WAN-off phase described in `WAN_OFF.md`. Read `FAILURE_CHECKS.md` for safe optional failures. Export final snapshots only after all selected phases. Preserve earlier evidence directories and failed logs. Do not publish state folders: they contain keys, tokens, private databases and potentially private logs.
