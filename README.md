# Infinite Porch 0.1.1 — physical qualification candidate

Required Rust: **1.89.0**. The locked UUID dependency rejects 1.88; 1.89 is the tested compiler. Node 22.12+ and Python 3.12+ are used by the build and acceptance tooling.

Continue the 0.1.0 architecture: Ed25519 peer identities, QUIC or TCP/Noise,
explicit trust and bounded grants, local Ollama, encrypted ciphertext storage,
messaging, CLI, SDK, React desktop control UI and signed Reality Ledger.

This candidate adds a first-class physical qualification workflow, present-time
provider verification, authenticated path observations, signed sanitized evidence,
recovery tests, schema migration, operator caps and platform helpers.

| Evidence class | Status |
| --- | --- |
| Implementation | Qualification CLI/UI, live adapter, diagnostics, export/validator, migrations and hardening implemented |
| Local verification | See `docs/receipts/qualification/command-gates.json` for executed checks |
| CI | Multi-platform workflow configured; no remote CI execution claimed |
| Physical computers / LAN / real weights | UNVERIFIED until separately collected evidence exists |
| Windows and macOS native operation | UNVERIFIED; source workflows and guides provided |
| Independent security review / production release | UNVERIFIED / not released |
| Federation, public relays, WASM, sharding, GPU pooling | FUTURE; not installed |

Start with [START_HERE.txt](START_HERE.txt). Exact two-PC procedures:
[Windows](docs/WINDOWS_QUALIFICATION.md), [Linux](docs/LINUX_QUALIFICATION.md),
[macOS](docs/MACOS_QUALIFICATION.md). The shared acceptance sequence is in
[Physical qualification](docs/PHYSICAL_QUALIFICATION.md).

```sh
cargo build --workspace --locked
npm ci
npm run build
cargo run -p porch-cli -- init --alias MIKEY-PC
cargo run -p porch-node -- --data state --ui apps/desktop/dist
# In another terminal:
target/debug/porch qualify status
target/debug/porch models scan
target/debug/porch qualify run --environment LOOPBACK
target/debug/porch qualify export qualification-local
target/debug/porch qualify validate qualification-local
```

The control API remains literal loopback, same-origin and token authenticated.
The peer transport is a separate surface. Sharing starts disabled; advertisements
provide no invocation authority. Qualification probes use a fixed public prompt
and do consume applicable grants. Skipped checks never count as passes.

## Architecture and operator reference

Connect computers you own with people you choose. Offer a little storage, share
an installed model, send a message, or run a bounded task on an approved device.
Each device keeps its own identity, permissions and evidence. Basic operation
needs no vendor account, cloud service, blockchain or central authority.

## Run from source

Requirements: Rust 1.89.0, Node 22.12+, npm, Python 3.12+, and a C compiler for bundled
SQLite. Linux x86_64 builds and local runtime are tested here. The CI configuration builds and tests Windows,
Linux and macOS; those remote jobs have not been run for this candidate.

```sh
npm ci
npm run build
cargo build --workspace --locked
./target/debug/porch --data state init --alias "Michael's computer"
./target/debug/porch-node --data state --ui apps/desktop/dist
```

Open `http://127.0.0.1:7331` and paste the operator token from `state/api.token`.
Keep that token private. The UI keeps it in memory for the current session.
Models, remote compute and storage are **off until you choose to share them**.
Use `porch doctor` to check local identity, disk access, clock and ledger.

On Windows, use `target\debug\porch.exe` and `porch-node.exe`. On macOS/Linux,
use the executable paths above. Stop the foreground daemon with Ctrl+C.

## Try three computers on one machine

```sh
python3 tests/acceptance.py
python3 scripts/demo.py
```

The first command runs the deterministic peer checks, then stops the nodes. The
second leaves ALPHA, BETA and GAMMA running for inspection and prints each UI
address and local token-file location. It explicitly offers a deterministic
**MOCK** model, hash compute and encrypted storage. Use a fresh demo state
directory; Ctrl+C stops all three. It does not simulate pooled GPU memory.

## Expose an installed Ollama model

With Ollama already running locally and the named model installed:

```sh
./target/debug/porch --data state model discover
./target/debug/porch --data state share model qwen3:8b
./target/debug/porch --data state grant issue --recipient PEER_ID --capability model.inference --resource qwen3:8b --max-calls 20 --ttl 3600 --out inference-grant.json
```

On that approved peer, import the grant and explicitly permit remote processing:

```sh
./target/debug/porch --data state grant import inference-grant.json
./target/debug/porch --data state model run qwen3:8b "hello" --privacy TRUSTED_PEERS
```

LOCAL_ONLY is the default. A failed real provider returns a failure; it never
substitutes the mock. Signing a receipt authenticates who reported a result; it
does not prove that a remote provider computed honestly.

## Join another PC

Start a node on each PC. Exchange the intended recipient's peer ID through a
channel you trust. On the Porch creator:

```sh
./target/debug/porch --data state qualify host "Oak Street" --recipient RECIPIENT_PEER_ID --out porch-invite.json
./target/debug/porch --data state identity show
```

Compare the entire fingerprint directly on the intended creator PC. Transfer the
invitation to its bound recipient and run:

```sh
./target/debug/porch --data state qualify join porch-invite.json --fingerprint VERIFIED_CREATOR_SHA256
./target/debug/porch --data state peers
./target/debug/porch --data state share refresh
```

The package pins the creator's cryptographic identity and reachable addresses.
Allow peer TCP/UDP port 7332 on the intended private network if needed. Keep the
control API on loopback. Invites are one-use, short-lived and grant membership,
not execution. Members approve direct relationships and grants separately.

## What's inside

| Component | Responsibility |
|---|---|
| `porch-core` | Versioned records, canonical signatures, encryption and deterministic routing |
| `porch-node` | libp2p transport, SQLite authority/accounting, providers, vault, messaging and API |
| `porch` | Operator CLI, pairing, grants, identity backup/rotation and diagnostics |
| React desktop UI | Ten operator views served by the local daemon |
| TypeScript SDK | Scoped application requests and independent ecosystem adapters |

Encrypted sessions use QUIC or TCP/Noise. Signed advertisements are claims;
trust, privacy and a bounded signed grant are checked before remote effects.
The scheduler filters first, then prefers local execution, available slots and
observed latency. Prompts and outputs stay out of the digest-based event ledger.
The provider necessarily sees input you explicitly send it.

## Verify and package

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run typecheck
npm test
python3 tests/cli_smoke.py
python3 tests/acceptance.py
python3 tests/qualification_acceptance.py
python3 tests/process_failures.py
python3 tests/process_failures.py --transport quic
python3 tests/transport_acceptance.py
python3 tests/limit_cli.py
python3 scripts/check_platform.py
python3 scripts/check_toolchain.py
npx playwright install chromium
python3 tests/ui_acceptance.py
python3 scripts/package.py
```

The delivered Linux portable package contains release binaries and the built
UI. `start.sh` starts a foreground node. `start.ps1` is the Windows equivalent
when built on Windows. There is no service installation or autostart.

See [architecture](docs/ARCHITECTURE.md), [authority](docs/AUTHORITY.md),
[security and limitations](docs/SECURITY.md), [testing evidence](docs/TESTING.md),
[operations](docs/OPERATIONS.md), [API](docs/API.md) and [CLI](docs/CLI.md).
The untouched current brief is [CONTROLLING_SPEC_0.1.1.txt](docs/CONTROLLING_SPEC_0.1.1.txt); the original 0.1.0 brief and baseline tag are preserved.
The complete delivery assessment is [DELIVERY.md](docs/DELIVERY.md).
The current traceability register is [REQUIREMENTS_0.1.1.md](docs/REQUIREMENTS_0.1.1.md).
