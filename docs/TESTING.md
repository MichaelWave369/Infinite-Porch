# Reproducible verification

Run from repository root after `npm ci`, `npm run build` and
`cargo build --workspace --locked`. No Ollama weights or outside Internet service
is needed for deterministic tests. Peer tests use encrypted real sockets, not a
replacement transport pretending to be the network.

| Gate | Command | Checked candidate evidence |
|---|---|---|
| Rust formatting | `cargo fmt --all -- --check` | receipts/format.log |
| Strict Rust lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` | receipts/clippy.log |
| Core/security/provider/recovery tests | `cargo test --workspace --locked` | receipts/rust-tests.log; 13 tests |
| TypeScript strict checks | `npm run typecheck` | receipts/typescript.log |
| Desktop and SDK build | `npm run build` | receipts/desktop-build.log |
| SDK authority/defaults | `npm test` | receipts/sdk-tests.log; two tests |
| CLI/identity lifecycle | `python3 tests/cli_smoke.py` | receipts/cli-smoke.json; eight checks |
| Three-process acceptance | `python3 tests/acceptance.py` | receipts/acceptance.json; 33 checks |
| UI acceptance | `python3 tests/ui_acceptance.py` | evidence/ui-receipt.json, real screenshots |
| Optimized node/CLI | `cargo build --release --workspace --locked` | receipts/release-build.log |
| Portable package | `python3 scripts/package.py --skip-build` | package manifest and checksums |

The three-process harness creates ALPHA (explicit mock model/storage), BETA
(hash/storage) and GAMMA (client). It pairs recipient-bound invitations, discovers
signed services, refuses unauthorized routing, executes a permitted remote model
job, validates a signed receipt/accounting, proves duplicate idempotency and
LOCAL_ONLY non-forwarding, revokes authority, encrypts/replicates/fetches/corrupts a
blob, sends a message, resumes an upload after provider restart, recovers a cached
origin receipt, and continues on a reachable island after ALPHA stops. It cleans
private test state unless `--keep-state` is requested; do not commit that directory.

Rust negative tests include forged signatures, signer/transport/requester mismatch,
unknown/revoked peers, replay, exhausted grant, expired invite/grant, ledger edit,
backward clock, provider failure/oversize/timeout, duplicate active work, active
revocation and uncertain restart refusal. Storage recovery injects the durable
rename-before-metadata crash state explicitly. Those fixtures do not establish
real power-loss durability or hardware resource isolation.

UI testing requires a Chromium installation (`npx playwright install chromium`).
`PORCH_BROWSER_PATH` can select an already installed compatible headless Chromium.
The test launches actual daemons, logs in, submits a remote mock job through the
form, navigates nine surfaces, exports ledger, checks memory-only credentials,
mobile layout, logout and browser exceptions. No API response is stubbed.

CI `.github/workflows/ci.yml` runs format/lint/Rust/TS/SDK/CLI/acceptance/build/package
gates on Linux, Windows and macOS, with UI checks on Linux. Configuration is
committed; remote CI execution has not happened because no remote repository was
provided. Dependency installation requires network; the built local runtime and
acceptance peer paths do not.

Initial failures are retained in receipts/acceptance-initial-failure.json and
failure-and-repair.json. A CBOR null/array serialization bug changed signing material;
the bounded JSON codec repaired it and a real-wire regression test preserves it.
Tests were not relaxed to accept an invalid signature. Later repaired cases and
the final command receipt document what was actually run.

No live Ollama, physical LAN mDNS, WAN NAT, native package signing, sustained load,
radio or independent audit result is claimed. Such qualifications remain REQUIRED
or FUTURE as classified in ROADMAP.md.
