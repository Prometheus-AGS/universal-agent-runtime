# Phase Reflection: fix-runtime-settings-namespace-routes

**Project:** universal-agent-runtime
**Date:** 2026-08-25
**Phase completion:** 100% of product goals; repository-wide frontend certification remains limited
**Changes completed:** 1 / 1

## Plan-to-delivery delta

The route defect and installed release were delivered, but execution diverged from the plan in three material ways. The merged origin/main dependency pins exposed stale Assistant UI and RMCP call sites, so the release candidate could not compile until those compatibility sites were repaired. Two requested repository-wide checks did not turn green: pnpm test still reports 12 failures and pnpm frontend:boundaries still reports three findings, all in files unchanged from origin/main and outside the settings transport. Artifact-refiner QA was skipped because the installed adapter lacks its required canonical prompts and schemas, and the independent critic could not be spawned under the session's higher-level policy.

The apparently changed provider UUIDs after restart were outer settings-row proxy IDs, not provider identities. Source inspection showed the Surreal adapter generates those UUIDs on every read. The durable provider data.id/key set, count, config hash, and seeded=0 startup result were preserved.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| Use canonical backend slugs for settings namespace reads | MET | The GET wrapper now uses namespaceToSlug(); four focused tests and the installed browser proof cover plural, hyphenated, unchanged, and non-success behavior. |
| Preserve terminal-run continuity through supported KBD rollover | MET | Upstream commit f1e58b25 is pushed and pinned; the signed successor run and phase are canonical, conflict-free, and no former phase is current. |
| Rebuild, install, and verify the LaunchAgent service | MET | Source and installed binary hashes match; health/readiness are 200; five durable provider IDs remain; installed Playwright passed 1/1 with no settings-route 404. |

## Delivered Changes

- `fix-settings-namespace-read-routes` — canonical settings GET route conversion, focused API and installed-service tests, release compatibility reconciliation, native install evidence, and KBD successor continuity contract (by: Codex; archived at `openspec/changes/archive/2026-08-25-fix-settings-namespace-read-routes/`)

## Technical Debt

- `frontend/src/features/providers/model/providers-store.ts` retains three boundary findings already present in origin/main.
- The merged baseline retains 12 failing provider-store and A2UI tests. This phase did not repair or certify those unrelated surfaces.
- The artifact-refiner installation is incomplete, so this phase has no independent artifact-refiner or artifact-critic receipt.
- The Surreal settings adapter returns transient outer row UUIDs. The durable provider identity contract remains data.id/key, but consumers that treat the outer row UUID as stable would be wrong.

## Architecture Integrity

- AGENTS.md violations: NONE observed in the implementation. KBD projections changed only through canonical commands; the KBD source change was authored in an external worktree and pinned by gitlink; no backend settings alias or provider persistence/payload/save behavior changed.
- Constraint violations: NONE introduced. The operator-owned untracked versions.toml and former prior-context.md remain unmodified and unstaged.
- Scope expansion: release-blocking Assistant UI and RMCP compatibility changes were documented in OpenSpec rather than added silently.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with artifact-refiner QA | 0/1 |
| Changes with executable scoped verification | 1/1 |
| Focused API tests | 4 passed |
| Installed browser scenarios | 1 passed |
| Refinement iterations | 0 |
| QA skip reason | Incomplete refiner package; independent critic prohibited by higher-level session policy |

No recurring artifact-refiner constraint pattern can be computed from zero QA runs.

## Cross-Tool Coordination Notes

- Progress tracking: RELIABLE — canonical revisions 651 through 689 preserve rollover, phase, task, completion, and stage transitions; progress.json reports 1/1 change and 6/6 tasks complete.
- Handoff quality: CLEAR — the operator supplied the route root cause and explicit rollout plan. One environmental hazard caused rework: a repository setup process overwrote the installed rollover CLI with 1.7.0 after installation, so later canonical mutations used the exact built CLI and the installed binary was restored.
- Control plane: unreachable at 127.0.0.1:7892; the canonical runtime committed locally and reported remote status unknown. This is not presented as remote synchronization evidence.
- Recommendation: verify the installed CLI hash after any repository bootstrap before emitting lifecycle commands.

## Lessons Learned

- Settings namespace keys and backend route slugs are separate contracts; perform conversion once at the transport boundary for both reads and writes.
- Compare provider data.id/key across restarts, not the transient outer Settings row UUID generated by the Surreal adapter.
- A clean origin/main merge can still expose dependency API drift. Run the production build early enough to separate migration work from the feature defect.
- Baseline failures must remain visible even when scoped change evidence passes; scoped certification is not repository-wide certification.
- KBD terminal history should continue through a signed successor event, never projection edits.

## Next Phase Focus

Do not create a successor phase in this closeout. Leave the run active with `/kbd-new-phase` as exact next work. Candidate future phases, if the operator selects them:

1. Repair the 12 origin/main provider-store/A2UI test failures and three provider-store boundary findings.
2. Restore the complete artifact-refiner package and independent critic path.
3. Decide whether outer settings-row UUID stability should become an API contract; do not fold that persistence decision into a route fix.

## Context for Next Phase

Use this reflection and `.prometheus/evidence/settings-namespace-routes/` as prior context for the next `/kbd-assess` invocation.

## Sycophancy Self-Check

This reflection leads with deviations, distinguishes scoped success from red repository-wide checks, names the unavailable independent QA, and does not treat transient row UUIDs or local canonical commits as stronger evidence than they are.
