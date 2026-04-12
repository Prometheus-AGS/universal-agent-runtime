---
name: entity-graph-migrate
description: "Phased migration from TanStack Query, Apollo, Redux, or SWR to @prometheus-ags/prometheus-entity-management—strangler patterns, query-key mapping, SSR hydration, and rollback-aware PR slices."
---

# entity-graph-migrate

**Migration** sub-skill: replace or shadow an existing client cache with the **normalized entity graph** while controlling risk.

## When to use

- `setup_spec.stack_report` shows **TanStack Query**, **Apollo**, **Redux**, **SWR**, or heavy **raw fetch**.
- You have an approved **`migration_plan.md`** from `entity-graph-setup/prompts/plan.md`.

## Supported patterns

| Pattern | Summary |
|---------|---------|
| **Parallel** | Feature flag; graph path coexists with legacy hooks. |
| **Vertical slice** | One `EntityType` end-to-end per PR. |
| **Read-first** | Graph reads; legacy writes until mutation parity. |

Full detail: `prometheus-entity-skills/entity-graph-setup/references/migration-patterns.md`.

## Workflow

1. Map **legacy keys** → **`serializeKey`** list keys; centralize in `keys.ts`.
2. Implement **normalize** functions per endpoint so **entities** land in `entities[type][id]`.
3. Replace **one** surface at a time; keep **acceptance criteria** from the plan.
4. For **Next.js**, add **hydration** if RSC already fetched JSON (see parent references).
5. **Backup** legacy code per `entity-graph-setup/CLAUDE.md` before deletion PRs.

## Deliverables

- Diff with **no** component-level `useGraphStore`.
- Updated **`registerSchema`** when list invalidation must mirror old `queryClient.invalidateQueries` behavior.
- `reflect_report.md` snippet for the slice.

## References

- `prometheus-entity-skills/entity-graph-setup/agents/migrator.md` — execution role.
- `prometheus-entity-skills/entity-graph-setup/references/codebase-analysis.md` — discovery order.

## Handoff

- Deeper UI: **`entity-graph-crud`**
- GQL-specific: **`entity-graph-graphql`**
