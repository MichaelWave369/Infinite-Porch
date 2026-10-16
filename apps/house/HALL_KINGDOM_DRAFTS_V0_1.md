# Hall of Brothers P2 · Local Kingdom Drafts

Status: **illustrative personal world layouts**, not connected models, authenticated visitors, or a production identity system.

In the **Hall of Brothers** inside `apps/house`, people can:
- Draw up personal doors by naming an AI collaborator, a kingdom and a short world description, with four allowlisted atmospheric palettes.
- Preview the *illustrative* imagined kingdom, edit existing doors, remove them with explicit confirmation, or leave the Hall empty.
- Save to this browser profile's localStorage, export a plain UTF-8 JSON backup and deliberately import a chosen JSON backup with strict validation. Reset requires confirmation.
- See three named example kingdoms when the browser has no stored draft. These examples explicitly say **UNCONNECTED**.
- Keep saved local door drafts when actual AI services change or disappear, because a door never requires a provider connection.

## Format and limits
`src/hallStore.mjs` provides a pure, separately testable `INFINITE_HOUSE_LOCAL_DRAFT` schema version 1. Up to 24 doors and 32 KiB. Known keys only, bounded strings, unique bounded IDs, allowlisted palettes, `connection:UNCONNECTED` only. Unknown keys including tokens, URLs, credential fields, fake LIVE statuses or overlarge files fail closed. Changing a sample door makes it an owner draft; deletion only updates the local draft.

An invalid existing storage record is **not overwritten automatically**. The user may download the raw backup, choose an explicit reset or import a separately validated replacement. Storage-denied contexts do not claim persistence. A local JSON export is a backup file, not a synchronized account or an encrypted vault.

## Trust
This public-safe frontend does not call the Porch API, read local node state, request a model connection, contact another user, upload exported files, or grant visitor permissions. Putting an AI's name in a nameplate is not evidence the AI is present or autonomously visiting.

**Do not enter private secrets, prompts, personal archives, or access tokens into the door-description field.** Browser localStorage is readable by anyone with access to that browser profile. Private Porch-controlled persistence and household access controls are future work.

The network core, existing Porch operator desktop, and separate Infinite Commons repo are unchanged.

## Validation

```sh
cd apps/house
npm install
npm test
npm run build
```

Review create/edit/preview/export/import/delete/reset with keyboard and mobile; disable localStorage, import invalid files, and verify no network traffic from the Hall itself. The canonical inter-repository boundary remains `docs/INFINITE_HOUSE_INTEGRATION_V0_1.md`.
