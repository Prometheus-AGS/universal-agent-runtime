# Agent: table-builder

**Goal:** Build TanStack Table `ColumnDef<TEntity>[]` using prometheus column helpers and align filters with `useEntityView`.

## Inputs

- `entity_spec.fields`
- Filter/sort capabilities (local vs remote)
- Row actions required (edit, delete, navigate)

## Steps

1. **Choose builders**

   - Strings → `textColumn`
   - Numbers → `numberColumn`
   - Dates → `dateColumn`
   - Booleans → `booleanColumn`
   - Enums → `enumColumn` with `enumOptions` for filter chips
   - Row chrome → `selectionColumn`, `actionsColumn`

2. **`meta.entityMeta`**

   - Set `field` to the entity key
   - Set `filterType` to drive toolbar controls (`text`, `number`, `date`, `boolean`, `enum`, `relation`, `none`)
   - `relation` type when cell shows joined label but filter targets FK

3. **Sorting**

   - Use `SortHeader` from library for consistent UI
   - Remote sort: ensure `listFetch` reads `ViewFetchParams` sort clauses and passes them to API

4. **Inline edit**

   - Only set `editable: true` when `onUpdate` path exists and you use `EntityTable` inline editor patterns from the library

5. **Row selection → CRUD**

   - On row click, call `crud.openDetail(id)` or `crud.select(id)` per UX

## Output

- `buildTaskColumns(): ColumnDef<Task>[]`
- Note mapping from UI filters → `FilterSpec` keys (same names as `field` when possible)

## Checklist

- [ ] Column `id` / `accessorKey` stable for TanStack
- [ ] No duplicate `field` metadata across columns
- [ ] Enum labels match form descriptors
