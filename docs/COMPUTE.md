# Bounded job execution

Installed job types are model.inference and compute.hash/sha256. No shell,
downloaded executable, generic filesystem access, arbitrary plugin or WASM host
is exposed. SINGLE_NODE and REMOTE_NODE refer to complete jobs on one host.
ENSEMBLE, PIPELINE and SHARDED_MODEL have no engine and refuse as unsupported.

The versioned Job has requester, ID, capability/resource, exact input/digest,
output token ceiling, timeout, privacy, execution mode, preferred executor and
signed grant. IDs are scoped to requester. Reuse with different manifests fails
JOB_ID_CONFLICT. Input is at most 64 KiB and jobs at most 30 seconds/4096 requested
output tokens. Two slots accept work without an unbounded pending job queue.

Admission validates trust, grant, scope, sharing, capacity and limits in a durable
transaction. The receipt contains exact hashes, requester/executor, provider,
model identifier when available, elapsed time, reported measurable usage, grant
nonce, status/reason and signature. The local ledger contains digest evidence;
the local jobs table retains private receipt output.

Ollama is accessed only on loopback, without redirects/proxy environment settings.
The adapter calls /api/tags and non-streaming /api/generate. It bounds request and
response sizes, passes num_predict, disables thinking and reports available token
counts. The node enforces duration and rejects reported output tokens exceeding
the request. Remote cancellation is polled and rechecked before output release.
Cancellation of an HTTP future does not prove cancellation of provider hardware.

The explicit mock is deterministic and labels output `mock:true`. It is useful
for network/authority proofs, not an inference benchmark. Builtin SHA-256 is a
real deterministic compute task. Ollama HTTP contract fixtures test discovery,
successful parsing, malformed/oversized output, unavailable provider, timeout
and active revocation; they do not run actual weights or validate VRAM isolation.

On restart a RUNNING job becomes UNCERTAIN and cannot reexecute. Completed jobs
return their verified original receipt. The origin stores its first exact route
and manifest before transport and retains result receipts across restart. A
retry must keep the same intent and executor. If that executor cannot answer,
the result remains ambiguous until inspected; a new ID is a new execution intent.
