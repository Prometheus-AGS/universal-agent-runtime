# devtools-event-bus Specification

## Purpose

`createDevtoolsEventBus(opts?)` wraps the W3 `subscribeDevtoolsEvent` engine tap with three
additional capabilities the Entity Explorer panel and Chrome extension require:

1. **Ring buffer** — retains the last N `DevtoolsEvent` objects (default 500, configurable)
   so subscribers that connect *after* ops have fired can receive history.
2. **Fan-out with replay** — N panel consumers subscribe to one bus instance; each new
   subscriber immediately receives the current buffer contents before transitioning to the
   live stream.
3. **Microtask-level burst coalescing** — a run of >threshold same-store events within a
   single microtask is collapsed into a synthetic `kind: "list"` event, preventing the Chrome
   message bridge from being flooded during bulk imports.

The bus holds exactly **one** subscription to `subscribeDevtoolsEvent`; that subscription is
released on `destroy()`. Multiple bus instances are independent.

## Requirements

### Requirement: Factory Function

`createDevtoolsEventBus(opts?: DevtoolsEventBusOptions): DevtoolsEventBus` SHALL be exported
from `src/devtools-event-bus.ts` and re-exported from `src/index.ts`.

```ts
interface DevtoolsEventBusOptions {
  bufferSize?: number;           // default 500; 0 = unbounded (not recommended)
  coalesceBurstThreshold?: number; // default 10; 0 = no coalescing
}

interface DevtoolsEventBus {
  subscribe(cb: (event: DevtoolsEvent) => void): () => void;
  getBuffer(): readonly DevtoolsEvent[];
  flush(): void;
  destroy(): void;
}
```

#### Scenario: Factory returns a live bus
- **WHEN** `createDevtoolsEventBus()` is called
- **THEN** the returned object MUST implement `subscribe`, `getBuffer`, `flush`, and `destroy`,
  and the bus MUST already be subscribed to `subscribeDevtoolsEvent` (one engine tap).

#### Scenario: Default options apply
- **WHEN** `createDevtoolsEventBus()` is called without arguments
- **THEN** `bufferSize` MUST default to `500` and `coalesceBurstThreshold` MUST default to `10`.

#### Scenario: Custom options respected
- **WHEN** `createDevtoolsEventBus({ bufferSize: 50, coalesceBurstThreshold: 5 })` is called
- **THEN** the ring buffer MUST cap at 50 entries and burst coalescing MUST activate after
  5 same-tick events.

---

### Requirement: Ring Buffer

The bus SHALL maintain a bounded ring buffer of `DevtoolsEvent` objects. When the buffer
is full, the oldest entry is evicted to make room for the newest.

#### Scenario: Buffer starts empty
- **WHEN** `createDevtoolsEventBus()` is called
- **THEN** `getBuffer()` MUST return an empty readonly array.

#### Scenario: Events populate the buffer
- **WHEN** N events fire through the engine tap
- **THEN** `getBuffer()` MUST return them in chronological order (oldest first).

#### Scenario: Buffer evicts oldest on overflow
- **WHEN** `bufferSize` is 3 and 4 events fire
- **THEN** `getBuffer()` MUST contain the 3 most recent events; the first event MUST have
  been evicted.

#### Scenario: `getBuffer()` returns a stable snapshot
- **WHEN** `getBuffer()` is called and a new event fires before the caller iterates the result
- **THEN** the previously-returned array MUST NOT change; each `getBuffer()` call returns a
  new snapshot (or a frozen copy).

---

### Requirement: Fan-Out Subscription

`bus.subscribe(cb)` SHALL register a listener that receives all future events dispatched by
the bus, and SHALL immediately replay the current buffer contents to the new listener in
registration order before returning.

#### Scenario: Replay on subscribe
- **WHEN** 3 events have fired and a new listener subscribes
- **THEN** the listener MUST receive the 3 buffered events synchronously (during the
  `subscribe()` call), in chronological order, before `subscribe()` returns.

#### Scenario: Live events after replay
- **WHEN** a listener has subscribed and an additional event fires
- **THEN** the listener MUST receive that event exactly once, after all replayed events.

#### Scenario: Multiple subscribers each get replay + live
- **WHEN** subscriber A subscribes after 2 events, then subscriber B subscribes after 3 events
- **THEN** A MUST receive events 1+2 at subscribe time, then event 3 live; B MUST receive
  events 1+2+3 at subscribe time.

#### Scenario: Unsubscribe stops delivery
- **WHEN** the `UnsubscribeFn` returned by `subscribe` is invoked
- **THEN** the unsubscribed callback MUST NOT receive any subsequent events; other subscribers
  MUST continue to receive them.

#### Scenario: Subscriber error isolation
- **WHEN** subscriber A throws and subscriber B is also registered
- **THEN** B MUST still receive the event; the error from A MUST be caught and logged via
  `console.warn`, not propagated.

---

### Requirement: Burst Coalescing

When the bus receives more than `coalesceBurstThreshold` events within a single microtask
tick, it SHALL coalesce the excess into a synthetic `{ kind: "list", ... }` summary event
before dispatching to bus subscribers.

#### Scenario: Below threshold — no coalescing
- **WHEN** `coalesceBurstThreshold` is 10 and 9 events fire synchronously
- **THEN** all 9 events MUST be dispatched individually to subscribers; no synthetic
  `kind: "list"` event is emitted.

#### Scenario: At/above threshold — coalescing fires
- **WHEN** `coalesceBurstThreshold` is 10 and 12 events fire synchronously in the same
  microtask
- **THEN** subscribers MUST receive the first 10 individual events, followed by one synthetic
  `{ kind: "list", key: "<store>", idCount: 2, at: <ISO> }` event representing the 2
  coalesced extras (or an equivalent grouping strategy).

#### Scenario: `flush()` forces pending coalesced burst
- **WHEN** a burst is in progress and `flush()` is called
- **THEN** the coalesced summary event MUST be dispatched synchronously before `flush()` returns.

#### Scenario: Coalescing disabled
- **WHEN** `createDevtoolsEventBus({ coalesceBurstThreshold: 0 })` is used
- **THEN** all events MUST be dispatched individually with no coalescing, regardless of volume.

---

### Requirement: Lifecycle and Destroy

`bus.destroy()` SHALL release the bus's engine-tap subscription and remove all bus
subscribers, leaving the engine tap as it was before the bus was created.

#### Scenario: Destroy releases engine tap
- **WHEN** `bus.destroy()` is called
- **THEN** no further events from `subscribeDevtoolsEvent` MUST reach the bus; events fired
  after `destroy()` MUST NOT appear in `getBuffer()` and MUST NOT be dispatched to any
  bus subscriber.

#### Scenario: Destroy is idempotent
- **WHEN** `bus.destroy()` is called twice
- **THEN** the second call MUST be a no-op; no error MUST be thrown.

#### Scenario: Destroy clears subscribers
- **WHEN** `bus.destroy()` is called with active bus subscribers
- **THEN** those subscribers MUST be removed; they MUST NOT receive events from any future
  bus instance using the same callback reference.

---

### Requirement: Production Tree-Shake Gate

The bus module SHALL be gated so production bundles that do not import it pay zero cost.

#### Scenario: No side-effects at import
- **WHEN** `devtools-event-bus.ts` is imported in a production bundle but
  `createDevtoolsEventBus` is never called
- **THEN** no engine-tap subscription MUST be established and no memory MUST be allocated
  for a buffer — the module has no module-level side effects.

#### Scenario: `src/index.ts` re-export does not pull in bus at runtime
- **WHEN** a consumer imports only `useGraphStore` from `@prometheus-ags/prometheus-entity-management`
- **THEN** `devtools-event-bus.ts` MUST NOT be evaluated by the bundler (tree-shakeable
  named export with no module-level side effects).

---

### Requirement: Test Coverage

A vitest file SHALL exercise every requirement above.

#### Scenario: Test file exists
- **WHEN** the test surface is inspected
- **THEN** `src/devtools-event-bus.test.ts` MUST exist with `describe` blocks per requirement
  and a minimum of 15 `it` assertions.

#### Scenario: Tests pass
- **WHEN** `pnpm test` runs in `~/.claude/worktrees/seim-entity-management`
- **THEN** all bus tests MUST pass; total suite count MUST grow by at least 15 tests with no
  regressions in the existing 124-test suite.
