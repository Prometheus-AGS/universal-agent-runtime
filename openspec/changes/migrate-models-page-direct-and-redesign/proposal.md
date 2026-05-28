# migrate-models-page-direct-and-redesign

## Why
Retire `models-browse-store` (bridge pattern) and adopt the terminal aesthetic on the models page.

## What changes
- `models-page.tsx` reads via `useModels()` + `useEntity*` hooks; no `useModelsBrowse()` references.
- Mutations through `optimisticUpsert/Remove` from `@/lib/realtime/optimistic`.
- Delete `frontend/src/stores/models-browse-store.ts`.
- Apply terminal aesthetic per `docs/admin-aesthetic-spec.md`.
- Playwright snapshot at `screenshots/models-page.png`.
- Flip Model row in `docs/migration-stale-data-audit.md` to `direct`.

## Impact
Behavioural parity. UI changes are visual only.
