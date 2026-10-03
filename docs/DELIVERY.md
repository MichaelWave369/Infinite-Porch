# Infinite Porch 0.1.0 delivery assessment

## Repository state

Independent new Rust/TypeScript monorepo. Branch
`feat/porch-v0.1.0-candidate`. The bundle's REPOSITORY_STATE.json records the exact
commit and clean tracked state. Its git bundle preserves that commit/history;
the source snapshot contains the same tracked files. No remote repository was
provided or created, and related projects were inspected read-only.

## Architecture and implemented behavior

Rust core/daemon/CLI, rust-libp2p authenticated peer sessions, SQLite authority and
evidence, encrypted filesystem CAS, same-origin React desktop and typed scoped SDK.
No mandatory cloud authority. Implemented local vertical spine: identity, invite/
trust, signed claims, bounded grants, deterministic routing, real remote jobs,
signed receipts, accounting and ledger.

The expanded candidate includes protected identity backup/rotation, client app
delegation, direct/pairwise channel messaging and bounded offline outbox, builtin
hash compute, loopback Ollama adapter, explicit mock inference, chunked encrypted
replication/readback/tombstones, resumable transfer and uncertain-job recovery,
logical addresses, operator UI/CLI/API/SSE and portable package configuration.

Mock inference is labeled and selected explicitly. Ollama contract tests use
simulated HTTP provider fixtures. The test network uses actual daemon processes
and encrypted loopback TCP/Noise paths, with QUIC listeners. It does not qualify
physical LAN, WAN/NAT or live model weights. No GPU pooling, private provider
attestation, running relay/federation/WASM or sharded execution is claimed.

## Test evidence

Format, strict Rust lint, TypeScript checks, Rust/desktop/SDK builds pass. The
candidate has 13 passing Rust tests, two SDK tests, eight CLI checks, 33 end-to-end
peer checks and eight real-UI checks. Raw command logs, repaired initial failures,
the final structured gate receipt and acceptance result live in docs/receipts.
The release acceptance result is recorded separately from the debug harness.
The Linux x86_64 portable package contains optimized node/CLI and built UI, with
SHA-256 manifest. Windows/macOS CI and package configuration exist but have not
run remotely; no native signing/notarization success is asserted.

## Run, demonstrate and share

From source root:

```sh
npm ci
npm run build
cargo build --workspace --locked
./target/debug/porch --data state init --alias "Michael"
./target/debug/porch-node --data state --ui apps/desktop/dist
```

Open `http://127.0.0.1:7331`, using the private local state/api.token. A delivered
portable Linux build runs with start.sh. The API remains loopback; peer traffic
uses separately configured transport listeners.

```sh
python3 tests/acceptance.py
python3 scripts/demo.py
```

The harness stops automatically and writes the final acceptance receipt. The
interactive three-daemon demonstration runs until Ctrl+C, prints UI addresses
and token paths and explicitly offers mock inference, compute and storage.

For a live already-installed Ollama model:

```sh
./target/debug/porch --data state model discover
./target/debug/porch --data state share model qwen3:8b
./target/debug/porch --data state grant issue --recipient PEER_ID --capability model.inference --resource qwen3:8b --max-calls 20 --ttl 3600 --out inference-grant.json
```

The approved requesting PC imports that grant then runs with TRUSTED_PEERS. The
default LOCAL_ONLY request stays on the origin. See README.md for both sides.

For another PC, start its node and exchange its public ID. On the owner:

```sh
./target/debug/porch --data state create "Oak Street"
./target/debug/porch --data state invite --recipient RECIPIENT_PEER_ID --ttl 600 --out porch-invite.json
```

Transfer the signed invite directly to the intended PC, then run
`porch --data state join porch-invite.json` there. Membership supplies no resource
authority. Verify pinned peer ID and intended LAN port reachability; issue and
import each bounded grant separately.

## UI, limitations and remaining work

Actual daemon UI evidence: evidence/home.png, models.png and mobile.png. The
browser test submitted a real remote mock job; no API state was substituted.

SECURITY.md and THREAT_MODEL.md explain local plaintext key/DB risks, malicious
providers, metadata exposure, cancellation uncertainty, bounded-load limits and
ledger rollback. ROADMAP.md explicitly separates REQUIRED production work from
FUTURE engines. Required work includes independent review/fuzz/load/crash tests,
OS secret/permission qualification, live Ollama/hardware policies, physical LAN/
QUIC/mDNS and WAN/authorized relay evidence, platform builds/signing and recovery/
metrics/app-registration completeness. Future work includes federation, mesh/
radio, additional providers, full ecosystem/media adapters and sharded models.

The untouched controlling brief, requirement register, freeze/build/deviation
receipts, final acceptance.json and Wayne handoff preserve scope and provenance.
Local builder verification is complete for the stated candidate. The original
production-capable objective remains open. No independent verification or public
release claim is made; final release authority stays human-owned.
