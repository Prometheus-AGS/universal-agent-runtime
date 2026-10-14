# Proposal

## Why

A storage engine alone does not give assistant users consent and deletion control. This change supplies the D3 controls acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Add owner-visible durable capture policy, provenance and retention controls to MemoryService APIs; new assistant automatic capture defaults off until opt-in. Require explicit retention choice at enablement. Use tombstones to immediately suppress deleted/expired content across records/index/history/queued capture; idempotent purge completes without retaining content in audit logs. Preserve existing explicit user memory behavior.

## Capabilities

### New Capabilities

- `dot-memory-controls`: Assistant capture and retention are owner controlled.

### Modified Capabilities

None.

## Impact

Code ownership: `src/uar/memory/auto_capture.rs`, `src/uar/memory/background.rs`, `src/uar/memory/service.rs`, `src/uar/api/memory.rs`. Dependencies: dot-12-memory-isolation, dot-08-durable-ingestion. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

