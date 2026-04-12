# Plan — entity-graph-crud

Translate `entity_spec` into an implementation plan before coding.

## 1. File map

List paths to create or modify, for example:

- `src/features/tasks/api.ts` — raw HTTP
- `src/features/tasks/useTaskCRUD.ts` — `useEntityCRUD` wiring
- `src/features/tasks/TaskColumns.tsx` — column defs factory
- `src/features/tasks/TaskFields.tsx` — `FieldDescriptor[]` factory
- `src/pages/tasks/TasksPage.tsx` — composition only

## 2. Hook wiring

Document:

- **`type`** (EntityType) and **`listQueryKey`** (stable array, include filters in serialized key via `useEntityView`)
- **`normalize`** signature and id extraction
- **`listFetch(ViewFetchParams)`** mapping from `FilterSpec` / `SortSpec` to API query params (reference `toRestParams` if applicable)
- **`detailFetch`** optional
- **`onCreate` / `onUpdate` / `onDelete`** delegating to API module

## 3. Component tree

ASCII or bullet tree:

- Page
  - Toolbar (create, refresh)
  - `EntityTable` bound to `crud.list`
  - `EntityDetailSheet` / `EntityFormSheet` controlled by `crud.mode`

## 4. State modes

Map UI regions to `CRUDMode`: `list` | `detail` | `edit` | `create`.

## 5. Column plan

Per column:

- Builder (`textColumn`, `numberColumn`, `dateColumn`, `enumColumn`, `booleanColumn`, `actionsColumn`)
- `meta.entityMeta.filterType`
- Editable inline? (ties to `InlineCellEditor` usage)

## 6. Form plan

Per field:

- `FieldDescriptor.type`
- Enum `options`
- `hideOnCreate` / `readonlyOnEdit` flags

## 7. Schema registration

- `registerSchema` payload
- All `listKeyPrefix` functions and example concrete keys

## 8. Risks

- Partial list + missing detail fields → mitigation: `detailFetch`
- Heavy joins → mitigation: lazy relation lists or separate hooks

## Output

Checklist of files with one-line purpose each + ordered implementation steps (API → hook → columns/fields → page).
