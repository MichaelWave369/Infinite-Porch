# Required release work and future extensions

The controlling goal is a production-capable first version. This delivered
candidate implements the local governed spine and records tested evidence; it
does not silently reinterpret that goal as already complete.

## REQUIRED before claiming that release

| Work | Acceptance evidence needed |
|---|---|
| Independent security review and dependency audit | Threat-driven review, fuzz corpus, advisory/license/SBOM receipt and repaired findings |
| Sustained hostile load and crash/disk-full testing | Bounded memory/CPU/disk/network under stress; no authority bypass; migration/recovery receipts |
| Real Ollama and hardware policies | Actual installed weights, token/time outcomes, changed/missing models, hardware cancellation and enforceable RAM/VRAM budgets |
| Physical LAN, QUIC and mDNS | Separate Windows/Linux/macOS PCs; discovery/pairing with outside Internet disconnected; verified encrypted paths |
| Cross-platform permissions and packaging | Windows ACLs/macOS secret protection; actual CI builds; portable launch/uninstall/upgrade tests |
| Native release signing | Sign Windows executable/installer with publisher certificate; macOS Developer ID, hardened runtime/notarization/stapling; Linux release signatures |
| WAN/NAT/relay capability in the stated first-version scope | Real quota-controlled, authorized forwarding; revocation; AutoNAT/DCUtR tests on independent NAT networks; no exit-proxy behavior |
| Operator recovery/backup UX | Protect all vault keys/state, inspect UNCERTAIN jobs, queued-message controls, full ledger export/checkpoints and migration backups |
| Complete status/metrics surfaces | Queued/active/uncertain job views, lifetime accounting, safe diagnostics and expired-record retention policies |
| Signed extensible app service registration | Bounded service schema, endpoint/capability allowlist, revocation, expiry and authorization tests; current manifests cover installed built-ins only |

These items are not described as future luxuries when they support the brief's
production promise. The repository contains portable build/packaging configuration;
external platform execution and code-signing credentials were not available in
this Linux workspace. The browser desktop UI is built; a native Tauri shell is
not implemented and is not required to pretend that an installer was produced.

## FUTURE, beyond this local job-level candidate

- Opt-in multi-Porch federation with distributed membership/conflict contracts.
- Private DHT, GossipSub channels and optional decentralized rendezvous after
  private discovery and authority are explicitly bounded.
- Community mesh, Wi-Fi Direct and radio drivers with real bandwidth/MTU limits
  and hardware/regulatory constraints.
- WASM sandbox, approved image providers and plugin engines with narrow effects.
- llama.cpp/vLLM/OpenAI-compatible provider adapters, measured performance history
  and deterministic multi-model proposals.
- ENSEMBLE, PIPELINE and genuinely distributed SHARDED_MODEL execution engines.
- Full PhiKernel, PhiVessel/Crane Fly, PhiBot and CommonLine voice/media integration.
- Multi-replica repair, erasure coding, tested backup policies and retained keys.
- Voluntary gifting/reciprocity/payment adapters; never implicit grants or
  speculative token economics.

Unsupported relay, federation and WASM operations refuse explicitly. Future
execution modes fail validation. Transport/SDK interfaces provide concrete seams,
not fabricated running features. Final release authority belongs to the human
operator after evidence is reviewed.
