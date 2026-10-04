# Two Linux PCs

Linux x86_64 portable binaries are built and locally exercised in this candidate.
Physical LAN, mDNS, actual weights and Internet-off operation remain UNVERIFIED.

On EACH machine extract its native portable archive. From that archive root:

```sh
chmod +x bin/porch bin/porch-node start.sh scripts/platform/install-user.sh
./scripts/platform/install-user.sh
PORCH="$HOME/.local/share/infinite-porch/0.1.2/bin/porch"
STATE="${XDG_STATE_HOME:-$HOME/.local/state}/infinite-porch"
"$PORCH" --data "$STATE" init --alias PC-A  # use PC-B on the other machine
"$HOME/.local/share/infinite-porch/0.1.2/bin/porch-node" --data "$STATE" --ui "$HOME/.local/share/infinite-porch/0.1.2/ui"
```

Leave the daemon running; define PORCH and STATE in a second control terminal.
Ordinary node use requires no root. If your firewall blocks the LAN, permit
TCP/UDP 7332 and UDP 5353 from the intended private subnet through your existing
firewall policy. Leave loopback control 7331 and local Ollama 11434 unexposed.

On B:

```sh
"$PORCH" --data "$STATE" identity show
```

Transfer B's public peer ID to A. On A:

```sh
B_PEER='PASTE_B_PEER_ID'
"$PORCH" --data "$STATE" qualify host 'Oak Street' --recipient "$B_PEER" --out invite.json
"$PORCH" --data "$STATE" identity show
```

Compare A's complete fingerprint directly on A. Copy invite.json to B, then:

```sh
A_PEER='PASTE_A_PEER_ID'
A_FINGERPRINT='PASTE_A_FINGERPRINT'
"$PORCH" --data "$STATE" qualify join invite.json --fingerprint "$A_FINGERPRINT"
"$PORCH" --data "$STATE" qualify status
```

On A, start local Ollama and choose an installed model (the example must appear
in scan). Do not enable an Ollama LAN listener:

```sh
"$PORCH" --data "$STATE" models scan
MODEL='qwen3:4b'
"$PORCH" --data "$STATE" share model "$MODEL"
"$PORCH" --data "$STATE" models verify "$MODEL"
"$PORCH" --data "$STATE" grant issue --recipient "$B_PEER" --capability model.inference --resource "$MODEL" --action run --ttl 1800 --max-calls 12 --calls-per-hour 12 --max-output-tokens 64 --out model-grant.json
"$PORCH" --data "$STATE" grant issue --recipient "$B_PEER" --capability message.direct --resource inbox --action send --ttl 1800 --max-calls 12 --calls-per-hour 12 --out message-grant.json
```

Copy the two grant files to B. On B:

```sh
MODEL='qwen3:4b'
"$PORCH" --data "$STATE" grant import model-grant.json
"$PORCH" --data "$STATE" grant import message-grant.json
"$PORCH" --data "$STATE" share refresh
"$PORCH" --data "$STATE" models list
"$PORCH" --data "$STATE" qualify run --peer "$A_PEER" --model "$MODEL" --message --environment PHYSICAL_LAN --separate-machines-confirmed
"$PORCH" --data "$STATE" qualify export evidence-baseline-B
"$PORCH" qualify validate evidence-baseline-B
```

A separately runs local qualification and exports evidence-baseline-A. Inspect
B's provider=ollama receipt, exact executor, route eligibility, digests, timing
and grant reference. Availability alone is not invocation proof.

Revoke on A using the nonce from model-grant.json, then prove a new refusal on B:

```sh
# A:
GRANT_NONCE=$(python3 -c "import json;print(json.load(open('model-grant.json'))['payload']['nonce'])")
"$PORCH" --data "$STATE" grant revoke "$GRANT_NONCE"
# B:
"$PORCH" --data "$STATE" qualify run --peer "$A_PEER" --model "$MODEL" --phase revoked --environment PHYSICAL_LAN --separate-machines-confirmed
"$PORCH" --data "$STATE" qualify export evidence-revoked-B
```

Optional storage: A voluntarily runs `porch share storage --limit 64MiB`, issues
B a blob.storage/vault/store grant with `--max-storage-bytes 8MiB`, and B imports
it. On B put a small nonsensitive file with `porch storage put probe.txt`, copy
its CID, then `porch storage replicate CID A_PEER_ID` and
`porch storage fetch CID A_PEER_ID --out retrieved.txt`. Compare the files.
Provider A sees ciphertext only. Pinning and replica verification do not promise
continuous repair or backup durability.

Stop/restart each node using the same state. Compare qualify status and doctor,
including active or UNCERTAIN jobs. Same-ID interrupted jobs stay pinned to the
original executor. Grant expiry and revocation survive restart. Collect new
exports and follow OFFLINE_LAN_TEST.md after issuing a new model grant.

Optional user service at the default install/state paths:

```sh
mkdir -p "$HOME/.config/systemd/user"
cp scripts/platform/infinite-porch.service "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now infinite-porch.service
journalctl --user -u infinite-porch.service
```

The template uses the default ~/.local/state path. Adjust it if XDG_STATE_HOME is
custom. Stop any foreground node first; the state lock rejects a second process.
This service template is not a physical systemd qualification receipt.
