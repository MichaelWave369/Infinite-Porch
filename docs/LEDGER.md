# Evidence ledger and accounting

Every ledger event is locally signed Ed25519 evidence with sequence, event type,
timestamp, subject, actor, relevant hashes and previous-event hash. An SQLite
transaction commits associated state and ledger events together where applicable.
Startup and `porch doctor` verify every surviving row's signature, chain, signer
and sequence. Corruption causes startup refusal; do not bypass verification.

Events include start, trust/pairing, grant issue/refusal/revocation, connection
changes, job admission/refusal/completion, storage chunks/replica verification/
tombstones, messages and scoped client authorization. Expiration fails closed
when authority is checked. The candidate has no autonomous grant-expiration
event sweeper; expiry state is computed from signed times.

SSE `/v1/events` publishes committed ledger entries through a bounded 256-entry
broadcast channel. A slow reader receives GAP and should refetch. Read/export
shows the last 200 events; the complete chain remains in the database. A local
tail deletion or rollback cannot be detected without an external checkpoint.
There is no consensus ledger, blockchain, token or automatic public replication.

Provider grant counters are durable authority accounting: total/hour calls,
cumulative storage reservations and readback transfer bytes. Receipts preserve
duration, input size and provider-reported tokens when available. GPU seconds,
VRAM, monetary cost and physical network bandwidth remain unknown. UI aggregate
job metrics cover the latest 200 retained-visible receipts, not lifetime totals.
Grant budgets do not silently reset across restart. A new signed grant is a new
operator allocation.

Communities can implement gifting/fairness/credits by reading these records and
proposing future policy. No accounting record or economic recommendation itself
issues authority. Payments and new enforcement engines require explicit adapters.
