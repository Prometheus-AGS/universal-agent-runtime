# AGENTS.md — entity-graph-crud

Instructions for AI agents using the **entity-graph-crud** skill package ([agentskills.io](https://agentskills.io)-style layout with prompts, agents, references, hooks, and nested skills).

## Role

You implement or refactor CRUD on **@prometheus-ags/prometheus-entity-management**. You optimize for **cross-view consistency**: updating an entity once updates every list, sheet, and joined detail that reads that ID from the graph.

## Mandatory pre-read

Before generating code, read in order:

1. The consuming repo’s **`CLAUDE.md`** / **`AGENTS.md`** (e.g. pnpm-only, global layering).
2. This skill’s **`CLAUDE.md`** (CRUD-specific layering).
3. **`references/crud-patterns.md`** and **`references/ui-component-catalog.md`**.

Skim **`src/crud/useEntityCRUD.ts`** and **`src/ui/EntitySheets.tsx`** in the library when behavior is ambiguous.

## Behavior

- **Interview first** when requirements are incomplete: follow **`prompts/specify.md`** verbatim enough to produce `entity_spec`.
- **Plan before code** for multi-file work: **`prompts/plan.md`** with file map and hook contract.
- **Match the host app**: Next.js App Router vs Vite + TanStack Router, styling system, existing `api/` or `server/` patterns.
- **Never** put `fetch`, `axios`, GraphQL clients, or Supabase calls in `*.tsx` components.
- **Never** call **`useGraphStore`** from component files; only from dedicated hook/store modules where the app architecture allows—and prefer library hooks.

## Specialist playbooks (`agents/`)

| File | Use when |
|------|-----------|
| **`agents/form-builder.md`** | `FieldDescriptor[]`, `EntityFormSheet`, `EntityDetailSheet`, create vs edit flags |
| **`agents/table-builder.md`** | `ColumnDef`, `meta.entityMeta`, `EntityTable` + `UseEntityViewResult` wiring |
| **`agents/relation-wirer.md`** | `registerSchema`, `listKeyPrefix`, `invalidateTargetLists`, `globalListKeys` |

## Sub-skill routing (slash commands)

| Command | Use when |
|---------|-----------|
| **`/entity-crud-page`** | New full screen: table + sheets + modes + delete |
| **`/entity-crud-form`** | Only sheets and field descriptors |
| **`/entity-crud-table`** | Only grid, columns, filters/sort alignment |
| **`/entity-crud-relations`** | Only schema registry and cascade behavior |

## Quality gate

1. **TypeScript** passes (`pnpm run typecheck` or app-specific `typecheck:vite` / `typecheck:next`).
2. **CRUD flow**: list load → select → detail → edit → save (rollback on error) → create → delete.
3. **Relations**: when schemas exist, FK changes refresh the right lists and `readRelations` stays coherent.
4. **No violations** of skill **`CLAUDE.md`** (grep for `useGraphStore` in components, `fetch` in components).

## Outputs

Provide unified diffs or full files with paths. Always document:

- **`EntityType`** string and **`listQueryKey`** shape (including how filters enter the key).
- Where **`registerSchema`** runs (bootstrap path).
- API base paths and **`normalize`** id extraction.

## Lifecycle hooks

If the host tool loads **`hooks/hooks.json`**, `on_activate` runs **`scripts/state-init.sh`**; orchestrator flags come from **`scripts/detect-orchestrators.sh`**.
