# Bounded failure testing

Hooks live in owned test processes and the Transport test wrapper, not in a
production chaos-control API. Commands:

```sh
cargo test --workspace --locked
cargo test -p porch-node --test faults
cargo test -p porch-node --test provider_runtime
cargo test -p porch-node --test recovery
python3 tests/process_failures.py
python3 tests/process_failures.py --transport quic
python3 tests/qualification_acceptance.py
```

| Fault | Installed test/procedure | Scope |
| --- | --- | --- |
| Dropped response | faults.rs mode 1 | execute once, lose reply, retry same manifest/executor |
| Duplicate request | faults.rs mode 2; security replay test | same signed nonce rejected; new wire nonce with same job returns original receipt |
| Corrupted response/payload | faults.rs mode 3; signature tests | binding/signature refuses; no altered result accepted |
| Delayed packet | faults.rs mode 4 | bounded application delay over loopback transport |
| Duplicate response | faults.rs mode 5 | request ID correlation accepts one response; no second provider effect |
| Job timeout / provider error | providers.rs, provider_runtime.rs | owned HTTP fixture, null output, no fallback |
| Expired/revoked/quota-exhausted grant | security.rs, providers.rs, acceptance.py | effect boundary and final release revalidation |
| Executor process death | process_failures.py | terminates only child daemon; RUNNING becomes UNCERTAIN |
| Requester process death | process_failures.py | origin's persisted intent reconciles original executor receipt |
| Network interruption | owned daemon stop + restart; loopback reconnect | no adapter/firewall/routing changes |
| Provider stops during inference | provider_runtime.rs HTTP death fixture | real Ollama stop remains external operator test |
| Corrupted stored blob | acceptance.py | disposable fixture ciphertext changed, digest/decryption refuses |
| Rename-before-commit crash | recovery.rs | precise durable state fixture, not power-loss proof |
| Restart, expired transfer, stale resource/service | recovery.rs, acceptance.py, qualification tests | persistent authority and signed-expiry checks |
| Invite theft/replay/forgery/race | faults.rs and qualification.rs | recipient pin, signature, one-use transaction, fingerprint |

To reproduce a specific physical interruption, use public diagnostic inputs,
record job ID/peer IDs first, stop only your own foreground test node with Ctrl+C,
restart with the same state, and retry the SAME ID. UNCERTAIN jobs never silently
execute again or move to a second executor. A new ID is a new job and needs a
valid grant. Never erase failed state or ledger entries to manufacture recovery.

A failed transport does not prove whether the provider executed. The persisted
origin intent and cached executor receipt preserve that uncertainty. An outgoing
job wait is bounded by its declared duration plus two seconds of transport
allowance (and the underlying transport timeout). This allowance does not extend
the grant expiry or executor's duration cap. On a job deadline the stale pinned
connection is disconnected so an explicit same-ID retry can reconnect. Other
inner transport errors do not tear down a healthy connection. There is no
automatic job reroute or automatic new execution.

A rapid QUIC restart can retain a dead UDP session until the first attempt
times out. Process tests record each attempt, retry at most three times only
for PEER_JOB_DEADLINE/PEER_TRANSPORT_FAILURE, and require the original cached
receipt or signed expected refusal. The first failed attempt remains in
bounded_reconnection_observations. Provider call counts must still prove no
duplicate effect. A generic connection failure cannot count as revocation proof.
The qualification messaging probe has a separate five-second diagnostic wait;
an offline peer produces a recorded FAIL, not an indefinite CLI hang.
