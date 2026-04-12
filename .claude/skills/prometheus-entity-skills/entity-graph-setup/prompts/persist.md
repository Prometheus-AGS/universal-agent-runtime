# Persist — entity-graph-setup handoff

Finalize session state and optional orchestrator routing.

## 1. Update skill state

If using `scripts/state-init.sh`, merge into `.entity-graph-skills/entity-graph-setup/state.json`:

- `updated_at` (UTC ISO)
- `phases_completed`: append `specify` | `plan` | `execute` | `reflect`
- `artifacts`: paths to `migration_plan.md`, `setup_spec.json`, `reflect_report.md`
- `orchestrators`: refresh output of `detect-orchestrators.sh`

## 2. Git hygiene

- Recommend a **single feature branch** per vertical slice (e.g. `feat/entity-graph-projects`).
- Tag or note **backup** ref before deleting legacy code (per `CLAUDE.md`).

## 3. Orchestrator routing

Parse `detect-orchestrators.sh` JSON:

- **`kbd.available`**: suggest next `/kbd-plan` or project-specific phase sync.
- **`openspec.available`**: propose formal change folder if the org uses OpenSpec for migrations.
- **`evolver.available`**: optional evolution cycle for iterative rollout.
- **`refiner.available`**: optional UI artifact refinement if demo pages were added.

Emit a short **next_actions** list tailored to which flags are `true`.

## 4. Sibling skills

Document handoff when appropriate:

| Next skill | When |
|------------|------|
| `entity-graph-crud` | Full table + sheets + `useEntityCRUD` |
| `entity-graph-graphql` | GQL client + `useGQLEntity` / `useGQLList` |
| `entity-graph-prisma` | Next.js route + Prisma + `toPrismaWhere` |
| `entity-graph-realtime` | WebSocket / Supabase / Convex adapters |

## Output

`persist_summary.md` with state path, branch suggestion, orchestrator JSON, and **next_actions**.
