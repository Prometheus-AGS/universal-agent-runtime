## Purpose

Defines reliable delivery and observable recovery for entity changes carried
from UAR's embedded SurrealDB SSE bridge into the frontend entity graph.

## Requirements

### Requirement: Embedded entity changes use the server event contract
The embedded SSE client SHALL consume the named `entity.change` event emitted
by `/api/uar/sync/stream` and SHALL map a valid
`{table, action, id, record, ts}` payload into one graph change using UAR's
canonical table-to-entity-type mapping.

#### Scenario: Named entity update is delivered
- **WHEN** the connected embedded stream emits `entity.change` with table
  `knowledge_bases`, action `update`, an id, and a full record
- **THEN** the entity graph receives exactly one `KnowledgeBase` update for that
  id with the full record

#### Scenario: Transport-only events do not mutate state
- **WHEN** the embedded stream emits `connected` or `heartbeat`
- **THEN** the entity graph receives no change

#### Scenario: Invalid event is ignored
- **WHEN** an event is unnamed, has an unrecognized name or action, refers to an
  unknown table, or lacks its required id or record fields
- **THEN** the entity graph receives no change and the active subscription
  remains usable

### Requirement: Embedded stream recovery is observable and single-connection
The embedded SSE client SHALL expose connection status through the existing
realtime adapter status contract and SHALL recover from a detected stream error
with capped backoff while maintaining at most one active connection per
subscription.

#### Scenario: Successful initial connection
- **WHEN** an embedded subscription opens successfully
- **THEN** its status transitions through `connecting` to `connected` and its
  reconnect attempt counter is reset

#### Scenario: Detected error schedules one replacement
- **WHEN** an active embedded stream reports an error
- **THEN** the failed connection is closed before one replacement is scheduled,
  status reports the failure and recovery transition, and the retry delay does
  not exceed 30 seconds

#### Scenario: Reconnect succeeds without parallel streams
- **WHEN** the scheduled replacement connection opens
- **THEN** status becomes `connected`, subsequent named events are delivered,
  and no failed predecessor remains active

#### Scenario: Unsubscribe cancels recovery
- **WHEN** a subscription is removed while connected or while a retry is pending
- **THEN** its active connection is closed, its pending retry is cancelled,
  status becomes `disconnected`, and no later connection is opened

### Requirement: Recovery does not overstate delivery guarantees
The embedded SSE client SHALL deliver each valid event it receives once and
MUST NOT fabricate checkpoint replay for the interval in which it was
disconnected.

#### Scenario: Post-reconnect update is not duplicated
- **WHEN** one valid entity update arrives after a successful reconnect
- **THEN** the graph applies that received update exactly once

#### Scenario: Disconnected interval has no replay claim
- **WHEN** the client reconnects after events may have been emitted while it was
  disconnected
- **THEN** delivery resumes from the replacement connection without reporting
  the missed interval as replayed or lossless

### Requirement: Browser evidence exercises the registered adapter
The live browser acceptance scenario SHALL bind its assertions to the embedded
stream used by the application and SHALL demonstrate a visible graph-backed
entity transition after recovery without reconnect-time page reload or manual
state replay.

#### Scenario: Visible state changes through the initial stream
- **WHEN** a known valid entity event is delivered through the application's
  registered embedded stream
- **THEN** the corresponding graph-backed screen displays the entity state

#### Scenario: Visible state changes after forced recovery
- **WHEN** the same registered stream is forced through its error path, a second
  real stream request opens, and one update is delivered on the replacement
  connection
- **THEN** the screen displays the updated entity exactly once without page
  reload, a separate probe connection, direct store injection, or manual replay

### Requirement: Browser preparation builds source dependencies
The browser preparation command SHALL build the source entity-management React
package through its declared workspace dependency graph before generating the
BDD tests.

#### Scenario: Clean source declarations are built in dependency order
- **WHEN** the React package consumes a declaration exported by entity-graph-core
- **THEN** preparation builds entity-graph-core before the React declaration bundle
- **AND** the command does not depend on a stale prebuilt core distribution
