# Security boundary and release limits

This is a local candidate with adversarial tests, not an audited production
security guarantee. The controlling brief's production-capable target remains
open until the REQUIRED items in ROADMAP.md are qualified.

Peer sessions authenticate device keys. Domain-separated Ed25519 records bind
effects to authenticated principals. Persistent replay protection, expiration,
issuer-record validation, revocation and transactional quotas deny unauthorized
effects. LOCAL_ONLY has both scheduler and outbound guard enforcement. Neither
an advertisement nor a model/agent role supplies authority.

The local API binds only to loopback, checks exact Host and Origin, requires a
bearer credential, refuses redirects, sends CSP/no-store/nosniff headers and bounds
bodies. Apps get finite scoped credentials; administrative control stays with
the operator credential. Treat localhost and the OS account as trusted: another
process in that account may read files or browser memory. This is not a sandbox
against an already-compromised machine. Sustained rate shaping,
OS key stores and platform ACL qualification remain required.

Keys, API tokens, prompts, messages and receipt outputs are sensitive local files
or database records. The database is not encrypted at rest. Unix directory/file
modes are set, but Windows ACLs and macOS vault integration are unverified. Protected
identity export uses Argon2id/AEAD. Network encryption and ciphertext replication
do not cure theft of a running device's local data.

Bounded queues, connection counts, frames, jobs, provider responses, blob sizes,
metadata pages and storage quotas reduce exhaustion. They do not establish measured
CPU/memory/bandwidth isolation. There is no GPU reservation, malicious-provider
attestation, process sandbox or verified forwarding engine. Inference results are
signed provider assertions. A provider can lie or retain inputs explicitly sent
to it. There is no private remote prompt guarantee against that provider.

Revocation suppresses active remote output and refuses new effects. It cannot
undo transmitted input or prove that Ollama stopped processing. Grants already
consumed before a crash remain consumed. Uncertain execution refuses replay.
The ledger detects edited surviving records, but a local attacker can erase a
valid tail or restore an old database without an external checkpoint noticing.

No public DHT, relay, anonymous exit proxy, general shell, auto-update, telemetry,
account login or hidden service installation is active. Networking dependencies
for future facilities do not mean those facilities are implemented.

Before release, perform independent security review/fuzzing, sustained load and
disk-full tests, dependency/license/advisory review, platform permission tests,
real Ollama/hardware tests and physical-network qualification. Pin dependency
lockfiles and review updates. No security scanner result is fabricated here.

0.1.1 caps are listed in config.example.json. To change them, stop the daemon,
place a limits object (partial fields take safe defaults) in limits.json, run
`porch --data YOUR_STATE configure-limits limits.json`, then restart. The state
lock refuses this operation while a daemon owns the state. Local peers cannot
change these operator caps. Lowering a storage cap never deletes existing data;
new writes fail when the combined local/remote item and partial-transfer total
would exceed it. Revoked grant nonces remain counted rather than being forgotten.

The clock permits at most two seconds of future creation skew, never adds grace
to expiry, and persists a lower wall-clock bound. Backward movement beyond two
seconds records a clock anomaly and refuses time-sensitive operations/startup.
Local deadlines and connection ages use monotonic elapsed time. Whole-state
rollback and an untrusted host clock still require external checkpoints.

Schema 1 migrates to 2 with a private SQLite backup before a transactional schema
change. A preexisting backup collision or unsupported future schema fails closed.
Missing identity beside an existing database refuses regeneration. Qualification
export whitelists public diagnostic receipts and a signed ledger-verification
statement; it does not copy the database, keys, tokens, arbitrary logs or unrelated
private prompts/outputs. IP/peer identifiers in diagnostic evidence are intentional
metadata: review the bundle before sharing it. Node signatures are not official
release signatures or independent validation.

See SUPPLY_CHAIN_REVIEW.md for two configuration-inapplicable Hickory advisories
and the retained paste maintenance warning. The reviewed gate fails on changed
features or newly discovered findings; it is not a claim of a clean upstream audit.
