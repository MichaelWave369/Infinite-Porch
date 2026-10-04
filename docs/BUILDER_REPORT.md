# Builder-Facing Report

**Project:** Infinite Porch

**Controlling specification:** CONTROLLING_SPEC_0.1.1.txt (unchanged)

**Milestone / candidate:** 0.1.1 physical qualification candidate

## What was built

First-class qualification CLI/UI, verified-fingerprint pairing, provider state
tracking, actual connection diagnostics, signed sanitized bundles, offline
validation, safe operator caps, private schema migration, recovery/fault tests
and Windows/Linux/macOS procedures. The existing architecture and protocol 1
are retained.

## What is real versus substituted

- Implemented: live loopback Ollama adapter and finite qualification operations.
- Demonstrated locally: 158 software assertions across Rust, CLI, SDK, UI, real
  loopback peers and fault scenarios; native package checks collected separately.
- Mocked/simulated: inference mocks and owned HTTP fixtures; no model weights.
- Unable to verify: physical LAN/WAN removal, real Ollama, Windows/macOS, systemd,
  idle host metrics and independent security review.

## Material deviations

Rust 1.89.0 is the lowest supported compiler for the locked UUID dependency.
Existing transitive futures/async-trait are direct dependencies; the build helper
rustversion resolves to 1.0.22. Final Rust audit uses an existing snapshot.
Reviewed dependency findings remain visible. See deviations.json.

## Human decision required

None for this authorized candidate build. Production release remains human-owned.

## One atomic next action

Run the two-PC Windows guide on the RTX 5070 and RTX 3060 machines with one
already installed Ollama model, preserving baseline and revoked exports.

## Evidence bundle

Source-stage evidence: receipts/qualification/. Exact commit, final package
verification and hashes: outer DELIVERY_REPORT.md and release-evidence/.

## Wayne handoff status

Candidate prepared for independent proving-ground verification. No independent
verification or production release claim is made by the Forge.
