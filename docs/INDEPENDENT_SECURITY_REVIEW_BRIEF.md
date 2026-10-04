# Independent security review brief — 0.1.2

Status: **UNVERIFIED**. Review preparation is complete only when the source, source history, lockfiles, native evidence and reproduction kit are present. No independent reviewer has signed off this candidate.

Review protocol 1 / schema 2 against the controlling brief, particularly privacy before routing, scope/quota/revocation, canonical/domain-separated signatures, peer parser resource limits, storage encryption/integrity, durable restart and uncertain execution, localhost authority, Windows ACL/helper behavior and the field certificate's resistance to contradictory evidence. Inspect the sensitive-module map and expected refusal cases in `security-review/`.

Request independent adversarial reproduction of parser/authority boundaries on native platforms and two physical PCs. Preserve failures rather than weakening gates. Provide exact source commit, platform/CPU, test identity/scenario references, affected boundary, attack steps, evidence hashes, severity and recommended repair; validate the repair before release. Do not include live tokens, private keys or private prompts in shared findings.

Michael retains release ownership. Native hosted success and a usable physical kit are engineering qualification; production remains gated on physical network/model evidence and independent security review.
