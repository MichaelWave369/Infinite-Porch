# Wayne Rider handoff manifest

Project: Infinite Porch. Candidate: 0.1.0. Prepared for independent, non-destructive
verification; no independent review has been run or claimed in this build.

Authority: untouched CONTROLLING_SPEC.txt and freeze receipt. No amended canon.
Final release authority remains with the human operator.

| Artifact | Role | Integrity / location |
|---|---|---|
| Controlling brief | Untouched source | SHA-256 4e668341934d471be3b912c41eed24a124a0992528841a3a1f75b592e6bd6386; docs/CONTROLLING_SPEC.txt |
| Candidate source | Implementation | Exact git commit and source bundle in outer REPOSITORY_STATE.json / git bundle |
| Requirement register | Coverage/status | docs/REQUIREMENTS.md, REQ-001–045 |
| Forge local gates | Builder evidence | docs/receipts/build.json, command-gates.json and raw logs |
| End-to-end result | Acceptance evidence | docs/receipts/acceptance.json, signed remote job receipt |
| UI evidence | Real operator surfaces | docs/evidence/*.png, ui-receipt.json |
| Portable build | Current OS executable/UI | portable manifest with SHA-256 file hashes |
| Deviations/failures | Preserved limitations/repairs | docs/receipts/failure-and-repair.json and deviation.json |

Mocks: deterministic inference is explicitly selected and labels output. Ollama
adapter unit tests use HTTP contract fixtures, not real weights. Simulations:
three real processes and loopback encrypted peer paths, not physical homes.
Uninstalled engines: relay, federation, WASM, ensemble/pipeline/sharded model.
Unavailable qualification: physical LAN/WAN, independent NATs, OS key-store/ACL
tests, Windows/macOS signing/native installers, live model hardware, external
ecosystem runtimes and independent security review. See ROADMAP.md for required
production work; these cannot be inferred from passing local gates.

Requested proving objectives: reproduce every deterministic gate; inspect signed
principals and canonical serialization; attack local/peer grant scope and replay;
test quota races and uncertain recovery; review secret persistence and ledger
rollback; qualify physical transport/provider/platform boundaries; preserve all
failures and verify repairs. Do not infer authority from an agent role, peer claim
or mock inference success.

Candidate prepared for Wayne Rider Proving Ground. No independent verification
or release claim is made by the Forge. This manifest does not automatically
invoke another skill or send a message to another person.
