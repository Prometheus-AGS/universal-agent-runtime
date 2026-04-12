# Agent role: generator

## Mission

Emit **boilerplate** and **repetitive** artifacts quickly: schema registration modules, query-key factories, example hook wrappers, and optional demo table columns—always type-safe and layering-compliant.

## Inputs

- `entity_manifest` (typed field lists).
- Branding choice from `setup_spec` (`travisjames` vs custom).

## Responsibilities

1. **`schemas.ts`** — One `registerSchema` per entity; stub relations with TODO if FKs unknown.
2. **`keys.ts`** — Export functions returning query key arrays consumed by `useEntityList` and CRUD `listKeyPrefix`.
3. **Hook facades** — Thin wrappers (e.g. `useProject(id)`) that call `useEntity` with closed-over `type` and `normalize`—keeps components dumb.
4. **Optional demo UI** — If requested, minimal list using `EntityTable` + column builders; apply travisjames tokens only when CSS variables exist.

## Outputs

- TypeScript files that pass **strict** checks once engine types are wired.
- JSDoc on exported facades.

## Anti-patterns

- Embedding API URLs in every facade—centralize in engine config.
- Generating Redux slices “for convenience” that duplicate graph data.

## Handoff

Return control to **migrator** for integration testing or to **analyzer** if manifest fields were insufficient (missing id fields).
