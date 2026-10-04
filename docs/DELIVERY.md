# Infinite Porch 0.1.1 candidate assessment

The engineering milestone continues baseline commit
`3a3ace51979110afc76fa2dda7a35108c41af15b` on
`feat/porch-v0.1.1-physical-qualification`. The original source, history and
`baseline-v0.1.0-candidate` tag are preserved. The current controlling brief is
CONTROLLING_SPEC_0.1.1.txt; REQUIREMENTS_0.1.1.md maps its 35 sections to code,
checks and remaining external evidence. Protocol version remains **1**.

The Rust core/node/CLI, libp2p transport, SQLite authority, encrypted vault, SDK
and React UI remain the architecture. This candidate adds qualification commands
and a tenth UI page, fingerprint-confirmed pairing, current provider observations,
actual connection diagnostics, signed sanitized export and an offline validator.
It adds safe configurable caps, schema-1 backup/migration to schema 2, clock
checks, state locking, platform helpers and reproducible local fault scenarios.

Repairs include redundant parallel connection attempts, stale QUIC session
recovery after a quick restart, oversized JSON frames hidden by a truncated
reader, unbounded qualification messaging waits, wrong database-owner migration,
provider digest changes, quota/revocation effect races and private error leakage.
Fault tests retry the same job ID, manifest and executor; an uncertain job never
silently executes again. A lost first transport attempt is retained in the
recovery receipt. See FAILURE_INJECTION.md and SECURITY.md for practical limits.

Local verification comprises 29 Rust tests, 33 original peer checks, 32
qualification checks, 10 process-failure checks in each of two transport
configurations, 12 transport checks, eight CLI checks, five cap checks, six
platform contract checks, 11 UI checks and two SDK tests. Format, strict lint,
types and builds passed. These are 158 heterogeneous local tests/checks before
the separate native release-package gate. They are not physical evidence.
The final optimized package checks, exact source commit, binary requirements and
archive hashes are collected after the source commit and appear in the outer
DELIVERY_REPORT.md, REPOSITORY_STATE.json and release-evidence/ directory.

All selected source-stage results and commands are in
receipts/qualification/command-gates.json. Earlier failures remain in raw logs;
they are not selected as final passes. The example signed bundle and UI download
validate offline. A signed node statement establishes who reported its public
probe results; it cannot establish physical ownership, WAN disconnection or an
honest remote provider by itself.

The live Ollama path is implemented and HTTP contracts are exercised, but no
Ollama process or weights are available here. The opt-in live suite records two
SKIPPED_ENVIRONMENT and three UNVERIFIED scenarios. Windows/macOS build and
qualification flows are supplied; native execution, Windows ACLs, PowerShell
parsing and macOS signing remain UNVERIFIED. CI is configured for all three
platforms; no remote CI run occurred. Linux executables are unsigned GNU builds
for the recorded runtime, not a universal Linux compatibility claim.

Cargo audit findings are retained with a bounded applicability review, including
two Hickory advisories and a paste maintenance warning. The final audit used an
existing RustSec snapshot and makes no freshness claim. npm audit reported zero
findings for the checked locked graph. See SUPPLY_CHAIN_REVIEW.md.

Exact external procedures are in WINDOWS_QUALIFICATION.md,
LINUX_QUALIFICATION.md, OLLAMA_QUALIFICATION.md and OFFLINE_LAN_TEST.md; the shared
pairing, revocation, restart and export sequence is in PHYSICAL_QUALIFICATION.md.
RELEASE_GATES.md and ROADMAP.md keep required physical/platform/security evidence
separate from future engines. No production qualification, independent review,
official signing or release authorization is claimed. WAYNE_HANDOFF.md prepares
the candidate for independent reproduction.
