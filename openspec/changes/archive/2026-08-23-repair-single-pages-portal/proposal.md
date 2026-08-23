## Why

UAR currently has two workflows competing to publish GitHub Pages, while the
intended Docusaurus workflow fails its own package-manager contract and calls a
nonexistent TypeScript documentation script. The result is a live Pages site
that can serve the SDK reference instead of the product portal and can silently
substitute placeholder output for a missing generated reference.

## What Changes

- Make `website/package-lock.json` and npm the only package-manager contract for
  installing and building the Docusaurus site.
- Generate Rust and TypeScript references through their real pinned commands,
  fail when either artifact is absent, and stage both beneath the completed
  Docusaurus build without placeholder fallbacks.
- Remove the standalone TypeScript SDK Pages publisher and retain
  `.github/workflows/docs.yml` as the sole Pages owner.
- Keep GitHub Actions limited to build, package, deploy, and deployed-route
  validation; all routine development checks remain local.
- Add local structural controls for the staging and workflow contract before
  final phase-level site certification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Make the accepted portal artifact deterministic: one npm
  site build, real Rust and TypeScript reference artifacts, no placeholders,
  one Pages publisher, and deployment-only workflow behavior.

## Impact

- **Documentation build:** `website/package.json`, its pinned lockfile contract,
  and documentation-only staging scripts.
- **GitHub Pages:** `.github/workflows/docs.yml` becomes the only publisher;
  `.github/workflows/typescript-sdk-docs.yml` is removed.
- **Dependencies and APIs:** No runtime dependency or public API changes. The
  existing Docusaurus and TypeDoc pins remain authoritative.
- **Runtime UX:** No React application or runtime behavior changes; the public
  documentation entrypoint becomes deterministic.
- **Provider compatibility:** No provider, model, inference, or credential
  behavior changes.
- **Realtime state:** No SSE, AG-UI, A2UI, or entity-state behavior changes.
- **KBD:** The registered `repair-single-pages-portal` change advances only
  after strict OpenSpec and bounded local evidence pass.
