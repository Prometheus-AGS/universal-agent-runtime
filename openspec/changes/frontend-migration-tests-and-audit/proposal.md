## Why

The migration touches ~17 views, 11 stores, 13 entity types, and 5 new realtime topics. The "no stale data anywhere" contract is only as strong as our ability to detect regressions. We need both an automated contract test that catches future breakage and a human-readable audit doc that lists every old fetch site → new entity hook so reviewers can spot-check coverage.

## What Changes

### Automated test

- Vitest + RTL contract test: two components both mount `useEntity("provider", "p1")` against a synthetic graph + mocked `EventSource`. Emit one SSE `update` event → assert both components re-render with the new value within the same tick.
- A second test for `useEntityList`: emit `create`/`delete` events → list rows mutate accordingly.

### Audit doc

- `docs/migration-stale-data-audit.md` — one table per entity type. Columns:
  - **Old fetch site** (file:line)
  - **Old store** (file)
  - **New consumer** (file:line, `useEntity*` call)
  - **Status** (migrated / out-of-scope / pending)
- Sweep `git grep "fetch\\(" frontend/src/` — every remaining fetch must be either inside `services/` or explicitly listed as "deliberately not graph-backed" with a reason.

### Docs

- README architecture section updated to describe the new data-flow contract.
- `CLAUDE.md` updated with the four invariants:
  1. Components → hooks → graph (never components → fetch directly).
  2. Mutations go through `useEntityCRUD`; SSE reconciles authoritative state.
  3. Each entity has at most one canonical fetcher under `services/entities/`.
  4. Realtime adapters live in `lib/realtime/`.

### Cleanup

- Remove the dev-only diagnostic listener from `bootstrap-entity-engine-and-realtime` (#1) now that the system is verified.
- Flip `VITE_ENTITY_MGMT_CHAT_RUNTIME` default to `true` in all builds; remove the flag in a follow-up.

## Acceptance

- Contract test passes; failing test reproduces a stale-data scenario.
- Audit doc exists and shows zero "pending" rows.
- README + `CLAUDE.md` reference the new contract.
- `git grep -nE "use(Provider|Agent|Model|Skill|Setting|Knowledge|Memory|AuthKeys|CompilerSessions|ToolsDiscovery|McpHealth)(Admin)?Store" frontend/src` returns zero hits.
