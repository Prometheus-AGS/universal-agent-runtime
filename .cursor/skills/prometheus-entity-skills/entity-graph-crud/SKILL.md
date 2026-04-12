---
name: entity-graph-crud
description: >
  Generate complete CRUD screens for any entity type using @prometheus-ags/prometheus-entity-management hooks and UI components.
  Analyzes entity shape, produces column definitions (TanStack Table + entityMeta), FieldDescriptor-based forms,
  EntityDetailSheet / EntityFormSheet wiring, and registerSchema + cascade invalidation—all strictly following
  Components → Hooks → Stores → APIs data flow.
---

# Entity Graph CRUD

## Purpose

Deliver production-ready CRUD surfaces (list, create, edit, detail, delete) for entities backed by **@prometheus-ags/prometheus-entity-management**. The graph is the single source of truth; lists hold **IDs only**; mutations flow through hooks and callbacks so every view stays globally reactive.

## When to use

- Building CRUD pages, admin panels, or internal tools for REST (or HTTP-shaped) APIs
- Scaffolding **table + side panel + form** flows aligned with `useEntityCRUD`
- Adding **`registerSchema`** and cascade rules so related lists and detail joins stay consistent after mutations
- Replacing ad-hoc TanStack Query or local `useState` rows with normalized graph reads

## Sub-skills (slash-style routes)

Invoke the focused playbook when scope is narrow:

| Route | Focus |
|-------|--------|
| **`/entity-crud-page`** | Full page: list + selection + detail + create/edit + delete |
| **`/entity-crud-form`** | `EntityFormSheet` / `EntityDetailSheet` + `FieldDescriptor[]` + buffer wiring |
| **`/entity-crud-table`** | `EntityTable` + `columns.tsx` helpers + filters/sort metadata aligned with `useEntityView` |
| **`/entity-crud-relations`** | `registerSchema`, `belongsTo` / `hasMany` / `manyToMany`, `listKeyPrefix`, invalidation keys |

Each route maps to `prometheus-entity-skills/entity-graph-crud/skills/<name>/SKILL.md`.

## Workflow (PMPO-style)

1. **`prompts/specify.md`** — Interview: entity shape, fields, relations, API surface, UI preferences → `entity_spec`.
2. **`prompts/plan.md`** — File map, hook wiring, component tree, column/form plans, schema registration.
3. **`prompts/execute.md`** — Generate code per **`CLAUDE.md`** (strict layering).
4. **`prompts/reflect.md`** — Architecture grep + `pnpm run typecheck` (+ example app scripts).
5. **`prompts/persist.md`** — Merge artifacts; optional `state-init.sh`; handoff notes.

## Orchestrator detection

From the **consuming workspace root** (paths assume the prometheus repo layout; adjust if vendored):

```bash
bash prometheus-entity-skills/entity-graph-crud/scripts/detect-orchestrators.sh
```

Emits JSON: `kbd`, `evolver`, `refiner`, `openspec` availability flags for follow-up automation.

## Key library APIs

| Area | Exports |
|------|---------|
| CRUD orchestration | `useEntityCRUD`, `CRUDOptions`, `CRUDState`, `CRUDMode`, `DirtyFields` |
| List + view | `useEntityView`, `ViewDescriptor`, `FilterSpec`, `SortSpec`, `toRestParams` |
| Relations | `registerSchema`, `getSchema`, `cascadeInvalidation`, `readRelations`, `EntitySchema` |
| UI | `EntityTable`, `EntityDetailSheet`, `EntityFormSheet`, `Sheet`, column builders, `SortHeader` |
| Single entity | `useEntity` (detail subscription inside `useEntityCRUD` when `detailFetch` provided) |

## References

- **`references/crud-patterns.md`** — Patterns A–E and anti-patterns
- **`references/ui-component-catalog.md`** — Props and composition cheat sheet
- **`CLAUDE.md`** — Non-negotiable layering for this skill
- **`AGENTS.md`** — Agent behavior, read order, quality gate

## Constraints

- **`pnpm`** only where the repo or consumer policy requires it (this monorepo: pnpm).
- **Components** must not import `useGraphStore` or call `fetch` / GraphQL clients directly.
- **Hooks** own orchestration; **API modules** or server handlers own I/O.
- Prefer one shared **`normalize(raw) => { id, data }`** for list rows, detail, and mutation results.
