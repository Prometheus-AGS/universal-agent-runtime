# Plan — entity-graph-setup migration strategy

Inputs: `setup_spec` from **specify**, optional `entity_manifest` from **entity-graph-detect**.

## 1. Goals

- [ ] Single **normalized graph** as source of truth for migrated surfaces.
- [ ] **Lists store IDs**; no duplicate entity payloads in React local state.
- [ ] **Hooks** own orchestration; **engine** owns I/O.

## 2. Strangler ordering

Pick one primary pattern (document why):

| Pattern | When |
|---------|------|
| **Parallel** | Keep old hooks live; new routes use graph hooks behind `NEXT_PUBLIC_*` / runtime flag. |
| **Vertical slice** | One entity (e.g. `Project`) fully migrated end-to-end first. |
| **Read-first** | Migrate reads to graph; writes stay on old layer until fetchers stable. |

## 3. File layout (proposal)

Typical additions (adjust to repo conventions):

- `src/lib/entity-graph/engine.ts` or `lib/prometheus/engine.ts` — `configureEngine`, shared fetch wrapper.
- `src/lib/entity-graph/schemas.ts` — `registerSchema` for each `EntityType`.
- `src/lib/entity-graph/keys.ts` — list query key builders (single source of truth with CRUD `listKeyPrefix`).
- Optional `src/components/providers/EntityGraphProvider.tsx` — children-only side effects (init schemas, devtools guard).

## 4. Task list (ordered)

Produce **numbered** tasks with **acceptance criteria**:

1. Add dependency `@prometheus-ags/prometheus-entity-management` (version pin strategy: semver / workspace / path).
2. Implement `configureEngine` with existing API client (auth headers, base URL).
3. Register schemas for wave-1 entities.
4. Replace **one** list screen: `useEntityList` + join to graph at render.
5. Replace **one** detail path: `useEntity` or `useSuspenseEntity` if Suspense already adopted.
6. Document **rollback**: branch/tag, files to revert.

## 5. SSR / Next.js specifics

- If RSC fetches data: plan **hydration** (`upsertEntity` / batch upsert on client from server props) — reference `examples/nextjs-app` patterns.
- Avoid importing graph store in **Server Components**; pass serializable props across the boundary.

## 6. Open questions

List blockers that need human answers before **execute**.

## Output artifact

`migration_plan.md` containing: pattern choice, file paths, ordered tasks, rollback, SSR notes, open questions.
