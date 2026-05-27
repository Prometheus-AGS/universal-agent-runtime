# migrate-compiler-page-direct-and-redesign

## Why
Compiler page is 59 LOC — quickest win. Retire `compiler-sessions-store`; establish reusable terminal-aesthetic empty-state and loading components.

## What changes
- Page reads via `useEntityList<CompilerSessionEntity>("CompilerSession")` (or new `useCompilerSessions` if needed).
- Retire `frontend/src/stores/compiler-sessions-store.ts`.
- Author `frontend/src/components/admin/empty-frame.tsx` (ASCII frame, reusable) + `loading-cursor.tsx` (flicker `▍`) here; consumed by later migrations.
- Apply aesthetic. Screenshot. Audit flip.

## Impact
Establishes the shared aesthetic components used by changes 6–11.
