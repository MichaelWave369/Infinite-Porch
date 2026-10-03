# Encrypted Porch Vault

Local plaintext is sealed with a new random 32-byte key and XChaCha20-Poly1305
nonce. Owner Peer ID is authenticated additional data. Content ID is SHA-256 of
ciphertext, including its nonce and tag. Only the owning node keeps the key in
private local metadata. A storage provider receives ciphertext, size, owner ID
and signed authority, never the plaintext decryption key.

Maximum ciphertext is 8 MiB, hence plaintext is at most 8 MiB minus 40 bytes.
Local owned ciphertext is bounded to 128 MiB. Remote allocation is explicitly
configured. Each provider checks its total stored/reserved allocation and the
grant's cumulative byte budget before accepting a transfer. At most 16 partial
uploads exist. Storage metadata has no path supplied by a peer: strict lower-case
hex content IDs select filenames inside the vault.

Begin returns the durable offset. Chunks are at most 64 KiB with exact monotonic
offsets and grant revalidation. Bytes are synced before committing an offset.
Restart truncates uncommitted trailing bytes. If a final file rename preceded
metadata commit, recovery validates full ciphertext size/digest and completes
metadata without repeating upload. Missing transfer files release the reservation;
expired uploads are cleaned when another begin is received. Consumed grant budget
is not refunded. Crash-state fixtures are not physical power-loss qualification.

Replication tracks desired and last-verified states separately. Completion verifies
ciphertext CID; the requesting owner then fetches the entire replica and compares
it to its local ciphertext. Retrieval checks digest and AEAD before returning local
plaintext. Corruption refuses; it never marks that replica repaired by guesswork.
Verified means last successful full retrieval, not continuously available backup.

Pinning protects owned objects from ordinary local deletion. Local tombstones remove
the decryption key and requestable local content; remote tombstones are separately
authorized operations. Neither proves secure erasure on malicious storage or old
backups. Deleting a peer replica clears its verification state. On Windows, replacing
a fetched local file has a short removal/rename gap; interruption requires retry.

Protect the whole data directory if you need recoverable storage. An identity-only
backup omits the locally stored per-blob decryption keys. Durable replication factors,
repair scanning, erasure coding, multi-owner metadata and verified backup policies
remain future work; do not treat one replica as a backup guarantee.
