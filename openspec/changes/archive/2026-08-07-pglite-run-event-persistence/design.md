## Context

The frontend already opens one IndexedDB-backed PGlite database and persists
threads/messages through `UarDb`, while runtime entities and normalized AG-UI
rows remain in the in-memory PEM/Zustand graph. C-06 now emits a typed event row
for every accepted official frame and terminal phase timings, which gives C-07
one normalized input but also makes per-token database writes unacceptable.

The installed PEM workspace exposes `createPGlitePersistenceAdapter`,
`startLocalFirstGraph`, and `registerEntityFromSql` through the existing
`platform/entities` facade. Its local-first runtime hydrates a graph snapshot,
subscribes to graph/action changes, persists with a debounce, and can replay
registered pending graph actions. PGlite's official API supports IndexedDB
storage, parameterized queries, multi-statement migrations, and transactions;
no dependency change is required.

Two observed contracts shape the design:

- Several official frames synthesized from one retained runtime event share a
  wire sequence but have distinct event identities. Wire sequence therefore
  cannot be the durable primary key.
- The current backend emits content deltas and run terminal frames but does not
  emit text/reasoning end frames. Coalesced content needs a terminal fallback or
  it would never be persisted on current streams.

## Goals / Non-Goals

**Goals:**

- Add additive, versioned `run` and `run_event` PGlite schema with terminal
  phase timings and deterministic per-run ordering.
- Persist non-content AG-UI rows incrementally and content/reasoning spans once
  per logical span rather than once per token.
- Use the server-assigned run identity when present so later checkpoint/resume
  APIs address the same run.
- Hydrate PEM graph state from PGlite before starting realtime synchronization.
- Generate RuntimeRun and RuntimeAgUiEvent validation schemas from the same SQL
  that creates their tables.
- Adopt PEM's local-first snapshot/action runtime without creating an
  application outbox table.

**Non-Goals:**

- Build the run trace UI, replay controls, or virtualization (C-11).
- Move existing thread/message tables to the singular target schema.
- Introduce Electric sync or a shared worker for multi-tab database ownership.
- Change provider/model request or server event contracts.
- Remove high-frequency event rows from the live entity graph; C-06 explicitly
  requires them there. C-07 bounds only durable `run_event` writes.

## Decisions

### 1. Add migration 002 without rewriting migration 001

Migration 002 creates `run` and `run_event` against the live `threads` and
`messages` table names. `run.phase_timings` is JSONB and `run_event.payload` is
the preserved normalized payload. Rewriting migration 001 would invalidate
already-initialized browser databases and is unnecessary for C-07.

`run_event` stores two order values:

- `seq`: a unique, monotonically increasing persistence ordinal within a run;
- `wire_sequence`: the original AG-UI sequence, which may repeat across frames
  derived from one retained source event.

The primary key is `(run_id, event_id)` and `(run_id, seq)` is unique. This keeps
replay idempotent without dropping frames that legitimately share wire order.
Alternative rejected: `(run_id, wire_sequence)` from the migration-plan sketch,
because C-06 proves that key is not unique.

### 2. Put coalescing in one platform persistence writer

`platform/pglite/run-event-persistence.ts` owns per-stream text/reasoning
buffers keyed by run, content kind, and message identity, and is the only code
that converts `AguiEventRow` into durable rows.
`TEXT_MESSAGE_CONTENT` and `REASONING_MESSAGE_CONTENT` append to memory. Their
matching `*_END` frame persists one aggregate row. `RUN_FINISHED` or `RUN_ERROR`
flushes any remaining aggregate first, covering the current transport that has
no end frames. All other accepted rows persist incrementally in adapter order.

The chat stream store supplies thread context and awaits the writer at durable
boundaries. It does not reinterpret official payload fields. Alternative
rejected: debounce each token independently, because that still produces many
rows and cannot express one logical content span.

### 3. Keep durable order independent from transport reconnects

The database allocates the next per-run `seq` when appending a new event and
uses `(run_id, event_id)` conflict handling for replay idempotency. The official
wire sequence remains queryable as `wire_sequence`. Event kind is a small
normalized value (`lifecycle`, `message`, `reasoning`, `tool`, `state`,
`custom`, or `raw`) used by the later trace query.

When a pre-content transport retry has no server run header, each retry uses a
distinct local fallback run identity. This keeps a failed attempt terminal
without preventing a later successful attempt from recording its own terminal
state. Cancellation is finalized from the awaited stream abort path rather than
as a detached database write.

### 4. Bootstrap local-first storage before realtime sync

Entity engine configuration and schema registration remain synchronous at app
startup. After `UarDb.open()` applies migrations, the database provider creates
PEM's PGlite persistence adapter, starts `startLocalFirstGraph`, and awaits
`runtime.ready`. Only then does it start the existing realtime transport and
render application children.

This ordering prevents an older persisted snapshot from overwriting server
events that arrived during hydration. Initialization is module-idempotent so
React Strict Mode cannot create duplicate graph subscriptions.

### 5. Use PEM pending-action persistence instead of an outbox table

The local-first runtime stores graph snapshot and pending action records in its
PGlite adapter table and replays registered keyed actions with the package's
retry policy. C-07 does not add migration `003-outbox.sql` or application-owned
retry machinery. A poison handler reports only action identity and error text,
never the action input, so secrets are not copied to logs.

### 6. Register schemas from migration SQL through the facade

The SQL constants used by migration 002 are passed to
`registerEntityFromSql` for RuntimeRun and RuntimeAgUiEvent. Existing
`registerSchema` relation metadata stays in place; SQL generation supplies the
field-validation schema rather than duplicating column definitions in another
hand-maintained object. All PEM imports remain centralized in
`platform/entities/index.ts`.

## Risks / Trade-offs

- **[Current streams omit `*_END`]** → Flush buffered spans before every run
  terminal and cover explicit-end plus terminal-fallback behavior in tests.
- **[Persistence failure could otherwise become an unhandled background
  rejection]** → Await durable writer operations in the async stream loop and
  route failures through the existing stream error path.
- **[Local-first hydration could race realtime sync]** → Make hydration a
  prerequisite of sync initialization and test call ordering.
- **[Wire sequence is not unique]** → Preserve it as data, never identity; use
  stable event id plus a distinct persistence ordinal.
- **[Graph snapshots still contain C-06 high-frequency live rows]** → Accept
  this explicit C-06 contract for now; C-07 prevents the separate `run_event`
  table from amplifying it into per-token durable rows.
- **[PGlite IndexedDB is not coordinated across tabs]** → Retain the existing
  single-instance browser contract; shared-worker ownership remains a later
  platform change.

## Migration Plan

1. Add migration 002 and schema constants; existing databases apply it once
   through `schema_migrations`, new databases apply migrations 001 then 002.
2. Add typed run/event queries and the coalescing writer with in-memory tests.
3. Add PEM adapter/runtime bootstrap after PGlite initialization and before
   realtime sync.
4. Wire the normalized chat stream projection to the writer and terminal run
   updates.
5. Verify migration idempotency, coalescing, replay deduplication, hydration
   order, typecheck/lint/boundaries, and focused tests.

Rollback removes the C-07 call sites and local-first bootstrap. The additive
tables and migration row may remain dormant; no existing thread/message data is
rewritten or deleted.

## Open Questions

None. The table identity, coalescing fallback, initialization order, and package
API availability are resolved by the current repository and installed package
sources.
