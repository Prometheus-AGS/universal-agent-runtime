# Goals

Phase: **uar-frontend-typecheck-cleanup**

Seeded from `uar-security-deps-and-hygiene`'s reflection.md — a thin
cleanup phase, deliberately scoped down at the user's choice. The
reflection's other three carryover items were explicitly excluded:
`task_7c2fd7ee` (SurrealQL `type::thing()`/`type::record()` fix) and
`task_188b4179` (`VectorMatcher::embed_batch` placeholder embeddings)
already have separate `spawn_task` sessions working on them, so a new
KBD phase around either would duplicate that work; the eval-gate
activation item is operator-only, not agent-actionable.

## G1 — Clear the pre-existing `bun run typecheck` backlog (P0, primary)

- 17 pre-existing errors have been carried, unaddressed, across 3+
  phases now (first noted in `uar-spec-v2-and-polish`, re-carried
  through `uar-security-deps-and-hygiene`): Base UI `Select` string vs.
  `null` nullability mismatches, `react-resizable-panels` API drift,
  and `recharts` type-export drift.
- **New finding this session, not yet in any prior phase's records**:
  running `bun run typecheck` right now doesn't even reach the
  TypeScript compiler — it fails immediately with
  `[ERR_PNPM_IGNORED_BUILDS] Ignored build scripts: @parcel/watcher@2.5.6`,
  a pnpm supply-chain safety gate asking for `pnpm approve-builds`.
  This is very likely unrelated to the 17 carried errors (it's a
  pre-install gate, not a type error) — plausibly introduced by the
  `frontend-pnpm-workspace-migration`/`replace-bun-with-pnpm-in-ci-frontend-job`
  changes visible in `openspec/changes/`. **First task for
  `/kbd-assess`**: resolve this gate (or confirm it's a one-time local
  interactive approval, not a repo-level regression) so the actual
  current error count and content can be re-verified fresh — don't
  assume the carried "17 errors, same 3 causes" description is still
  accurate until the command actually runs again.
- Fix or explicitly re-scope each of the 3 known causes once the
  command runs cleanly:
  - Base UI `Select` nullability (`string | null` vs. component's
    expected prop type)
  - `react-resizable-panels` API drift
  - `recharts` type-export drift

## Explicitly out of scope for this phase

- `task_7c2fd7ee` (SurrealQL `type::thing()` fix) — separate spawn_task
  in flight.
- `task_188b4179` (embedding placeholders) — separate spawn_task in
  flight.
- Eval-gate activation — operator-only, requires secrets/workflow
  dispatch this agent cannot perform.

## Success criteria

- `bun run typecheck` actually runs to completion (the pnpm gate is
  resolved or confirmed to be a local-only, one-time approval).
- Zero TypeScript errors, or every remaining error explicitly
  dispositioned with rationale (e.g. an upstream type-export bug not
  fixable from this repo) rather than silently re-carried a 4th time.
- No regression in `cargo test --lib`/`cargo test --test integration`
  if any fix touches shared build tooling (e.g. `package.json`,
  `pnpm-workspace.yaml`).

---

## Instructions

Review and refine the goals above before running `/kbd-assess`. When
ready:

```
/kbd-assess uar-frontend-typecheck-cleanup
```
