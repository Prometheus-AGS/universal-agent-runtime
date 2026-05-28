# entity-surreal-live-adapter Specification

> **Reconciliation note.** This spec replaces an earlier draft that targeted a `SyncAdapter.start/stop` lifecycle and a global single-stream subscription model. The corrected spec below matches the actual `RealtimeAdapter` contract in `prometheus-entity-management/src/adapters/types.ts` and the per-`ChannelConfig` registration model used by `RealtimeManager.register(adapter, channels[], normalize?)`. The superseded text is preserved under `openspec/changes/archive/2026-05-27-ssed-entity-surreal-live-adapter/specs/entity-surreal-live-adapter/spec.md`. Capability ID is unchanged.

## Purpose

A `SurrealLiveAdapter` for `prometheus-entity-management` that implements the existing `RealtimeAdapter` contract using SurrealDB `LIVE SELECT`. The adapter is registered via `RealtimeManager.register(adapter, channels[], normalize?)`; each `ChannelConfig` becomes one independent `LIVE SELECT` subscription. The adapter performs an initial `SELECT` seed for each channel before opening the live subscription, maps SurrealDB live actions (`CREATE | UPDATE | DELETE | CLOSE`) to `EntityChange` objects, surfaces connection state via the optional `onStatusChange` callback, reconnects with exponential backoff on transient failures, and supports optional checkpoint-based replay on reconnect. A companion `entity-realtime-surreal-live` skill in `prometheus-skill-system/skills/react/prometheus-entity-skills/` documents the wiring patterns for end-user apps.

## Requirements

### Requirement: Adapter Factory
The package SHALL export `createSurrealLiveAdapter(opts: SurrealLiveAdapterOptions): RealtimeAdapter` from `src/adapters/surreal-live.ts`, re-exported from `src/index.ts`. The returned object MUST satisfy `RealtimeAdapter` (not `SyncAdapter`) — SurrealDB driver query/execute access is the consumer's responsibility, not the adapter's.

#### Scenario: Public export
- **WHEN** a consumer imports `createSurrealLiveAdapter` from `@prometheus-ags/prometheus-entity-management`
- **THEN** the import MUST resolve and the returned object MUST satisfy the `RealtimeAdapter` interface (`name: string`, `subscribe(config, handler) → UnsubscribeFn`, optional `onStatusChange`).

#### Scenario: Adapter name
- **WHEN** a consumer reads `adapter.name`
- **THEN** the value MUST be a non-empty string. The default value (when `opts.name` is not supplied) MUST be `"surreal-live"`. A consumer-supplied `opts.name` overrides the default.

#### Scenario: Type exports
- **WHEN** a consumer imports `SurrealTableConfig` and `SurrealLiveAdapterOptions` types
- **THEN** both MUST be re-exported as type-only exports from the package root.

#### Scenario: Manager registration
- **WHEN** a consumer calls `manager.register(adapter, channels, normalize?)`
- **THEN** the manager MUST invoke `adapter.subscribe(...)` exactly once per `ChannelConfig` entry in `channels[]`, and each returned `UnsubscribeFn` MUST be tracked for later cleanup.

### Requirement: Per-Channel Subscription Model
Each `ChannelConfig` SHALL produce one independent SurrealDB `LIVE SELECT` subscription. The adapter MUST NOT collapse multiple channels into a single SurrealDB subscription, even when their `type` (table) is identical.

#### Scenario: One LIVE SELECT per channel
- **WHEN** the adapter is registered with `channels: [{type: "user"}, {type: "task"}]`
- **THEN** the adapter MUST issue two independent `LIVE SELECT` queries (one per channel) and MUST maintain two independent reconnect/replay state machines.

#### Scenario: Channel filter clause
- **WHEN** a `ChannelConfig` includes a `filter` object (e.g. `{type: "task", filter: {project_id: "abc"}}`)
- **THEN** both the initial seed `SELECT` and the `LIVE SELECT` MUST include a `WHERE` clause derived from the filter; the same filter is reused on reconnect.

#### Scenario: Channel id-scoped subscription
- **WHEN** a `ChannelConfig` includes `id: "<entity-id>"` without a `filter`
- **THEN** the subscriptions MUST be scoped to that single record id (e.g. `LIVE SELECT * FROM user:<id>`).

#### Scenario: UnsubscribeFn cleanup
- **WHEN** the `UnsubscribeFn` returned from `subscribe(...)` is invoked
- **THEN** the adapter MUST `KILL` the corresponding SurrealDB live query, stop any in-flight reconnect backoff for that channel, and release per-channel resources without affecting other channels' subscriptions.

### Requirement: Initial Seed via First Handler Invocation
The adapter SHALL hydrate the graph with current rows by emitting an initial `ChangeSet` through the subscription's `handler` callback before live deltas begin, when `initialQueryStrategy` is `select-then-live` (the default).

#### Scenario: Default seeding
- **WHEN** `subscribe(config, handler)` is called with `opts.initialQueryStrategy` unspecified or `"select-then-live"`
- **THEN** the adapter MUST execute `SELECT * FROM <table>[ WHERE <filter>]` once, convert each row to an `EntityChange` with `op: "insert"` and the table name as `type`, batch them into a single `ChangeSet`, and invoke `handler(changeset)` once with that ChangeSet **before** any live delta is emitted.

#### Scenario: Empty seed
- **WHEN** the initial `SELECT` returns zero rows
- **THEN** the adapter MUST still invoke `handler(changeset)` once with a `ChangeSet` whose `changes` array is empty, so downstream consumers can observe "seeded with nothing" as distinct from "not yet seeded".

#### Scenario: Live-only mode
- **WHEN** `opts.initialQueryStrategy: "live-only"` is supplied
- **THEN** the adapter MUST NOT execute the initial `SELECT`; only the live subscription is opened.

#### Scenario: Ordering guarantee
- **WHEN** the adapter is in `select-then-live` mode and live actions begin arriving during the seed query
- **THEN** the adapter MUST buffer those live actions until the seed `ChangeSet` has been delivered, then flush the buffered actions in arrival order — no live delta may reach the handler before the seed.

### Requirement: Action Payload Mapping
The adapter SHALL map SurrealDB live notifications (`CREATE | UPDATE | DELETE`) to `EntityChange` objects with the documented `op`, `type`, `id`, and `data` / `patch` fields.

#### Scenario: CREATE → insert
- **WHEN** the SurrealDB driver emits a live notification with `action: "CREATE"`, `result: <full-row>`
- **THEN** the adapter MUST emit `{ op: "insert", type: <table>, id: <record-id>, data: <full-row> }` inside a `ChangeSet` delivered to the channel's handler.

#### Scenario: UPDATE → update
- **WHEN** the SurrealDB driver emits a live notification with `action: "UPDATE"`, `result: <full-row>`
- **THEN** the adapter MUST emit `{ op: "update", type: <table>, id: <record-id>, data: <full-row> }`. The full row is provided (SurrealDB live notifications carry the whole record), not a diff; `patch` is left undefined.

#### Scenario: DELETE → delete
- **WHEN** the SurrealDB driver emits a live notification with `action: "DELETE"`, `result: <record-with-id>`
- **THEN** the adapter MUST emit `{ op: "delete", type: <table>, id: <record-id> }`. The `data` field MAY be the last-known row or `undefined`; consumers MUST treat `op === "delete"` as authoritative regardless.

#### Scenario: CLOSE / unknown action
- **WHEN** the SurrealDB driver emits a notification with `action: "CLOSE"` or an unrecognised action string
- **THEN** the adapter MUST NOT emit a `ChangeSet`. For `CLOSE` specifically, it MUST treat the channel as disconnected and enter the reconnect path (see Reconnection requirement). For unknown actions, it MUST log a warning and skip.

#### Scenario: Table → EntityType derivation
- **WHEN** mapping a notification to `EntityChange.type`
- **THEN** the adapter MUST use `ChannelConfig.type` (the configured `EntityType`), not the raw table name parsed from the SurrealDB record id, so consumers can use a different `EntityType` label than the table name when the optional `normalize` argument is in play.

### Requirement: Status Surface via onStatusChange
The adapter SHALL expose connection state through the optional `onStatusChange` callback declared in `RealtimeAdapter`.

#### Scenario: Status callback registration
- **WHEN** a consumer (or the `RealtimeManager`) calls `adapter.onStatusChange(cb)`
- **THEN** the adapter MUST register the callback and return an `UnsubscribeFn` that detaches it.

#### Scenario: Status values
- **WHEN** the adapter transitions between connection states
- **THEN** it MUST invoke every registered status callback with one of `"connecting" | "connected" | "disconnected" | "error"`, matching the `AdapterStatus` enum in `types.ts`.

#### Scenario: Per-adapter, not per-channel
- **WHEN** any one of the adapter's channels disconnects
- **THEN** the adapter's status MUST be the worst state across all channels — `"disconnected"` or `"error"` if any channel is in that state; `"connecting"` if any is mid-reconnect with all others healthy; `"connected"` only when every channel is healthy. A single status stream represents the adapter as a whole.

### Requirement: Reconnection with Backoff
The adapter SHALL recover from transient SurrealDB connection failures on a per-channel basis with exponential backoff.

#### Scenario: Backoff schedule
- **WHEN** a channel's live subscription drops due to a transient error
- **THEN** the adapter MUST schedule a reconnect after `min(initialDelayMs * 2^attempt, maxDelayMs)` jittered by ±25%, where `initialDelayMs` defaults to 500 and `maxDelayMs` defaults to 30000. Defaults MAY be overridden via `opts`.

#### Scenario: Reconnect re-issues subscription
- **WHEN** reconnect succeeds
- **THEN** the adapter MUST re-issue the same `LIVE SELECT` (same table + filter + id scope as the original `ChannelConfig`) and emit `AdapterStatus = "connected"`.

#### Scenario: Attempt counter resets on success
- **WHEN** a channel has been `"connected"` for at least `connectedSettleMs` (default 30000)
- **THEN** the per-channel attempt counter MUST reset to 0 so the next disconnect starts fresh from `initialDelayMs`.

#### Scenario: Permanent error
- **WHEN** the SurrealDB driver reports an authentication or schema error (non-transient)
- **THEN** the adapter MUST NOT attempt to reconnect, MUST emit `AdapterStatus = "error"`, and MUST surface the error message to any registered `onStatusChange` callback in a single error-state notification.

### Requirement: Checkpoint Replay on Reconnect
The adapter MAY support a checkpoint-based replay on reconnect to recover changes that occurred while disconnected, when the consumer supplies a per-channel checkpoint store.

#### Scenario: Replay opt-in
- **WHEN** `opts.checkpointStore` is provided
- **THEN** the adapter MUST persist a per-channel checkpoint value (e.g. `updated_at`) on each successfully-delivered `EntityChange`, and on reconnect MUST execute `SELECT * FROM <table> WHERE <filter> AND updated_at > <stored>` before opening the new `LIVE SELECT`, emitting the recovered rows as an initial `ChangeSet` for that channel.

#### Scenario: Replay opt-out
- **WHEN** `opts.checkpointStore` is absent
- **THEN** the adapter MUST NOT attempt replay on reconnect. The seeded subscription resumes from the moment of reconnect; any deltas that occurred during the disconnect window are lost (the consumer is responsible for accepting that trade-off).

#### Scenario: Per-channel checkpoint keys
- **WHEN** multiple channels target the same table with different filters
- **THEN** the checkpoint MUST be keyed by `(adapter.name, channel.label || channel.type + filter-hash)` so distinct channels do not overwrite each other's checkpoints.

### Requirement: List Refresh Hints
The adapter SHALL populate `ChangeSet.affectedListKeys` when emitting non-trivial changes, so list-rendering consumers can refresh deterministically.

#### Scenario: Insert / delete populates list keys
- **WHEN** the adapter emits a `ChangeSet` containing at least one `op: "insert"` or `op: "delete"` change for an entity that the channel knows belongs to a list (per `opts.listKeyResolver`)
- **THEN** the `ChangeSet.affectedListKeys` array MUST contain the resolved list-key strings, deduplicated.

#### Scenario: Update without listKeyResolver
- **WHEN** the adapter emits an update and no `listKeyResolver` is supplied
- **THEN** the `ChangeSet.affectedListKeys` MUST be `undefined` (or omitted). The `RealtimeManager`'s 16ms coalesce window handles the list-refresh decision in that case.

### Requirement: Companion Skill
The `prometheus-skill-system` repo SHALL carry a companion skill at `skills/react/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md` describing the consumer wiring pattern.

#### Scenario: Skill exists
- **WHEN** an agent surveys `prometheus-entity-skills`
- **THEN** the skill directory MUST exist with a non-empty `SKILL.md` containing front matter, usage example, gotchas, and a reference to this specification.

#### Scenario: Skill ↔ spec alignment
- **WHEN** the spec is verified
- **THEN** the skill's documented adapter shape (export name, options, return type, registration sequence) MUST match this spec verbatim. A mismatch fails verification.

### Requirement: Test Coverage
The adapter SHALL ship with vitest coverage that mirrors the patterns used by `electricsql-tenant.test.ts`.

#### Scenario: Fake SurrealDB client
- **WHEN** the test file runs
- **THEN** it MUST use a hand-rolled fake `Surreal` client (no real network), and MUST cover seed delivery, live action mapping (CREATE / UPDATE / DELETE), reconnect backoff, replay (with and without `checkpointStore`), per-channel cleanup via `UnsubscribeFn`, status transitions, and `affectedListKeys` derivation.

#### Scenario: No leaked subscriptions
- **WHEN** the test invokes `UnsubscribeFn` for each channel
- **THEN** the fake client MUST observe a corresponding `KILL` call; the test asserts zero leaked live subscriptions remain.
