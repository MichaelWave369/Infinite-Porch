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
