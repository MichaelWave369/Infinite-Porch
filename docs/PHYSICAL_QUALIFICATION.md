# Two-computer physical qualification

External evidence remains UNVERIFIED in this candidate. The tooling makes the
acceptance sequence reproducible; it cannot remotely prove ownership, physical
separation, router topology or upstream WAN disconnection.

Use two separately owned PCs on the same private LAN. Install the native build,
initialize independent state directories and keep both foreground nodes running.
The platform guides give complete commands. All commands below are local operator
commands; no cloud account, public bootstrap, relay, or public Ollama listener is
needed. The control API stays on 127.0.0.1:7331. The peer port is TCP/UDP 7332.

1. On B, run `porch identity show`; send its peer ID to A.
2. A runs `porch qualify host "Oak Street" --recipient B_PEER_ID --out invite.json`.
3. On A, display `porch identity show`. Compare its full SHA-256 fingerprint
   directly on A's screen or through an already authenticated channel.
4. Copy only invite.json to B. B runs `porch qualify join invite.json --fingerprint HOST_FINGERPRINT`.
5. On both, run `porch qualify status`. Confirm the intended peer IDs and aliases,
   active trust, the same Porch ID, encrypted authenticated direct transport,
   actual non-loopback addresses and `/infinite-porch/rpc/1` after exchanging RPCs.
6. On A, run `porch models scan`, `porch share model EXACT_MODEL`, and
   `porch models verify EXACT_MODEL`. Verification uses a fixed public prompt.
7. B runs `porch share refresh` then `porch models list`. A model advertisement
   is a signed availability claim and gives B no invocation authority.
8. A issues an exact-resource grant to B, with at least 16 output tokens and
   enough duration for the diagnostic. Import its JSON on B. Use the precise
   commands in the platform guides; do not copy A's api.token or identity.key.
9. B runs `porch qualify run --peer A_PEER_ID --model EXACT_MODEL --environment PHYSICAL --separate-machines-confirmed`.
10. Inspect the route, executor, provider `ollama`, input/output digests, duration,
    grant nonce and signed execution receipt. The mock provider yields PARTIAL,
    never a real-model PASS. A failed job yields FAIL; unavailable environments
    yield explicit SKIPPED_ENVIRONMENT/UNVERIFIED results.
11. Export on B: `porch qualify export evidence-baseline-B`. Validate with
    `porch qualify validate evidence-baseline-B`. Export A's separately run local
    checks as well. Inspect signatures and matching peer identities on both.
12. A revokes the grant. B runs the same qualification command with `--phase revoked`.
    A new attempt must return a revoked-grant refusal, recorded REFUSED_EXPECTED.
    An unrelated missing model or network failure does not pass this test.
13. Stop and restart both foreground nodes with the same state paths. Compare
    identity, trust, Porch, grant revocation, storage objects, active/UNCERTAIN
    jobs and ledger continuity. Retry the same interrupted job ID if testing
    reconciliation; it must not execute a second time or move to another executor.
14. Follow OFFLINE_LAN_TEST.md with separately granted messaging and another
    bounded model grant. Keep all online/offline/restored exports separately.

A PHYSICAL label is an operator attestation. IP classification alone is not
physical proof: private addresses can traverse VPNs or routed private networks.
Record the topology and WAN manipulation independently. Separate-machine and
WAN-condition scenarios stay PARTIAL pending that corroboration. Restart and
platform/security external gates are intentionally not inferred from one run.
The UI never converts these records into a production-ready badge.

Invitation safety: qualification host requires the intended recipient's peer ID;
qualification join requires the host's public-key fingerprint. Invitations last
10 minutes and are one-use. SHA-256 is shown in full, not a custom shortened word
scheme. An already stolen invitation cannot be used by a different recipient.
For manual address correction, use a full literal-IP multiaddress pinned with
`/p2p/PEER_ID`, e.g. `porch peer connect /ip4/192.168.1.10/tcp/7332/p2p/PEER_ID`.
Choose an address actually observed on the intended host, not a virtual adapter.
