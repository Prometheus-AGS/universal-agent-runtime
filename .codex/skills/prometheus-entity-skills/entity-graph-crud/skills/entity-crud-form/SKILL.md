---
name: entity-crud-form
description: >
  Build EntityFormSheet and EntityDetailSheet configurations from FieldDescriptor arrays: field types,
  enum options, readonly/create/edit flags, custom editControl/render, and wiring to useEntityCRUD buffers and actions.
---

# `/entity-crud-form` — Forms and sheets

## When to use

- You already have **`useEntityCRUD`** (or will add it) and need **create/edit/detail** UI only
- Replacing ad-hoc forms with declarative **`FieldDescriptor<T>[]`**
- Adding **custom controls** (FK pickers, rich text) via `editControl` / `render`

## Core types (`@prometheus-ags/prometheus-entity-management`)

- **`FieldType`**: `text` | `textarea` | `number` | `email` | `url` | `date` | `boolean` | `enum` | `custom`
- **`FieldDescriptor<TEntity>`**: `field`, `label`, `type`, optional `required`, `placeholder`, `options`, `hint`, `render`, `editControl`, `hideOnCreate`, `hideOnEdit`, `readonlyOnEdit`

## Steps

1. Map each `entity_spec` field to **`FieldType`** and labels (humanize keys).
2. System fields (`id`, `createdAt`, …): **`hideOnCreate`**, **`readonlyOnEdit`**, optional **`render`** in detail.
3. **Enums**: supply `options: { value, label }[]` aligned with API vocabulary.
4. **Relations as FK**: detail view may **`render`** resolved label from `crud.relations`; edit may use **`editControl`** that calls `setField` / `setCreateField` (passed from parent callbacks—**not** fetch inside control).
5. **`EntityFormSheet`**: `crud`, `fields`, `mode` (`"create"` | `"edit"`), `open`, `onClose`; surface `createError` / `saveError`.
6. **`EntityDetailSheet`**: `crud`, `fields`, `open`, `onClose`; optional `showEditButton` / `showDeleteButton`.

## Checklist

- [ ] No network I/O inside `render` / `editControl`
- [ ] Dates round-trip consistently (ISO strings vs `Date`)
- [ ] Required fields match server validation expectations
- [ ] `EntityFormSheet` `open` tracks `crud.mode` precisely to avoid focus traps

## Playbook

Follow **`agents/form-builder.md`** for the full decision table and outputs.

## Parent skill

**`prometheus-entity-skills/entity-graph-crud/SKILL.md`**
