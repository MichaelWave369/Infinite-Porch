# Protocol v1

Peer protocol `/infinite-porch/rpc/1`; Identify version `/infinite-porch/1`;
localhost API `/v1`. An incompatible libp2p protocol is not negotiated. Envelope
version/domain mismatches fail signature verification. Typed effect records use
serde with unknown-field rejection. Operator control is `{operation,args}`.

Every signed record contains `version`, `domain`, `signer`, `public_key`,
`payload`, and a hexadecimal Ed25519 `signature`. Peer ID derives from the
included libp2p protobuf public key and must equal signer. Signature material
is the canonical object containing version/domain/signer/key/payload, excluding
the signature. The full signed record's canonical SHA-256 is its record hash.
`porch-core/src/lib.rs` is the executable schema source of truth.

Canonical JSON recursively sorts object keys by Rust string order, preserves
array order, uses serde JSON string escaping and accepts only signed/unsigned
64-bit integer numbers. Depth is at most 32. Floats fail closed. This is the
Porch v1 contract, not a claim of RFC 8785 conformance. Wire JSON uses the bounded
rust-libp2p codec. Its signing-material round-trip has a regression test.

| Record / domain | Important bindings |
|---|---|
| `porch.request.v1` | nonce, request ID, time window, finite operation and args |
| `porch.response.v1` | original nonce/hash, signer, status, code and output |
| `porch.grant.v1` | issuer, recipient, capability/resource/action, scope, limits and expiry |
| `porch.advertisement.v1` | provider, selected claims, observed facts and expiry |
| `porch.service.v1` | service ID, provider, protocol, endpoint, grant policy and expiry |
| `porch.job.receipt.v1` | job/request/input/output hashes, executor, provider, status and usage |
| `porch.blob.receipt.v1` | ciphertext ID/size, owner, executor and grant nonce |
| `porch.message.receipt.v1` | message ID/hash, sender and recipient |
| `porch.ledger.v1` | sequence, previous hash, actor, subject, event and time |
| `porch.api.client.v1` | local delegated token hash, scopes and expiry |

Request lifetime is at most 60 seconds. Created time may lead the receiver by
at most two seconds. Replay nonces are durable; the table is bounded at 4096.
Authenticated transport peer must equal request signer, and a job's requester
must equal that peer. Responses bind to the actual request, not merely its ID.

Frame limit is 256 KiB. Input is 64 KiB UTF-8 bytes; output response JSON is
128 KiB; blob ciphertext is at most 8 MiB in 64 KiB chunks. The local encrypted
upload endpoint accommodates its hex expansion. Finite peer operations are
resources, porch.join, job.run, blob.begin/chunk/get/delete and message.send.
Unknown operations refuse. No generic executable endpoint exists.

Logical addresses support peer, service, blob, porch and model namespaces.
Service lookup returns only fresh signed provider records from approved peers.
Aliases are operator-pinned, unique active local labels. Model logical addresses
are intent descriptors; model availability and authority are resolved at job
submission. They do not implement a global DNS directory.

Schema version 1 initializes the current tables, including client delegation
and outbound retry reservations. No older public schema has been released.
Future schemas require explicit migrations and compatibility tests; current
nodes refuse a higher SQLite user_version. Updates never auto-change trust.
