# Handoff in — Diagnose database-backed readiness latency

Parent: skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture.

## Why this child exists

The operator explicitly requested a diagnostic child after a release deployment
and approved shared SurrealDB restart failed to establish stable UAR readiness.
The Presentation implementation remains accepted (three changes, twelve tasks).
This child does not reopen that implementation or substitute for its pending
spec sync/archive approval and reflection.

## Inputs

- `docs/releases/local-install-2026-09-05.md`: build, installed identity, restart
  order, configuration preservation and observed operational outcomes.
- `.prometheus/session-log.md`: September 5 deployment/recovery chronology.
- `.prometheus/gotchas.md`: readiness versus liveness, database restart ordering,
  and credential-safe fixture handling.
- `versions.toml`: authoritative dependency and architecture pins.
- `src/server.rs`: actual readiness handler, middleware and startup composition.
- `src/uar/persistence/providers/surreal.rs`: actual database adapter operations.
- Parent `assessment.md`, `plan.md`, `execution.md`, and `goals.md`: phase scope
  and acceptance evidence. Early assessment gaps are historical, not current.

Latest recorded baseline, not a fresh probe: UAR liveness returned200 in0.021005s;
readiness exceeded15seconds and then returned408 after30.022509s. Authenticated
WebSocket `RETURN true` passed in5705ms and9109ms; another startup query timed
out at15seconds. SurrealDB PID29223 and UAR PID34726 were left running with
unchanged configuration and database location. Reconfirm identities read-only
before any new diagnostics; never print environment or credential values.

## Success criteria and deliverables

Produce `assessment.md`, then a bounded diagnostic `plan.md` before executing
experiments. Record commands, timings and redacted results in `execution.md`.
Return `diagnosis.md` and `handoff-out.md` with confirmed findings, rejected
hypotheses, unresolved uncertainty and a minimal remediation proposal. Use an
artifact-only independent critic for the diagnostic conclusion. If evidence
cannot distinguish causes without new authority, name the required next action
instead of declaring a root cause. No fix is authorized by creating this child.

## Creation compatibility note

The installed kbd-new-child shell helper strips a legacy trailing childPointer
when choosing artifact paths, but its runtime branch uses activePath.phaseId as
the parent. Those differ at the current nested waypoint. This child is created
with canonical native phase commands and explicit artifacts under the runtime's
active parent, retaining boundary evaluations and the child:before hook. No
generated waypoint/progress projection or external skill script is hand-edited.
