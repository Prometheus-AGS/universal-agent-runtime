# Execute — entity-graph-setup code generation

Inputs: approved `migration_plan.md`, `setup_spec`, optional `entity_manifest`.

## Rules

- Follow **`CLAUDE.md`** in this skill and **`prometheus-entity-skills/_shared/references/architecture-rules.md`**.
- Reuse the project’s **existing** HTTP client (axios, ky, `fetch` wrapper) **inside** engine configuration—not inside new component code.
- Use **`pnpm add`** / **`npm install`** / **`yarn add`** consistent with the detected lockfile.

## Step A — Dependency

Add:

```text
@prometheus-ags/prometheus-entity-management
```

If the workspace is the **library monorepo** itself, wire the app via **tsconfig paths** (no duplicate install).

## Step B — Engine bootstrap

Generate `configureEngine` with:

- `fetchEntity` / `fetchList` implementations delegating to real endpoints.
- **normalize** functions mapping API JSON → `{ id, ...record }` for `upsertEntity` / list id extraction.
- Sensible **`staleTime`** (start with library default 30s unless product says otherwise).

## Step C — Schema registration

For each entity in wave 1:

- `registerSchema({ type, relations?, globalListKeys? })` matching real list keys (`serializeKey`).

Emit TypeScript that **typechecks** against exported `EntitySchema` types.

## Step D — Hook migration (slice 1)

- Replace one list: `useEntityList({ queryKey, fetchList: ... })` or equivalent pattern from plan.
- Ensure **components** only import **hooks** and presentational props—no `useGraphStore`.

## Step E — Developer ergonomics

- Optional: `useGraphDevTools` gated behind `import.meta.env.DEV` or `process.env.NODE_ENV === "development"`.
- Short **README snippet** in the PR body or `docs/entity-graph.md` (only if the repo already uses such docs).

## Output

1. Unified diff or per-file patches.
2. **Install command** copy-paste block.
3. **Manual verification** steps (which page to open, what network tab should show).
