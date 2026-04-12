---
name: entity-crud-page
description: >
  Scaffold a full CRUD page (list, create, edit, detail, delete) using useEntityCRUD with EntityTable,
  EntityDetailSheet, and EntityFormSheet. Enforces Components → Hooks → Stores: page composes only;
  useXxxCRUD hook wraps library CRUD; API module holds fetch/mutations.
---

# `/entity-crud-page` — Full CRUD page

## When to use

- New admin or app screen for **one** primary `EntityType` with full lifecycle
- Replacing fetch-heavy pages that keep duplicate row state in React
- You need **modes** (`list` | `detail` | `edit` | `create`) coordinated in one surface

## Preconditions

- `entity_spec` from **`prompts/specify.md`** (fields, mutations, optional relations)
- Implementation plan from **`prompts/plan.md`**

## Implementation checklist

1. **API module** (no React): `list`/`detail`/`create`/`update`/`delete` HTTP or RPC wrappers; map errors to `Error`.
2. **Bootstrap**: call **`registerSchema`** once per related type if relations exist (see **`/entity-crud-relations`**).
3. **Hook** `useXxxCRUD.ts`: call **`useEntityCRUD<T>`** with:
   - `type`, `listQueryKey`, `listFetch`, `normalize`, optional `detailFetch`
   - `onCreate` / `onUpdate` / `onDelete` delegating to API module
   - `initialView` for default filter/sort; `createDefaults` if needed
4. **Columns**: factory `buildXxxColumns()` using **`agents/table-builder.md`**; pass **`crud.list`** as `EntityTable`’s **`viewResult`**.
5. **Fields**: factory `buildXxxFields()` using **`agents/form-builder.md`**.
6. **Page** `XxxPage.tsx`:
   - `const crud = useXxxCRUD()` (from hook file only)
   - **`EntityTable`**: `viewResult={crud.list}`, `onRowClick` → `crud.openDetail` or `crud.select`, `selectedId={crud.selectedId}`
   - **`EntityDetailSheet`**: `open={crud.mode === "detail"}`, wire `onClose`, `startEdit`, `deleteEntity`
   - **`EntityFormSheet`**: `open={crud.mode === "create" || crud.mode === "edit"}`, `mode` prop, save/cancel
   - Toolbar: `crud.startCreate()`, `crud.list.refetch` if exposed
7. **Reflect**: **`prompts/reflect.md`**

## Deliverables

| Artifact | Role |
|----------|------|
| `XxxPage.tsx` (or route file) | Composition only; no store, no fetch |
| `useXxxCRUD.ts` | `useEntityCRUD` wiring + stable `useRef` callbacks if needed |
| `xxxApi.ts` (or shared client) | Network + serialization |
| `xxxColumns.tsx` / `xxxFields.tsx` | Reusable builders |

## Parent skill

**`prometheus-entity-skills/entity-graph-crud/SKILL.md`**, **`../CLAUDE.md`**, **`../AGENTS.md`**
