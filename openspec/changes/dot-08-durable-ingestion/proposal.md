# Proposal

## Why

Accepted uploads retain bytes only in memory and ignore enqueue failure. This change supplies the R6 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Persist immutable source bytes and owner-scoped job metadata in one supported-backend transaction before 202. Workers discover committed jobs independently of the in-memory queue, claim idempotent document/chunk work and expose durable failed/retryable states. Use the database initially; do not add an object store. Preserve upload limits and tenant authorization.

## Capabilities

### New Capabilities

- `dot-durable-ingestion`: Accepted uploads own durable recoverable ingestion work.

### Modified Capabilities

None.

## Impact

Code ownership: `src/uar/api/knowledge.rs`, `src/uar/rag/ingestion_worker.rs`, `src/uar/persistence`, `migrations`. Dependencies: dot-02-principal-credentials. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

