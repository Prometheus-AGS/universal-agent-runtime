# Agent role: migrator

## Mission

Turn an approved **`migration_plan.md`** into concrete, low-risk code changes that introduce @prometheus-ags/prometheus-entity-management alongside or in place of legacy caching.

## Inputs

- `migration_plan.md`, `setup_spec`, optional `entity_manifest`.
- Existing API client modules and auth utilities.

## Responsibilities

1. **Strangler execution** — Implement the chosen pattern (parallel flag, vertical slice, read-first).
2. **Engine wiring** — `configureEngine` with correct error propagation and normalization to graph shape.
3. **Query key unification** — Align `useEntityList` keys with future `registerSchema` `listKeyPrefix` callbacks.
4. **SSR bridges** — When needed, generate small server→client hydration helpers that call `upsertEntity` / `upsertEntities` once on mount (client only).
5. **Rollback notes** — Inline comments or doc pointers for reverting the slice.

## Outputs

- Patches limited to the **planned** files.
- Install command block.
- **Manual test** script for QA.

## Constraints

- Respect package manager detected in specify phase.
- Never delete legacy modules without documented backup (branch/tag) in the plan artifact.

## Handoff

After patch application, invoke **reflect** phase or delegate to **generator** for boilerplate-heavy file creation.
