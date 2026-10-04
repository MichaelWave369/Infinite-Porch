# Two Windows PCs — physical qualification for 0.1.2

Use the Windows native portable ZIP identified in the delivery report. The
report and native receipts record qualification on the exact candidate commit.
Hosted Windows tests cover the software, installer, state ACLs, field setup,
identity preservation and firewall dry plan. Physical LAN, household firewall
enforcement, WAN-off LAN, physical remote Ollama and independent review remain
UNVERIFIED until their own evidence is collected. This candidate is **NOT
PRODUCTION QUALIFIED**.

The full operator procedure is [field-kit/windows/START_HERE.md](../field-kit/windows/START_HERE.md).
Use the copy inside the extracted Windows native package; it validates the
package manifest and starts its bundled binaries. The outer delivery's field
kit is a source reference. The portable package requires no Rust, Node, Git or
Visual Studio installation on the field PCs. Its desktop is served by the
native node; there is no signed installer or Electron/Tauri executable.

On both intended PCs:

1. Obtain the Windows ZIP and its checksum from the delivery report, compare
   SHA-256, and extract into a directory the operator owns. The binaries and
   manifest are unsigned; hashes alone do not authenticate a publisher.
2. Open **PowerShell 7.5+** in the package's `field-kit/windows` directory and
   review the scripts. If downloaded scripts are blocked, remove the download
   marker only from the reviewed scripts as described in `START_HERE.md`.
   Do not change a managed or machine-wide execution policy.
3. Inspect the Windows network profile and `firewall-status.ps1`. Preview the
   kit's application-bound Private/LocalSubnet peer rules with
   `firewall-enable-private.ps1 -DryRun`. Actual rule changes require a human
   decision and an elevated shell. Never expose control port 7331 or Ollama
   port 11434. The kit's rollback removes only its own rule group.
4. Run setup with different aliases and the same public session label:

```powershell
# First PC (for example, the RTX 5070 PC)
.\Porch-FieldLab.ps1 setup -Alias MIKEY-5070 -Session field-001
# Second PC (for example, the RTX 3060 PC)
.\Porch-FieldLab.ps1 setup -Alias MIKEY-3060 -Session field-001
```

These are alternative aliases for the PC-A / PC-B examples in `START_HERE.md`.
Reuse the same state, ports and session on every later command. Setup preserves
the existing identity and checks the private state DACL. Keep state folders,
keys, operator tokens and private databases on their original PCs.

Compare the intended peer identities and full host SHA-256 through a channel
already trusted, then use `pair-host` and `pair-join`. Discovery establishes
reachability; membership and resource grants require their own explicit
decisions. The word phrase is a comparison aid, not a replacement for the full
fingerprint.

On the model owner, use an exact already installed Ollama tag. `model-host`
verifies real local invocation before exposure and issues a bounded model
grant plus a separate inbox grant. `storage-host` issues a bounded encrypted
storage grant. Transfer and import only the public invitation and grants.
On the requester, run `qualify` with `-SeparateMachinesConfirmed`; `-Guided`
coordinates baseline, owner-mediated revocation, requester restart and
independently exported owner evidence. The physical kit does not install
Ollama or change router, adapter or OS-service settings.

Collect any optional WAN-off evidence **before revoking** the model grant;
see [WAN_OFF.md](../field-kit/windows/WAN_OFF.md). A human removes only the
upstream Internet connection while retaining the LAN. Router observations,
bilateral path evidence and actual messaging/inference are needed; failed
external probes alone are inconclusive.

Complete revocation/refused execution and restart on both PCs, then independently
export each participant snapshot. Correlate the two public exports as described
in `START_HERE.md`. Preserve failed phases and earlier evidence. Only a matching
kit-owned PID, executable path and start time may be stopped. Missing or
contradictory origin/executor records fail correlation. Successful hosted tests
cannot promote a physical gate, and successful physical records cannot confer
production qualification or independent security review.

Read [FAILURE_CHECKS.md](../field-kit/windows/FAILURE_CHECKS.md) before optional
failure experiments. GPU use, VRAM and sustained household hardware load require
their own observations; selecting a GPU-equipped PC does not prove GPU execution.
The original 0.1.1 manual remains preserved in the baseline source history.
