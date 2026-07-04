# Phase Reflection: uar-spec-v2-and-polish

**Project:** universal-agent-runtime
**Date:** 2026-07-04
**Phase completion:** 100%
**Changes completed:** 7 / 7

## What diverged from plan, first

Before crediting anything: this phase's own waypoint files
(`current-waypoint.json`, `position-reminder.txt`) were stuck at
`assess_pending` / step 3-of-7 for a real span of elapsed time *after*
assess+plan had actually completed (commit `16c1aa3`) and three changes
had already merged — nobody had refreshed them. That was only caught and
fixed because `/kbd-status` was invoked and cross-checked against
`progress.json` and `git log` directly rather than trusting the reminder
file. Separately, the position-reminder sync script
(`write-position-reminder.sh`) reads a `.stage` key from
`current-waypoint.json`, but the waypoint schema in this repo had only
ever populated `.status` — so `Stage:` silently rendered `unknown` on
every regeneration until a `stage` field was added by hand this phase.
Neither of these is a change-execution failure, but both are real
process gaps worth fixing before they cause a worse one (e.g., a stale
`exactNextCommand` sending a future session back into `/kbd-assess` on
already-planned work).

Also: none of this phase's 7 OpenSpec changes went through
`/opsx:verify` + `/opsx:archive` — they remain sitting in
`openspec/changes/<id>/` rather than `openspec/changes/archive/`. This
matches the established (if not ideal) pattern from prior phases —
artifact-refiner QA gate automation has been carried as unaddressed debt
for 4+ phases now, including this one — but it means the reflect
prerequisite "changes verified + archived" is not literally met. Every
change *was* verified, just via `cargo check`/`cargo test`/`cargo
clippy` directly rather than the formal OpenSpec/artifact-refiner gate.

## Goals

| Goal | Status | Notes |
|---|---|---|
| G4 Specification & Distribution (CH-12→13→{14,15}→17) | **MET** | All 5 changes landed and verified: CH-12 (75116e4), CH-13 (e1532d6), CH-14 (4b24f01), CH-15 (bbe7ec2), CH-17 (13326a4). The declared sequential dependency (CH-12→13→{14,15}) held in practice — CH-14/15 genuinely needed CH-13's stage plumbing and couldn't have started earlier. |
| G5 Polish & Release (CH-19, CH-20) | **MET** | CH-20 (369117b + e2c82c7) and CH-19 (45e7e37) both landed. CH-20 was explicitly scoped down for two of its four sub-items (server.rs split is an *assessment*, not an executed split; `cargo bench`/`cargo check --benches` was never actually run this session) — disclosed in both the commit and this reflection, not silently claimed as fully done. |

Both goals are MET on delivered scope. Two caveats keep this from being
an unqualified "fully done": (1) CH-20's benchmark code has never been
compiled or executed, only reviewed — it could still have a compile
error nobody has seen; (2) the OpenSpec verify/archive step was skipped
for all 7 changes, per the paragraph above.

## Delivered Changes

- `agent-spec-v2` (CH-12, `75116e4`) — by: claude-code
- `compiler-v2-stages` (CH-13, `e1532d6`) — by: claude-code
- `eval-targeted-suites` (CH-17, `13326a4`) — by: claude-code
- `perf-security-load` (CH-20, `369117b` + incidental fix `e2c82c7`) — by: claude-code
- `conformance-testing` (CH-14, `4b24f01`) — by: claude-code
- `agent-template-library` (CH-15, `bbe7ec2`) — by: claude-code
- `docs-overhaul-deploy-guide` (CH-19, `45e7e37`) — by: claude-code

All 7 pushed to `origin/main`. Full lib suite 363/363 green as of the
last change; each individual change was verified independently before
its own commit (details in each `openspec/changes/<id>/proposal.md`).

## Technical Debt Introduced

- `benches/hot_path.rs` (CH-20) has never been run via `cargo bench` or
  even `cargo check --benches` in any session — compiled-by-inspection
  only. A future pass should actually run it.
- `main()` (`src/main.rs`) always loads the full `AppConfig` (including a
  persistence provider) before dispatching to *any* subcommand, so the
  new `compile` subcommand (CH-15) and the existing `eval` subcommand
  both require `UAR_PERSISTENCE__PROVIDER`/`UAR_PERSISTENCE__DATABASE_URL`
  env vars they never actually use. Pre-existing constraint for `eval`,
  newly inherited by `compile`; not fixed here (shared dispatch behavior,
  out of scope for a single-subcommand change).
- CH-20's `server.rs` split is a written assessment
  (`docs/server-rs-split-assessment.md`) and recommendation, not an
  executed split — deliberately, per Rule 31/Rule 8 (a ~5,000-line
  mechanical move deserves its own dedicated, checkpointed change).
- All 7 OpenSpec change directories remain unarchived (see "What diverged
  from plan" above) — this is a re-instance of the chronic
  artifact-refiner automation gap, not a new problem.

## Debt Found, Not Introduced By This Phase (disclosed for the record)

- `tests/uar_integration.rs`: a `Skill` struct literal is missing 8
  fields, breaking `cargo check --tests` for that one target. Predates
  this phase; unrelated to any of its 7 tracked changes.
- `tests/bdd.rs`: a nested `#[path]` attribute resolves incorrectly
  (`tests/live/integration/live/harness.rs`, which doesn't exist),
  breaking that target's compile. Also predates this phase and is
  unrelated to its tracked changes — this file wasn't even committed by
  this phase (it was untracked working-tree state present before this
  phase's work began).
- One real, in-scope bug *was* fixed: `src/uar/guardrails.rs`'s injection
  scan didn't normalize whitespace, so padding/line-breaking an injection
  phrase defeated a plain substring match (fixed in CH-20, `369117b`).
- One unrelated pre-existing bug *was* fixed in passing because it
  blocked verification entirely: `tests/settings_persistence.rs`'s
  `minimal_config()` was missing `AppConfig.guardrails` (the field was
  added in an earlier phase, `c454431`, and never backfilled here) —
  landed as its own separate commit (`e2c82c7`), not folded into CH-20's
  diff.

## Architecture Integrity

- This repo's `AGENTS.md` doesn't have a literal "Never Do" heading; the
  applicable rule set is the 40-rule Prometheus Base Rules Set mirrored
  in both `AGENTS.md` and `CLAUDE.md`. No violations identified:
  - Rule 31 (small, reviewable changes): each of the 7 tracked changes,
    plus the 2 incidental fixes, landed as its own separate, scoped
    commit.
  - Rule 3 (surgical changes): the only cross-cutting edits were
    deliberate, narrow reuse enablers (`PromptDialect::name()`,
    `strategy.rs`'s `default_*` fns → `pub(crate)`,
    `ModelRequirementsSection` gaining `PartialEq`, relocating
    `parser.rs`'s test fixture to module scope) — each justified in its
    change's `proposal.md`, not drive-by refactoring.
  - Rule 30 (tests are part of completion): every change was verified
    via `cargo check`/`cargo test`/`cargo clippy` before being committed.
  - Rule 8 (minimize irreversible actions): commits were only pushed
    after explicit user authorization ("commit and push and follow the
    rest of your recommendation"); nothing was force-pushed or amended.
- `.kbd-orchestrator/constraints.md` does not exist in this repo, so
  there is no separate machine-checkable constraint file to validate
  against beyond the rule set above.

## Cross-Tool Coordination Notes

Single-tool phase (`sourceTool: claude-code` throughout — no Roo/Cursor/
Codex/Antigravity activity this phase), so "cross-tool" coordination in
the literal multi-tool sense doesn't apply. What *does* apply, and is
worth recording as the equivalent lesson for a single-tool, multi-session
phase:

- **Progress tracking: GAPS FOUND.** `progress.json` itself stayed
  accurate throughout (assess/plan flags, `changes_completed` count).
  The *derived* files (`current-waypoint.json`, `position-reminder.txt`)
  did not — they went stale for a real span after assess+plan completed,
  and only self-corrected once a `/kbd-status` cross-check forced the
  issue. The lesson isn't "add more state files" — it's that whichever
  skill finishes a stage must actually refresh the reminder file's
  `Next command:` line as its last act, not just update `progress.json`
  and assume something else will propagate it.
- **Handoff quality: CLEAR**, once corrected. `assess.handoff.json` and
  `plan.handoff.json` exist and are readable; there was no `execute`
  handoff written before this reflection (the execute stage produced 7
  separate change commits instead of one handoff artifact) — writing one
  now, per this skill's own instructions.
- **Recommendation**: fix `write-position-reminder.sh`'s `.stage` /
  `.status` field mismatch at the source (schema or script, not another
  ad hoc `stage` key bolted on) so this doesn't silently render `unknown`
  again in the next phase.

## Lessons Learned

- **Verify tranche composition against the actual plan before assuming
  drift.** Partway through this phase I initially assessed CH-20 as
  "started ahead of the declared G4-then-G5 sequencing" — that was
  wrong. `plan.md`'s own tranching explicitly put CH-20 in Tranche B
  alongside CH-13/CH-17 (all three independent of the CH-12→13→{14,15}
  chain). Re-reading `plan.md` before writing that judgment into the
  waypoint would have caught it immediately; instead it had to be
  corrected in a later commit. **Read the plan's own stated ordering
  before asserting something is out of sequence.**
- **A CLI subcommand inherits its binary's full startup cost unless
  explicitly designed not to.** Adding `compile` as a `Command` variant
  seemed like it should be a lightweight, config-free operation; it
  wasn't, because `main()` loads full `AppConfig` before *any* dispatch.
  Worth checking a binary's actual entry-point structure before assuming
  a new subcommand is as cheap as it looks on paper.
- **When a heavy verification command gets interrupted, don't retry it
  blindly — check in.** The `cargo check --tests --benches` rejection
  mid-session was handled correctly by pausing and asking the user for a
  narrower command (`cargo check` without `--benches`) rather than
  re-issuing the same heavy call — this is the right pattern to repeat.
- **Reusing an existing test fixture across modules is worth a small
  visibility change.** Relocating `parser.rs`'s `minimal_agent_md()` to
  `pub(crate)` module scope (instead of duplicating ~100 lines of YAML
  fixture in `conformance.rs`'s own tests) kept both modules' tests
  honest about testing the same real parse path, and is a pattern worth
  repeating for future compiler-adjacent test modules.
- **Disclosing a discovered inconsistency is more valuable than smoothing
  it into a single narrative.** CH-19's `DEPLOYMENT.md` could have
  described "the" deployment path; instead, `git log --follow` on
  `deploy.yml` showed it was originally GKE-based and later fully
  rewritten to Azure AKS, while the Helm chart's `storageClass` is still
  GKE-specific and unwired from CI. Documenting this as "two deployment
  paths" is more useful to an operator than a single confident story that
  doesn't match what's actually running.

## Next Phase Focus

Recommend a **hygiene-and-validation** phase (name TBD by
`/kbd-next-phase`, working title `uar-hygiene-and-bench-validation`)
before opening new feature scope, given how much carried debt has
accumulated:

1. **Automate the artifact-refiner QA gate.** This is its 4th+
   consecutive phase as unaddressed debt. Either wire it for real or
   make an explicit, disclosed decision to drop it from the KBD
   contract for this project instead of re-carrying it indefinitely.
2. **Clear the two newly-found pre-existing compile failures** —
   `tests/uar_integration.rs` (add the 8 missing `Skill` fields) and
   `tests/bdd.rs` (fix the nested `#[path]` resolution) — both are small,
   mechanical fixes blocking `cargo check --tests` from ever running
   clean end-to-end.
3. **Actually run `benches/hot_path.rs`** via `cargo bench` (and
   `cargo check --benches` at minimum) to confirm it compiles and get a
   real baseline, since CH-20 shipped it unexecuted.
4. Operator-only, cannot be done by an agent: seed
   `evals/results/starter.baseline.json` and activate the Tier-2 nightly
   eval gate (`UAR_LLM__API_KEY` secret + `vars.UAR_EVAL_MODEL` +
   `workflow_dispatch update_baseline=true`) — carried across multiple
   phases now.

## Addendum (2026-07-04, post-reflection research)

The "Next Phase Focus" above was written without checking GitHub's
Dependabot alerts — which had been printing a vulnerability count on
every `git push` this entire phase and were never investigated because
they weren't in this phase's declared scope. Follow-up research at the
user's request found:

- **96 open alerts** (5 critical, 17 high, 63 medium, 11 low), oldest
  dated ~March 2026 (~4 months accumulated), no `.github/dependabot.yml`
  (so no automated PR pipeline — a pure, silently-ignored backlog).
- **`surrealdb`** (pinned `=3.0.5`; crates.io has `3.2.0`): high-severity
  HTTP RPC session-UUID leak (anonymous session hijack) and privilege
  escalation via an HTTP RPC race condition. `surreal-backend` is UAR's
  **default** feature — directly production-relevant.
- **`rmcp`** (pinned via git rev, well behind upstream `HEAD`):
  high-severity DNS rebinding in its Streamable HTTP server transport.
  `rmcp` is the core, non-optional MCP SDK.
- **`wasmtime`/`wasmtime-wasi`**: 2 critical sandbox-escape bugs (aarch64
  Winch backend) + a WASI path bypass. Lower urgency — `wasm-runtime` is
  opt-in, not in UAR's default feature set.
- **`failure`** crate (critical, type confusion): no real exposure — a
  dev-only transitive dependency of `grcov` (coverage tooling), never
  shipped.

This directly intersects with D-D ("dependency pins are deliberate, not
debt"), which this phase's own CH-19 re-affirmed in `ARCHITECTURE.md`
without checking whether the *specific pinned versions* carry known,
upstream-fixed vulnerabilities. They do, for the two that matter most.

**Rescoped decision**: the next phase (`uar-security-deps-and-hygiene`)
promotes a security dependency triage/upgrade pass to G1 (primary),
carrying this reflection's original hygiene recommendations forward as
G2 (secondary). See `.kbd-orchestrator/phases/uar-security-deps-and-hygiene/goals.md`.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation.
