# PhiOS / PhiVessel / PhiBot integration seam

Read-only contracts inspected: PhiOS README and PHIOS_API_KEY_BOUNDARY_V0.1;
SuperPhiVessel README and GOVERNANCE; PhiBot README and ACCEPTANCE. No code was
copied, no repository was changed, and Infinite Porch requires none of them.

The TypeScript SDK provides independent observation/request adapters. PhiOS can
read approved peers, selected capabilities and model records using a scoped local
credential. PhiKernel must make its own human-authorized decision before asking
for a Porch effect. Porch then validates that app delegation, filters routing and
requires the resource owner's peer grant. A PhiOS role/key is not peer authority.

PhiVessel observes models and constructs a pure proposal with
`execution_authority:false`. Separate `execute` submits it through PorchClient.
Crane Fly/councils can select preferred_peer as a recommendation, while hard
privacy/trust/authority/capacity rules still determine eligibility. PhiBot's
adapter calls the same governed run API. Both receive signed receipts and can
link their record hashes into their own evidence.

These are implemented adapter seams with SDK tests. They are not a running
PhiKernel connection, full provider-catalog integration, secret exchange,
distributed agent swarm or inherited authority. Rich storage/messaging app
delegations and ecosystem end-to-end verification require explicit future contracts.
