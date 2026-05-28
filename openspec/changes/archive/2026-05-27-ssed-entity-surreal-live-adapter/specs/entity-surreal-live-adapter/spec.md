## ADDED Requirements

### Requirement: Adapter Factory
The package SHALL export `createSurrealLiveAdapter(opts: SurrealLiveAdapterOptions): SyncAdapter` from `src/adapters/surreal-live.ts`, re-exported from `src/index.ts`.

#### Scenario: Public export
- **WHEN** a consumer imports `createSurrealLiveAdapter` from `@prometheus-ags/prometheus-entity-management`
- **THEN** the import MUST resolve and the returned function MUST produce a `SyncAdapter` instance compatible with `realtime-manager.registerAdapter`.

#### Scenario: Type exports
- **WHEN** a consumer imports `SurrealTableConfig` and `SurrealLiveAdapterOptions` types
- **THEN** both MUST be re-exported as type-only exports from the package root.

### Requirement: Initial Seed
The adapter SHALL hydrate the graph with current rows before opening the live subscription when `initialQueryStrategy` is `select-then-live` (default).

#### Scenario: Default seeding
- **WHEN** the adapter is started with `initialQueryStrategy` unspecified or set to `select-then-live`
- **THEN** for each table, the adapter MUST execute `SELECT * FROM <table>[ WHERE <where>]` once, convert each row to an `EntityChange` with `op: "insert"`, and emit a single `ChangeSet` containing all seeded rows.

#### Scenario: Live-only mode
- **WHEN** `initialQueryStrategy: "live-only"` is supplied
- **THEN** the adapter MUST NOT execute the initial `SELECT`; it MUST open only the live subscription.

#### Scenario: onSynced fires after seed
- **WHEN** the initial seed completes for all configured tables (`select-then-live` mode)
- **THEN** the adapter MUST invoke `onSynced` (if supplied) exactly once.

#### Scenario: onSynced fires immediately in live-only mode
- **WHEN** `live-only` is used
- **THEN** the adapter MUST invoke `onSynced` (if supplied) immediately after the live subscriptions are established (no rows to seed).

### Requirement: Live Subscription
The adapter SHALL open one `LIVE SELECT` per `SurrealTableConfig` and map incoming actions to `EntityChange` values.

#### Scenario: Live query opened per table
- **WHEN** the adapter starts subscriptions
- **THEN** for each table, the adapter MUST issue `LIVE SELECT * FROM <table>[ WHERE <where>]` and retain the returned live-query UUID for later cancellation.

#### Scenario: CREATE action
- **WHEN** SurrealDB pushes a `CREATE` notification with `result: row`
- **THEN** the adapter MUST emit `{ op: "insert", type, id, data: normalize?(row) ?? row }`.

#### Scenario: UPDATE action
- **WHEN** SurrealDB pushes an `UPDATE` notification with `result: row`
- **THEN** the adapter MUST emit `{ op: "upsert", type, id, data: normalize?(row) ?? row }`.

#### Scenario: DELETE action
- **WHEN** SurrealDB pushes a `DELETE` notification carrying at least an `id`
- **THEN** the adapter MUST emit `{ op: "delete", type, id }` with no `data` field.

#### Scenario: idColumn override
- **WHEN** a `SurrealTableConfig` specifies a non-default `idColumn`
- **THEN** the adapter MUST read the entity id from that column on every row.

#### Scenario: normalize applied
- **WHEN** a `SurrealTableConfig` supplies a `normalize` function
- **THEN** every `data` field emitted MUST be the result of `normalize(row)`, never the raw row.

### Requirement: Reconnection with Backoff
The adapter SHALL handle websocket disconnections with exponential backoff and SHALL re-issue live queries on reconnect.

#### Scenario: Backoff sequence
- **WHEN** the underlying SurrealDB websocket closes unexpectedly
- **THEN** the adapter MUST attempt reconnection with sleeps of 1 s, 3 s, 9 s, then a cap of 30 s thereafter (per attempt), continuing until reconnect succeeds.

#### Scenario: Live queries re-issued
- **WHEN** the websocket reconnects after a disconnect
- **THEN** the adapter MUST re-issue each `LIVE SELECT` (acquiring new live-query UUIDs) before declaring the stream healthy.

#### Scenario: Status callbacks
- **WHEN** connection state transitions occur
- **THEN** the adapter MUST notify status callbacks registered via the `SyncAdapter` API with `"connecting"`, `"online"`, `"reconnecting"`, or `"error"` as appropriate.

### Requirement: Checkpoint Replay
The adapter SHALL optionally replay missed changes on reconnect when `checkpointResume` is configured.

#### Scenario: Checkpoint persistence
- **WHEN** `checkpointResume` is supplied and an `EntityChange` is emitted
- **THEN** the adapter MUST invoke `checkpointResume.saveCheckpoint(<offset>)` where the offset is sourced from the row's `checkpointResume.columnName` (typically `updated_at`).

#### Scenario: Replay on reconnect
- **WHEN** the adapter reconnects and a previous checkpoint exists (`loadCheckpoint` returns non-null)
- **THEN** before re-attaching the live stream, the adapter MUST execute `SELECT * FROM <table> WHERE <columnName> > <checkpoint>` for each configured table and emit the resulting rows as `upsert` changes.

#### Scenario: No checkpoint configured
- **WHEN** `checkpointResume` is omitted
- **THEN** the adapter MUST behave as before — no checkpoint persistence, no replay query on reconnect.

### Requirement: List Refresh Hints
For each emitted `ChangeSet`, the adapter SHALL include `affectedListKeys` so the graph can refresh derived lists.

#### Scenario: affectedListKeys derivation
- **WHEN** the adapter emits a `ChangeSet` containing one or more changes for entity type `T`
- **THEN** the `affectedListKeys` array MUST include every list key registered against `T` (matching the existing pattern used by `createElectricAdapter`).

### Requirement: Companion Skill
The `prometheus-entity-skills` package SHALL ship a new skill `entity-realtime-surreal-live/` documenting setup and usage.

#### Scenario: Skill present
- **WHEN** the skill set is inspected after this change
- **THEN** `skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md` MUST exist with YAML front matter, `# What you get`, `# Setup`, `# Patterns`, `# Gotchas` sections, and at least one code example showing `createSurrealLiveAdapter` registration.

#### Scenario: Cross-reference
- **WHEN** the skill is read
- **THEN** it MUST reference the test file at `src/adapters/surreal-live.test.ts` as the canonical behavior reference.

### Requirement: Test Coverage
The adapter SHALL ship a vitest suite covering the documented behaviors.

#### Scenario: Test surface
- **WHEN** the test file at `src/adapters/surreal-live.test.ts` runs
- **THEN** it MUST cover, at minimum: seed path (select-then-live + live-only), CREATE/UPDATE/DELETE mapping, `normalize` application, reconnect with backoff, checkpoint replay (configured and unconfigured), and `affectedListKeys` derivation.

#### Scenario: Suite passes
- **WHEN** the test file is run via `pnpm test src/adapters/surreal-live.test.ts`
- **THEN** every assertion MUST pass without network calls (the SurrealDB client is mocked).
