# Proposal

## Why

Resume requires a live run before loading its checkpoint; unsupported methods can succeed as no-ops. This change supplies the R5 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Persist a complete owner-bound resume envelope atomically with the checkpoint and load it before live-manager lookup. Reconstruct through host admission using protected history/current bindings and explicit host secret/resource reattachment. Replace persistence no-op defaults with unsupported errors; never deserialize runnable authority from caller input. Before reconstruction can execute, acquire a durable CAS claim bound to source run, checkpoint revision and executor incarnation; duplicate resume returns the same continuation identity or an explicit conflict, never new execution. Reuse the existing epoch/ownership vocabulary. A predecessor that may still execute or has uncertain effects remains blocked until trusted exclusion/reconciliation; change 11 supplies abrupt-owner exclusion, so 06 alone does not certify abrupt takeover.

**BREAKING for unsafe legacy behavior:** callers can no longer rely on subject-only ownership, volatile production authority or successful no-op durability; migration and explicit unsupported outcomes are required.

## Capabilities

### New Capabilities

- `dot-checkpoint-resume`: Checkpoints restore authorized execution in a fresh process.

### Modified Capabilities

None.

## Impact

Code ownership: `src/uar/api/routes.rs`, `src/uar/runtime/manager.rs`, `src/uar/persistence/mod.rs`, `src/uar/persistence/providers/surreal.rs`, `tests/test_checkpoint.rs`. Dependencies: dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

