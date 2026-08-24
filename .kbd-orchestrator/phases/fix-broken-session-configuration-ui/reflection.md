# Phase Reflection: fix-broken-session-configuration-ui

**Project:** Universal Agent Runtime, with one controlled Prometheus Entity Management upstream change
**Date:** 2026-08-23
**Phase completion:** all four scoped changes delivered; no aggregate readiness percentage is reported because the phase plan prohibits one
**Changes completed:** 4 / 4

## Delta from plan

The product scope did not expand, but execution exposed four process deltas. First, the signed `v3.0.2` tag resolves to `f29a701649799df3ff64f5f986e3c016246d34b6`, while the implementation branch started from current `origin/main` `e25210010a8eb4e575f7e4fc6e04be598a8c8213`, whose public manifests are also 3.0.2. The negative control was corrected to run from a clean detached checkout of the signed tag after the independent judge rejected evidence tied only to the untagged main SHA. Second, the upstream implementation needed adversarial corrections for GraphQL partial commits, view-projection ordering, duplicate-ID merge origins, and missing-entity lifecycle completion. Third, the KBD control plane at port 7892 was unavailable, so the canonical runtime committed locally and regenerated noisy projections for unrelated legacy phases. Fourth, the second UAR OpenSpec archive initially refused to proceed because its MODIFIED block would have dropped the dependency-drift scenario added by the first archive; the later delta was amended to preserve the scenario before archiving.

## Root causes

- A version label alone was insufficient provenance because the signed tag and current 3.0.2-manifest branch point were different commits.
- The original upstream design treated list ingestion mainly as a publication-count problem; critic review showed that atomicity also had to cover descriptor batches, projections, lifecycle, and pre-ingestion merge origins.
- Earlier KBD task registration mixed canonical ordinal IDs with display-label IDs, leaving cancelled duplicate task records even though their canonical tasks and changes completed.
- Serial OpenSpec MODIFIED deltas targeted the same requirement, so the later delta had to carry forward the scenario added by the earlier archive.

## Corrective actions completed

- Retained a SHA-validating signed-release reproduction script and its observed mismatch failure.
- Added one public core atomic ingestion action and routed every classified core, React, GraphQL, and PGlite list path through it.
- Kept UAR on exact registry Entity Management/Core 3.0.2 and isolated the upstream correction as proposed 3.0.3 in PR #41.
- Added persistent UAR architecture rules and deterministic negative fixtures for render-body setters, facade bypass, per-row graph mutations, and duplicate entity caches.
- Reconciled all four changes into canonical KBD DONE state and archived their OpenSpec deltas into the appropriate main specifications.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| Adopt the reviewed Entity Management release | MET | UAR resolves exact registry `@prometheus-ags/prometheus-entity-management` 3.0.2 and `@prometheus-ags/entity-graph-core` 3.0.2 through one application core singleton. |
| Make Session Configuration responsive and effective | MET | The installed sheet opened in 81 ms, used the configured-model projection with no `/api/models` request, persisted save/reopen, preserved cancel isolation, and routed genuine inference according to turn, session, then agent precedence. |
| Correct sheet spacing | MET | Computed styles observed 16 px horizontal padding and a 24 px content gap at 320, 768, 1024, and 1440 pixels. |
| Prevent recurrence | MET | Repository instructions and the local boundary scanner reject the observed React/entity failure shapes; 19 forbidden fixtures failed for their intended rules and allowed fixtures passed. |
| Fix the general fetched-list defect at its owner | MET WITH LIMIT | Upstream source now emits one successful ingestion publication for 1, 12, and 7,248 rows and fails closed on a later side-batch error. PR #41 is open. No npm publication or UAR adoption of 3.0.3 occurred. |

## Delivered changes

- `adopt-entity-management-3-0-2` — exact released dependency and singleton lockfile baseline (by: Codex)
- `repair-session-configuration-entity-flow` — registered Provider, Model, AgentSession, and AgentSessionDraft domain flow plus effective backend model routing (by: Codex)
- `prevent-session-configuration-regressions` — standing instructions, deterministic architecture controls, and one bounded installed-service proof (by: Codex)
- `fix-atomic-fetched-list-ingestion` — framework-neutral atomic ingestion, binding migrations, fixed-group 3.0.3 version metadata, and upstream PR #41 (by: Codex)

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with artifact-refiner runtime logs | 0 / 4 |
| Changes with row-form verification | 4 / 4 |
| Changes with independent adversarial review where required | 3 / 3 |
| First-pass adversarial approval | 0 / 3 |
| Final critic/judge disposition | PASS / APPROVE |

The artifact-refiner PMPO runtime was unavailable, so no `.refiner/artifacts/<change>/refinement_log.md` is claimed. The execution used the repository-approved direct static substitute, strict OpenSpec validation, observed functional evidence, and history-blind critic/judge review. Earlier review failures are retained as negative evidence rather than collapsed into the final approval.

## Technical Debt and Limits

- UAR intentionally remains on released 3.0.2. The upstream fix is not available to UAR until PR #41 merges, a 3.0.3 package is published, and a separate reviewed dependency change adopts it.
- KBD retains cancelled duplicate display-label task records for two completed changes. The changes are canonically DONE, but raw task counters display 5/6 and 10/11. This is projection history, not missing implementation.
- Canonical `position.json` records this phase COMPLETE at revision 569, while the generated `current-waypoint.json` cursor and `exactNextCommand` still point at this phase's Execute stage. The stale projection was not hand-edited.
- The upstream repository-wide `openspec validate --all --strict` reports five unrelated pre-existing v4 change failures. The affected `v3-framework-neutral-core` spec passes strict validation.
- No Linux, Windows, mobile, remote-host, non-`server-full`, non-Chromium, soak, load, or general performance claim was made.

## Architecture Integrity

- AGENTS.md violations introduced: NONE observed.
- GitHub Actions policy violations: NONE; product verification ran locally and workflows were unchanged.
- Entity ownership: graph-backed records flow through registered platform contracts and domain hooks; Zustand remains limited to transient UI/process state.
- Unrequested product changes: NONE.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND — upstream OpenSpec truth was complete before UAR KBD task projections were reconciled, and earlier display-label task IDs polluted raw counters.
- Handoff quality: CLEAR for repository/worktree ownership and evidence locations; weak for canonical task-ID syntax.
- Corrective recommendation: always register and transition KBD tasks by canonical ordinal, and record the upstream commit/tag distinction before the first negative-control run.

## Lessons Learned

- A release negative control must validate checkout HEAD, signed tag commit, and supplied SHA before importing source.
- Atomic list ingestion is a semantic transaction, not merely a batched notification: primary rows, side descriptors, lifecycle, sync state, lists, and projections must commit together.
- Entity Management must own server business records; feature components should subscribe narrowly through domain hooks and keep only disposable widget state locally.
- Serial OpenSpec MODIFIED deltas against one requirement must be rebased conceptually before archive or the archive gate will correctly reject scenario loss.
- A cancelled duplicate task can make generated counters look incomplete after the actual change is complete; canonical task ordinals prevent this ambiguity.

## Next Phase Focus

No automatic product phase is started. After PR #41 merges and 3.0.3 is published, use a separate reviewed dependency-adoption change if UAR should consume it. Until then, UAR's supported dependency remains exact registry 3.0.2.

## Context for Next Phase

Use the signed 3.0.2 negative-control evidence, the installed-session browser report, the archived OpenSpec specifications, and this reflection as prior context. Do not infer that upstream 3.0.3 has been published or adopted.
