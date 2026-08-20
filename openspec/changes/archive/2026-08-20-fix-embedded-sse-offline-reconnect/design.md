## Context

See `proposal.md` for motivation. The embedded persistence path registers a
private adapter from `frontend/src/entities/sync.ts` against
`/api/uar/sync/stream`. The endpoint emits named `connected`, `heartbeat`, and
`entity.change` events; only `entity.change` carries graph data, shaped as
`{table, action, id, record, ts}`. The current adapter listens only through
`onmessage`, expects different fields, exposes no status, and leaves recovery
entirely to browser behavior. The shared `/api/live` adapter already establishes
the repository pattern of closing a failed EventSource before an explicit
backoff retry.

## Goals / Non-Goals

**Goals:**

- Make the embedded adapter implement the endpoint's actual named-event and
  payload contract.
- Expose `RealtimeAdapter` status and own a single reconnect state machine with
  deterministic cleanup.
- Make unit and browser tests observe the adapter registered by the application,
  including one visible graph-backed state transition after reconnect.

**Non-Goals:**

- Changing the Rust endpoint, shared `/api/live` adapter, or persistence
  provider selection.
- Replaying events emitted while disconnected, adding an offline write queue,
  or claiming lossless delivery.
- Refactoring transport bootstrap, changing UI design, or adding dependencies.

## Decisions

### 1. Keep the adapter in `entities/sync.ts` and export its factory

The change will rename/export the existing private factory and leave
`initSyncTransport` in place. This is the smallest change that makes the real
adapter directly testable without introducing a new service/module boundary.

Alternative considered: extract a new realtime module. Rejected because it
adds file movement and import churn without changing behavior.

### 2. Consume only the named `entity.change` event

The adapter will register `addEventListener("entity.change", ...)`. It will
validate `table`, `action`, and `id`, map the table through the canonical UAR
topic/entity names, use `record` as `EntityChange.data`, and ignore malformed or
unknown events. `connected` and `heartbeat` remain transport signals and never
mutate graph state.

Alternative considered: change the server to emit unnamed messages matching
the current client shape. Rejected because the server's named-event contract is
already explicit and the client is the isolated broken consumer.

### 3. Own reconnect after `onerror`

Each subscription owns one EventSource, attempt counter, and optional timer.
On error it emits `error`, closes and clears the old source, and schedules one
replacement using `min(baseDelay * 2^attempt, 30_000)`. Opening a source resets
the attempt counter and emits `connected`; starting a connection emits
`connecting`. Unsubscribe marks the subscription stopped, clears the timer,
closes the source, and emits `disconnected`.

Closing before scheduling is load-bearing: it disables native recovery on the
failed instance so native and application reconnect paths cannot open parallel
streams. The delay is configurable only through an internal factory option for
fast deterministic tests; no public API is introduced.

Alternative considered: rely exclusively on native EventSource reconnect.
Rejected because the observed browser control could not force or observe that
path deterministically, and it provides no adapter-level status or timer
cleanup contract.

### 4. Instrument the application source from Playwright

The BDD step will install an init-script wrapper around native EventSource
before application bootstrap. The wrapper records only instances targeting
`/api/uar/sync/stream` and otherwise preserves native behavior. The scenario
will dispatch a named entity event through the captured application source,
force its error event, observe a second real request/open, and dispatch an
update for the same entity. A graph-backed Knowledge screen must show the
updated row once without reconnect-time reload or `replayRuntime`.

Alternative considered: create a second probe EventSource. Rejected because it
does not prove the registered adapter delivered or recovered. Direct store
injection and manual runtime replay are also rejected because they bypass the
transport under test.

### 5. Fix existing-entity projection in the source package

The first corrected browser run proved that the registered adapter updated the
normalized graph while `useEntityView` retained rendered items behind an
unchanged ID array. The source package will subscribe its projected `items` to
the stable entity snapshots for those IDs. The same correction applies to
`useEntityQuery`, the documented replacement with the identical projection
shape. UAR will consume the tested upstream commit through its existing
submodule; the Knowledge feature will not gain a refresh or local cache bypass.

Alternative considered: reload the Knowledge list or force an ID-list change
from the adapter. Rejected because both hide a source-package reactivity defect
and break the normalized graph's single-source-of-truth contract.

### 6. Build the source package through its declared dependency graph

The BDD preparation script will invoke Turbo from the entity-management root
with the React package dependency filter. This builds `entity-graph-core`
before the React declarations and removes reliance on whatever `dist` happened
to exist in the submodule checkout.

Alternative considered: keep invoking React's `tsup` directly and prebuild core
manually during certification. Rejected because the checked-in preparation
command would remain non-reproducible and fail on a clean source checkout.

## Risks / Trade-offs

- **[Risk] Browser native reconnect competes with the adapter retry.** → Close
  the failed EventSource synchronously before scheduling its replacement; unit
  and browser checks assert one replacement request and one delivery.
- **[Risk] Synthetic browser event injection proves the client boundary but not
  persistence polling.** → Keep the existing real endpoint connection and real
  replacement request, and scope the claim to adapter delivery/recovery. The
  Rust polling implementation is unchanged and is not recertified here.
- **[Risk] Events emitted while disconnected are lost.** → State this limit in
  the spec and verification. Checkpoint replay requires a separate server
  contract and is not silently simulated.
- **[Risk] Advancing the source submodule includes upstream changes after UAR's
  old detached pin.** → Base the repair on upstream `main`, run upstream package
  gates first, then run UAR type/lint/unit/build/browser gates against the exact
  commit before accepting the pointer.
- **[Trade-off] Explicit retry adds client state.** → Reuse the repository's
  existing capped-backoff shape and keep it subscription-local with focused
  fake-timer cleanup tests.

## Migration Plan

1. Land the adapter and focused unit tests without changing configuration or
   persisted data.
2. Land the upstream source/compatibility PR and open the canonical generated
   rc.2 version PR after its package gates pass.
3. Advance UAR's source submodule to the tested source/compatibility commit and
   observe the pre-recorded browser failure become a no-reload pass. The
   separate unmerged rc.2 PR is version evidence, not the pinned source commit.
4. Roll back by reverting the UAR commit and upstream repair; the backend
   endpoint and other provider transports remain untouched.
