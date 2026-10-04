# REQUIRED and FUTURE after the 0.1.1 engineering candidate

## REQUIRED before physical/production qualification

| Gate | Evidence still required |
| --- | --- |
| Two independently owned physical PCs | Recipient-bound invitations, human fingerprint comparison, real LAN authenticated path |
| Real Ollama weights | Advertisement without authority, bounded grant, actual remote inference, signed digests/provider/grant receipt |
| Revocation and recovery on physical machines | Refusal after revocation, node restart, preserved identity/trust/membership/ledger, interrupted-job reconciliation |
| Internet-off LAN | Human WAN disconnection with LAN preserved; messaging, fresh discovery, model job, receipts and clean restoration |
| Windows native readiness | MSVC build/runtime, paths, state ACLs, firewall, PowerShell helpers, install/upgrade/uninstall checks |
| macOS if claiming support | Native build/runtime, permissions/firewall, package behavior; signing/notarization before production distribution |
| Independent security review | Physical-network attacks, fuzzing, sustained hostile load, disk-full/power-loss recovery, dependency/license review |
| Hardware/provider budgets | Measured CPU/RAM/VRAM/bandwidth isolation and cancellation behavior; the current limits bound requests, not GPU isolation |
| Official release provenance | Publisher signing and reviewed source/build provenance; current portable archives are unsigned |
| Durable operator recovery | Protected whole-state/vault backup, external ledger checkpoints and reviewed UNCERTAIN-job procedures |

Implemented local gates and their executed evidence are listed in RELEASE_GATES.md
and receipts/qualification/command-gates.json. CI configuration is not CI execution.
Physical procedures and six-state evidence bundles are provided; external evidence
cannot be inferred from LOOPBACK or SIMULATED results.

## FUTURE, outside this qualification cycle

- Public federation, private DHT, optional public rendezvous/relay and NAT traversal.
- Mesh/radio/Wi-Fi Direct drivers and independent network experiments.
- WASM marketplaces, extra provider/plugin engines and arbitrary execution modes.
- Model tensor sharding, GPU pooling, ensemble/pipeline coordination.
- Native desktop shells, mobile apps, browser extensions and broader integrations.
- Autonomous administration, currencies, payments or blockchain systems.
- Group forward secrecy, anonymous communication and global identity directories.

These facilities are not prerequisites invented for the scoped 0.1.1 engineering
candidate. Unsupported federation/relay/WASM engines refuse explicitly.
FEDERATED_ALLOWED uses only currently eligible trusted-peer routes and reports
federation NOT_INSTALLED. Architecture remains the existing governed job spine.
Final production release ownership stays with the human operator after evidence.
