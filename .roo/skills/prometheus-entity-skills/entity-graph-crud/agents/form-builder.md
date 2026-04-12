# Agent: form-builder

**Goal:** Produce `FieldDescriptor<TEntity>[]` and wire them to `EntityFormSheet` / `EntityDetailSheet` + `useEntityCRUD` buffers.

## Inputs

- `entity_spec.fields` with types and constraints
- Whether mode is create, edit, or read-only detail
- Optional design-system overrides via `editControl` / `render`

## Steps

1. **Map types → `FieldType`**

   | Source | `FieldType` |
   |--------|-------------|
   | string (short) | `text` |
   | string (long) | `textarea` |
   | number | `number` |
   | boolean | `boolean` |
   | ISO date | `date` |
   | email / url | `email` / `url` |
   | enum | `enum` + `options` |

2. **Labels and UX**

   - Humanize field keys (`dueDate` → “Due date”).
   - `placeholder` for empty states; `hint` for validation rules.

3. **Create vs edit**

   - System fields (`id`, `createdAt`): `hideOnCreate`, `readonlyOnEdit`.
   - Relations displayed as read-only resolved labels in detail (`render`) while edit uses FK picker in `editControl` if required.

4. **Sheet wiring**

   - `EntityFormSheet`: pass `crud`, `fields`, `mode` (`"create"` | `"edit"`), `open`, `onClose`.
   - `EntityDetailSheet`: pass `crud`, `fields`, `open`, wire edit/delete buttons to `crud.startEdit`, `crud.deleteEntity`.

5. **Validation**

   - Required fields: `required: true` on descriptor; enforce server errors surfaced via `crud.saveError` / `createError`.

## Output

- Exported factory: `buildTaskFields(): FieldDescriptor<Task>[]`
- Short usage snippet in page component

## Checklist

- [ ] No API calls inside `render` / `editControl`
- [ ] Enum options match server vocabulary
- [ ] Dates round-trip ISO consistently with API
