# Networking and partition behavior

rust-libp2p carries direct QUIC or TCP/Noise/Yamux sessions. Identify checks the
Porch protocol; Ping reports observed latency. Default peer listeners are TCP
7332 and UDP 7332. The privileged API stays at 127.0.0.1:7331. mDNS discovers LAN
addresses and dials only identities already approved locally. Manual pinned
multiaddresses and signed invitations work without any bootstrap directory.

The verified acceptance path is loopback TCP/Noise among three OS processes.
QUIC listeners are implemented, but listener existence does not prove a successful
QUIC session. Physical Ethernet/Wi-Fi, multicast discovery and public Internet
paths have not been field-tested. Direct routable Internet addresses can be
configured; automatic NAT traversal is not installed or qualified.

| Networking facility | Candidate state |
|---|---|
| TCP/Noise/Yamux, request-response | Implemented; actual peer path tested |
| QUIC | Implemented listener/transport; session qualification required |
| mDNS | Implemented for approved identities; physical LAN test required |
| Identify, Ping | Implemented; latency stays unknown until observed |
| Kademlia DHT | Dependency evaluated, no behavior enabled; no public records |
| GossipSub | Deferred; private channels currently use pairwise messages |
| AutoNAT, relay, DCUtR | Dependencies available; no active engine or WAN claim |
| Rendezvous/bootstrap, federation | Deferred; not a central authority substitute |
| Radio/Wi-Fi Direct/custom mesh | Real Transport trait boundary; no hardware driver |

Network queues allow 64 commands/outbound requests; receive execution concurrency
is 32. libp2p connection limits bound pending and established sessions (64 total
established, two per peer). These are admission bounds, not a measured bandwidth
shaper or protection against every handshake denial-of-service attack.

Cached resource advertisements expire after 60 seconds. Routing refreshes only
approved peers with bounded concurrency/timeouts. When one node disappears,
services on a still-connected pair continue. The acceptance harness needs no
external DNS, cloud or outside Internet; this establishes a LAN-like simulation,
not a five-house wireless field proof or universal outage survival guarantee.

Resumable encrypted uploads preserve durable offsets across provider restart.
The messaging outbox stores up to 64 deliveries and tries at most ten times per
message. Grants and clocks still apply while partitioned. Jobs are not automatically
rerouted after ambiguous execution. There is no conflict-prone global replicated
authority database: effect authority belongs to the resource owner.

The Transport trait exposes authenticated request-response and a frame limit.
Low-bandwidth drivers can refuse an unsupported frame size. Actual fragmentation,
MTU discovery, radio-specific cryptography/regulatory constraints and mesh routing
remain explicit future implementation work.
