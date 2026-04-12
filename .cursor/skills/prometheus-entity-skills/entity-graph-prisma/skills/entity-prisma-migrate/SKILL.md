---
name: entity-prisma-migrate
description: >
  Slash command /entity-prisma-migrate — refactor existing manual Prisma + fetch + useState patterns
  to @prometheus-ags/prometheus-entity-management hooks driven by createPrismaEntityConfig, registerSchema, and
  useEntityView/useEntityCRUD while preserving API routes and auth.
---

# /entity-prisma-migrate

## Identify legacy patterns

- Components calling `fetch("/api/...")` directly
- `useEffect` + `useState` mirroring server rows
- Duplicate copies of the same record in multiple components

## Migration steps

1. Ensure API responses match adapter expectations (adjust server if needed).
2. Introduce `createPrismaEntityConfig` for one model at a time.
3. Replace list state with `useEntityList` or `useEntityCRUD` list path.
4. Replace detail state with `useEntity` or CRUD selected id + detail fetch.
5. Register schemas **before** routes render (`registerSchema`).
6. Remove redundant local state; keep only UI-only state (dialogs, form drafts per CRUD edit buffer rules).

## CRUD edit buffer

`useEntityCRUD` keeps edits in React state until save — do not move draft edits into graph `patches` unless using `applyOptimistic` intentionally.

## Validation

- After each model migration, run typecheck and smoke test create/update/delete.
- Verify cascade invalidation updates related lists when FKs change.

## Reference

- Library: `src/crud/useEntityCRUD.ts`, `src/crud/relations.ts`
