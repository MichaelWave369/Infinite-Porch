# Infinite House × Infinite Porch × Infinite Commons — phase 0 contract

Status: DESIGN + PUBLIC-SAFE VISUAL PROTOTYPE; no live cross-repository session or multiplayer is claimed.
Date: 2026-10-16. Applies to Infinite Porch 0.1.2 candidate and the existing Infinite Commons experience.

## Three independent pieces, one experience

1. **Infinite Porch (this repo):** Local owner authority, device identities, explicitly trusted peers, bounded grants, encrypted transport, local/private storage, messaging, inference, receipts. Existing Rust daemon, operator dashboard and SDK remain authoritative. No application visual can bypass their controls.
2. **Infinite House (new optional UI in apps/house):** An original nostalgic 1980s point-and-click home, bedroom, library/den, living room, kitchen, basement arcade, garage, attic, Hall of Brothers and back porch. The UI provides presentation and *requests*, not resource authority.
3. **Infinite Commons (separate repo, MichaelWave369/Infinite-Commons):** Existing Midnight Sierra/React/Three.js public experience with the optional Shasta Campus. Commons retains its own source, world, Python artifact handoff and review gates. The campus lives *inside* Commons, not as a substitute for Commons.

**The visitor journey:** enter personal House -> explore private/public rooms -> visit the Hall of Brothers' illustrative doors -> step onto back porch -> explicitly open the public Commons demo. The public link is not a cross-node identity or visit session.

## Critical separation of identities

- A machine's **Porch peer ID** authenticates a device; it is not automatically a human, resident, avatar, or AI.
- A **human household owner** selects the house's visibility and invited visitors. Authenticating/authorizing a human and linking that account to a device requires a later, separately reviewed design.
- An **AI connection** is a model/provider adapter, not a claim of independently embodied autonomous presence. An agent's visual door/world metadata cannot grant model invocation, file access or network routing.
- A **represented visitor** is a UI/session concept only until strong binding, admission and owner permission are implemented and verified. No inferred consent.
- The Hall of Brothers is an **owner-curated roster of worlds and connections**, not a model-provider directory granting access to every node.

## Phase 0 (this PR) acceptance

- Entirely additive frontend: no Rust daemon, existing desktop operator UI, SDK, Commons repo, grants or protocol changes.
- Static React/Vite house starts locally. Ten navigable rooms and contextual hotspots, an illustrative Hall of Brothers, and a back-porch *external demo link*.
- Room manifest has unique IDs, valid exits and public-only destinations, checked with built-in Node tests.
- No private token, model execution, credentials, LAN scanning, network API, hidden downloads, or remote asset fetching; no automatic audio.
- Original scenes use local CSS/DOM; future generated artwork must have provenance, size caps, attribution and asset review before publication.
- Keyboard-capable controls; reduced-motion preferences disable decorative animations.
- Existing backend acceptance and qualification are unchanged. This frontend passing tests cannot promote Porch 0.1.2 to production.

## Next governed milestones (separate PRs)

### P1 — Visual polish and room authenticity
Illustration overlays per room (rain, CRT, lamps), keyboard/semantic navigation, offline/reduced-motion/performance tests, original art assets, responsive Quest-friendly fallback. Make a scene look delightful before building the whole world at once.

### P2 — Persistent house and hall *metadata*
Versioned room/door/kingdom manifests; local import/export, user-approved edits and original-file preservation. No model credentials inside manifests. Offline doors remain visible without impersonating unavailable AIs.

### P3 — Scoped local Porch adapter
A separately issued `client.issue` credential for allowlisted *read-only* observations and explicitly chosen actions using the existing SDK and loopback API. Do not ask for/persist the operator token in a published site. A local companion/host process may be needed; design before implementation. Never expose localhost control to public Pages or casually rely on permissive CORS.

### P4 — Commons transition contract
Align named destination IDs with Commons' existing semantic IDs (e.g. `commons.arrival`). Navigation intent is untrusted presentation data; user clicks initiate travel. No auth codes, private identifiers, bearer tokens or prompts in URLs. Return-home UX may use local state. Preserve Midnight Sierra's painted mode and existing Shasta campus.

### P5 — Visits and shared spaces
Define household/avatar identities, invitation grants, guest room allowlists, join/leave and AI-to-AI requests with explicit operator approval; test with two physical machines, LAN-off WAN, revocation and replay resistance before describing it as live multiplayer. Commons currently has no production multiplayer/federation.

### P6 — Publishable static shell
Separate public preview from private local access. Static Pages may host only art and safe public metadata. Qualify accessibility, mobile, performance, deployment and provenance. Do not claim live interconnected houses or deployed Commons site without receipts.

## Invariants

- **Ledger above ego:** never label a screenshot, fixture, offline preview, or mocked AI response as a verified visitor/event.
- **Private by default:** rendering a door or discovering a provider must never imply authorization.
- **Independent roots:** Infinite Porch and Infinite Commons both remain useful when the other is offline.
- **Preserve originals:** original 38k-line Python `infinite_porch_FINAL.py` is a read-only heritage/reference artifact, not an executable dependency or material to overwrite.
- **No ghost revocation:** cached public exported data cannot be forcibly recalled from parties who saved a copy.
- **Human-directed travel:** opening a door is a deliberate action; spontaneous autonomous agent visitation is not enabled.

## Phase 0 run

```sh
cd apps/house
npm install
npm test
npm run build
npm run dev
```
Open Vite's localhost URL; these commands do not start a Porch node or a Commons daemon.
