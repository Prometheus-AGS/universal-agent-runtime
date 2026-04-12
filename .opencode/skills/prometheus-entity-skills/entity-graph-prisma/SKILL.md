---
name: entity-graph-prisma
description: >
  Analyze schema.prisma and integrate Prisma-backed APIs with @prometheus-ags/prometheus-entity-management:
  TypeScript entity shapes, registerSchema relation graphs, createPrismaEntityConfig for REST list/detail/CRUD,
  toPrismaWhere/toPrismaOrderBy from FilterSpec, Next.js App Router CRUD routes, and migration from ad-hoc
  Prisma calls to the library’s adapter patterns. Optional prisma-entity-graph-generator for codegen.
---

# Entity Graph — Prisma

This skill connects **Prisma ORM** (server-side) to the **client entity graph** via REST (or similar HTTP) APIs that accept Prisma-shaped `where` / `orderBy` query parameters — the pattern implemented in `createPrismaEntityConfig` (`src/adapters/prisma.ts`).

## When to use

- Greenfield: Prisma schema exists; you need list/detail/CRUD hooks on the client aligned with `useEntity`, `useEntityList`, `useEntityView`, `useEntityCRUD`.
- Brownfield: Replace manual `fetch` + local React state with graph-backed hooks while keeping Prisma on the server.
- Codegen: Introduce or configure **prisma-entity-graph-generator** (external package) to emit descriptors/config from DMMF.

## Library anchor APIs

| Export | Role |
|--------|------|
| `createPrismaEntityConfig` | Builds `entity`, `list`, `crud`, and `schemas()` for Prisma-style JSON query params |
| `toPrismaWhere` / `toPrismaOrderBy` | Compile `FilterSpec` / `SortSpec` → Prisma filter objects (`src/view/prisma-compile.ts`) |
| `registerSchema` | Register relation graph for cascade invalidation (`src/crud/relations.ts`) |

## Sub-skills

| Command | Purpose |
|---------|---------|
| `/entity-prisma-setup` | End-to-end: parse schema → types → config → register schemas |
| `/entity-prisma-generator` | Install/configure prisma-entity-graph-generator |
| `/entity-prisma-api` | Next.js Route Handlers using `PrismaClient` + `where`/`orderBy` from query |
| `/entity-prisma-migrate` | Refactor manual Prisma usage → `createPrismaEntityConfig` |

## PMPO

Use `prompts/specify.md` → `plan.md` → `execute.md` → `reflect.md` → `persist.md`.

## Critical rule

**Never import `@prisma/client` in client-side React bundles.** Prisma runs on the server (Route Handlers, Server Actions, Node API). The browser uses hooks + HTTP only.

## Package manager

Use **pnpm** in this monorepo.
