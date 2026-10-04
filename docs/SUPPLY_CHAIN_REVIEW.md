# 0.1.1 supply-chain review

See raw cargo-audit.json, npm-audit.json, hickory-features.txt and
supply-chain-review.json under docs/receipts/qualification. npm audit reported zero
findings in this run. Cargo audit reported two Hickory advisories and one
maintenance warning; they remain visible in the delivered raw output.

| Finding | Classification and reason |
| --- | --- |
| RUSTSEC-2026-0118, hickory-proto 0.25.2 | Unreachable in this configuration: DNSSEC features and DnssecDnsHandle are absent from the enabled feature graph. |
| RUSTSEC-2026-0119, hickory-proto 0.25.2 | Unreachable in this integration: libp2p-mdns 0.48.0 parses packets using Message::from_vec, but its output encoder manually appends bytes and never uses the affected Hickory BinEncoder. |
| RUSTSEC-2024-0436, paste 1.0.15 | Build-time maintenance risk in transitive macros; retained for this bounded candidate, requiring future upstream migration. |

This is an applicability assessment by the builder, not independent security
review or a claim that Hickory itself is patched. scripts/audit_gate.py checks
exact dependency versions, disabled DNSSEC and the inspected mDNS source path;
any new advisory, warning, feature expansion or dependency change fails the gate
for fresh review. No global audit ignores or blind major upgrades are used.

The locked graph keeps libp2p 0.56.0. CLI futures and node async-trait become
direct dependencies while reusing existing transitive libraries for bounded
response streaming and the JSON transport codec. Lock regeneration in the
available offline index resolved the build helper rustversion from 1.0.23 to
1.0.22; its checksum is locked. No runtime major-version upgrade or new service
or model dependency was introduced. Rust 1.88 was rejected by uuid 1.27.0's declared 1.89 requirement;
1.89.0 is consistently pinned and locally compiled/tested. cargo-audit is a
separate developer tool, not a runtime dependency.

An earlier cargo audit fetched RustSec's advisory database. The final audit gate
ran with `--offline` against that existing snapshot: it makes no freshness claim.
Default CI performs a fresh bounded fetch and fails if it cannot fetch.
The supplied raw output records IDs, dates, descriptions and upstream URLs.
The mDNS integration source hashes are in the review receipt. This assessment
must be revisited before adding DNS resolution, DNSSEC, different discovery or
unbounded advertisement/connection behavior.
