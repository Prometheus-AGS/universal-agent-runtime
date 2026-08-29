# Phase Reflection: allow-loopback-tools-without-jwt

Project: Universal Agent Runtime
Date: 2026-08-29
Phase completion: 100%
Changes completed: 1/1

## Delta

The plan expected a code freeze before certification and one final quality gate. Delivery required two production-correction rounds, two complete certification restarts, and a final KBD record reconciliation. The first isolated critic found incorrect rollback ownership, a settings-notification race, an incomplete direct-HTTP authorization change, and missing durable evidence. The second found an over-broad HTTP bypass and stale recovery binaries. The third found stale KBD completion and publication claims. Those findings were corrected before the final PASS. PR #274 subsequently merged, so the previous reflection's statement that publication remained pending is obsolete.

## Root Cause

Runtime authority crossed two enforcement boundaries: `RunManager` and the global HTTP Cedar middleware. The initial implementation changed only one. The first correction then bypassed route classification too early, expanding a tool-specific exception into a broader authorization exception.

Governance settings publication and notification had separate owners. A database notification could arrive before the in-process runtime snapshot changed, allowing clients to refetch stale authority.

Rollback documentation conflated posture-derived initialization with persisted operator state. Recovery artifacts also lacked commit-qualified identities. KBD projections then lagged the verified and published state.

## Goal Achievement

| Goal | Result | Evidence |
| --- | --- | --- |
| Eligible exact-loopback, JWT-disabled runtimes default governance Off | MET | Persistence, restart, and installed-runtime evidence |
| Operators can switch governance On or Off from settings | MET | Serialized mutation tests, authoritative revisions, and focused browser coverage |
| Governance Off bypasses governance for tool execution | MET | Deterministic web-search and direct HTTP tool regressions plus installed native-tool execution |
| Ineligible or unverifiable postures fail closed | MET | JWT, non-loopback, ingress, persistence-failure, and mutation-failure coverage |
| The inactive-governance warning appears once per process | MET | Unit coverage and installed warning-cardinality receipt |
| Capability inversion and non-tool authorization boundaries remain intact | MET | Tool-route-only bypass and actor-creation denial control |

Overall goal completion: 100%.

## Delivered Change

One change, `allow-loopback-tools-without-jwt`, completed all 42 OpenSpec tasks and all 9 KBD work packages. It added the verified loopback/JWT-disabled eligibility posture, persisted governance defaulting, the live settings switch, one-warning-per-process behavior, a coherent runtime authority snapshot, serialized mutation confirmation, and the narrowly scoped direct tool-execution bypass. Non-tool actions and all ineligible postures remain governed.

## Corrective Actions

- Shared one coherent governance gate across runtime tool execution and direct HTTP middleware.
- Limited Governance Off bypass to `POST` tool-execution routes.
- Made durable write, cache update, runtime publication, notification scheduling, and response ordering explicit.
- Distinguished API-owned persisted values from posture-owned defaults in rollback behavior.
- Rebuilt forward and rollback binaries from final commits and recorded commit-qualified names and SHA-256 digests.
- Corrected KBD completion, certification, publication, and archive projections after the final evidence existed.

## Artifact Quality Summary

- Changes with phase-scoped artifact-refiner QA: 0/1; no phase-named refiner log exists.
- Changes with independent artifact review: 1/1.
- First-pass independent-review pass rate: 0/1.
- Changes requiring material refinement: 1.
- Material correction rounds: 2.
- Isolated review sequence: FAIL, FAIL, PASS.
- Recurring findings were authority-boundary scope, notification ordering, rollback ownership, evidence retention, recovery identity, and state-accounting accuracy.

## Verification Outcome

The archived verification records 10 requirements and 55 scenarios satisfied. Observed gates included Rust formatting, locked `server-full` compilation, the complete Rust suite, frontend build, 80 files and 406 frontend tests, typecheck, lint, GitHub Actions policy validation, focused Governance Playwright 5/5, support-matrix validation, release-local contracts with six negative controls, strict OpenSpec validation, release builds, rollback proof, installed health/readiness, warning cardinality, governance toggling, tool execution, and non-tool denial. PR #274 merged at 2026-08-28T09:29:58Z.

## Technical Debt and Remaining Risk

- The phase had no live third-party search MCP. Search-specific behavior was proved deterministically; installed live evidence used a native tool. A later runtime release independently completed a live streaming `web_fetch` call, but that is follow-on confirmation, not original phase evidence.
- The current KBD position reminder still names an obsolete apply command even though canonical progress is complete. This is state-projection debt.
- This phase predated mandatory stage handoffs. Its missing execution handoff was backfilled during reflection from the archived 42/42 completion and certification records.
- The Reflect memory-writeback hook resolves a missing external script path and its ingestion hook produced an empty-source wiki entry. The hook remained non-blocking and the phase memory was appended directly; the hook package still needs repair outside this phase.
- Existing PGlite direct-eval build warnings were not introduced by this phase.
- Current optional-integration startup warnings belong to later runtime work and are not phase regressions.

## Architecture Integrity

Capability inversion remains intact: agent kernels did not acquire write authority. The host-owned governance gate controls mutation and execution. The exception is posture-derived, fail-closed, and scoped to tool execution. Business authority remains in explicit runtime state rather than UI-local state. No phase constraints file exists, and the final implementation has no known `AGENTS.md` violation.

## Cross-Tool Notes

OpenSpec archive evidence was the strongest completion record. KBD progress was corrected only after independent review exposed stale projections, and the generated position reminder still lags canonical completion. Future phases should create an execution handoff and update phase completion, publication, and archive projections in the same canonical transition that records their receipts.

## Lessons

1. A policy exception is incomplete until every enforcement boundary uses the same predicate.
2. Settings notifications must follow authoritative runtime publication.
3. Rollback state ownership must distinguish persisted operator intent from posture-derived defaults.
4. Recovery artifacts need commit-qualified identities and hashes.
5. Certification must restart after a production correction.
6. Workflow projections are evidence-bearing artifacts and require the same accuracy review as code.

## Next Phase Focus

Do not invent a successor phase. The next operator-selected phase should first dispose of the remaining clean-startup-log evidence and archive `refresh-liter-surreal-dependencies`, then explicitly configure or disable optional integrations that currently log degraded startup, and repair the stale KBD position projection.

## Context for the Next Phase

`main` and `origin/main` are synchronized and clean. PR #274 is merged and its OpenSpec change is archived. The later integrated runtime release is installed through the LaunchAgent and has separate live evidence for startup skill reconciliation, A2UI rendering, and `web_fetch`. Those later changes should not be folded back into this phase's completion claim.

## Sycophancy Check

The strict Reflect analyzer returned `sycophancy_score: 0.01785714365541935` and `s08_detected: false`. It reported one low-severity S-07 length finding and made no correction in detect-only mode. The complete response is retained in `sycophancy/reflect-2026-08-29T05-58-53Z.json`.
