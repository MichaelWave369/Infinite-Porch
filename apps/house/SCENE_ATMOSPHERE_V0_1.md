# Infinite House · Room Atmosphere v0.1 (P1 preview)

**Status: illustrative visual improvement only.** Does not connect to an AI, authorize travel, discover files or enable real multiplayer.

## What changed from P0

- Each of the ten rooms has its own original hand-authored SVG visual composition in `src/RoomScene.tsx`. The previous reused bedroom-like props for all rooms.
- Distinct visual anchors: bedroom CRT/bed/rain window; hallway framed doors; living-room couch/TV; kitchen refrigerator/cabinets; den library shelves; basement arcade cabinets; garage workbench; attic trunks; Hall of Brothers corridor; Shasta-facing back porch with a symbolic beacon.
- Animation is confined to small decorative details: starlight, CRT glow, little LEDs, lamp warmth, distant flame and rain.
- **Pause ambient motion** is an actual toggle. OS reduced-motion preference always wins. Nothing autoplays audio.
- The SVGs contain no external URLs, third-party artwork or runtime code. UI hotspots and explicit exit buttons remain accessible outside the drawings.

## Not yet part of this PR

- Integrating the prior AI-generated bedroom painting as a layered source image. That art is not bundled here; any binary scene artwork must be separately reviewed for license/owner approval, viewport fit and asset provenance.
- Advanced room-specific draggable objects, physics, WebXR, local files, model access, visiting other porches, and any live connection to Commons.
- A released experience: this is a reviewed React prototype and remains outside the Porch networking qualification claim.

## Acceptance

```sh
cd apps/house
npm install
npm test
npm run build
npm run dev
```

Inspect all ten visual scenes at large and small widths. Tab through the room directory, hotspots, door buttons and Commons link. Toggle ambient motion; test with an OS-level reduced-motion preference. Ensure the external link still opens only the public Commons preview and no local operator token is requested.

## Integration boundaries

No changes to `crates/*`, `apps/desktop`, `packages/sdk`, Rust locks, Porch protocol, or the separate `Infinite-Commons` repo. Read `docs/INFINITE_HOUSE_INTEGRATION_V0_1.md` for the authoritative integration contract. A drawn AI kingdom door is not an authenticated remote peer.
