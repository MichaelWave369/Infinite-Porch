# Operator CLI

Use `porch --help` and each subcommand's `--help`. Global options: `--data state`,
`--api http://127.0.0.1:7331`, `--token-file PATH`. The CLI reads its local token
file; it never asks for a cloud account. Privileged API URLs must be literal
loopback HTTP addresses. JSON stdout is usable by scripts. Refused result status
exits 2; transport/validation errors exit nonzero.

`porch --version` identifies the executable as `porch 0.1.1`; the Rust crate's
internal name is porch-cli.

| Need | Command |
|---|---|
| Initialize / diagnose | `porch init --alias "Michael"`; `porch status`; `porch doctor` |
| Show identity | `porch identity show` (daemon running) |
| Community | `porch create "Oak Street"`; `porch invite --recipient PEER_ID --out invite.json`; `porch join invite.json`; `porch leave` |
| Direct trust / path | `porch peer approve PEER_ID ALIAS`; `porch peer connect MULTIADDRESS`; `porch peer revoke PEER_ID` |
| Resources | `porch resources`; `porch share compute`; `porch share compute --disable`; `porch share storage --limit 100MiB` |
| Models | `porch model discover`; `porch model list`; `porch share model qwen3:8b`; `porch share model qwen3:8b --private` |
| Explicit test provider | `porch share mock porch-mock` |
| Governed inference | `porch model run qwen3:8b "hello" --privacy TRUSTED_PEERS --peer PEER_ID --max-tokens 512` |
| Grant exchange | `porch grant issue --recipient PEER_ID --capability model.inference --resource qwen3:8b --out grant.json`; `porch grant import grant.json` |
| Revoke / inspect | `porch grant revoke GRANT_NONCE`; `porch grants`; `porch jobs`; `porch ledger` |
| Encrypted vault | `porch storage put file.bin`; `porch storage replicate CID PEER_ID`; `porch storage fetch CID PEER_ID --out recovered.bin` |
| Pin / delete | `porch storage pin CID`; `porch storage pin CID --unpin`; `porch storage delete CID`; `porch storage delete-remote CID PEER_ID` |
| Messaging | `porch message PEER_ID "hello" --queue`; `porch channel PORCH_ID "hello" --queue`; `porch retry-messages` |
| Address / path state | `porch resolve porch://peer/PEER_ID`; `porch network`; `porch peers`; `porch share refresh` |

Storage grants use `--capability blob.storage --resource vault --action store
--max-storage-bytes 4MiB`. Message grants use `message.direct`, `inbox`, `send`.
Grant flags include TTL, total/hour calls, input bytes, output tokens, optional
duration (`--max-duration-ms`), Porch scope and exact input digest. Full help documents accepted flags. A GPU
share command is deliberately absent because no VRAM reservation is enforced.

Scoped apps can be issued credentials through the finite control interface:

```sh
porch call client.issue --json '{"holder":"PhiBot","resources":["sha256"],"privacy":["LOCAL_ONLY"],"max_calls":50,"ttl_seconds":3600}'
porch call client.revoke --json '{"nonce":"CREDENTIAL_NONCE"}'
```

The first result contains a secret returned once; save it privately and give the
application that scoped secret, not `api.token`. JSON quoting differs in Windows
shells; use the desktop operator or PowerShell escaped quotes as appropriate.
`porch call` is finite daemon control, never a shell executor.

Identity backup/restore/rotation work offline. Output file writes fail if a file
already exists, so backup, invitation, grant and retrieval outputs do not silently
overwrite prior artifacts. See IDENTITY.md for secret handling and rotation.

0.1.1 physical workflow:

```sh
porch qualify host "Oak Street" --recipient PEER_ID --out invite.json
porch qualify join invite.json --fingerprint VERIFIED_HOST_SHA256
porch qualify status
porch models scan
porch models refresh
porch models verify EXACT_INSTALLED_MODEL
porch qualify run --peer PEER_ID --model EXACT_MODEL --environment PHYSICAL --separate-machines-confirmed
porch qualify export fresh-evidence-directory
porch qualify validate fresh-evidence-directory
```

Validation is offline and also accepts the UI's exported JSON container. Run
phases are baseline, revoked, restart, offline and restored. `--message` adds a
public probe with its own inbox grant. Physical/WAN attestations cannot prove
the topology by themselves. See the platform guides for both sides of pairing
and grant exchange. Use `porch --data STATE configure-limits limits.json` only
while the daemon is stopped; validated limits take effect at restart.
