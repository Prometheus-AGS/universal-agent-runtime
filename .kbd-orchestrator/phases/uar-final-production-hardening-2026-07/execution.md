# Execution — uar-final-production-hardening-2026-07

Selected backend: Codex + OpenSpec
Active scope: changes 20–24 only
Canonical progress: `progress.json`
Source plan: `plan.md`

> EXECUTION LOCK: The job is 24/24 production completion. Before every action ask whether it directly advances changes 20–24. CI is asynchronous evidence. Never babysit workflows while release work can advance.

## Dispatch

| Change | Execution state | Next conversion |
|---|---|---|
| `align-release-workflow-platforms` | implementation complete | EVIDENCE_PENDING → DONE from immutable candidate artifacts |
| `certify-operational-resilience` | implementation complete | EVIDENCE/TIME_BOUND → DONE from retained resilience and soak reports |
| `produce-supply-chain-artifacts` | implementation complete | EVIDENCE_PENDING → DONE from published and independently verified evidence |
| `certify-release-candidate` | implementation complete | AUTHORIZATION/EVIDENCE/TIME_BOUND → DONE from immutable RC and external validation |
| `release-1-0-0` | implementation complete | AUTHORIZATION/EVIDENCE → DONE from no-rebuild GA and public verification |

## Action filter

Allowed:

- Complete a pending evidence-producing release operation.
- Fix a demonstrated Stable Linux/macOS product defect.
- Reconcile OpenSpec/KBD state with direct evidence.
- Request authorization exactly when an external effect is ready.

Disallowed:

- Polling or narrating CI as the primary activity.
- Restarting the full matrix for cosmetic or Experimental Windows status.
- Reopening changes 1–19 without a demonstrated regression.
- Adding speculative features, tests, or refactors.
- Running broad compilation during implementation instead of the agreed `cargo check` checkpoint.

## Evidence handling

- Runs are asynchronous; inspect them after completion or on a failure notification.
- Bind evidence to one immutable source SHA.
- Do not mark time-bound or externally authorized tasks complete early.
- Do not leave implemented work labeled merely `PENDING`; use progress notes to expose the real state.

## Approval gates

- Merge PR #98.
- Create candidate/GA tags.
- Publish GitHub release and GHCR artifacts.
- Accept external-install and operating-period evidence.

## Blockers

No known implementation blocker. External/time-bound evidence and publication authorization remain.

## Reflection handoff

Reflection must verify 24/24, immutable SHA equality, supported-platform evidence, public artifact integrity, and explicit retention of Windows as Experimental.
