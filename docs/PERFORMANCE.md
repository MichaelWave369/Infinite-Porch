# Local performance baseline

Single debug-build loopback sample from qualification-acceptance.json. These
values are observations, not service guarantees or physical LAN measurements.
The receipt microbenchmark is a separate local Rust debug example.

| Measurement | Observed |
| --- | --- |
| Startup A | 118.4 ms |
| Startup B | 116.0 ms |
| Startup C | 116.0 ms |
| Manual signed discovery | 41.2 ms |
| Pairing | 188.9 ms |
| Message round trip | 39.3 ms |
| Mock job round trip | 117.8 ms |
| Dispatch/receipt overhead | 94.8 ms |
| 8 MiB SHA-256 sample | 1685.6 MiB/s |
| 100 receipt signs | 25.9 ms |
| 100 receipt verifications | 1061.8 ms |

Pairing includes a deliberately rejected fingerprint and stolen-invite attempt.
Discovery is an explicit refresh, not mDNS convergence. Job overhead subtracts
the signed mock-provider duration from API round trip; it includes discovery,
routing and receipt transport. Hash throughput uses Python hashlib on a warm
8 MiB memory buffer; it measures hashing only, not encryption, disk or replication.

Receipt timing is for 100 domain-separated JSON assertions in the Rust debug
build. It does not establish production signature or model throughput. Raw
values and commands are preserved in receipt-performance.json and its log.

Idle RSS and CPU are UNVERIFIED: this managed runtime lacks compatible native
/proc measurements. Real LAN startup/discovery/latency, model throughput, GPU
load and sustained concurrency remain UNVERIFIED. Capture those on each real
PC during the external procedure rather than extrapolating these values.
