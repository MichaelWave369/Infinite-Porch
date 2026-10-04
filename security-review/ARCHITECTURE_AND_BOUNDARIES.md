# Architecture and trust boundaries

Operator CLI / React UI → literal-loopback HTTP + operator bearer → finite control operations → node policy / durable SQLite → authenticated libp2p peer RPC → executor grant validation → installed builtin or loopback Ollama provider.

| Boundary | Enforcement | Sensitive implementation |
|---|---|---|
| Browser to operator API | Exact Host/Origin, root/scoped credential, bounded bodies, no token persistence | `crates/porch-node/src/api.rs`, `clients.rs`; CLI `http` |
| Discovery to trust | Signed advertisement conveys visibility; identity and recipient-bound invitation establish explicit trust | `network.rs`; core `Signed`, `Invite`; `qualification.join` |
| Trust to authority | Recipient/capability/resource/action/expiry/quota exact grants; persistent revocation; post-run checks | node `validate_grant`, `execute_job`, `run`, `dispatch_remote`; core `Grant` |
| Input to route | LOCAL_ONLY blocks outbound jobs; peer eligibility/authority checked before input transmission | core `schedule`; node `run` |
| Peer bytes to parser | Protocol `/infinite-porch/rpc/1`; capped JSON/EOF, strict typed fields and signatures | `wire_codec.rs`, core `Signed`, `WireRequest`, `Job` |
| Model adapter | Literal loopback HTTP, no proxy/redirect, pinned installed digest, bounded response and deadline | `models.rs`; provider code in node `lib.rs` |
| Storage | XChaCha20-Poly1305 ciphertext, owner-bound AAD, SHA-256 CID, chunk quotas and verified retrieval | core `seal`/`unseal`; `storage.rs` |
| Persistence / recovery | Single-state lock, signed ledger, migration backup, no key recreation with existing DB, uncertain jobs not rerun | `db.rs`, `lib.rs`, migration/recovery tests |
| Evidence | Fixed public probe, finite export filenames/hashes/signatures, independent participant role/digest binding | `qualification.rs`, `field.rs` |
| Windows operator kit | Per-user ACL checks, explicit pairing, exact owned PID, Private application/subnet peer firewall | `field-kit/windows/*.ps1` |

Cryptography/library inventory: Ed25519 identity/signatures via libp2p-identity; peer encryption/authentication via libp2p Noise and QUIC/TLS; SHA-256 via sha2; XChaCha20-Poly1305 via chacha20poly1305; protected identity backups use the existing Argon2id-derived encryption path. Canonical JSON sorting and domain separation are critical. Exact dependency versions are in Cargo.lock, which controls the release, rather than this prose inventory. Check upstream advisories and transitive crypto implementations.

Community membership is not a resource grant. Node signatures prove identity provenance and integrity, not physical topology, ownership, independent review or official release signing. A privileged local operator can change observations or local files; do not treat that actor as a remote unprivileged peer.
