# Plan — Map Prisma to entity graph

## Entity type naming

- Default: Prisma model name === graph `EntityType` (e.g. `Task`).
- When JSON uses different keys, document mapping table.

## Relation → registerSchema

For each `createPrismaEntityConfig`:

- `belongsTo`: FK on this model pointing to parent
- `hasMany`: inverse of child’s FK
- `manyToMany`: explicit join table model or implicit — align with library’s `ManyToManyRelation` shape in `src/crud/relations.ts`

## Endpoints per model

| Model | List URL | Detail URL | Notes |
|-------|----------|------------|-------|
| Task | GET /api/tasks | GET /api/tasks/:id | |

## API decoding

Plan how server parses:

```ts
const where = req.nextUrl.searchParams.get("where");
const parsedWhere = where ? JSON.parse(where) : {};
```

Add **zod** / **superstruct** validation before Prisma.

## Client modules

- One `createPrismaEntityConfig` export per root model (or grouped by bounded context).

## Generator decision

- If >10 models with repetitive CRUD, plan **prisma-entity-graph-generator**; else hand-write first model as template.

## Next

`execute.md`
