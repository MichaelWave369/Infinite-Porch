# Signed qualification evidence

```sh
porch qualify run --environment LOOPBACK
porch qualify export evidence-directory
porch qualify validate evidence-directory
# UI export creates one JSON container instead:
porch qualify validate porch-qualification-bundle.json
```

Each export includes run.json, node.json, results.json, receipts.json,
ledger-verification.json, sanitized-log.json and manifest.json. The manifest
hashes the exact UTF-8 pretty JSON file bytes and is signed by the node identity.
The signed run binds environment, peers, scenarios and its public diagnostic
receipts. Validator checks domain/version/signature/peer binding, hashes, complete
file set, result schema, timestamps, peers, receipt output digests and the public
prompt's input digest. It rejects modified, malformed and mismatched evidence.
The on-disk reader applies file and aggregate size bounds; there are no arbitrary
export filenames. Existing evidence directories are not overwritten.

Every scenario contains expectation, observation, result, evidence_file,
timestamp and involved peers. Allowed results are PASS, FAIL, REFUSED_EXPECTED,
SKIPPED_ENVIRONMENT, UNVERIFIED, PARTIAL. Skipped and unverified never count as
passes. Physical separation and WAN condition are operator attestations; the
build does not award a production qualification badge.

Only qualification-created receipts for the fixed PUBLIC_PROMPT are exported.
No unrelated job outputs, private prompts, raw daemon logs, databases, grants
containing local credentials, API tokens or private keys are copied. Public keys,
peer IDs, resource names and advanced socket observations are included and may
still disclose network topology; review before sharing. Historical 0.1.0 refusal
strings are not copied because they may contain private parser data.

Ledger-verification.json is a signed full-local-chain verification statement with
record count and tip hash. It is not a complete chain export or independent chain
verification. Node signatures prove the signer and payload integrity, not physical
truth, official publisher trust, owner identity or independent security review.
The example sanitized bundle is a LOOPBACK rehearsal with explicit mock inference.
No runtime identity key, token, database or vault key is bundled with it.
