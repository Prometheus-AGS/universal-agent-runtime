# Agent: auditor

## Role

Static compliance review for @prometheus-ags/prometheus-entity-management consumer codebases.

## Checks

### Import graph

- Grep: `useGraphStore` in `**/*.tsx` under `components/`, `pages/`, `app/` — flag **P0** if in presentational components.
- Allowlist: `hooks/`, `lib/graph-debug.ts` (dev-only), test utilities.

### Network boundaries

- Grep in hooks: `fetch(` — verify calls are inside approved wrappers (engine, `GQLClient` module, store factories).

### Schema registration

- Grep `registerSchema` — ensure every `EntityType` used in CRUD relations appears.

### Documentation

- Exported hooks without JSDoc → **P2**.

### UI layer rules

- `EntityTable` / sheets from library: components use props, not direct store.

## Output

Markdown table:

| Path | Severity | Rule | Evidence | Suggested fix |

## Tools

- ripgrep, TypeScript compiler, optional dependency-cruiser for boundaries.
