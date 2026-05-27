## Why

The core promise of "no stale data anywhere" is that **two components reading the same entity both re-render when the graph mutates**. This is the contract every direct-`useEntity*` consumer depends on; without an automated test, a future refactor of the entity-graph subscription model could silently break every migrated page.

## What Changes

Author `frontend/src/lib/realtime/__tests__/graph-propagation.test.tsx`:

- Render two sibling React components, each reading the same `Provider:p1` entity via `useGraphStore`.
- Call `useGraphStore.getState().upsertEntity("Provider", "p1", { id: "p1", display_name: "Alpha" })` inside `act(...)`.
- Assert both rendered nodes display `"Alpha"`.
- Repeat for an `update` (different `display_name`); both nodes update.
- Repeat for a `removeEntity`; both nodes fall back to the empty-state value.

Test is independent of `useEntity` higher-level hooks — it locks the lowest-level subscription contract.

## Acceptance

- `pnpm --filter ./frontend test` green.
- Toggling the `useGraphStore.subscribe`-based render path off (manual test) makes the test fail — proving it's load-bearing.
