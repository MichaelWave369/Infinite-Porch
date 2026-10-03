# Concrete threat model

Trusted base: local operator's OS account, the daemon code and installed provider.
Untrusted: network traffic, peer self-reports, storage providers, model output,
invitation delivery channels and dependency updates. Tests prove listed cases in
the stated environment; they do not prove absence of every attack.

| Threat | Risk | Current mitigation / evidence | Remaining risk | Next mitigation |
|---|---|---|---|---|
| Malicious peer | Unauthorized effects | Transport/signature principal binding, finite capabilities, real-peer forgery tests | Handshake floods, implementation bugs | Fuzzing and soak tests |
| Compromised Porch member | Lateral access | Membership gives no compute/storage grant; effect-time checks | Existing allowed inputs can be retained | Least privilege and incident drills |
| Stolen device | Keys, local prompts, messages | Unix private directory/files; protected export test | Running key and DB are plaintext at rest | OS key stores, ACL tests, disk encryption guidance |
| Malicious compute provider | False result/accounting | Signed receipt hashes bind assertion to provider | Signature is not computation attestation | Verifiable deterministic checks and attestation where feasible |
| Malicious requester | Oversized or expensive work | 64 KiB input, slots, call/byte/time limits; quota tests | Ollama hardware resources not isolated | Worker sandbox and GPU/RAM budgets |
| Spam/flooding | CPU/memory/network exhaustion | Connection/ingress/outbound/replay/outbox bounds | Bandwidth and local API flooding remain | Rate shaping and sustained stress tests |
| Sybil identities | Fake people consume access | No auto-trust, recipient-specific grants | Operator can approve deceptive people | Out-of-band identity verification and abuse controls |
| Metadata leakage | Expose membership/endpoints | No global directory; discovery restricted to approved peers | LAN mDNS/Identify and traffic timing reveal endpoints/IDs | Private discovery mode and traffic analysis review |
| Replay | Repeated execution/budget bypass | Durable nonces and job ID/hash reservation; replay test | Local state rollback can rewind evidence | Trusted/external checkpoints |
| Forged advertisements | Route to impersonator | Signed expiring claims, active trust and final provider authority | Authorized host can overstate capacity | Probe/attest claims and retain measured observations |
| Malicious storage host | Reads, discards or retains data | Client AEAD, digest readback, quotas and deletion request | Can discard ciphertext or ignore secure erasure | Multiple independent verified replicas and repair policy |
| Corrupted stored content | Silent bad retrieval | SHA-256 and AEAD; corrupted remote/local fixture tests | Loss if all copies and keys disappear | Tested backup and key recovery procedure |
| Malicious AI provider | Leakage and harmful output | Explicit remote privacy/grant; provider-specific receipt | Provider sees plaintext and can lie | Provider trust policy; application output validation |
| Prompt/data leakage | Accidental remote routing | LOCAL_ONLY default and outbound guard; no-forward test | Explicit remote consent allows provider retention | Per-data classification and stronger local secrets |
| Dependency compromise | Supply-chain execution | Committed Cargo/npm locks and independent install | No advisory audit or signed SBOM completed | Release audit, SBOM and signed provenance |
| Bootstrap outage | Service loss | No required bootstrap; actual local peer path test | No route survives if all physical paths fail | Multiple independently operated optional rendezvous paths |
| Clock rollback | Extend authority | Persisted lower bound; backward-clock negative test | Device clock rollback of whole DB or forward jump | Trusted clock policy and durable checkpoint |
| Ambiguous execution after crash | Duplicate external effect | UNCERTAIN refusal; pinned original route and cached result | Cannot prove provider stopped before crash | Recovery inspection and provider idempotency contracts |
| Local ledger deletion | Hide evidence | Surviving chain signatures/hash checks | Tail deletion/old backup can remain internally valid | Operator-owned external signed checkpoints |

Private channels use independent pairwise grants and encrypted sessions. This is
not a group forward-secrecy protocol or anonymous messaging network. Device
revocation is local to each resource owner; no global revocation oracle exists.
