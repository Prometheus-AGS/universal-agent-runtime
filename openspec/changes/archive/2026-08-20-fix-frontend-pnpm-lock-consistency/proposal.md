## Why

The nested frontend workspace lock is stale against the committed
entity-management submodule manifests, so pnpm 11.15.0 rejects a clean frozen
install before screen certification can start. The root-workspace lock contract
does not cover this independently active pnpm execution root.

## What Changes

- Reconcile `frontend/pnpm-lock.yaml` with the committed frontend manifests and
  pinned entity-management manifests under pnpm 11.15.0.
- Preserve pre-existing common dependency resolutions that are not required by
  those manifest changes.
- Require every independently active pnpm workspace root to remain
  frozen-installable before source-bound certification evidence is minted.
- Record the stale-lock negative control, deterministic regeneration, exact
  delta classification, clean frozen install, and focused frontend checks.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `frontend-build-tooling`: extend lock consistency from the repository-root
  pnpm workspace to every independently active nested pnpm workspace root.

## Impact

- Dependency metadata: `frontend/pnpm-lock.yaml` only; no manifest, root lock,
  submodule pin, product source, provider integration, or public API changes.
- Runtime UX and realtime state: no behavior changes; the corrected lock makes
  the already-implemented frontend and embedded SSE recovery reproducible in a
  clean checkout.
- KBD workflow: this child remains the active phase until independent review
  passes, then returns control to the parent `screen-by-screen-validation`
  change without changing the parent release denominator.
