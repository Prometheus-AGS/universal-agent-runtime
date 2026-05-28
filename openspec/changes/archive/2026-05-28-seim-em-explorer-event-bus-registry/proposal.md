## Why

The W3 `subscribeDevtoolsEvent` tap is a raw, single-source, unbuffered push stream — it
notifies about the *default* `useGraphStore` instance only, delivers events only to subscribers
who are already registered when an op fires, and fans out to a flat `Set` of listeners with no
coalescing or replay. The Entity Explorer panel (W6) and Chrome extension (W8) both need more:

1. **Buffer replay on connect.** The panel opens *after* events have already fired (user clicks
   the FAB). Without a buffer, the Events tab is empty until the next op. The bus must maintain a
   ring buffer (default 500 events) and replay it to every new subscriber at registration time.

2. **Fan-out stability.** Both the injected panel (W6) and the Chrome devtools page (W8) will
   subscribe simultaneously. The bus must be a stable fan-out multiplexer on top of the existing
   `subscribeDevtoolsEvent` channel — a single subscription to the engine, fanned out to N panel
   consumers.

3. **Multi-store support.** Apps that create more than one graph store instance (e.g., one per
   workspace tab, one per schema namespace) need each store's events to flow into the shared bus
   so the panel can observe all of them. The registry maps store instances to their bus
   subscriptions and exposes a `getRegisteredStores()` snapshot for the Subscriptions tab.

4. **High-frequency coalescing.** Bulk import ops (50+ upserts in one microtask tick) would
   flood the Chrome message bridge if forwarded individually. The bus coalesces a burst of >N
   same-store events within a single microtask into a synthetic `kind: "list"` summary event
   before dispatching to bus subscribers.

The existing `devtools.ts` (`useGraphDevTools`) is a *snapshot* hook — it pulls state on render.
W5 adds the *push stream* layer that feeds the Events tab and future time-travel replay. No
existing files are changed except `src/index.ts` (re-exports only).

## What Changes

**New file:**

```
src/devtools-event-bus.ts
```

Public API exported from this file:

```ts
// Bus lifecycle
createDevtoolsEventBus(opts?: DevtoolsEventBusOptions): DevtoolsEventBus
type DevtoolsEventBusOptions = { bufferSize?: number; coalesceBurstThreshold?: number }
interface DevtoolsEventBus {
  subscribe(cb: (event: DevtoolsEvent) => void): () => void   // replay + live
  getBuffer(): readonly DevtoolsEvent[]                       // current ring snapshot
  flush(): void                                               // drain pending coalesced burst
  destroy(): void                                             // unsubscribe from engine tap
}

// Multi-store registry (built on top of the bus)
registerStore(bus: DevtoolsEventBus, store: StoreApi<any>, name: string): () => void
getRegisteredStores(): RegisteredStore[]
type RegisteredStore = { name: string; unsubscribe: () => void }
```

**Modified file:** `src/index.ts` — add re-exports for `createDevtoolsEventBus`,
`registerStore`, `getRegisteredStores`, `DevtoolsEventBusOptions`, `DevtoolsEventBus`,
`RegisteredStore`.

All new exports are tree-shaken in production via the existing `process.env.NODE_ENV` guard
pattern established in W3.

## Capabilities

### New Capabilities

- **`devtools-event-bus`**: `createDevtoolsEventBus(opts?)` returns a `DevtoolsEventBus`
  instance that wraps `subscribeDevtoolsEvent` with a ring buffer (default 500), fan-out to
  N panel subscribers with replay-on-connect, and microtask-level burst coalescing into
  `kind: "list"` summary events. All bus instances are independent; the engine tap is held
  for the bus's lifetime and released on `destroy()`.

- **`devtools-store-registry`**: `registerStore(bus, store, name)` wires an additional Zustand
  store instance into an existing bus by calling `store.subscribe` and routing state-change
  diffs into the bus as `DevtoolsEvent` objects. `getRegisteredStores()` returns the live
  registry snapshot consumed by the panel's Subscriptions tab.

### Modified Capabilities

- **`src/index.ts`**: Re-exports only. No logic change.
