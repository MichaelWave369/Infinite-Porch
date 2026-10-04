# Audit and reproduction commands

Use Rust 1.89.0 with rustfmt/clippy, Node 22.12+ and Python 3.12. Native Windows additionally needs MSVC build tools and PowerShell 7. Work from a fresh clone/bundle and disposable state.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
npm ci
npm run typecheck
npm run build
npm test
python tests/acceptance.py
python tests/qualification_acceptance.py
python tests/field_acceptance.py
python tests/native_runtime.py
python tests/process_failures.py
python tests/process_failures.py --transport quic
python tests/transport_acceptance.py
python tests/limit_cli.py
python tests/field_scripts.py
python scripts/check_platform.py
npm audit --audit-level=high
cargo install cargo-audit --version 0.22.2 --locked
python scripts/audit_gate.py
```

`PORCH_EVIDENCE_ROOT` sends test receipts to a fresh directory; preserve the baseline receipts. For UI, install the pinned Playwright Chromium then `python tests/ui_acceptance.py`. `python scripts/native_qualification.py --out <fresh-dir>` executes and logs the full native gate sequence. Archive validation is `python scripts/verify_package.py <zip>`; fresh native package execution is `python tests/package_acceptance.py --archive <zip> --out <receipt.json>`.

Seeded Rust property targets are `crates/porch-node/tests/properties.rs`: canonical signature mutation/domain separation, invitations and grant authority/time/scope, evidence hashes and physical-class promotion, stored caps/content IDs. `wire_codec.rs` tests strict read/write/padding limits. The ordinary suite uses 128 seeded cases; `PORCH_PROPERTY_CASES=16384 cargo test --locked --test properties` expands it, with a 65536 cap. The separate scheduled/manual workflow preserves normal PR run duration. These are bounded property/mutation checks, not a claim of coverage-guided fuzzing. Prioritize future cargo-fuzz targets for nested signed invitation/grant/frame/advertisement parsers, evidence manifests and state metadata; retain crashing inputs as private minimal reproductions if they contain sensitive data.
