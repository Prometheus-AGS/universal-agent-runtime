## Why

After change `providers-page-direct-reads` the page reads from the graph but still routes mutations through `useProvidersAdmin` → `useProvidersAdminStore`. To retire the store entirely, the three mutation paths (`configureProvider`, `setDefault`, `removeProvider`) must call the service directly and patch the graph instead of asking a Zustand action to do it.

This is the **mutation surgery** that lets the next change delete the store.

## What Changes

For each mutation:

- **`configureProvider`** (create) — calls `services/providers-api.ts::configureProvider` directly. On success, re-runs `loadProvidersIntoGraph()` so the new row + updated `ProviderMeta` reach the graph. **Non-optimistic** per the global rule for create paths.
- **`setDefault`** — calls `services/providers-api.ts::setDefaultProvider`. Optimistically `upsertEntity("ProviderMeta", "current", { default_id: id })` first; rollback to the previous `default_id` on failure. The SSE bridge will reconcile authoritative state regardless.
- **`removeProvider`** — captures the entity snapshot, calls `services/providers-api.ts::deleteProvider`, optimistically `removeEntity("Provider", id)`. On failure, re-upsert the snapshot.

Local component state replaces store-level fields:
- `saving: boolean` — local useState.
- `removing: string | null` — local useState (id being removed).
- `error: string | null` — local useState.
- `clearError()` — becomes `setError(null)`.

## Acceptance

- All three mutations work end-to-end.
- `setDefault` flips the default badge within one frame.
- Forcing a server rejection on `setDefault` rolls back the badge.
- `removeProvider` removes the row instantly; a forced failure restores it.
- `configureProvider` creates a new row that appears within ~200 ms (post-fetch + SSE) — non-optimistic is acceptable for the rare create.
- `useProvidersAdmin` is no longer referenced from `providers-page.tsx`.
