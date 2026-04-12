# prisma-entity-graph-generator (conceptual guide)

The package name **prisma-entity-graph-generator** is a **pluggable Prisma generator** pattern (similar to `prisma-client-js`). This repository ships the **runtime** adapter `createPrismaEntityConfig`; a separate generator package (if you author or install one) can emit:

- `EntityType` constants
- `createPrismaEntityConfig({...})` stubs per model
- `registerSchema` bootstrap snippets
- Zod schemas for `where` allowlisting

## prisma schema block (illustrative)

```prisma
generator entityGraph {
  provider = "prisma-entity-graph-generator"
  output   = "../src/generated/entity-graph"
}
```

## Inputs

- DMMF from `prisma generate`
- Optional config: API base path, id field overrides, models to skip

## Outputs (suggested)

| File | Content |
|------|---------|
| `models.ts` | string union of entity types |
| `configs/*.ts` | `createPrismaEntityConfig` per model |
| `register-all.ts` | single `registerSchemas()` call |

## When to use

- Many models with identical CRUD surface
- Need strict sync between Prisma schema and client config

## When to skip

- Few models — hand-written config is clearer
- Heavily custom endpoints per model

## Integration with this monorepo

Point generator output at the **application** repo, not `@prometheus-ags/prometheus-entity-management` core `src/`, unless you are developing the generator itself.

## Verification

After `prisma generate`:

- `pnpm exec tsc --noEmit`
- Diff review: ensure relations match Prisma `@@index` / FK constraints
