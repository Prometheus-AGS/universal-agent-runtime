## ADDED Requirements

### Requirement: Fresh-install schema is current before first use
The frontend SHALL apply every migration in `MIGRATIONS`
(`frontend/src/platform/pglite/migrations.ts`) to the local PGlite database before
the database is returned to any caller, including when the database was created
from the packaged schema seed. The packaged seed SHALL be generated at the latest
migration version, and the seed asset the application loads SHALL be the file the
seed builder (`scripts/build-pglite-seed.mjs`) names for that version.

#### Scenario: First session on a fresh browser profile persists chat messages
- **WHEN** a browser profile with no `/pglite/uar-threads` IndexedDB database opens the console and the operator sends a message in a new conversation in that first session
- **THEN** the message row is written with the columns `run_id`, `client_turn_id` and `history_position` present
- **AND** after a page reload the conversation history shows that message without any earlier reload having been required

#### Scenario: Seed older than the migration list
- **WHEN** the loaded seed records fewer migrations in `schema_migrations` than `MIGRATIONS` declares
- **THEN** the missing migrations are applied after the seed loads and before the database is returned to callers
- **AND** migrations already recorded are not reapplied

#### Scenario: Seed drift is detected
- **WHEN** `pglite:seed:check` runs and the seed file for the latest migration version is missing, or its recorded migration digest or schema catalog differs from `MIGRATIONS`
- **THEN** the check exits non-zero and names the mismatch

### Requirement: Local database runs in a leader-elected worker
The frontend SHALL open the local PGlite database through `PGliteWorker`
(`@electric-sql/pglite/worker`) so WASM Postgres runs in a dedicated worker and exactly
one tab's worker has the IndexedDB database `uar-threads` open at a time. The `live`
extension SHALL remain available to main-thread callers.

#### Scenario: Single tab hosts the database in a worker
- **WHEN** the console opens in one tab
- **THEN** a dedicated worker hosts PGlite and no `PGlite` instance is created on the main thread
- **AND** `navigator.locks.query()` reports exactly one held lock whose name begins with `pglite-election-lock:` followed by the fixed election identity

#### Scenario: Two tabs share one database
- **WHEN** a second tab opens on the same origin while the first tab is open
- **THEN** exactly one worker holds the election lock
- **AND** the second tab reads the threads and messages the first tab wrote
- **AND** a thread created in either tab is listed in the other without an IndexedDB open error

#### Scenario: Run snapshot subscription works through the worker
- **WHEN** a tab subscribes to a run snapshot through `subscribeRunSnapshot` and another tab appends events for that run
- **THEN** the subscriber receives the updated snapshot with the appended events in order

### Requirement: Worker election identity does not depend on the build
The worker election identity SHALL be a fixed application constant that does not depend
on the worker script URL, so tabs loaded from different builds elect against the same lock.

#### Scenario: Tabs from different builds elect against one lock
- **WHEN** one tab was loaded from build A and another from build B while both are open
- **THEN** both register the same election lock name and only one worker holds it

### Requirement: Database initialization runs in the worker at election time
Asset loading, seed selection, migrations and database creation SHALL run inside the
worker `init`, evaluated when that worker wins the election, so a worker that becomes
leader later does not apply a seed over an existing database.

#### Scenario: Seed is not applied over an existing database
- **WHEN** a follower tab's worker wins the election after the database already exists
- **THEN** the seed is not loaded
- **AND** the threads and messages written before the handover are still present

### Requirement: Leader change does not lose or duplicate client writes
The frontend SHALL treat `LeaderChangedError` from the database as retryable for every
keyed write (thread upsert, message upsert, run start and finish, event append). The
retry SHALL be bounded by named constants, and a write that still fails after the bound
SHALL reject to its caller and SHALL NOT be reported as persisted.

#### Scenario: Leader tab closes during message writes
- **WHEN** the leader tab closes while a follower tab is persisting chat messages
- **THEN** the surviving tab becomes leader and continues to answer queries
- **AND** every message the operator saw acknowledged is present exactly once after the handover

#### Scenario: No leader becomes available within the bound
- **WHEN** a write fails with `LeaderChangedError` and no worker becomes leader within the retry bound
- **THEN** the call rejects to its caller
- **AND** no success is reported and no row is created by the failed attempt

### Requirement: One tab owns the graph snapshot row and pending-action replay
At most one tab, the `PGliteWorker` leader, SHALL write or remove the PEM graph
snapshot stored under key `uar:entity-graph` and replay pending actions. A tab that is
not the owner SHALL NOT write the snapshot or replay pending actions. A tab that
becomes the owner SHALL adopt pending-action records present in the stored snapshot that
its own runtime lacks, replay the adopted records using the configured retry policy, and
persist its graph, without waiting for a further graph change.

#### Scenario: A non-owner tab changes the graph
- **WHEN** a non-owner tab changes graph state that would schedule a persist
- **THEN** the stored snapshot row and its `updated_at` are not changed by that tab

#### Scenario: Ownership moves when the owner closes
- **WHEN** the owner tab closes while another tab is open
- **THEN** the surviving tab becomes owner and the stored snapshot is replaced by that tab's graph without waiting for a further graph change

#### Scenario: A stale tab never overwrites a newer snapshot
- **WHEN** tab B holds an older graph than the one the owner tab A last stored, and B is not the owner
- **THEN** the stored snapshot continues to equal A's last persisted graph

#### Scenario: Owner reloads while another tab stays open
- **WHEN** the owner tab holds a pending action, reloads while a second tab stays open, and the second tab becomes owner
- **THEN** the pending action is neither dropped from the stored snapshot nor replayed more than once

### Requirement: Existing local data survives the move to the worker
A browser profile whose `uar-threads` database was created by the main-thread build
SHALL open under the worker build with its threads, messages, runs, run events and
graph snapshot intact, without loading the seed and without a destructive migration.

#### Scenario: Profile created by the previous build upgrades
- **WHEN** a profile that holds threads, messages, a run with events and a graph snapshot written by the main-thread build is opened by the worker build
- **THEN** every thread, message, run and event is readable and the graph hydrates to the stored snapshot

### Requirement: Legacy localStorage migration runs once on the main thread
The one-time migration of legacy `uar-thread-registry` and `uar-chat-messages`
`localStorage` data SHALL run on the main thread (workers have no `localStorage`) and
SHALL be serialized across tabs so each legacy record is imported once.

#### Scenario: Two tabs open with legacy data present
- **WHEN** two tabs open concurrently on a profile that still holds legacy `localStorage` thread and message data
- **THEN** each legacy thread and message appears exactly once in the database
- **AND** the migration flag is set and the legacy keys are removed

### Requirement: Retained AG-UI event entities are bounded
The graph SHALL retain at most `RUNTIME_AG_UI_EVENT_RETENTION` `RuntimeAgUiEvent`
entities, evicting the oldest first, and eviction SHALL NOT delete rows from the PGlite
`run_event` table. Plan fixes the constant's value from measurement.

#### Scenario: Event entities are capped
- **WHEN** more than `RUNTIME_AG_UI_EVENT_RETENTION` AG-UI events are ingested in one session
- **THEN** the graph holds exactly `RUNTIME_AG_UI_EVENT_RETENTION` `RuntimeAgUiEvent` entities and the stored snapshot size stops growing with further events
- **AND** the PGlite `run_event` row count for each run equals the events persisted for it

### Requirement: Graph persistence has a maximum wait
The application SHALL set `persistDebounceMs` explicitly, and the owner tab SHALL start a
persist no later than `GRAPH_PERSIST_MAX_WAIT_MS` after the first unpersisted graph
change by calling the runtime's `persistNow()`. Plan fixes both values from measurement.

#### Scenario: Long streaming reply keeps persisting
- **WHEN** a reply streams graph updates at intervals shorter than the debounce for longer than `GRAPH_PERSIST_MAX_WAIT_MS`
- **THEN** `lastPersistedAt` in the graph sync status advances at least once per `GRAPH_PERSIST_MAX_WAIT_MS` while the stream runs

#### Scenario: Tab is killed mid-stream
- **WHEN** a tab is closed without unload handlers during a long stream
- **THEN** after reload the hydrated graph is no older than `GRAPH_PERSIST_MAX_WAIT_MS` plus one persist duration before the kill

## MODIFIED Requirements

### Requirement: PEM local-first graph lifecycle
The application SHALL persist and hydrate the PEM graph through the installed
PGlite persistence adapter and local-first runtime instead of an
application-owned outbox table. Persistence of the graph snapshot and replay of
pending actions SHALL be owned by one tab at a time.

#### Scenario: Application starts with a graph snapshot
- **WHEN** PGlite contains a persisted PEM graph snapshot
- **THEN** local-first hydration completes before the realtime transport subscribes
- **AND** application children render only after both initialization steps succeed

#### Scenario: Graph state changes
- **WHEN** the PEM graph or a keyed graph action changes after initialization in the tab that owns the graph snapshot
- **THEN** the package local-first runtime persists the graph snapshot and pending-action records through PGlite

#### Scenario: Pending actions exist at startup
- **WHEN** a persisted graph snapshot contains registered pending actions and connectivity is available
- **THEN** the package runtime replays them using its configured retry policy in at most one tab at a time
- **AND** no application outbox table or parallel replay loop is used

#### Scenario: Two tabs start with the same pending actions
- **WHEN** two tabs hydrate a snapshot that contains the same pending action and both start while connectivity is available
- **THEN** the action is replayed once and is not replayed again by the second tab
