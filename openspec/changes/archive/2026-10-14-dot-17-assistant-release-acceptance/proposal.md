# Proposal

## Why

Source evidence and existing image publication do not certify an always-on assistant. This change supplies the All acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Assemble a local production-boundary scenario runner and evidence matrix connecting each spec scenario to exact source revision, backend, entry point and observed result. Run one complete always-on journey per profile with memory, schedule, follow-up, connected effect, approval and crash recovery. Keep implementation, product evidence, certification and publication statuses independent.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `product-validation-evidence`: Both required deployment profiles pass before first release. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `tests`, `docs`, `.kbd-orchestrator/phases/dot-feature-support`. Dependencies: dot-01-release-provenance, dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-09-idempotent-settlement, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-12-memory-isolation, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups, dot-16-assistant-capability-surface. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

