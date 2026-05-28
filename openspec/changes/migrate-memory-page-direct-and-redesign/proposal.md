# migrate-memory-page-direct-and-redesign

## Why
Retire `memory-admin-store` (bridge); adopt terminal aesthetic. Preserve search affordances.

## What changes
- Reads via `useMemory()`; ad-hoc search endpoints stay direct (one-shot, not cached in graph).
- Mutations through `optimisticUpsert/Remove`.
- Delete `frontend/src/stores/memory-admin-store.ts`.
- Apply aesthetic; screenshot; audit flip.

## Impact
Preserves search UX; data path shortened to `SSE → graph → page`.
