---
name: entity-crud-table
description: >
  Build EntityTable views with column helpers (textColumn, numberColumn, enumColumn, etc.), SortHeader,
  selectionColumn, and meta.entityMeta for filters; sync sorting and search with useEntityView via UseEntityViewResult.
---

# `/entity-crud-table` — Table and list UI

## When to use

- List/grid is the primary deliverable; sheets may be minimal or deferred
- You need **toolbar search**, **column filters**, **pagination** (`loadMore` | `pages` | `none`), or **inline edit**
- Remote vs local completeness is already decided for `useEntityView` (or default remote)

## EntityTable props (library)

Key **`EntityTableProps`** fields:

| Prop | Role |
|------|------|
| **`viewResult`** | **`UseEntityViewResult<T>`** from `crud.list` or standalone `useEntityView` |
| **`columns`** | `ColumnDef<T>[]` with optional **`meta.entityMeta`** |
| **`getRowId`** | Default `r => String(r.id)` |
| **`selectedId`** / **`onRowClick`** | Highlight + selection integration with CRUD |
| **`onCellEdit`** | When using editable columns—must delegate to hook-layer `onUpdate`, not raw API |
| **`paginationMode`** | `"none"` \| `"loadMore"` \| `"pages"` |
| **`pageSize`** | Default 50 |
| **`searchFields`** | Fields used for client/search wiring with `viewResult.setSearch` |
| **`toolbarChildren`** | Extra toolbar controls (create button, density) |
| **`showToolbar`** | Default true |

## Column helpers (`src/ui/columns.tsx`)

- **`selectionColumn`**, **`textColumn`**, **`numberColumn`**, **`dateColumn`**, **`enumColumn`**, **`booleanColumn`**, **`actionsColumn`**, **`SortHeader`**
- Set **`meta.entityMeta.filterType`** (`text`, `number`, `date`, `enum`, …) so the toolbar filters match column semantics
- **`editable: true`** only when **`onCellEdit`** commits through the CRUD hook

## Wiring

```tsx
<EntityTable<Task>
  viewResult={crud.list}
  columns={taskColumns}
  selectedId={crud.selectedId}
  onRowClick={(row) => crud.openDetail(String(row.id))}
  paginationMode="loadMore"
/>
```

## Playbook

**`agents/table-builder.md`** — per-column plan and filter alignment.

## Parent skill

**`prometheus-entity-skills/entity-graph-crud/SKILL.md`**
