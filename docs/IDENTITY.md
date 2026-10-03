# Device identity

Each installation creates an established libp2p Ed25519 keypair locally. The
public key derives its Peer ID. An alias is a local label, not identity or proof
of personhood. Several devices may share a human operator, but every device
must be paired and authorized separately. No biometrics or human directory.

`identity.key` contains private protobuf key material. On Unix, the node directory
is mode 0700 and private files are created 0600. The running device key is not
encrypted with an OS key vault in this candidate. Windows ACL and macOS Keychain
integration remain required release work. An attacker who controls the OS can
read device keys and private local detail records.

The CLI deliberately exports protected identity backups using Argon2id and
XChaCha20-Poly1305. Passwords must contain at least 12 bytes. Supply them through
a named environment variable; avoid putting them in command history. Restoring
uses a fresh directory and does not replace existing keys. This backs up identity
only; it does not back up storage decryption keys, trust or database state.

```sh
porch --data state identity backup --out identity.backup --password-env PORCH_BACKUP_PASSWORD
porch --data restored identity restore identity.backup --password-env PORCH_BACKUP_PASSWORD
porch --data state identity rotate --new-data rotated --out rotation-proof.json
```

Rotation creates a fresh device and a cross-signed, one-day proof. An operator on
each peer imports it with `porch peer rotate rotation-proof.json`. The old peer
and its grants are revoked. The new key carries no resource authority or Porch
membership: approve/rejoin and issue new grants explicitly. A stolen key cannot
be remotely erased. Revoke it on each affected peer and rotate local credentials.

Recipient-bound, one-use invitations pin the creator key and signed Porch ID.
Their use establishes initial trust/membership only. An unbound bearer invite
is supported but can be accepted by whoever obtains it; use recipient binding.
