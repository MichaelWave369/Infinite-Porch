# Human-controlled upstream WAN phase

Before revoking the model grant, retain baseline evidence. Record the current peer socket/path on both PCs. A human removes **only the router's upstream Internet link**, keeping both PCs on the same powered Ethernet/Wi-Fi LAN. Do not disable PC adapters or modify router settings. If the connection cannot be isolated safely, record NOT_RUN.

On PC-B:

```powershell
.\Porch-FieldLab.ps1 offline -Peer '<PC-A-ID>' -Model '<exact-installed-tag>' -Session field-001 -SeparateMachinesConfirmed -WanConditionConfirmed -Out .\offline-B
```

The helper records pre/offline peer paths, local socket facts, public message/inference receipts and two external TCP reachability observations. It does not send prompts outside Porch. A failed external socket probe is inconclusive by itself: a firewall, destination outage or network policy can produce the same result. Attestation is also inconclusive alone. Preserve router/link observation, matching timestamps and evidence from both peers. PC-A exports a snapshot during this phase and records its own upstream observation. Restore the upstream link by hand and retain another run. The machine report keeps WAN-off UNVERIFIED pending review of these combined facts; no automatic physical WAN/traversal qualification occurs.
