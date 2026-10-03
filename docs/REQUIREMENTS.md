# Controlling requirement register

Source: unchanged CONTROLLING_SPEC.txt, SHA-256
`4e668341934d471be3b912c41eed24a124a0992528841a3a1f75b592e6bd6386`.
IDs correspond to numbered source sections, each preserving its contained
requirements. I = implemented with local evidence; P = partial, stated gap;
E = future extension/interface; D = deliverable prepared. None means production
qualification or independent verification. Authority is the supplied brief;
the source's section 1 permits explicit extension boundaries for work that cannot
reasonably be completed here. ROADMAP.md classifies REQUIRED versus FUTURE work.

| ID | Requirement / class | Authority | Implementation target | Verification hook / evidence | Status | Dependency / deviation |
|---|---|---|---|---|---|---|
| REQ-001 | Independent people-owned network; normative | §1 | core/node/CLI/UI | 33-check peer acceptance | P | Production/field scope remains open |
| REQ-002 | Sovereignty, local first, human authority; invariant | §2 | authority/SDK/scheduler | No-grant/local-only/client escalation negatives | I | Local operator OS is trusted base |
| REQ-003 | Read-only related contracts, independent adapters; normative | §3 | ADR-001, SDK | Provenance source paths; SDK tests | I | Full ecosystem runtime integration E |
| REQ-004 | Device identity and mature cross-platform peer daemon; normative | §4 | core, network, node | Actual TCP/Noise security tests and build | P | Linux qualified locally; QUIC/LAN/platform work REQUIRED |
| REQ-005 | Logical addressing, signed records, alias pinning; normative | §5 | address.resolve, signed services | Signature tests and live discovery | P | Model address is intent descriptor; no global DNS |
| REQ-006 | Create/join/leave/invite/revoke/private community; normative | §6 | node community/pairwise trust | Recipient-bound invite and expiry tests | P | One active Porch; group channels pairwise |
| REQ-007 | Deny-by-default bounded grants; invariant | §7 | grant core + transactional checks | Replay/principal/quota/revoke tests | I | VRAM/relay engines not installed |
| REQ-008 | Selected signed expiring claims and observations; normative | §8 | advertisement, network observations | Signed discovery/routing | P | GPU/RAM/vision/cost unknown, never fabricated |
| REQ-009 | Versioned safe distributed jobs and receipts; normative | §9 | run/execute_job, hash + providers | Job/duplicate/timeout tests | I | Builtin hash and inference only |
| REQ-010 | Provider-neutral routing, Ollama/mock/privacy; normative | §10 | models + scheduler + SDK | HTTP contract fixtures, remote mock acceptance | P | Live weights/hardware qualification REQUIRED |
| REQ-011 | Signed private inference provenance; normative | §11 | Receipt and ledger | Signature/digest checks, acceptance receipt | I | Resource facts provider-reported where indicated |
| REQ-012 | Deterministic hard filters then explainable order; normative | §12 | schedule + route | Privacy/authority-before-speed test | P | Hardware reservations/performance metrics absent |
| REQ-013 | Encrypted CAS/replicas/quotas/recovery/deletion; normative | §13 | storage | Full retrieval, corruption, resume/crash fixtures | I | Last verification is not durable backup proof |
| REQ-014 | Signed direct/channel messages, presence, queue; normative | §14 | node messaging + network | Actual direct peer delivery | P | Pairwise channels/outbox implemented; group/media test extension |
| REQ-015 | Voluntary bounded visible revocable relays; normative | §15 | finite unsupported operation + Transport seam | Relay-not-installed refusal | E | Authorized forwarding REQUIRED before full v1 scope claim |
| REQ-016 | Partition/stale clocks/reconnect/resume; normative | §16 | DB/time/transport/storage | Provider/origin restart, interrupted upload, island job | P | Loopback partition tested; physical outage REQUIRED |
| REQ-017 | Signed append-only local evidence; normative | §17 | db ledger + SSE | Tamper/startup chain tests | P | Expiry sweeper/full export/checkpoints REQUIRED |
| REQ-018 | Resource accounting, no speculative economics; normative | §18 | grant counters + receipts | Consumed calls once, transfer/storage facts | P | Hardware/lifetime metrics and fairness extension open |
| REQ-019 | Understandable nine-view desktop, headless daemon; normative | §19 | React/TypeScript | UI navigation/form/export screenshots | P | Browser desktop built; pending job views REQUIRED |
| REQ-020 | Serious operator CLI; normative | §20 | porch binary | Eight CLI smoke checks | I | GPU command absent pending real enforced VRAM |
| REQ-021 | Versioned local API/events/typed clients; normative | §21 | Axum + SDK | Auth/Host/Origin/client scope + SDK tests | I | Same-origin browser desktop |
| REQ-022 | Signed expiring app service discovery; normative | §22 | installed service manifests | Signed built-in/model discovery | P | Generic app registration REQUIRED |
| REQ-023 | Governed agents, inspectable authority; invariant | §23 | scoped clients + adapters | Agent cannot issue grants/escalate privacy | I | Token holder is not human identity proof |
| REQ-024 | Hostile-input/resource/replay/security protection; normative | §24 | finite parsers/quotas/limits | Rust and acceptance negatives | P | Audit/fuzz/load/OS secret controls REQUIRED |
| REQ-025 | Private discovery, no public content logs; normative | §25 | no public DHT, digest ledger | Code + private ciphertext acceptance | P | Local DB not encrypted; traffic metadata remains |
| REQ-026 | Protocol/schema compatibility and policy-safe updates; normative | §26 | envelope v1 / DB version | Version/future engine test | P | Explicit future migration/upgrade tests REQUIRED |
| REQ-027 | Clear monorepo dependency direction; normative | §27 | Cargo workspace + npm workspaces | Workspace/desktop/SDK builds | I | Three focused Rust crates instead of speculative split |
| REQ-028 | Versioned deterministic canonical schemas; normative | §28 | porch-core + serde | Actual wire signature round-trip test | I | JSON selected; not RFC8785 claim |
| REQ-029 | Durable SQLite/CAS and local secret separation; normative | §29 | DB/meta/vault/key files | Identity/storage/recovery tests | P | OS vault and private-detail at-rest encryption REQUIRED |
| REQ-030 | At least three-node deterministic CI network; normative | §30 | acceptance + interactive demo | 33-check result, CI config | I | Explicit mock provider; simulated LAN-like paths |
| REQ-031 | Negative failures and first-class refusal; normative | §31 | security/provider/recovery tests | Forgery, quotas, revocation, timeout, corrupt blob | P | Hostile-load/actual relay/physical cases remain |
| REQ-032 | Structured diagnostic observations, no telemetry; normative | §32 | tracing, doctor, UI, ledger | Doctor/real UI/route receipts | I | Lifetime metrics caveat in LEDGER.md |
| REQ-033 | Real build/package/run paths and signing gaps; normative | §33 | package.py, CI, launchers | Linux release and portable manifest | P | Windows/macOS execution/signing external REQUIRED |
| REQ-034 | All named guides and normal-person README; normative | §34 | docs + README | Document/path/link review | D | Included in source/package |
| REQ-035 | Concrete mitigations and remaining risks; normative | §35 | THREAT_MODEL, SECURITY | Explicit threat matrix | D | Independent review REQUIRED |
| REQ-036 | PhiOS governed infrastructure seam; future | §36 | SDK local read/run contracts | Scope enforcement tests | E | No PhiKernel runtime integration claim |
| REQ-037 | PhiVessel proposal then bounded execution; future | §37 | PhiVesselPorchAdapter | Pure proposal has no fetch/effect | E | Full Crane Fly/catalog integration future |
| REQ-038 | CommonLine independent adapter boundary; future | §38 | CommonLinePorchAdapter | SDK + message acceptance | E | Voice/WebRTC/media engine future |
| REQ-039 | Replaceable constrained physical transport seam; future | §39 | Transport trait/frame_limit | Real libp2p implementation obeys boundary | E | Hardware/radio/MTU drivers unimplemented |
| REQ-040 | Job-level SINGLE/REMOTE; explicit future engines; normative + future | §40 | Job modes + scheduler | Future engine/version negatives, real remote job | I | No sharded/ensemble claim |
| REQ-041 | End-to-end create/join/grant/job/revoke/offline proof; normative | §41 | actual OS-process harness | acceptance.json and signed job receipt | P | Deterministic mock; live Ollama/physical PCs REQUIRED |
| REQ-042 | All available gates and honest CI; normative | §42 | command receipt + CI | Format/lint/build/13 Rust/SDK/CLI/33 peer/UI | P | Remote CI/platform/live tests unavailable/unrun |
| REQ-043 | Vertical runnable delivery; process | §43 | complete local spine expanded | Commit/source/test receipts | I | Optional engines remain explicit seams |
| REQ-044 | No central SaaS/tokens/shell/fabricated claims; invariant | §44 | all boundaries + limitations | Code/tests + threat model | I | Production claims withheld pending REQUIRED evidence |
| REQ-045 | Inspect/build/test/docs/state/receipt delivery; process | §45 | bundle + DELIVERY + handoff | Exact committed snapshot and package manifest | D | Local repository; no external remote created |

The status column is a section-level rollup, not a claim that every example in
that section is implemented. This table, subsystem guides, test receipt and
REQUIRED/FUTURE roadmap preserve the distinctions among implemented, mocked,
simulated, unverified and deferred behavior.
