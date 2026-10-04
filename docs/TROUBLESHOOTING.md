# Qualification troubleshooting

Run `porch doctor` and `porch qualify status` first. Doctor reports separate
IDENTITY, DATABASE, LEDGER, CONTROL API, NETWORK LISTEN, LAN DISCOVERY, PEER
CONNECTIVITY, NAT STATUS, OLLAMA, MODEL INVENTORY, STORAGE, AUTHORITY, CLOCK and
PACKAGING results with remediation. A successful category does not imply an
overall production pass.

| Observation | Response |
| --- | --- |
| NODE_STATE_ALREADY_IN_USE | Stop the other node using this state. Do not delete its lock file while it runs. |
| EXISTING_STATE_IDENTITY_MISSING_RESTORE_BACKUP | Stop; restore the original protected identity. Never initialize a replacement identity over an existing database. |
| DATABASE_SCHEMA_UNSUPPORTED | Preserve state and use the matching version. Future schemas are not downgraded. |
| MIGRATION_BACKUP_ALREADY_EXISTS_REVIEW_REQUIRED | Preserve both databases; inspect the previous migration attempt. Do not automatically overwrite the backup. |
| CLOCK_UNCERTAIN_BACKWARD_JUMP | Correct the clock. Keep the persisted lower bound and recorded anomaly; do not extend grants. |
| PAIRING_FINGERPRINT_MISMATCH | Compare the intended host's full fingerprint directly. Do not retry with a guessed fingerprint. |
| No eligible executor | Inspect route.candidates and excluded reasons: privacy, trust, grant bounds, model, signed expiry and capacity. |
| PEER_JOB_DEADLINE | Preserve original job ID. Retry to reconcile on the original executor; no automatic second execution. |
| DUPLICATE_JOB_UNCERTAIN | Executor stopped during a running job. Preserve it for reconciliation; it is not automatically rerun. |
| Model advertisement but no grant | Issue/import a bounded exact-resource grant from that executor. Membership alone gives no authority. |
| MODEL_VERSION_CHANGED_RESHARE_REQUIRED | Rescan and explicitly re-share the new digest; issue a new grant. |
| Ollama unreachable | Start the local provider, check literal loopback port, then scan. No mock fallback is installed. |
| Peer connection fails after reboot | Confirm current LAN address. Refresh mDNS or manually connect using the intended peer's pinned address. Persisted addresses may be stale after DHCP change. |
| GUI cannot authenticate | Use its own local api.token and literal http://127.0.0.1:7331; token is kept in browser memory only. |
| Evidence export refuses | Run qualification first, choose a new directory, and retain failures. |
| Private guest Wi-Fi / client isolation | Join the same reachable trusted LAN or remove isolation through your own network policy. |

IPv4/IPv6 private/link-local direct paths are classified LAN_DIRECT; this is an
IP observation, not automatic proof of physical topology. Advanced outbound
local ephemeral sockets are UNVERIFIED because the libp2p endpoint API exposes
only the remote dial address. Inbound local sockets, encryption, authenticated
peer, negotiated RPC, RTT and connection age are observed where available.
