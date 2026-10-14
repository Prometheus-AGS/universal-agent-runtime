# Proposal

## Why

Persisted approval rows do not alone reconstruct a safe executable continuation. This change supplies the Approvals acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Connect existing durable approval records to restored run/checkpoint disposition. Expose current pending/expired/invalidated decisions through existing approval reads/events; claim approved continuation once under current owner, binding and policy. Preserve timeout/denial semantics and invalidate stale approval rather than replaying a captured future.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-approval-workflow`: Recovered approvals require current executable authority. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/uar/runtime`, `src/uar/api/routes.rs`, `openspec/specs/tool-approval-workflow/spec.md`. Dependencies: dot-06-fresh-process-resume. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

