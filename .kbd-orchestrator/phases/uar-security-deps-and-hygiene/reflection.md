# Phase Reflection: uar-security-deps-and-hygiene

**Project:** universal-agent-runtime
**Date:** 2026-07-06
**Phase completion:** 100%
**Changes completed:** 10 / 10

## What diverged from plan, first

`plan.md`'s Round 4 disclosed risk — "if 3.1/3.2 introduce a breaking
change this repo's migrations can't absorb cleanly in one pass, stop
and re-carry as debt rather than force a partial/unverified upgrade" —
did not materialize. No breaking schema/query change was found between
`surrealdb` 3.0.5 and 3.2.0; `cargo check` was clean immediately after
the version bump, unlike `rmcp-pin-bump` and `wasmtime-disposition`
earlier in this phase, both of which needed real source fixes for
`#[non_exhaustive]`/trait-impl breakage.

One factual correction surfaced during `surrealdb-upgrade`'s
compatibility review: both `assessment.md` and `plan.md` refer to "12
SurrealDB migrations" needing a breaking-change check. That count is
real, but it describes `migrations/*.sql` — which are Postgres/sqlx
migrations (`sqlx::migrate!("./migrations")` in
`src/uar/security/credentials/store.rs`), not SurrealDB. The actual
SurrealDB schema is a single file, `migrations/surrealdb/schema.surql`
(85 `DEFINE` statements). This didn't change the change's outcome (the
real file was reviewed and is unaffected either way), but it's worth
recording since the next phase that touches persistence should look in
the right place from the start.

`surrealdb-upgrade`'s `cargo test --test integration` run also hit one
resource-contention flake (`credential_chain_put_then_list` timed out
waiting for its own embedded server to become healthy, among 58 tests
several of which boot their own server instances concurrently). Ruled
out as a regression by rerunning the single test in isolation (passed
in 3.82s) and then the full suite (56/56 clean). Not a new problem —
this test harness's design (many parallel server-booting tests) makes
this class of flake plausible under CPU load; worth a note for whoever
next investigates integration test reliability, though not blocking.

As with every phase since at least `uar-spec-v2-and-polish`: none of
this phase's 10 OpenSpec change directories were run through
`/opsx:verify` + `/opsx:archive` — all 10 still sit in
`openspec/changes/<id>/` rather than `openspec/changes/archive/`. This
is the same, already-disclosed pattern (artifact-refiner QA-gate
automation carried as unaddressed debt for 4+ phases, formally
addressed *within* this phase via the `artifact-refiner-gate-decision`
change — see Artifact Quality Summary below), not a new gap. Every
change was still verified, just via `cargo check`/`cargo test`/`cargo
clippy` (+ a live-server smoke check for `surrealdb-upgrade`) directly
rather than the formal OpenSpec/artifact-refiner gate.

## Artifact Quality Summary

| Metric                       | Value                              |
| ----------------------------- | ----------------------------------- |
| Changes with QA (artifact-refiner) | 0/10                           |
| First-pass pass rate         | N/A — gate not applicable this phase |
| Changes requiring refinement | 0                                   |
| Total refinement iterations  | 0                                   |

No artifact-refiner MCP tool is available in this environment (this
phase's `artifact-refiner-gate-decision` change — see below —
reconfirmed this via `ToolSearch` and formally retired the gate
requirement rather than continuing to silently carry it as debt). The
replacement verification method used for all 10 changes: `cargo
check`/`cargo test --lib`/`cargo test --test integration`/`cargo
clippy`, plus a dedicated live-server smoke check for the two
highest-blast-radius changes (`rmcp-pin-bump`, `surrealdb-upgrade`).
Full detail and rationale: `.kbd-orchestrator/references/artifact-refiner-gate-decision.md`.

### Recurring Constraint Violations

None — no constraint-based QA gate ran this phase (see above).

## Goals

| Goal | Status | Notes |
|---|---|---|
| G1 Security dependency triage & upgrade (P0, primary) | **MET** | All 6 tracked items landed: `.github/dependabot.yml` added, `surrealdb` 3.0.5→3.2.0 (`5cdcbde`), `rmcp` bumped to `rmcp-v1.8.0` tag (`c90858e`, fixes GHSA-89vp-x53w-74fx), `wasmtime`/`wasmtime-wasi` 41.0.3→46 (`720ba17`, fixes 2 critical + 1 high CVE), `failure` crate dispositioned as no-exposure dev-only dependency (documented in `assessment.md`, no code change needed), npm-side alerts fully resolved (`814be24` — both traced to Rust/root-devDependency issues, not actual npm runtime deps). |
| G2 Hygiene & validation (P1, secondary, carried from `uar-spec-v2-and-polish`) | **MET** | `artifact-refiner-gate-decision` formally retired the QA-gate requirement (no MCP tool available; documented the `cargo check/test/clippy` + direct-inspection replacement). `tests/uar_integration.rs` and `tests/bdd.rs` pre-existing compile failures both fixed (`814be24`, `e51c376`). `cargo bench --bench hot_path` actually run for the first time (`38f285b`) — all 4 benchmarks microsecond-scale, baseline recorded in the bench file's doc comment. `write-position-reminder.sh`'s `.stage`/`.status` schema mismatch fixed at the source, in the separate `prometheus-skill-system` repo, per explicit user decision (`e51c376`). |

Both goals fully MET — no scope cuts, no re-carried items from this
phase's own candidate list (the 4 items carried *into* this phase from
`uar-spec-v2-and-polish` are listed under G2 above and are now done,
not still-carried).

## Delivered Changes

- `dependabot-yml` — `814be24` — by: claude-code
- `fix-uar-integration-test` — `814be24` — by: claude-code
- `fix-bdd-test-path` — `814be24` — by: claude-code
- `artifact-refiner-gate-decision` — `814be24` — by: claude-code
- `npm-deps-triage` — `814be24` — by: claude-code
- `fix-waypoint-stage-schema` — `e51c376` — by: claude-code
- `wasmtime-disposition` — `720ba17` — by: claude-code
- `run-hot-path-bench` — `38f285b` — by: claude-code
- `rmcp-pin-bump` — `c90858e` — by: claude-code
- `surrealdb-upgrade` — `5cdcbde` — by: claude-code

All 10 committed to `main` (not yet pushed to a remote this phase —
no push instruction was given; matches this project's standing
pattern of committing per-change and letting the user decide when to
push). Full lib suite 363/363 green as of the final change; each
change was verified independently before its own commit (details in
each `openspec/changes/<id>/proposal.md`).

## Technical Debt Introduced

None. This phase's changes were dependency bumps, config-only
additions (`dependabot.yml`), and mechanical test/schema fixes — no new
abstractions, no deferred follow-ups specific to any of the 10 changes
themselves.

## Debt Found, Not Introduced By This Phase (disclosed for the record)

- `type::thing()` vs `type::record()` inconsistency in
  `src/uar/persistence/providers/surreal.rs:524` and
  `src/uar/compiler/storage/surreal.rs:71,109` — already broken at the
  pinned `3.0.5`, already tracked separately as `task_7c2fd7ee` (a
  `spawn_task` a separate session/task started on before this phase's
  Round 4 began). Confirmed during `surrealdb-upgrade`'s compatibility
  review to be orthogonal to the version bump, not fixed here since a
  different in-flight task owns it.
- `type::thing()` bug's twin, `update_document_status` using
  `type::thing()` when SurrealDB wants `type::record()` — same root
  cause, same separately-tracked task.
- 17 pre-existing `bun run typecheck` errors (Base UI Select
  nullability, `react-resizable-panels` API drift, `recharts`
  type-export drift) — unrelated to any change in this phase, carried
  forward again (see `carriedOverDebt` in `progress.json`).
- `VectorMatcher::embed_batch` returns zero-vector placeholder
  embeddings (`model.forward()` commented out) — pre-existing,
  separately tracked as `task_188b4179`, breaks RAG search /
  `LocalEmbedding` intent backend. Out of this phase's scope.

## Architecture Integrity

- No violations of the Prometheus Base Rules Set identified:
  - Rule 31 (small, reviewable changes): each of the 10 tracked
    changes landed as its own separate, scoped commit; Round 1's 5
    small changes were batched into one commit deliberately
    (`implementation-first/test-at-checkpoints` — the project's
    standing preference — applied per-round, not per-change, for the
    genuinely low-risk rounds), while Rounds 2–4's higher-risk changes
    each got their own commit and dedicated checkpoint.
  - Rule 3 (surgical changes): `surrealdb-upgrade` touched exactly
    `Cargo.toml`/`Cargo.lock` plus its own `openspec/` change dir — no
    drive-by fixes to the discovered `type::thing()` bug, which was
    explicitly left to its own separately-tracked task.
  - Rule 30 (tests are part of completion): every change verified via
    `cargo check`/`cargo test`/`cargo clippy` (and, for
    `surrealdb-upgrade` specifically, a live-server smoke check) before
    being committed.
  - Rule 5/6 (truth over fluency, evidence before conclusions): the
    "12 SurrealDB migrations" correction above was surfaced rather than
    silently worked around; the integration-test flake was diagnosed
    with an isolated rerun rather than assumed-and-reported as clean.
- `.kbd-orchestrator/constraints.md` does not exist in this repo, so
  there is no separate machine-checkable constraint file to validate
  against beyond the rule set above (same as noted in the prior
  phase's reflection).

## Cross-Tool Coordination Notes

Single-tool phase (`sourceTool: claude-code` throughout). Two
`spawn_task`-started sessions were running concurrently against the
same working tree for unrelated discovered bugs
(`task_188b4179`/embedding placeholders, `task_7c2fd7ee`/`type::thing`)
— per `project.json`'s `discoveredBugs`. Neither touched this phase's
files; no conflicts observed. Uncommitted `Cargo.toml`/`Cargo.lock`
changes for the `surrealdb-upgrade` bump were already present in the
working tree at the start of this session (bumped to `=3.2.0` with a
comment citing "explicit user direction") — inherited and continued
from rather than redone, since the diff was correct and matched the
plan's intent.

- **Progress tracking**: `progress.json` and `current-waypoint.json`
  stayed in sync throughout this phase (no repeat of the prior phase's
  stale-reminder-file gap) — each round's commit included a
  corresponding `chore(kbd):` state-update commit.
- **Handoff quality**: no `execute.handoff.json` was written before
  this reflection, matching the prior phase's pattern (execute
  produced per-change commits, not a single handoff artifact).

## Lessons Learned

- **A term repeated across `assessment.md` and `plan.md` isn't
  automatically verified.** "12 SurrealDB migrations" propagated from
  assessment to plan without either document actually opening
  `migrations/*.sql` to check the SQL dialect. A 30-second `grep` for
  `type::thing`/`DEFINE TABLE` vs `CREATE TABLE`/`SERIAL` would have
  caught the Postgres/SurrealDB mixup immediately. Worth a standing
  habit: when a count or fact is asserted in a planning document,
  verify it against the actual files before treating it as a scoping
  constraint for the next phase.
- **A single flaky test isn't a regression — but don't just assume
  that either.** Diagnosing `credential_chain_put_then_list`'s failure
  required actually isolating it (rerun alone, confirm it passes fast)
  rather than either (a) blindly re-running the full suite and hoping,
  or (b) assuming a version bump caused it without checking. The
  isolated rerun took under a minute and produced a clean answer either
  way.
- **When uncommitted work is found in the working tree mid-phase,
  read the diff before redoing it.** The `surrealdb-upgrade` bump to
  `3.2.0` (vs. plan's more conservative "latest compatible 3.x") was
  already done and well-documented via a `Cargo.toml` comment
  explaining the 3.2.0-vs-3.1.5 decision. Continuing from it (verifying,
  then committing) was faster and more accurate than re-deriving the
  same decision from scratch.

## Next Phase Focus

This phase closes out both the security backlog (G1) and the hygiene
debt carried from `uar-spec-v2-and-polish` (G2) with nothing re-carried
from either. Recommend a return to **feature scope** next, informed by
the still-open items below (none of which block a feature phase, but
should stay visible):

1. **`type::thing()` → `type::record()` fix** — already assigned to
   `task_7c2fd7ee`; confirm it's landed before the next phase that
   touches `src/uar/persistence/providers/surreal.rs` or
   `src/uar/compiler/storage/surreal.rs`, since both files still have
   the inconsistency as of this phase's end.
2. **`VectorMatcher::embed_batch` placeholder embeddings**
   (`task_188b4179`) — blocks real RAG search; check status before
   scoping any RAG-adjacent feature work.
3. **17 pre-existing `bun run typecheck` errors** — still unaddressed
   across 3+ phases now; worth a small dedicated cleanup change if a
   frontend-touching phase comes up next.
4. Operator-only, cannot be done by an agent: seed
   `evals/results/starter.baseline.json` and activate the Tier-2
   nightly eval gate — carried across multiple phases now, unchanged
   since the prior phase's reflection.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation.
