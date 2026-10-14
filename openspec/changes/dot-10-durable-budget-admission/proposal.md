# Proposal

## Why

Enforcing trackers start empty and independent processes can each admit spend. This change supplies the R8 enforcement acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Make aggregate reservation/admission atomic in durable storage with owner scope, limit generation and stable settlement reference. Restore enforcing state before admission, reconcile team/global reservations without duplicate liability and retain unknown spend until reconciliation. Migrate legacy history only where provenance permits; block new budgeted work if the aggregate cannot be established.

## Capabilities

### New Capabilities

- `dot-durable-budgets`: Budget admission uses shared durable reservations.

### Modified Capabilities

None.

## Impact

Code ownership: `src/uar/runtime/cost_budget.rs`, `src/uar/runtime/manager.rs`, `src/uar/compiler/collaboration/team_execution`, `src/uar/persistence`. Dependencies: dot-09-idempotent-settlement. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

