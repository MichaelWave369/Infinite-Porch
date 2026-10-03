# Privacy

LOCAL_ONLY is the default. Input is filtered before routing and guarded again
before RPC transmission. A refusal for no eligible host sends no job input to
peers. TRUSTED_PEERS and PORCH_ALLOWED are deliberate permissions to send plaintext
input across an encrypted session to an approved provider. That provider can
retain it; session encryption protects transport, not against its recipient.

No global user directory or public community announcement exists. A signed
advertisement reveals only selected model/service claims, voluntary quota, alias
and a small set of observations to approved peers. Actual GPU/RAM details remain
unknown. mDNS and Identify still reveal transport-level presence to a LAN/connected
peer; complete traffic anonymity is not a candidate claim.

The local ledger stores hashes, identities, reason codes, times and resource facts.
Prompt text and model output are omitted from its event payloads. Private output
receipts and messages remain in the protected local database. Reason text from
errors can expose local paths; treat diagnostic logs and exported receipts as
sensitive unless reviewed. The UI ledger export contains only the visible 200
events and is not a full backup.

Blob providers receive client-side ciphertext, IDs, owner and size. Decryption keys
stay in the origin database. Local browser/API clients can retrieve plaintext when
explicitly authorized by the operator token. No telemetry, remote log upload,
hidden agent installation or cloud credential collection exists.

Deleting local metadata/tombstones does not guarantee erasure from backups, disks,
recipient devices or screenshots. The daemon has no retention scheduler in this
candidate. Back up and protect the entire state directory, and verify recipients
before sharing invitation or grant packages.
