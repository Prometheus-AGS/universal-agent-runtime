# add-memory-fetcher-and-hook

## Why
Memory page is bridged via `memory-admin-store`. Add the entity-graph scaffold first.

## What changes
- New `frontend/src/entities/fetchers/memory.ts` — `loadMemoryIntoGraph()`.
- New `frontend/src/entities/hooks/use-memory.ts` — `useMemory()` (list) + `useMemoryItem(id)` if needed.

## Impact
Additive scaffolding; no consumer rewires.
