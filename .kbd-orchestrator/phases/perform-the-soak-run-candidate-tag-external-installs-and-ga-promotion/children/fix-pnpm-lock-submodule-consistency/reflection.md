# Phase Reflection: fix-pnpm-lock-submodule-consistency

**Project:** universal-agent-runtime
**Date:** 2026-08-20
**Phase completion:** reported per goal; no aggregate percentage
**Changes completed:** 1 / 1

## Delta Between Plan and Delivery

The planned operator lock candidate was frozen-installable but was not the
minimum change it claimed to be. The first independent review compared it
directly with `HEAD` and found two unrelated movements: an existing
`@eslint/config-array` edge moved from `minimatch` 10.2.5 to 10.2.6, and
`y-webrtc` moved from `ws` 8.21.0 to 8.21.1. Restoring both exposed a second
failure that metadata-only validation had hidden: a clean full install needed
the new direct `ws` 8.21.1 package record as well as the preserved 8.21.0
record. The final lock retains both versions for their distinct causal edges.

Evidence work also required correction. The first clean-install receipt used
the dirty worktree, did not execute the root supply-chain validator, and several
receipt blocks did not contain commands capable of producing their recorded
output. The corrected evidence uses a disposable external worktree with empty
dependency directories, records the validator output, and retains exact
replayable commands. Fresh history-free critic and judge reviews both passed.

## Goals

| Goal | Status | Notes |
|---|---|---|
| Make the committed root pnpm lock match the pinned entity-management manifest. | MET for the child candidate | The corrected root lock describes the pinned submodule importer and preserves unrelated resolved edges. Commit and parent resume are handled by the child handoff. |
| Prove frozen installation leaves the lock unchanged. | MET | A clean full frozen install linked 1,345 packages, validated 1,482 supply-chain entries, exited 0, and preserved SHA-256 `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`. |
| Return control to `screen-by-screen-validation` for clean certification. | MET at child exit | The child contains no parent browser run or certification claim. The parent must recertify a new immutable source commit after this child commit. |

## Delivered Change

- `fix-pnpm-lock-submodule-consistency` — corrected the root workspace lock,
  added and synced the frozen-workspace requirement, retained positive and
  negative controls, and archived the OpenSpec change. (by: Codex)

## Technical Debt and Limits

- Install verification disabled lifecycle scripts. It proves frozen dependency
  resolution and the supply-chain lock policy, not install-script behavior.
- Parent browser behavior, generated bundles, release checks, and external
  installation remain unverified by this child.
- OpenSpec 1.5.0 does not address a dated archived change through
  `validate <archive-id> --type change`; archived completeness is checked with
  `validate --archived`, while the active change and synced canonical spec are
  strict-validated separately.

## Architecture Integrity

- AGENTS.md violations introduced: NONE observed.
- Manifest or submodule changes: NONE.
- Scope violations: NONE in the reviewed child candidate. Operator settings,
  the frontend lock, generated static output, parent certification artifacts,
  and `.refiner/registry.json` remain excluded.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact QA | 1 / 1 |
| First independent review | BLOCK |
| Final independent review | critic PASS; judge PASS |
| Refinement iterations | 2 |
| Final blocking constraints | 5 satisfied |

The artifact preserves the initial review failures and the missing-`ws` clean
install failure. Schema validity was supplemented with exact constraint IDs,
chronological checkpoints, active/history identity, source hashes, and literal
receipt replay.

## Cross-Tool Coordination Notes

- Progress tracking had a gap: OpenSpec reached archive before the canonical KBD
  change was transitioned from in progress. The runtime is reconciled before
  child exit.
- The initial child-creation script emitted `child_label: unbound variable`
  after creating the phase. Canonical phase activation recovered the position,
  but the script should treat post-create projection output as fallible.
- History-free review was decisive because the clean regeneration comparison
  alone did not reveal which movements were causal relative to `HEAD`.

## Lessons Learned

- Audit a generated lock candidate directly against `HEAD`; comparison with a
  second regeneration can make shared noncausal drift look intentional.
- A frozen lock-only install is not a substitute for a clean full frozen
  install. The latter can expose missing package records that metadata accepts.
- One direct pin and one unchanged transitive edge can legitimately require two
  versions of the same package. Preserve each edge's causal version.
- Evidence commands must be capable of emitting every line attributed to them.

## Next Phase Focus

Return to the parent Execute phase and resume `/opsx:apply
screen-by-screen-validation`. Certify from the new committed source with empty
dependency directories, frozen installation, fresh processes, and regenerated
source-bound evidence. Do not reuse the pre-child browser bundle.

## Sycophancy Self-Check

- S-08: this reflection leads with the incorrect first candidate and evidence
  defects before reporting goal results.
- S-03: lifecycle-script, browser, generated-bundle, and release limits remain
  explicit.
- S-02: goal results are grounded in the retained commands, outputs, hashes,
  and independent reviews.
- The optional sycophancy-correction tool is unavailable; this manual check is
  recorded instead.
