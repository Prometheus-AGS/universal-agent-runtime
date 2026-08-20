## Why

The embedded SurrealDB SSE bridge cannot currently deliver its named
`entity.change` events into the frontend entity graph because the server and
client disagree on both the event name and payload fields. The existing
screen-validation reconnect proof also watches a separate probe instead of the
registered application adapter, so release certification cannot truthfully
claim no-reload recovery.

## What Changes

- Align the embedded client with the `/api/uar/sync/stream` named-event and
  payload contract so supported entity changes reach the graph.
- Give the embedded adapter observable connection status, bounded recovery from
  detected stream errors, and complete unsubscribe cleanup without parallel
  connections or duplicate delivery.
- Add focused unit controls and one live browser scenario bound to the
  EventSource registered by the application, including a forced-error negative
  path and visible post-reconnect state change without page reload or manual
  replay.
- Make the existing BDD preparation path build the source React package and its
  upstream workspace dependencies instead of relying on stale declaration
  output.
- Explicitly retain the current boundary: recovery resumes delivery from the
  new connection and does not promise checkpoint replay for events emitted
  while disconnected.

## Capabilities

### New Capabilities

- `embedded-sse-sync`: Defines the UAR frontend contract for consuming the
  embedded SurrealDB SSE bridge, reporting status, reconnecting after detected
  transport errors, cleaning up, and proving exactly-once handling per received
  event.

### Modified Capabilities

- None. The existing `entity-surreal-live-adapter` capability specifies the
  reusable direct SurrealDB driver adapter, not UAR's Axum-hosted embedded SSE
  bridge, and its requirements remain unchanged.

## Impact

- **Runtime UX:** graph-backed screens can observe embedded entity changes and
  recover after a detected stream error without a page reload.
- **Realtime state:** affects only the embedded `/api/uar/sync/stream` client
  registration, mapping, status, reconnect, and cleanup behavior.
- **Provider compatibility:** PostgreSQL and remote SurrealDB continue using the
  shared `/api/live` adapters unchanged; no LLM provider behavior changes.
- **Code and tests:** UAR changes are limited to `frontend/src/entities/sync.ts`,
  its focused unit test, the embedded scenario feature/steps under `tests/bdd/`,
  and the root BDD preparation build order. The normalized projection repair and
  pnpm consumer contract are delivered in the upstream entity-management
  source/compatibility PR.
- **APIs and dependencies:** no backend endpoint, public API, package
  declaration, or lockfile change. UAR advances the existing entity-management
  submodule pin to the tested upstream source/compatibility head; the separate
  generated `3.0.0-rc.2` PR records the requested version without publication.
- **KBD workflow:** yes. This proposal belongs to the active
  `fix-embedded-sse-offline-reconnect` child and must complete its verification
  and handoff before `screen-by-screen-validation` resumes.
