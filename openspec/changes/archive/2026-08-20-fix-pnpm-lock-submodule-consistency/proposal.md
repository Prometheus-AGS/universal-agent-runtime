## Why

Advancing the workspace-owned entity-management submodule changed its manifest
without updating the root `pnpm-lock.yaml`, so a clean frozen install rejects
the committed candidate before browser certification can build it. The lock
must describe the exact committed workspace before source-bound certification
can resume.

## What Changes

- Adopt the already exercised root lock candidate that matches the pinned
  entity-management manifest while preserving its existing resolved versions.
- Require root workspace manifests, including workspace submodule manifests,
  to remain consistent with the committed root lock under frozen pnpm 11
  installation.
- Record the stale-lock failure as the negative control and prove the corrected
  lock is unchanged by both frozen lock-only and frozen installation commands.
- Return KBD control to `screen-by-screen-validation` only after strict
  OpenSpec validation and independent review.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-build-tooling`: Add frozen root-lock consistency for committed
  workspace manifests and workspace-submodule advances.

## Impact

- Changes only the root `pnpm-lock.yaml`, the `frontend-build-tooling` contract,
  and the child OpenSpec/KBD evidence and handoff artifacts.
- No manifest or dependency intent changes; no new dependency is introduced.
- Runtime UX, provider compatibility, API behavior, and realtime state are
  unchanged. This repair only makes the already planned frontend/runtime build
  graph reproducible.
- KBD child state must be completed and returned to the parent before the
  immutable screen certification run is restarted.
