# Proposal

## Why

Detached ledger writes can outlive terminal completion and have no idempotency identity. This change supplies the R8 settlement acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Replace detached acceptance with durable idempotent settlement keyed by stable run/call/attempt and scope identity. Preserve per-call/team charging without double counting. Separate execution outcome from accounting state; a persistence outage leaves durable reconciliation work or visible unresolved accounting, never an invented settlement.

## Capabilities

### New Capabilities

- `dot-spend-settlement`: Usage settlement has stable durable identity.

### Modified Capabilities

None.

## Impact

Code ownership: `src/uar/runtime/run_cost.rs`, `src/uar/persistence`, `migrations`, `tests/run_cost_settlement.rs`. Dependencies: dot-02-principal-credentials, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

