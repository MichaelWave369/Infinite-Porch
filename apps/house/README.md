# Infinite House (phase 0)

A self-contained, **illustrative** React/Vite experience for the private-home metaphor of Infinite Porch. Original 1980s-inspired painted-with-CSS stage, ten rooms, room hotspots, the Hall of Brothers, and a back porch that links to the **public Commons preview**, if deployed. All rooms/doors are sample presentation only.

**No runtime networking, no real AI identities, no permission or login, no multiplayer.** The real Porch 0.1.2 daemon and its authenticated operator dashboard are unchanged. Never paste operator or provider API credentials into this demo.

## Try it

Use Node >=22.

```sh
cd apps/house
npm install
npm test
npm run build
npm run dev
```

The app deliberately has its own package.json **outside the root's existing npm workspaces**, so it cannot silently alter the qualified `apps/desktop` package or root lockfile. Its dependency lockfile and dedicated CI are follow-up prerequisites before a tagged reproducible release; do not label this as a release build.

## Scope

- Nav: bedroom, hallway, living room, kitchen, upstairs den, basement arcade, garage, attic, Hall of Brothers, back porch.
- Pin/hotspot descriptions are informative **not real actions**. Door buttons inspect example world metadata, they do not invoke an LLM.
- The back-porch link is a normal new-tab external URL to the existing public Commons GitHub Pages *target*. Deployment and identity handoff are not guaranteed.
- Animation is CSS-only, decorative, automatically disabled under `prefers-reduced-motion`.
- Full keyboard navigation through native buttons/links.
- All room metadata lives in `src/rooms.json`; `tests/manifest.test.mjs` verifies its invariants.

Read [the cross-repository plan](../../docs/INFINITE_HOUSE_INTEGRATION_V0_1.md) before attaching any Porch API.

## Public GitHub Pages preview (proposed)

PR #5 adds a **static public React showcase** at:

`https://michaelwave369.github.io/Infinite-Porch/`

This is a **deployment target, not proof the site is live**. After review and merge, the repository owner must confirm **Settings → Pages → Build and deployment → Source: GitHub Actions** and check the `Deploy Infinite House to GitHub Pages` workflow. The workflow has a `workflow_dispatch` option if a manual retry is required. The only uploaded artifact is `apps/house/dist`. No daemon, `.env`, credential file, private workspace, or Rust binary is published.

```sh
cd apps/house
npm install
npm test
HOUSE_BASE_PATH=/Infinite-Porch/ npm run build
node scripts/verify-pages.mjs
```

On Windows PowerShell, use `$env:HOUSE_BASE_PATH="/Infinite-Porch/"` before running the build. Unset it or use `/` for ordinary localhost development.

**Important storage warning:** GitHub Pages project sites under `michaelwave369.github.io` share a browser origin. Other pages and scripts served from that same origin can access this preview's `localStorage` Hall drafts. These local-only decorative drafts are **not private/encrypted Porch data**, are not synchronized across devices, and are not uploaded by this application. **Never enter passwords, keys, confidential transcripts, personal details or other secrets.** Use the JSON export for a deliberate personal backup only. A real personal Porch will use separately verified scoped access.

Public demo verification and owner setup: [PAGES_PUBLIC_DEMO_V0_1.md](PAGES_PUBLIC_DEMO_V0_1.md).
