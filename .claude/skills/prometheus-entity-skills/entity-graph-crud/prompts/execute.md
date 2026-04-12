# Execute — entity-graph-crud

Generate code according to the plan and **skill `CLAUDE.md`**.

## Order of implementation

1. **API layer** (`fetch` wrappers, error handling). No React imports.
2. **Bootstrap** `registerSchema` if relations exist (app init file).
3. **Custom hook** wrapping `useEntityCRUD` with all callbacks.
4. **Column definitions** (TanStack `ColumnDef` + `entityMeta`).
5. **Field descriptors** for sheets.
6. **Page component** composing table + sheets; wire `open`, `onClose`, `crud` props.

## Coding rules (recheck while typing)

- **No `useGraphStore` in `*.tsx` components.**
- **No `fetch` in components**—only in API module or server actions consumed by hooks indirectly.
- **`normalize`** must be shared between list and mutation results where applicable.
- **Lists:** ensure `listQueryKey` reflects filter/sort state so cache lanes stay correct.

## Patterns to prefer

- `useEntityCRUD` for full list+detail+edit+create cycle.
- `useEntityView` parameters mirror toolbar filters (remote mode when completeness is not local).
- `actionsColumn` for row-level edit/delete with confirmation for delete.

## Inline editing

If using `EntityTable` inline edits:

- Commit path should call `onUpdate` from hook layer, not a component calling API directly.

## Styling

Reuse project tokens; do not hardcode colors inconsistent with app theme.

## Output format for the agent

- Provide complete files or patches with paths.
- End with a short **wiring summary**: which hook the page calls, which keys are used, where `registerSchema` runs.
