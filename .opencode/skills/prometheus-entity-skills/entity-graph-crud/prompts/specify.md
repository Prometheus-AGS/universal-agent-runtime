# Specify — entity-graph-crud interview

Run this phase before writing code. Capture answers in `entity_spec` (JSON or structured markdown).

## 1. Entity identity

- **Entity type string** (graph key): e.g. `"Task"`, `"Product"`.
- **ID field** on wire + runtime: `id`, `uuid`, composite?
- **Display title** for sheets and page headings.

## 2. Field inventory

For each field:

| Column | Questions |
|--------|-----------|
| Name | Exact key on entity object |
| Type | string / number / boolean / date / enum / relation id / text |
| Required on create? | |
| Editable? | Read-only after create? |
| PII / secret? | Mask in UI? |
| Default | For create buffer |

## 3. List vs detail shape

- Does the **list endpoint** return full rows or partial projections?
- If partial, which fields require **`detailFetch`**?

## 4. Sorting and filtering

- Which fields are **server-filterable** vs **client-only**?
- Default sort column + direction.
- Free-text search across which fields (if any)?

## 5. Mutations

- Create/update/delete **HTTP methods and URLs** (or GraphQL operations if using GQL hooks elsewhere).
- **Optimistic** behavior expectations (toggle/slider → `applyOptimistic`?).
- **Error format** (message string, field errors).

## 6. Relations (for `registerSchema`)

For each relation:

- Cardinality: `belongsTo`, `hasMany`, `manyToMany`
- Foreign key or array field name
- Target entity type
- **List query key** pattern for child collections (`listKeyPrefix`)
- Lists to invalidate on FK change (`invalidateTargetLists` / `globalListKeys`)

## 7. UI preferences

- Table density, selection column yes/no
- Sheet width, placement (library sheets are right-side drawers)
- Primary actions in toolbar vs row actions
- Design system (shadcn, MUI, plain Tailwind)

## 8. Tech context

- Next.js vs Vite, SSR needs (hydration from server props into graph)
- Auth headers / tenant scoping for fetches

## Output artifact

Produce `entity_spec` object:

```json
{
  "type": "Task",
  "idField": "id",
  "title": "Tasks",
  "fields": [],
  "listFetch": { "mode": "full|partial" },
  "view": { "filters": [], "sort": [] },
  "mutations": { "create": true, "update": true, "delete": true },
  "relations": [],
  "ui": {}
}
```

## Done when

Stakeholder confirms field list and mutation endpoints; relations sketched or explicitly “none”.
