# AGENTS.md — entity-graph-prisma

## Mission

Map **Prisma models** to **@prometheus-ags/prometheus-entity-management** entity types, REST endpoints, and relation schemas — without leaking Prisma into the client.

## Workflow

1. Locate `schema.prisma` (often `prisma/schema.prisma`).
2. Use **schema-parser** agent output as source of truth for models, fields, enums, relation names.
3. Choose HTTP contract for list (`where`, `orderBy`, pagination) compatible with `listSearchParams` in `src/adapters/prisma.ts`.
4. Generate or hand-write `createPrismaEntityConfig` per aggregate root.
5. Call `registerSchema(...taskConfig.schemas())` during app bootstrap (client).

## Files to touch (typical Next.js app)

- `prisma/schema.prisma` — already exists
- `src/lib/prisma.ts` — PrismaClient singleton
- `app/api/<model>/route.ts` — list/create
- `app/api/<model>/[id]/route.ts` — get/patch/delete
- `src/entity-config/<model>.ts` — `createPrismaEntityConfig` + hooks re-export

## Verification

- `pnpm prisma validate`
- `pnpm run typecheck` for app
- Manual CRUD through example-style UI

## Orchestrators

Run `scripts/detect-orchestrators.sh` from repo root; attach outputs to KBD state if used.
