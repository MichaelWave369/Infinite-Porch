# Safe optional failures

`Porch-FieldLab.ps1 failure-check -Peer <intended-peer>` checks refusal of a nonexistent model. `revoke` followed by `refused-check` exercises cached revoked authority with a signed refusal. `stop` only terminates the kit-owned Porch process; `setup` restarts it without changing its identity. Preserve evidence before and after each check.

You may manually exit your own Ollama process, observe bounded provider failure, and launch it again. Never stop unrelated services. Use an isolated test Porch/grant to exercise a two-call quota or deliberately short expiry; do not exhaust grants used for the main field sequence. Use new job IDs and retain expected refusal receipts. Do not tamper with the live vault to test corruption: copy a public test ciphertext outside the state directory, change the copy, and compare its SHA-256 to the CID. That proves only the copy-integrity check; it does not prove a live remote retrieval refused corrupted storage. The deterministic acceptance suite contains an isolated corruption/retrieval test.

Network cancellation, residential NAT, sustained load and GPU/VRAM checks remain separate scenarios. Do not represent a command that was not executed as PASS. Preserve local daemon logs privately; export only the whitelisted public qualification artifacts.
