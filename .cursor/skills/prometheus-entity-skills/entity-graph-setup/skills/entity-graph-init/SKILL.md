---
name: entity-graph-init
description: "Bootstrap @prometheus-ags/prometheus-entity-management in a React app—install dependency, configureEngine, optional schema registration stubs, and a minimal useEntity/useEntityList proof while enforcing Components → Hooks → Graph layering."
---

# entity-graph-init

First-time **initialization** of @prometheus-ags/prometheus-entity-management in a **React** (Vite or Next.js) codebase.

## When to use

- The app has **no** graph integration yet; you need a **thin vertical proof** (one list + one detail).
- You want the **correct** `configureEngine` placement and import paths before broader migration.

## Prerequisites

- Completed **`setup_spec`** from `entity-graph-setup/prompts/specify.md` (or equivalent context).
- Package manager identified from lockfiles.

## Workflow

1. **Install** `@prometheus-ags/prometheus-entity-management` (or path alias in this monorepo).
2. **Create** `configureEngine` module using the project’s HTTP client for auth/base URL.
3. **Call** `configureEngine` once at app bootstrap (client entry or `use client` provider).
4. **Add** `registerSchema` stubs for entities touched in the proof (relations optional).
5. **Wire** one screen with `useEntityList` + one with `useEntity` (or Suspense variants if the app already uses Suspense boundaries).
6. **Verify** with `reflect` checklist from parent skill `prompts/reflect.md`.

## Deliverables

- `engine.ts` (or equivalent) with typed `fetchEntity` / `fetchList`.
- `schemas.ts` with `registerSchema` calls.
- One migrated route or component **without** `useGraphStore` in `.tsx` components.

## References

- `prometheus-entity-skills/_shared/references/library-api.md` — Engine + hooks entry points.
- `prometheus-entity-skills/_shared/references/architecture-rules.md` — Layering.
- Parent: `prometheus-entity-skills/entity-graph-setup/SKILL.md`

## Constraints

- Do not remove legacy data code in this sub-skill—**init runs parallel** unless the user explicitly requests replacement.
