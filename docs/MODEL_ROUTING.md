# Model routing

The default classification is LOCAL_ONLY. The origin's RPC guard independently
refuses any outbound job marked LOCAL_ONLY. TrustedPeers requires approved
direct trust and an imported provider grant. PorchAllowed additionally requires
the selected peer to share the active Porch. FederatedAllowed may use the same
already-approved direct subset; it creates no federation permission or discovery.
No untrusted or federated executor is available in this candidate.

Candidate filtering precedes ordering: privacy, active trust, Porch membership,
authority, capability match, fresh signed advertisement and advertised slot
capacity. Output includes every excluded candidate/reason and the chosen peer.
Eligible ordering is local first, then queue depth, observed latency and Peer ID.
Unknown latency uses a conservative ordering value, not a fabricated observation.
The provider rechecks real capacity and all limits before effect admission.

Current load uses active job slots. Queue depth is zero because excess work refuses
instead of entering a job queue. CPU parallelism is locally observed; model/storage
capacity is configured/provider-reported. GPU/RAM/VRAM allocation, throughput, cost,
model quality and performance histories are unknown and do not receive invented
scores. Hardware-aware routing and reservation enforcement are required before
claiming GPU/RAM sharing guarantees.

Ollama model digests are discovered when a model is configured. They identify a
provider report, not an attested measurement of weights used for each inference.
Only operator-selected models are advertised. Describing or recommending a route
does not grant a model permission to execute it.

The SDK's PhiVessel `propose` returns a pure request and `execution_authority:false`.
`execute` then enters the same scoped client/node boundary. PhiBot executes only
through that API. This is the extension seam for Crane Fly/councils/history routers,
not a running integration with those repositories or a model that authorizes itself.
