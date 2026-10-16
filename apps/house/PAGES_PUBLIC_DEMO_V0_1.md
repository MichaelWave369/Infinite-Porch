# Infinite House GitHub Pages public preview v0.1

## Purpose

A publicly accessible, static React/Vite **demonstration** of the ten illustrated Infinite House rooms and the local-only Hall of Brothers door-drafting interface, separate from the authenticated local Rust node.

**Target URL (not confirmed live by this document):** https://michaelwave369.github.io/Infinite-Porch/

**Project path:** `/Infinite-Porch/` with exact casing. The build uses `HOUSE_BASE_PATH=/Infinite-Porch/` (localhost defaults to `/`). This repo does not deploy to `/Infinite-Commons/`; that remains a distinct repository and runtime.

## Files and workflow

- `.github/workflows/deploy-infinite-house-pages.yml` builds on pushes to `main` modifying `apps/house/**` or this workflow, plus manual `workflow_dispatch`. Never deploy on PRs.
- The existing `house-preview.yml` tests every proposed change to the House, **including the Pages subpath build**.
- Build runs in `apps/house`: install dependencies, Node tests, TypeScript/Vite build, then `scripts/verify-pages.mjs`. Verifier checks generated asset paths, contents of the public output folder and absence of obvious operator-route code.
- Only `apps/house/dist` is uploaded as Pages artifact; GitHub Actions `pages:write` and `id-token:write` are scoped to the deployment job, never to the build.
- `github-pages` environment records the actual URL and deployment receipt. A green React build is not proof the public site is deployed.
- This release does not add any remote analytics, SDK calls, tokens, localhost daemon access, live visiting, WebXR, backend server or automatic model discovery.

## Owner enablement after PR merge

1. In the GitHub repo choose **Settings → Pages → Build and deployment → Source → GitHub Actions**. GitHub connector access here cannot change this setting for the owner. Do not assume it is already enabled.
2. Inspect **Actions → Deploy Infinite House to GitHub Pages** for a green Build AND Publish workflow. If no deployment appears, manually invoke **Run workflow** on `main`.
3. Open https://michaelwave369.github.io/Infinite-Porch/ in a fresh tab. Verify it renders without 404s for JS/CSS, the ten rooms change, the Hall samples are explicitly `UNCONNECTED`, and the optional Commons link remains a public preview.
4. Check a small mobile viewport, keyboard-only navigation, reduced-motion OS preference and pause toggle. Test Hall create/edit/export/import/delete/reset with NONPRIVATE sample text. Reload and verify saved browser-only draft, then clear the draft. Run import validation against intentionally malformed JSON.
5. If the deploy workflow is red or the URL 404s, **do not claim LIVE**; check `github-pages` environment status, Pages source selection, and Actions logs.

## Security and privacy

Public Pages is **not a private home**. The URL contains static code and public-safe data only. All decorations and example Hall doors are unconnected; no Porch operator credential, local model inventory, federated visitor session, private grant or file is embedded.

The Hall design editor uses browser `localStorage` with explicit JSON import/export. This is not encrypted storage or remote synchronization. GitHub Pages project paths share a browser origin under `michaelwave369.github.io`; scripts hosted on that origin can read data saved by other project paths. Do not store private information, access keys, or personally sensitive details in public Hall drafts. Exported JSON files are user-chosen local backups, not remotely recalled artifacts.

To create real personal houses, first design an owner-authenticated local host, session and origin boundary plus scoped Porch credentials and explicit guest consent. These are separate reviewed milestones, not implied by the static launch.

## Known limitation

The standalone `apps/house` package currently has no committed lockfile; CI uses `npm install` and top-level exact versions. A dedicated audited lockfile and `npm ci` should be added before claiming a reproducible, pinned public release. The public demo is an early candidate and must not be represented as production network qualification.
