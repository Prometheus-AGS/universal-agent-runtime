## Why

UAR still resolves its entity-management runtime from the checked-out `3.0.0-rc.1` workspace even though `3.0.2` is the published release. The application must consume the exact released package and its compatible core peer before the session configuration repair is built against it.

## What Changes

- Replace the UAR frontend's `workspace:*` entity-management dependency with exact registry version `3.0.2`.
- Resolve `@prometheus-ags/entity-graph-core` at `3.0.2` as the package's singleton peer and reconcile both UAR lockfiles without changing the vendored workspace sources.
- Prove that application and test code still import entity-management symbols only through `frontend/src/platform/entities`.
- Record the published tarball integrity and resolved graph so a local workspace package cannot silently substitute for the release.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-architecture-boundaries`: Require the UAR product to resolve its entity-management runtime from the exact supported registry release while retaining one facade and one compatible core singleton.

## Impact

- Affects `frontend/package.json`, `frontend/pnpm-lock.yaml`, and the root `pnpm-lock.yaml`.
- Does not edit `frontend/packages/prometheus-entity-management/**`, the operator-owned `versions.toml`, backend behavior, provider compatibility, or realtime semantics.
- KBD tracks this as the first UAR change because the following changes compile and verify against the released package.
