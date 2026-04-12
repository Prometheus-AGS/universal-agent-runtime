# UI component catalog — @prometheus-ags/prometheus-entity-management

Public exports from **`@prometheus-ags/prometheus-entity-management`** for CRUD UIs. Import from the package name or tsconfig path alias used by your app.

---

## Sheets (`src/ui/EntitySheets.tsx`)

| Export | Purpose |
|--------|---------|
| **`Sheet`** | Base right-side panel: overlay, **Esc** closes, optional `footer`, `width` (default `w-[480px]`) |
| **`EntityDetailSheet`** | Read-oriented detail; **`crud: CRUDState<T>`**, **`fields`**, `title`, `description`, optional `children`, `showEditButton`, `showDeleteButton`, `deleteConfirmMessage` |
| **`EntityFormSheet`** | Create/edit form with dirty tracking; **`mode`**: `"create"` \| `"edit"`**,** **`open`**, **`onClose`**, **`crud`**, **`fields`** |

### `FieldDescriptor<TEntity>`

| Member | Role |
|--------|------|
| **`field`** | Key on entity (string keyof) |
| **`label`** | Visible label |
| **`type`** | `text` \| `textarea` \| `number` \| `email` \| `url` \| `date` \| `boolean` \| `enum` \| `custom` |
| **`required`**, **`placeholder`**, **`hint`** | UX + validation hints |
| **`options`** | Enum `{ value, label }[]` |
| **`render`** | Read-only cell in detail / readonly modes |
| **`editControl`** | Custom editor `(value, onChange, entity) => ReactNode` |
| **`hideOnCreate`**, **`hideOnEdit`**, **`readonlyOnEdit`** | Lifecycle flags |

### `CRUDState` (subset for sheets)

Relevant fields from **`useEntityCRUD`**:

- **Modes:** `mode`, `setMode`, `startEdit`, `startCreate`, `cancelEdit`, `cancelCreate`
- **Selection:** `selectedId`, `select`, `openDetail`
- **Buffers:** `editBuffer`, `setField`, `setFields`, `createBuffer`, `setCreateField`, `setCreateFields`
- **Actions:** `save`, `create`, `deleteEntity`, `applyOptimistic`
- **Status:** `isSaving`, `saveError`, `isCreating`, `createError`, `isDeleting`, `deleteError`, `dirty`

---

## Table (`src/ui/EntityTable.tsx`)

| Export | Purpose |
|--------|---------|
| **`EntityTable`** | Data grid: TanStack Table, toolbar search, sort handoff to **`useEntityView`**, selection, optional inline edit, pagination modes |
| **`InlineCellEditor`** | Single-field inline editor (Enter commit, Esc cancel) |

### `EntityTableProps<T>` (summary)

| Prop | Type / default | Notes |
|------|----------------|-------|
| **`viewResult`** | `UseEntityViewResult<T>` | **Required** — from `crud.list` or `useEntityView` |
| **`columns`** | `ColumnDef<T>[]` | Use library column builders + `meta.entityMeta` |
| **`getRowId`** | `(row) => string` | Default `String(row.id)` |
| **`selectedId`** | `string \| null` | Highlight active row |
| **`onRowClick`** | `(row) => void` | e.g. open detail |
| **`onCellEdit`** | `(row, field, value) => void` | Must forward to hook-layer mutation |
| **`onBulkAction`** | `(rows) => ReactNode` | Toolbar region for selection |
| **`paginationMode`** | `"none"` \| `"loadMore"` \| `"pages"` | Default `"loadMore"` |
| **`pageSize`** | number | Default **50** |
| **`searchPlaceholder`**, **`searchFields`** | string, string[] | Wired to `viewResult.setSearch` |
| **`toolbarChildren`** | `ReactNode` | Extra controls |
| **`showToolbar`** | boolean | Default **true** |
| **`emptyState`** | `ReactNode` | Empty list UX |
| **`className`** | string | Wrapper |

**Peer dependency:** `@tanstack/react-table` in the consuming app.

---

## Columns (`src/ui/columns.tsx`)

| Helper | Typical use |
|--------|-------------|
| **`selectionColumn`** | Row checkboxes |
| **`textColumn`** | Strings; optional `filterType` override |
| **`numberColumn`** | Numbers + `format` |
| **`dateColumn`** | ISO / date strings |
| **`enumColumn`** | Status columns + **`enumOptions`** for filters |
| **`booleanColumn`** | Toggles / checkmarks |
| **`actionsColumn`** | Per-row actions menu |
| **`SortHeader`** | Shared sort affordance |

### `EntityColumnMeta<TEntity>`

- **`field`**, **`filterType`** (`text`, `number`, `date`, `dateRange`, `boolean`, `enum`, `relation`, `none`)
- **`enumOptions`**, **`relationEntityType`**
- **`editable`**, **`hideable`**

---

## Hooks (CRUD and lists)

| API | Role |
|-----|------|
| **`useEntityCRUD`** | List + detail + buffers + mutations + `cascadeInvalidation` |
| **`useEntityView`** | Filtered/sorted list: local / remote / hybrid completeness |
| **`useEntity`**, **`useEntityList`** | Lower-level single entity or list subscription |
| **`useSuspenseEntity`**, **`useSuspenseEntityList`** | Suspense variants (when enabled in your React version setup) |
| **`registerSchema`**, **`readRelations`** | Relation graph + detail joins |

---

## Typical composition order

1. **`useXxxCRUD`** (or `useEntityView` + `useEntity`) in a **hook file**.
2. **`EntityTable`** with **`viewResult={crud.list}`** and column defs.
3. **`EntityDetailSheet`** when **`crud.mode === "detail"`**.
4. **`EntityFormSheet`** when **`crud.mode === "create"`** or **`"edit"`**.

---

## Styling

Sheets and table use **Tailwind** utilities (`bg-background`, `text-muted-foreground`, borders). Ensure CSS variables or theme tokens exist (e.g. shadcn-style `background`, `muted-foreground`).
