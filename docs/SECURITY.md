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
against an already-compromised machine. Rate-limiting privileged local clients,
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
