# Reproduce the 0.1.1 local checks

Use Rust **1.89.0**, Node 22.12+ and Python 3.12+. From source root first run
`npm ci`, `npm run build` and `cargo build --workspace --locked`. All deterministic
provider cases use explicit mocks or owned HTTP fixtures. Peer cases use actual
daemon processes and encrypted sockets on loopback.

| Gate | Command | Selected source-stage result |
| --- | --- | --- |
| Toolchain consistency | `python3 scripts/check_toolchain.py` | Rust 1.89.0 consistent |
| Rust format | `cargo fmt --all -- --check` | PASS |
| Strict lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| Rust regressions | `cargo test --workspace --locked` | 29 tests, including security, limits, provider, migration and recovery |
| Debug build | `cargo build --workspace --locked` | PASS |
| TypeScript | `npm run typecheck` | PASS |
| Desktop and SDK build | `npm run build` | PASS |
| SDK defaults/authority | `npm test` | 2 tests |
| CLI | `python3 tests/cli_smoke.py` | 8 checks |
| Original three-node acceptance | `python3 tests/acceptance.py` | 33 checks |
| Qualification/API/privacy/export | `python3 tests/qualification_acceptance.py` | 32 checks |
| Process interruption/reconciliation | `python3 tests/process_failures.py` | 10 checks, BOTH transport mode |
| QUIC interruption/reconciliation | `python3 tests/process_failures.py --transport quic` | 10 checks, QUIC-only mode |
| TCP, QUIC and BOTH paths | `python3 tests/transport_acceptance.py` | 12 checks |
| Offline cap configuration | `python3 tests/limit_cli.py` | 5 checks |
| Platform command contracts | `python3 scripts/check_platform.py` | 6 checks; native Windows/macOS UNVERIFIED |
| UI | `python3 tests/ui_acceptance.py` | 11 checks, four screenshots |
| Dependency review | `python3 scripts/audit_gate.py`; `npm audit --audit-level=high` | Rust findings reviewed, npm zero findings |
| Portable native build | `python3 scripts/package.py` | Run after a source commit; output must use a fresh directory |
| Extracted native package | `python3 tests/package_acceptance.py --archive ARCHIVE --out package-result.json` | Collected after source commit in release-evidence |

The source-stage command-gates.json under receipts/qualification selects final
logs and structured receipts and hashes those evidence files. Earlier attempts
and repaired failures remain visible. Original 0.1.0 results are historical;
they are not summed again into current totals. The two process modes exercise
the same ten assertions on distinct transport configurations.

UI testing requires compatible Chromium (`npx playwright install chromium`).
`PORCH_BROWSER_PATH` selects an existing headless browser. This environment's
standard download failed; the successful UI run used a separately installed
Chromium 153.0.8010.0. The UI test launches real nodes, logs in, submits a remote
mock job, navigates ten pages, validates qualification attestation/count/export,
renders a hostile alias as text, checks memory-only tokens and 390px layout,
logs out and requires no page exceptions. No API response is substituted.

The package test extracts a fresh archive, verifies every manifest hash, runs
the bundled CLI/node/UI, initializes private state, executes a builtin job,
exports/validates a qualification run and verifies the ledger. On Unix it also
checks private modes and the user install helper. It never uses production state.

The final local Rust audit ran with `--offline`, using an earlier fetched advisory
snapshot. CI uses the default fresh fetch; a fetch failure remains a failure.
Neither route suppresses new findings. See SUPPLY_CHAIN_REVIEW.md for retained
findings and exact applicability conditions.

The live suite is separate:
`python3 scripts/qualify_ollama.py --data state --model EXACT_MODEL --out live.json`.
Use `--cli` for a bundled executable. Missing Ollama/weights are skipped rather
than passed. Physical LAN, mDNS, outside-Internet removal, native Windows/macOS,
hardware load/durability and independent review require the external procedures.
Multi-platform CI is configured, not remotely executed in this delivery.
