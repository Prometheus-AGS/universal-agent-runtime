# Phase Reflection: handle-kbd-uninitialized-runtime

**Project:** universal-agent-runtime
**Date:** 2026-08-25
**Phase completion:** 100%
**Changes completed:** 1 / 1

## Delta From Plan

- The planned upstream base was `f1e58b25`; execution fetched current upstream `main` at `1308e4b7`, which already contained that rollover commit through merged PR #67. The review branch therefore started from the newer conflict-free mainline rather than replaying an obsolete base.
- Importing a legacy phase before the first typed mutation exposed a stale local-state comparison inside `migrate_legacy_ledgers(true)`. The one-line state refresh was necessary so the projection-ahead guard compares against the just-committed import. This is a narrow migration-path correction beyond the plan's wording, but the alternative was a first mutation that initialized a run and then still rejected `stage enter` because its phase did not exist.
- Upstream PR #68 and UAR PR #272 both merged during phase closeout. The latter automatically closed issue #265, preserving the issue as audit history.

## Root Causes

- Mutation commands shared a replay-only precondition with inspection commands, so a valid identity-only registration could never cross the signed initialization boundary.
- Canonical empty state has two observable representations—`NotInitialized` and revision zero—but the legacy-aware initializer handled neither consistently.
- Legacy import committed canonical state without refreshing the local snapshot used by the projection-ahead guard.
- KBD completion dimensions are scoped to the run, not a phase, so summaries from the prior phase remained visible after creating this phase in the same run.

## Corrective Actions

- Split mutation initialization from read-only replay and cover both empty-state representations.
- Import compatible legacy phase state before the requested command, then compare projections against the returned canonical state.
- Reuse one process fixture for debug, release, and installed binaries, including non-zero rejection behavior.
- Replace stale completion summaries through typed KBD events and record the cross-phase projection behavior as a follow-up lesson.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| Resolve GitHub issue 265 with evidence | MET | Both review layers merged, the installed CLI passes 3/3 issue scenarios, UAR `main` pins the fix, and issue #265 closed automatically. |
| Preserve registered project history and typed command correctness | MET | Status stays non-mutating, one signed initialization/import boundary precedes the mutation, later commands reuse the run, and rejected commands exit non-zero without a committed command revision. |
| Close the issue only after the reported failure modes are handled or proven obsolete | MET | The issue is retained as audit history and will close through `Fixes #265` only when UAR PR #272 merges. |

## Delivered Changes

- `fix-kbd-uninitialized-runtime` — upstream mutation-only initialization, safe legacy import state refresh, actionable status/errors, process tests, release installation, exact UAR pin, OpenSpec contract, and evidence (by: Codex).

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with artifact-refiner QA | 0 / 1 |
| Strict OpenSpec verification | 2 / 2 repositories |
| Focused installed process scenarios | 3 / 3 |
| Refinement iterations | 1 missing-scenario correction |

Artifact-refiner was not invoked because the active environment prohibited subagent dispatch. The OpenSpec verification pass found the missing empty-project scenario before publication; that scenario was added and passed against both source and installed binaries.

## Technical Debt

- The upstream broad `kbd-runtime` repository-ledger fixture remains red on clean `origin/main` because aggregate-only legacy completion cannot be safely reconstructed. This phase preserved the guard and documented the baseline rather than weakening it.
- Unix-socket CLI transport remains optional and deferred; canonical local fallback is still the correctness path.

## Architecture Integrity

- AGENTS.md violations: NONE. KBD implementation was changed and reviewed upstream; UAR records only the exact gitlink.
- Constraint violations: NONE. `versions.toml` and the former phase `prior-context.md` remained unstaged and unmodified.
- Capability inversion: preserved. Signed runtime mutations remain in the trusted CLI/runtime host layer.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND — completion summaries are run-scoped, so the new phase initially projected settings-phase Evidence, Certification, and Publication summaries. Typed completion events corrected them; no waypoint was hand-edited.
- Handoff quality: CLEAR — Codex, Claude Code, Cursor, and OpenCode KBD payloads are byte-equivalent excluding `.DS_Store`, and all share the installed host CLI.
- Recommendation: phase creation should explicitly reset or re-scope non-implementation completion dimensions when several phases share one run.

## Lessons Learned

- A registered canonical runtime can appear as either `NotInitialized` or revision zero; mutation preconditions must treat both as the same empty state.
- After an import commits, every subsequent safety comparison must use the returned canonical state rather than the pre-import snapshot.
- An installed-binary proof should reuse the exact compiled-process scenarios through an explicit binary-path override.
- Full-suite failures need a clean-main comparison before they are attributed to a patch; preserve data-loss guards even when repository fixtures are stale.
- KBD completion dimensions are run-scoped, so a new phase in the same run can inherit truthful-but-wrong summaries unless explicitly refreshed.

## Next Phase Focus

No new implementation phase is required for issue #265. Both dependency layers are merged, the default-branch gitlink resolves to upstream commit `602750ec`, and the issue is closed. Use `/kbd-new-phase` for unrelated work.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation. The installed CLI already contains the fix; `sovereign-sync` was unchanged and must not be restarted for this issue.
