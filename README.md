# Infinite Porch

Connect computers you own with people you choose. Offer a little storage, share
an installed model, send a message, or run a bounded task on an approved device.
Each device keeps its own identity, permissions and evidence. Basic operation
needs no vendor account, cloud service, blockchain or central authority.

**0.1.0 is a runnable local candidate, not a qualified production release.** The
three-process acceptance demo uses real encrypted peer connections and explicit
mock inference. Live Ollama, physical LAN discovery, WAN traversal and native
Windows/macOS packages still require qualification. [Required work and future
extensions](docs/ROADMAP.md) are kept separate.

## Run from source

Requirements: Rust 1.99.0, Node 22+, npm, Python 3.10+, and a C compiler for bundled
SQLite. Linux is verified here. The CI configuration builds and tests Windows,
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

The first command runs the complete automated proof, then stops the nodes. The
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
./target/debug/porch --data state create "Oak Street"
./target/debug/porch --data state invite --recipient RECIPIENT_PEER_ID --ttl 600 --out porch-invite.json
```

Transfer that package to the recipient and run:

```sh
./target/debug/porch --data state join porch-invite.json
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
| React desktop UI | Nine operator views served by the local daemon |
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
The untouched build brief is [CONTROLLING_SPEC.txt](docs/CONTROLLING_SPEC.txt).
The complete delivery assessment is [DELIVERY.md](docs/DELIVERY.md).
