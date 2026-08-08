## 1. Schema and Database Contract

- [x] 1.1 Add migration 002 SQL for run and run-event records with phase timings, stable event identity, persistence ordinal, wire sequence, and required indexes.
- [x] 1.2 Extend UarDb with typed run upsert/terminal update, idempotent ordered event append, run listing, and per-run event reads.
- [x] 1.3 Add focused migration/database tests for additive idempotency, repeated wire sequences, duplicate event identities, and offline ordered reads.

## 2. PEM Local-First Bootstrap

- [x] 2.1 Extend the platform entity facade and register RuntimeRun/RuntimeAgUiEvent field schemas from migration SQL without removing existing relation metadata.
- [x] 2.2 Initialize the PEM PGlite adapter and local-first runtime after database migration/hydration and before realtime sync, with module-idempotent lifecycle handling.
- [x] 2.3 Add focused bootstrap tests proving graph hydration precedes realtime subscription and no application outbox schema or replay loop is introduced.

## 3. Run Event Persistence

- [x] 3.1 Implement the platform run-event persistence writer with normalized kinds, server run identity, replay idempotency, and terminal run updates.
- [x] 3.2 Coalesce text/reasoning content in memory and flush once at explicit end or current-transport terminal fallback.
- [x] 3.3 Add focused writer tests for content coalescing, terminal fallback, same-wire-sequence frames, RAW preservation, and terminal phase timings.

## 4. Stream Integration and Verification

- [x] 4.1 Wire the chat stream store to await durable normalized event boundaries and preserve its existing message/entity reduction behavior.
- [x] 4.2 Pass frontend typecheck, lint, architecture boundaries, focused tests, strict OpenSpec validation, artifact refinement, diff integrity, and isolated adversarial review.
