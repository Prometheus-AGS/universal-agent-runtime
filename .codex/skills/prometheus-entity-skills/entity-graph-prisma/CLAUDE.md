# CLAUDE.md — Entity Graph + Prisma

## Architectural boundary

```
Browser: Components → Hooks → (fetch) → REST JSON
Server:  Route Handler → PrismaClient → database
```

- `createPrismaEntityConfig` is **client-side factory** that produces fetchers; those fetchers call **your** HTTP API, not Prisma directly.
- API must decode `where` / `orderBy` JSON query params into `Prisma.*Args` safely.

## createPrismaEntityConfig

Defined in `src/adapters/prisma.ts`.

- **`type`**: EntityType key in the graph (e.g. `"Task"`).
- **`endpoint`**: Base URL; list = `GET endpoint`, detail = `GET endpoint/:id`.
- **`idField`**: Primary key on normalized JSON (default `id`).
- **`relations`**: Drives `prismaRelationsToSchema` → `registerSchema` entries (`belongsTo`, `hasMany`, `manyToMany`).

Returns:

- `entity(id)` → `EntityQueryOptions` for `useEntity`
- `list(...)` → `ListQueryOptions` for `useEntityList` with Prisma query serialization
- `crud(...)` → partial `CRUDOptions` for `useEntityCRUD`
- `schemas()` → pass to `registerSchema`

## toPrismaWhere / toPrismaOrderBy

- Consume **view layer** `FilterSpec` / `SortSpec` (from `useEntityView` / CRUD), not raw URL strings.
- Unsupported filter ops (`between`, `custom`, …) are omitted — handle in API if needed.

## Server-side validation

- Never pass client `where` directly to Prisma without **allowlisting** fields and ops (security).
- Consider per-model max depth / banned relations.

## DMMF (optional codegen)

- `@prisma/internals` / `Prisma.dmmf` can drive generators in Node **build** scripts only.
- Do not ship DMMF to the browser.

## TypeScript entity shapes

For maximum flexibility with graph storage:

```typescript
export interface TaskEntity extends Record<string, unknown> {
  id: string;
  title: string;
  projectId: string;
  // …known fields
  // index signature satisfied via extends Record<string, unknown>
}
```

Avoid pretending unknown API fields don’t exist; the graph stores `Record<string, unknown>` per entity.

## Next.js App Router

- `PrismaClient` should be a **singleton** in development (`globalThis` pattern) to avoid connection exhaustion.
- Use dynamic route segments `[id]` for detail/update/delete.

## pnpm

All installs use `pnpm add`, `pnpm dlx`, etc.
