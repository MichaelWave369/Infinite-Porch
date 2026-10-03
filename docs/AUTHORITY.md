# Authority

Reachability, capability advertisements, community membership and model proposals
confer no execution permission. Sharing a capability and issuing a grant are
separate operator choices. The provider validates immediately before admission
and again before returning a remote provider result. Storage chunks each recheck
authority. Revocation is durable and closes revoked peer connections.

Grants bind issuer/recipient, capability, exact resource, action, optional Porch,
limits, created/expires times, nonce and optional exact input SHA-256. Signer must
equal issuer; recipient must equal authenticated requester. The original issued
record hash must still match the provider's database. A valid signature alone
cannot invent an unissued or revoked grant.

| Capability | Resource | Action | Bound effects |
|---|---|---|---|
| `model.inference` | Exact configured/exposed model | run | Input, output tokens, calls/hour, total calls, duration |
| `compute.hash` | sha256 | run | Deterministic input hash, same job limits |
| `blob.storage` | vault | store | Owner ciphertext upload, readback and tombstone; byte/call budgets |
| `message.direct` | inbox | send | Message bytes, calls and optional Porch channel scope |

Grants live at most one day. Default job grants allow 20 calls, 20/hour, 64 KiB
input, 512 requested output tokens and 30 seconds. Issuing zero calls or exceeding
hard ceilings fails. Byte allocation is cumulative across uploads; deletion or
expiry does not refund consumed grant budget. Storage readback has a four-times
granted-byte transfer budget. These are conservative accounting rules, not money.

SQLite IMMEDIATE transactions consume quota and reserve a job together. A duplicate
does not consume another call. A crash after external admission cannot provide
exactly-once proof of provider side effects; it leaves UNCERTAIN and requires human
inspection instead of replay. Revocation cannot retract an input already received
by a peer or guarantee immediate hardware cancellation inside Ollama.

The operator token is local administrative authority. Give apps a `client.issue`
credential instead: selected reads, finite non-admin operations, exact resources,
allowed privacy classes, expiry and request count. Apps cannot issue peer grants,
approve peers, alter sharing or delegate more authority. Credentials can be revoked
with `client.revoke`. The SDK adapters use the same API boundary. Model proposals
are pure data until a separately authorized execution request is made.

Client credential revocation blocks new requests. It does not roll back already
accepted local work or remove an already-authorized queued message. Remote work
still requires the provider's independent peer grant. There is no software proof
that the holder of the operator token is a particular human.
