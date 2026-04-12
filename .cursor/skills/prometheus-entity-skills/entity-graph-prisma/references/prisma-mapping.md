# Prisma → entity graph mapping

## Model → EntityType

| Prisma | Graph |
|--------|-------|
| `model Task` | `type: "Task"` in `createPrismaEntityConfig` |

Use consistent PascalCase strings across GraphQL (if any), REST, and `registerSchema`.

## Primary key

- Single-field `@id` → `idField: "id"` (default).
- Composite keys → use string serialization for graph `EntityId` (e.g. `` `${a}:${b}` ``) **only** if all hooks agree; prefer surrogate UUID in API responses when possible.

## Foreign keys

- Prisma scalar FK `projectId` on `Task` → `belongsTo` relation in config:

```typescript
relations: {
  project: { type: "Project", foreignKey: "projectId", relation: "belongsTo" },
}
```

## Lists and includes

- `createPrismaEntityConfig` builds `toPrismaInclude` hints for related loading in **some** code paths; API routes should explicitly `include` only what’s allowed.
- Client graph still stores **one row per entity**; avoid duplicating nested graphs unless you also upsert related types.

## Enums

- Serialize as string in JSON; filter UI can use enum column builders from `src/ui/columns.tsx`.

## Json columns

- Treat as opaque `unknown` in TS; avoid deep filtering through `toPrismaWhere` unless you add custom ops.

## Soft delete

- If `deletedAt` exists, default `findMany` should filter `deletedAt: null` server-side; client filters may still send status.
