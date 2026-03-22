# Infinite Porch

Infinite Porch is the private, chamber-oriented downstream consumer for the PHI369 Labs / Parallax Pantheon stack.

Infinite Porch is the private, chamber-oriented downstream consumer for Phi-Pantheon, built to load deterministic bootstrap contracts, channel-selected candidates, and Porch-safe artifact bundles from local handoff outputs.

It is designed to load deterministic local bootstrap contracts and artifact bundles produced by Phi-Pantheon, then resolve Porch-safe views such as:
- private directory/summaries
- private graph
- private chronicle
- session history
- delta records
- private protocol artifacts / step results
- candidate/channel/bootstrap state

Infinite Porch is currently in private build-phase integration with Phi-Pantheon.
It is local-first, deterministic, and does not depend on remote sync or server-side ingestion in this phase.

## Usage

Run the deterministic local bootstrap proof:

```bash
porch bootstrap-check
```

Optionally target a different repository root containing `pantheon_data`:

```bash
porch bootstrap-check --repo-root /path/to/repo
```
