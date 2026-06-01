## Why

The Provider admin page shows which configured provider is currently the **default**. The backend exposes this as a single `default_id` field on the `/api/uar/providers` response. To read it via the entity graph (instead of a Zustand store cell), the SPA needs a graph-backed representation. A small singleton `ProviderMeta` entity (id `"current"`) is the least-invasive option — no new tables, no response-shape changes.

## What Changes

- Add `ProviderMetaEntity { id: "current"; default_id: string | null }` to `frontend/src/entities/types.ts`.
- Register schema in `frontend/src/entities/schemas.ts`: `registerSchema({ type: "ProviderMeta" });`.
- Extend `loadProvidersIntoGraph()` in `frontend/src/entities/fetchers/providers.ts` to also call `upsertEntity("ProviderMeta", "current", { id: "current", default_id })` after the catalog/configured fetch.
- Add `frontend/src/entities/hooks/use-provider-default.ts` exporting `useProviderDefault(): string | null` that reads the singleton from `useGraphStore`.

## Acceptance

- After page mount the graph contains exactly one `ProviderMeta` row keyed `"current"`.
- `useProviderDefault()` returns the current default id, or `null` when none.
- No page change yet — purely additive.
