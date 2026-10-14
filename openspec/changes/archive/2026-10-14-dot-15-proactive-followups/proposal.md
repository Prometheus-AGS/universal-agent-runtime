# Proposal

## Why

Existing observers need user-controlled follow-up decisions and durable delivery. This change supplies the D2 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Persist opt-in rules bound to owner/source/grant and authorized conversation. Consume durable occurrences and commit admission/suppression reasons plus delivery outbox. First-release destination is a durable owner conversation inbox; existing external connectors are allowed only when individually declared supported with idempotency/reconciliation and approval policy. No new email/push connectors. Recheck quiet hours, consent, cancellation and current destination authority before dispatch.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-scoped-observers`: Follow-up decisions and delivery remain governed and durable. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/uar/runtime/observer`, `src/uar/api`, `src/uar/persistence`, `src/uar/compiler/collaboration/connector_effect`. Dependencies: dot-13-memory-controls, dot-14-durable-schedules. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

