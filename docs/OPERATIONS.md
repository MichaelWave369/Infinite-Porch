# Install, run and recover

From source, install Rust 1.99.0, Node 22+, Python 3.10+ and an OS C compiler.
Use committed lockfiles with `npm ci` and `cargo build --locked`. Run `npm run
build` before launching the daemon so static UI assets exist. The node remains
usable headlessly without the UI. Development Vite is a visual preview only;
use the daemon's same-origin UI for privileged interaction.

The Linux portable bundle contains `bin/porch`, `bin/porch-node`, `ui/`, docs,
sample configuration and a SHA-256 manifest. Extract it and run `./start.sh`.
Windows bundles created on Windows use `./start.ps1`; macOS builds use start.sh.
Verify manifest file hashes before trusting artifacts. The executable is not
release-signed; see ROADMAP.md. It runs in the foreground and installs no service.

The default directory is `state`. `porch init --alias NAME` creates an identity,
local token and initial config.json. On restart, persisted operator sharing choices
take precedence over initial config.json. Explicit daemon CLI overrides are then
applied and validated. Use the API/CLI sharing operations to change live policy.
Readiness logs show peer ID, API and token-file path, never the token itself.

Use an account-private directory. Keep identity.key, api.token, porch.sqlite and
the entire vault/transfers metadata private. On Unix the node sets directory/file
permissions. Validate platform permissions before use elsewhere. Back up a stopped
whole directory (or use a consistent SQLite snapshot plus files). The encrypted
identity export alone cannot recover encrypted blobs. A lost per-blob key is lost
data even if ciphertext replicas survive.

Stop with Ctrl+C. No auto-update or migration authority changes policy silently.
Before changing builds, stop, back up the directory, inspect release/migration
notes and test on a copy. Current unsupported future schemas refuse. Never delete
authority tables to make a failing startup appear healthy.

| Symptom | Action |
|---|---|
| API authentication/origin refusal | Use exact printed 127.0.0.1 API URL and local token; avoid proxies, public hostnames or Vite for controls |
| Peer unreachable | Confirm pinned peer ID, actual full multiaddress and intended TCP/UDP firewall path; no automatic NAT hole punching is promised |
| No eligible model | Configure/install exact model, refresh signed resources, approve direct trust, exchange bounded grant and choose privacy deliberately |
| Ollama unavailable/timeout | Start local Ollama, verify /api/tags and installed model; do not replace a real failure with mock |
| Job UNCERTAIN | Inspect executor/job ledger and private provider logs; reuse same ID to obtain original result if available; do not replay blindly |
| Corrupted blob | Refuse it, inspect original/other verified copy and restore explicitly; preserve the failing ciphertext for diagnosis privately |
| Ledger or clock refusal | Preserve state and logs; inspect actual tampering/time source; do not disable chain/expiry checks |
| Metadata/storage quota exhausted | Revoke unnecessary authority and inspect allocation; history is bounded, with retention/archival procedures still required |
| Browser has no dashboard | Build UI and set --ui to its dist directory; API/headless functions remain usable |

Peer revocation is local to each owner. A Porch owner revoking a device does not
magically overwrite another member's direct trust database; notify operators
through their chosen process and revoke on every affected owner. Never assume
deletion on a malicious remote disk or that one verified replica equals backup.

For release packaging, `python3 scripts/package.py` builds current-OS optimized
executables and UI. Windows release needs a publisher certificate and Authenticode
signature; a native installer would also need packaging/install tests. macOS release
needs Developer ID signing, hardened-runtime configuration, notarization and
stapling if distributed as an app/installer. These require external credentials
and platform runners; none is claimed completed here. Linux builds here target
x86_64 Ubuntu/glibc, not every Linux distribution or architecture.
