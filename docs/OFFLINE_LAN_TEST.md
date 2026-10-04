# Internet-off LAN test — external evidence required

Never disable adapters, alter routing, or change firewall rules through this
qualification tool. A human controls the upstream WAN while preserving the LAN.
For Mikey's Starlink/Netgear setup, keep both PCs connected to the same local
router/switch and remove only the router's upstream Internet connection. Do not
unplug the LAN link or switch off the local Wi-Fi network. Verify the actual
wiring/topology before choosing the upstream cable or router control.

Before disconnecting: complete physical pairing, verify authenticated encrypted
non-loopback paths, share/verify a real model on A and import a bounded model
grant on B. A separately grants B message.direct/inbox/send. For messages both
ways, B grants A its own inbox too. Allow enough calls and expiry for all phases.
If a previous phase revoked the model grant, issue/import a NEW grant first.

On B, baseline:

```sh
porch qualify run --peer A_PEER_ID --model EXACT_MODEL --message --environment PHYSICAL --separate-machines-confirmed
porch qualify export evidence-online-B
porch qualify validate evidence-online-B
```

Record both PCs' actual IP addresses, local router/switch path, transport,
identity, model provider, grants and the observed WAN condition. Export A's
local checks separately. The direct destination should be LAN_DIRECT; inspect
advanced sockets. That label means a private/link-local IP destination and does
not by itself prove same LAN or absence of a VPN/WAN route.

Human disconnects only the upstream WAN. Demonstrate outside Internet unavailable
on both PCs and local addresses unchanged. Keep nodes running. On B:

```sh
porch share refresh
porch qualify run --peer A_PEER_ID --model EXACT_MODEL --message --phase offline --environment PHYSICAL --separate-machines-confirmed --wan-condition-confirmed
porch qualify export evidence-offline-B
porch qualify validate evidence-offline-B
```

Confirm messaging delivery, fresh signed resource/model discovery, actual remote
Ollama execution, matching grant/executor, signed receipt verification and a
local direct path. Repeat reciprocal messaging or local diagnostics on A and
export separately. Record the human WAN manipulation and independent observations
alongside the signed node bundles; this operator attestation is PARTIAL by itself.

Reconnect upstream WAN without changing the LAN. On B:

```sh
porch qualify run --peer A_PEER_ID --model EXACT_MODEL --message --phase restored --environment PHYSICAL --separate-machines-confirmed --wan-condition-confirmed
porch qualify export evidence-restored-B
porch qualify validate evidence-restored-B
```

Compare identities, Porch, trust, grants and ledger continuity across phases.
Capture any failure instead of replacing it with a pass. LOOPBACK/SIMULATED or
UNKNOWN/RELAY/WAN_DIRECT paths cannot prove offline LAN independence. No outside
Internet probe or untrusted URL is sent by the qualification command.
