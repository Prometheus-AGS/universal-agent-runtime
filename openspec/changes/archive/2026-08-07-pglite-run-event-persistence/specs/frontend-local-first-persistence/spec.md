## ADDED Requirements

### Requirement: Versioned local run and event schema
The frontend SHALL apply an additive PGlite migration that stores client-owned
runs, terminal phase timings, and ordered normalized run events without
rewriting existing thread or message data.

#### Scenario: Existing browser database upgrades
- **WHEN** a database containing migration 001 opens after C-07
- **THEN** migration 002 creates the run and run-event tables and indexes
- **AND** the existing threads and messages remain readable

#### Scenario: Migration is reapplied
- **WHEN** a database that already recorded migration 002 opens again
- **THEN** no schema statement is reapplied and no run/event row is changed

#### Scenario: Terminal phase timings persist
- **WHEN** a run reaches a finished, error, or cancelled terminal state with completed phase timings
- **THEN** its durable run row records the terminal status, finish timestamp, and complete phase-timing map

### Requirement: Bounded durable AG-UI event writes
The frontend SHALL persist accepted normalized AG-UI rows in adapter order while
coalescing text and reasoning content deltas into one durable row per logical
message span.

#### Scenario: Non-content event arrives
- **WHEN** the adapter emits a lifecycle, tool, state, custom, or raw event row
- **THEN** the row is persisted incrementally with its stable event identity, original wire sequence, normalized kind, timestamp, and payload

#### Scenario: Content span has an explicit end
- **WHEN** one or more text or reasoning content rows are followed by their matching end frame
- **THEN** the content deltas are persisted once as one ordered aggregate row at that boundary
- **AND** no individual content delta row is written

#### Scenario: Current transport terminates without an end frame
- **WHEN** buffered text or reasoning content is followed directly by RUN_FINISHED or RUN_ERROR
- **THEN** each buffered logical span is persisted once before the terminal event

#### Scenario: Multiple frames share one wire sequence
- **WHEN** distinct official event identities carry the same wire sequence
- **THEN** every identity is retained with a unique persistence ordinal
- **AND** the original wire sequence remains queryable without being used as row identity

#### Scenario: Replay repeats an event identity
- **WHEN** persistence receives an event identity already stored for the run
- **THEN** it creates no duplicate row and does not disturb the existing durable order

### Requirement: Durable run identity and offline reads
The frontend SHALL use the server-assigned run identifier when supplied and
SHALL expose typed run/event reads from PGlite for refresh and offline consumers.

#### Scenario: Server assigns a run identifier
- **WHEN** a chat response includes the server run header or an official row includes runId
- **THEN** subsequent run and event writes use that server identifier

#### Scenario: Trace is read after refresh
- **WHEN** the in-memory stores start empty after a page refresh
- **THEN** a consumer can list persisted runs and load ordered events for a selected run from PGlite without a network request

### Requirement: PEM local-first graph lifecycle
The application SHALL persist and hydrate the PEM graph through the installed
PGlite persistence adapter and local-first runtime instead of an
application-owned outbox table.

#### Scenario: Application starts with a graph snapshot
- **WHEN** PGlite contains a persisted PEM graph snapshot
- **THEN** local-first hydration completes before the realtime transport subscribes
- **AND** application children render only after both initialization steps succeed

#### Scenario: Graph state changes
- **WHEN** the PEM graph or a keyed graph action changes after initialization
- **THEN** the package local-first runtime persists the graph snapshot and pending-action records through PGlite

#### Scenario: Pending actions exist at startup
- **WHEN** a persisted graph snapshot contains registered pending actions and connectivity is available
- **THEN** the package runtime replays them using its configured retry policy
- **AND** no application outbox table or parallel replay loop is used

### Requirement: SQL-derived runtime entity schemas
The frontend SHALL generate RuntimeRun and RuntimeAgUiEvent validation schemas
from the same SQL definitions used by migration 002 while retaining existing
entity relation metadata.

#### Scenario: Entity schemas register at bootstrap
- **WHEN** entity schema bootstrap runs
- **THEN** registerEntityFromSql receives the run and run-event CREATE TABLE definitions through the platform entity facade
- **AND** the registered SQL field schemas coexist with the existing graph relation schemas
