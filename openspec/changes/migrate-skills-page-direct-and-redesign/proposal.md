# migrate-skills-page-direct-and-redesign

## Why
`useSkills` already exists. Retire `skills-admin-store`; route the optimistic toggle through the shared helper; redesign.

## What changes
- Reads via `useSkills()` directly.
- Toggle/edit mutations go through `optimisticUpsert` (matches Provider/Agent playbook).
- Delete `frontend/src/stores/skills-admin-store.ts`.
- Apply aesthetic; screenshot; audit flip.

## Impact
Consistency with the rest of the migrated admin surface.
