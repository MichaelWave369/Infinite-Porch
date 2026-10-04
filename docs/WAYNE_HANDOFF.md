# Wayne Rider Handoff Manifest

**Project:** Infinite Porch

**Candidate version:** 0.1.1

**Requested Wayne autonomy:** Level 2 non-destructive verification unless separately authorized

## Authority

- Controlling specification: CONTROLLING_SPEC_0.1.1.txt, unchanged; its explicit
  continuation brief outranks the preserved 0.1.0 brief.
- Accepted amendments: no material scope amendment; bounded local substitutions
  and environment limits recorded in deviations.json.
- Human release authority: Michael; no production action is authorized here.

## Artifacts

| Artifact | Role | Version | Hash | Location |
| --- | --- | --- | --- | --- |
| Current controlling brief | Untouched source | 0.1.1 | 37a97328fa947db3f249d26d99da6d47efe8519fb5f6780cb3dddf89a2d2e5c7 | CONTROLLING_SPEC_0.1.1.txt |
| Prior candidate | Preserved baseline | 0.1.0 | Uploaded ZIP 18409ce824a552862a812cba3ee3b6fba74fa7477935decf7f934f81c22e5936 | baseline-v0.1.0-candidate tag; git bundle |
| Candidate source/history | Implementation | 0.1.1 | Exact commit in final REPOSITORY_STATE.json | outer source/ and InfinitePorch.gitbundle |
| Local evidence | Builder checks | 0.1.1 | Selected file hashes in command-gates.json | receipts/qualification/ |
| Native package/UI | Unsigned GNU binaries | 0.1.1 | Final portable manifest/checksum | outer linux/ and release-evidence/ |
| Sanitized example | Signed public probes | schema 1 | Signed manifest file hashes | receipts/qualification/example-sanitized-bundle/ |
| Final delivery | All requested details | 0.1.1 | Outer checksum sidecar | DELIVERY_REPORT.md and delivery-manifest.json |

## Requirement coverage

- Register: REQUIREMENTS_0.1.1.md and .json; Q-001 through Q-035.
- External portions: physical pairing/LAN/Ollama/restart/offline, native
  Windows/macOS and independent security review remain UNVERIFIED.
- Deferred engines: section 29; REQUIRED/FUTURE distinctions in ROADMAP.md.

## Forge evidence

Build/command outputs and selected hashes: receipts/qualification/command-gates.json.
Supply chain: raw cargo/npm audits, supply-chain-review.json, SUPPLY_CHAIN_REVIEW.md.
Deviations: receipts/qualification/deviations.json. Post-commit release-build,
package/runtime/integrity results: outer release-evidence/. Initial failures are
retained rather than overwritten into passes.

## Declared limitations

Mocks are explicitly selected; dynamic Ollama HTTP fixtures use no weights.
Loopback encrypted TCP and QUIC are real transports, not physical topology.
Relay/federation/WASM/sharding/GPU pooling remain uninstalled. No Windows/macOS
binaries or official binary signing are claimed from this Linux environment.
A quick QUIC restart can lose its first attempt before a bounded same-ID retry;
that observation remains in the receipt. Provider cancellation is not hardware
isolation. Node attestations and signed receipts do not prove an honest provider
or external WAN conditions. Reviewed audit findings remain visible.

## Requested proving objectives

Reproduce software and package gates from a fresh clone of the bundle. Independently
review signed principals, codec byte caps, quotas/revocation, migration/clock and
uncertain recovery. Execute physical two-PC/live-model/revocation/restart/offline
procedures and native platform/ACL checks. Keep failures and validate every export.

## Authority statement

Candidate prepared for Wayne Rider Proving Ground. No independent verification
or release claim is made by the Forge. Final release authority remains human-owned.
This handoff does not invoke another agent or send anyone a message.
