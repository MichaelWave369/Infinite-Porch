# Qualification milestone boundary

Controlling source: CONTROLLING_SPEC_0.1.1.txt and its freeze receipt. Continue
the baseline source/history; preserve the original specification and protocol 1.
Authorized scope is qualification, live-provider verification, bounded failure
testing, physical procedures, diagnostics, migration, local control security,
safe caps, dependency review, evidence integrity and candidate packaging.

Interfaces affected: finite root-operator CLI/API and qualification UI; bounded
Transport codec/dial/diagnostic methods; schema-2 metadata and recovery; provider
inventory and digest verification; safe local configuration; native package and
platform helpers. Peer trust, signed grant authority and executor receipt rules
remain the governing boundaries. No cloud control or public engine is added.

Implementation sequence: inspect/freeze baseline; align the minimum compiler;
add qualification and provider state; harden boundary/failure behavior; execute
regressions; complete physical guides and traceability; commit source; build
optimized exact-commit binaries; verify extracted native package; collect final
report/history/hash manifests into the named candidate ZIP.

Local checks and selected receipts are in TESTING.md and command-gates.json.
Rollback: retain the baseline tag and git bundle. A migrated database has a
private schema-1 backup; do not downgrade a live schema-2 state directory or
silently replace identity. Restore a reviewed backup into separate offline state.

Known risks: hostile providers and host administrators, metadata exposure,
external clocks, hardware cancellation uncertainty, real power-loss durability,
quick QUIC reconnect deadlines, native OS permission behavior and retained
dependency maintenance findings. Independent review and physical validation
remain required; see SECURITY.md, RELEASE_GATES.md and ROADMAP.md.

Stopping condition: all available local software/build/package gates completed,
full source/history and Linux/UI artifacts delivered with hashes, all requested
physical procedures executable from the guides, and remaining environment gates
explicitly UNVERIFIED. This prepares a candidate; production release and
independent proving-ground verification remain human-owned.
