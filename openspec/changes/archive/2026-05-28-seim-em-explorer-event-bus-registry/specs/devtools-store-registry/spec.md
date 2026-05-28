# devtools-store-registry Specification

## Purpose

`registerStore(bus, sourceSubscribeFn, name)` wires an additional event source — any function
with the same signature as `subscribeDevtoolsEvent` — into an existing `DevtoolsEventBus` so
that events from multiple independent store instances (e.g., two isolated graph stores, one per
schema domain) all flow into a single bus and appear in the Entity Explorer's Events tab.

`getRegisteredStores()` returns a snapshot of the current registry, consumed by the panel's
Subscriptions tab to show which stores are actively connected.

The registry is module-level (singleton): all calls to `registerStore` / `getRegisteredStores`
within a process share one map. This mirrors the singleton design of the W3 engine tap and
`devtoolsSubscribers` set in `engine.ts`.

## Requirements

### Requirement: Public API Surface

`registerStore` and `getRegisteredStores` SHALL be exported from `src/devtools-event-bus.ts`
and re-exported from `src/index.ts`.

```ts
type DevtoolsSourceFn = (cb: (event: DevtoolsEvent) => void) => () => void;

interface RegisteredStore {
  name: string;
  active: boolean;          // false after the returned unsubscribe fn is called
}

function registerStore(
  bus: DevtoolsEventBus,
  source: DevtoolsSourceFn,
  name: string,
): () => void;  // unsubscribeFn — removes registration and stops routing

function getRegisteredStores(): ReadonlyArray<RegisteredStore>;
```

#### Scenario: Exports resolve
- **WHEN** a consumer imports `registerStore` and `getRegisteredStores` from
  `@prometheus-ags/prometheus-entity-management`
- **THEN** both MUST resolve to callable functions.

---

### Requirement: Store Registration and Event Routing

`registerStore(bus, source, name)` SHALL call `source(handler)` immediately, where `handler`
routes every received `DevtoolsEvent` into `bus` as if it were emitted by the bus's own engine
tap. Routed events enter the bus's ring buffer and are dispatched to all bus subscribers.

#### Scenario: Events from a registered source reach bus subscribers
- **WHEN** a source is registered and emits a `{ kind: "upsert", ... }` event
- **THEN** all subscribers on `bus` MUST receive that event, and `bus.getBuffer()` MUST
  include it.

#### Scenario: Multiple sources fan into the same bus
- **WHEN** two sources (`sourceA`, `sourceB`) are registered on the same bus
- **THEN** events from both sources MUST reach bus subscribers; events from each source MUST
  appear in `bus.getBuffer()` interleaved in arrival order.

#### Scenario: Same name registered twice is an error
- **WHEN** `registerStore(bus, source, "workspace")` is called and then
  `registerStore(bus, source2, "workspace")` is called
- **THEN** the second call MUST throw an `Error` with a message containing the duplicate name.
  The first registration MUST remain active.

#### Scenario: Registered source does not bypass burst coalescing
- **WHEN** a registered source emits a burst exceeding `coalesceBurstThreshold`
- **THEN** the bus MUST apply the same coalescing rules as for its own engine-tap events.

---

### Requirement: Unregistration

The `UnsubscribeFn` returned by `registerStore` SHALL, when called, stop routing events from
the source into the bus AND mark the store entry as `active: false` in the registry.

#### Scenario: Unsubscribe stops event routing
- **WHEN** the `UnsubscribeFn` is called and the source subsequently emits an event
- **THEN** bus subscribers MUST NOT receive that event and `bus.getBuffer()` MUST NOT
  include it.

#### Scenario: Unsubscribe marks store as inactive in registry
- **WHEN** the `UnsubscribeFn` is called
- **THEN** `getRegisteredStores()` MUST return the entry with `active: false` (the entry is
  retained for observability — it is not removed from the snapshot).

#### Scenario: Unsubscribe is idempotent
- **WHEN** the `UnsubscribeFn` is called twice
- **THEN** the second call MUST be a no-op; no error MUST be thrown.

---

### Requirement: Registry Snapshot

`getRegisteredStores()` SHALL return a `ReadonlyArray<RegisteredStore>` snapshot of all
stores that have ever been registered in this process, in registration order, with their
current `active` flag.

#### Scenario: Empty registry
- **WHEN** no stores have been registered
- **THEN** `getRegisteredStores()` MUST return an empty array.

#### Scenario: Snapshot reflects current state
- **WHEN** two stores are registered and one is subsequently unregistered
- **THEN** `getRegisteredStores()` MUST return both entries; the unregistered one MUST have
  `active: false`, the other `active: true`.

#### Scenario: Snapshot is immutable
- **WHEN** the caller mutates the returned array (e.g., `arr.push(...)`)
- **THEN** subsequent calls to `getRegisteredStores()` MUST NOT reflect the mutation. The
  return value MUST be a new array (or frozen) on each call.

---

### Requirement: Bus Destroy Interaction

When `bus.destroy()` is called, all stores registered against that bus SHALL be implicitly
unregistered: event routing stops and the registry marks their entries as `active: false`.

#### Scenario: Bus destroy cascades to registry entries
- **WHEN** a store is registered on a bus and `bus.destroy()` is called
- **THEN** `getRegisteredStores()` MUST show the store as `active: false`, and subsequent
  events from the source MUST NOT reach any subscriber.

#### Scenario: Registry entries from a destroyed bus can be re-registered on a new bus
- **WHEN** a store was registered on bus A (now destroyed) and is then registered on bus B
  with the same name
- **THEN** the registration on bus B MUST succeed (the duplicate-name guard applies only to
  *active* registrations, not destroyed ones).

---

### Requirement: Module-Level Reset (Test Utility)

A `__resetStoreRegistry()` function SHALL be exported from `src/devtools-event-bus.ts` for
use in tests only. It SHALL clear the registry map so each test starts from a clean state.
It MUST NOT be re-exported from `src/index.ts`.

#### Scenario: Reset clears all entries
- **WHEN** `__resetStoreRegistry()` is called after registrations
- **THEN** `getRegisteredStores()` MUST return an empty array.

---

### Requirement: Test Coverage

A vitest file SHALL exercise every requirement above, co-located with the bus tests.

#### Scenario: Test coverage
- **WHEN** `pnpm test` runs
- **THEN** the test suite MUST include at least 10 assertions covering the registry requirements;
  the existing 124 + ≥15 bus-test baseline MUST have no regressions.
