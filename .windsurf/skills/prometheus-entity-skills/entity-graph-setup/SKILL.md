---
name: entity-graph-setup
description: "Analyze a React/Vite/Next.js codebase and introduce @prometheus-ags/prometheus-entity-management from scratch—detect legacy data layers, infer entity types, emit schema registrations, and produce phased migration plans without breaking architecture rules."
---

# entity-graph-setup

Analyze a target **React / Vite / Next.js** codebase and set up **@prometheus-ags/prometheus-entity-management** from scratch. Detects existing data layers (TanStack Query, Apollo Client, Redux, SWR, raw `fetch`), maps **entity types** from API responses or GraphQL schemas, generates **`registerSchema`** registrations and hook wiring, and produces **migration plans** that preserve behavior while moving canonical state into the normalized graph.

## When to use

- Greenfield integration of @prometheus-ags/prometheus-entity-management into an app.
- Incremental migration off **TanStack Query**, **Apollo Client**, **Redux**, **SWR**, or ad-hoc fetch logic.
- Inferring **entity types** and relations from REST routes, OpenAPI, Prisma models, or GraphQL SDL.
- Aligning a team on **Components → Hooks → Graph** before deeper CRUD/realtime work.

## Sub-skills (invoke explicitly)

| Route | Focus |
|-------|--------|
| **`entity-graph-init`** | First-time bootstrap: dependency install, `configureEngine`, provider/bootstrap file, minimal `useEntity` proof. |
| **`entity-graph-migrate`** | Side-by-side or phased replacement of an existing cache layer; feature flags, strangler patterns. |
| **`entity-graph-detect`** | Static scan of API routes, types, and hooks to emit `entity_manifest` + suggested `EntitySchema[]`. |

## Process (PMPO)

1. **Specify** — Interview: data sources, auth, SSR, ID shapes, must-not-break flows (`prompts/specify.md`).
2. **Plan** — Choose migration strategy, file layout, entity mapping, risk list (`prompts/plan.md`).
3. **Execute** — Generate install steps, code, and docs pointers (`prompts/execute.md`).
4. **Reflect** — Typecheck, smoke-test list/detail, confirm graph population (`prompts/reflect.md`).
5. **Persist** — Merge artifacts, update state, optional orchestrator handoff (`prompts/persist.md`).

## Orchestrator detection

From the **consuming workspace root**:

```bash
bash prometheus-entity-skills/entity-graph-setup/scripts/detect-orchestrators.sh
```

Emits JSON flags for **KBD** (`.kbd-orchestrator/project.json`), **Evolver** (`.evolver/`), **Refiner** (`.refiner/`), **OpenSpec** (`openspec/`). Route follow-up automation (plans, specs, refinements) when any flag is `true`.

## Shared references (library monorepo)

When this skill runs inside **@prometheus-ags/prometheus-entity-management**, prefer:

- `prometheus-entity-skills/_shared/references/library-api.md`
- `prometheus-entity-skills/_shared/references/architecture-rules.md`
- `prometheus-entity-skills/_shared/references/branding.md` (travisjames.ai tokens for examples)
- `prometheus-entity-skills/_shared/references/schemas/*.schema.json`

## Constraints

- **Never** bypass **Components → Hooks → Stores** layering when generating UI-facing code.
- **Package manager:** detect `pnpm` / `npm` / `yarn` from lockfiles; **prefer pnpm** when the repo already uses it (this monorepo is **pnpm-only**).
- Do **not** delete legacy data code without an explicit **backup branch** or patch file noted in the migration plan.

## Related skills in this package

- `entity-graph-crud` — screens and `useEntityCRUD`
- `entity-graph-graphql` — GQL client + hooks
- `entity-graph-prisma` — Prisma route compilers
- `entity-graph-realtime` — adapters + manager
