# Deterministic lifecycle management

## Why

Embedded shutdown reaches application cleanup yet retains database locks; concurrent credential migration can lose meaningful transaction outcomes; sidecar fixture workers survive body failure. Detached work, cyclic ownership, inconsistent endpoint parsing, and timer-based readiness obscure completion.

## What Changes

Implement the operator-approved unified lifecycle plan WP1-WP8: trusted host supervision and atomic admission; endpoint and ownership contracts for persistence and memory; pinned engine completion; ordered teardown on all exits; event-driven subscription and work recovery; transactional migration outcomes; hermetic, acknowledged fixtures.

## Capabilities

- `runtime-resource-lifecycle`: explicit owned/borrowed resources, task outcomes, shutdown acknowledgements, and recovery state.
- `credential-migration-outcomes`: atomic receipts, concurrent joining, typed conflict and unknown commit outcomes.
- `realtime-lifecycle-recovery`: retained readiness, explicit gaps, subscribe-before-snapshot and explicit reconnection.

## Impact

Server, runtime, persistence/memory, realtime APIs/consumers, credential stores, sidecar process controls, fixtures and pinned first-party dependency integration. Preserve SurrealDB 3.3.0 and SQLx 0.8.6 bases. Verification is CLOSED; no authoring status certifies behavior.
