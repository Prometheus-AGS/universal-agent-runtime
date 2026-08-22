# Phase Reflection: fix-frontend-pnpm-lock-consistency

**Project:** universal-agent-runtime
**Date:** 2026-08-20
**Phase completion:** reported per goal; no aggregate percentage
**Changes completed:** 1 / 1

## Delta Between Plan and Delivery

The implementation remained lock-only, but the evidence required four
refinement iterations rather than the planned single convergence pass. The
first candidate receipts did not bind every displayed result to an executable
command. The first causal classifier then assigned all dependency movements to
the same 44-edge fallback, which made its zero-unclassified result true but not
informative. Replacing that fallback exposed three pnpm auto-peer projections
whose lock importer sections do not exist in the current manifests. The final
classifier resolves 41 direct importer edges and three removed auto-peer
projections against their actual `peerDependencies` and replacement
`devDependencies` paths.

OpenSpec archive validation also differs from active validation in version
1.5.0. The active change and canonical spec passed strict validation before
archive. After archive, the dated change is checked through `validate
--archived`; addressing it as an active change reports no delta even though the
archived delta file is retained. The task wording was corrected to reflect the
CLI's actual contract.

## Root Cause

The stale nested lock was treated as if the root workspace lock covered every
pnpm execution root. It does not. `frontend/` is an independently active pnpm
workspace whose importer graph includes pinned submodule manifests, so its lock
must be reconciled and frozen-tested independently.

The evidence defects came from proving resolver determinism before proving
causality. Two identical regenerations show repeatability, but do not show that
every changed record was required by the source delta. Pnpm's auto-installed
peer projections added a second trap: importer keys cannot always be mapped
back to the same manifest section by name alone.

## Corrective Actions

- Reconciled only `frontend/pnpm-lock.yaml` and retained unrelated common
  resolutions.
- Added an exact accepted-to-raw resolver patch and a fail-closed classifier for
  all 693 lock mutations.
- Resolved every named edge against its actual manifest section, including the
  three removed auto-peer projections.
- Ran nested frozen metadata and clean empty-dependency-tree installs, then
  Tier 0 and the focused SSE unit test while hashing both locks.
- Retained the stale-lock negative control and all intermediate review failures.
- Required independent history-free critic and judge PASS decisions before
  archive.

## Goals

| Goal | Status | Notes |
|---|---|---|
| Make the nested frontend lock reproducible under pnpm 11.15.0. | MET | Frozen lock-only and clean empty-dependency-tree installs exited 0 and retained nested SHA-256 `43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593`. |
| Preserve unrelated dependency resolutions. | MET | All 693 mutations are classified; common package bodies have 0 mutations and only 3 common snapshot-body changes have manifest causes. |
| Return control to `screen-by-screen-validation`. | MET at child exit | The child contains no parent browser result. Parent certification must start from the new immutable child commit and mint fresh evidence. |

## Delivered Change

- `fix-frontend-pnpm-lock-consistency` — reconciled the independently active
  frontend workspace lock, synced the frozen-workspace requirement, retained
  positive and negative controls, and archived the OpenSpec change. (by: Codex)

## Technical Debt and Limits

- Lifecycle scripts were disabled for dependency installation. The child proves
  dependency resolution, not install-script behavior.
- Parent browser behavior, generated bundles, release checks, and external
  installation remain unverified by this child.
- The historical raw resolver bytes are reconstructed from the retained exact
  patch because the scratch worktrees were later reused.

## Architecture Integrity

- AGENTS.md violations introduced: NONE observed.
- Manifest, product-source, root-lock, and submodule changes: NONE.
- Scope violations: NONE in the reviewed child candidate. Operator settings,
  generated static output, parent certification artifacts, and
  `.refiner/registry.json` remain excluded.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact QA | 1 / 1 |
| Initial independent review | BLOCK |
| Final independent review | critic PASS; judge PASS |
| Refinement iterations | 4 |
| Final blocking constraints | 5 satisfied |

The retained artifact includes the rejected receipt and classifier versions,
eleven chronological checkpoints, exact hashes, schema validation, and
byte-identical active/history persistence.

## Lessons Learned

- Validate the lock belonging to the command's actual workspace root; a green
  root frozen install does not validate a nested lock.
- Deterministic regeneration is not causal proof. Classify every lock mutation
  against the source delta and retain unrelated resolutions.
- Pnpm importer projections can originate from auto-installed peers. Resolve
  evidence anchors against the manifest section that actually declares them.
- A receipt command must be capable of producing every output line attributed
  to it.

## Next Phase Focus

Return to the parent Execute phase and resume `/opsx:apply
screen-by-screen-validation`. Create certification from the new child commit
with frozen dependency installation, fresh processes, and regenerated
source-bound evidence. Do not reuse the pre-child browser bundle.

## Sycophancy Self-Check

- S-08: this reflection leads with the evidence and classifier failures.
- S-03: install-script, browser, generated-bundle, release, and external-install
  limits remain explicit.
- S-02: each goal result names retained commands, outputs, or hashes and the
  independent reviews.
- The strict Reflect analyzer reported no S-08 inversion. It returned score
  `0.01785714365541935` with one low-severity S-07 length observation; no
  factual or structural correction was required.
