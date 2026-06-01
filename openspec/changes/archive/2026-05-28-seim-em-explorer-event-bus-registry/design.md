## Context

W5 builds on the W3 `subscribeDevtoolsEvent` tap. Everything lives in one new file
(`src/devtools-event-bus.ts`) plus minimal re-exports in `src/index.ts`. The bus is the
bridge between the engine tap and the panel (W6) / extension (W8).

## Goals / Non-Goals

**Goals:**
- Ring buffer with configurable size and automatic overflow eviction
- Single engine-tap subscription per bus instance; released on `destroy()`
- Replay buffer to new subscribers synchronously at registration time
- Microtask-level burst coalescing into `kind: "list"` summary events
- Module-level store registry with `active` tracking for the Subscriptions tab
- Full vitest coverage (≥25 new tests over the 124 baseline)

**Non-Goals:**
- React hooks (those live in W6)
- Persistence across page reload (in-memory only)
- Cross-origin / cross-frame event routing (Chrome extension bridge is W8)

## Decisions

**D1. Single file: `src/devtools-event-bus.ts`**
Both the bus and the registry live in one file. They share a small amount of internal state
(`_injectEvent` path) and separating them would require exposing internal APIs. One file, two
exported APIs, tested together.

**D2. Ring buffer as a fixed-length array with a write pointer**
A classic circular buffer: `buffer: DevtoolsEvent[]`, `head: number` (write pointer),
`count: number`. Avoids the allocation churn of `Array.shift()` on every overflow at the cost
of ~8 bytes of extra state. `getBuffer()` reconstructs in chronological order from
`(head - count + bufferSize) % bufferSize`.

```ts
function getBuffer(): readonly DevtoolsEvent[] {
  if (count < bufferSize) return buffer.slice(0, count);
  return [...buffer.slice(head), ...buffer.slice(0, head)];
}
```

**D3. Replay is synchronous, inside `subscribe()`**
When a new subscriber calls `bus.subscribe(cb)`, the bus iterates `getBuffer()` and calls
`cb(event)` for each buffered event before adding `cb` to the live-subscriber set and
returning the unsubscribe function. This guarantees the subscriber's first N calls happen
before it enters the live stream — important for the Events tab initial render.

**D4. Burst coalescing uses `Promise.resolve().then()` (microtask queue)**
When the bus receives an event and the in-flight burst count hits `coalesceBurstThreshold`,
it schedules a flush via `Promise.resolve().then(flush)`. This means the flush happens at
the end of the current microtask checkpoint, after all synchronous op-site calls. The
`flush()` method is also public so tests and the extension bridge can force it synchronously.

```ts
let pendingBurst: DevtoolsEvent[] = [];
let flushScheduled = false;

function handleEngineEvent(event: DevtoolsEvent) {
  pendingBurst.push(event);
  if (pendingBurst.length <= threshold) {
    dispatchOne(event); // below threshold — forward immediately
    pendingBurst = [];
    return;
  }
  if (!flushScheduled) {
    flushScheduled = true;
    Promise.resolve().then(flush);
  }
}
```

`flush()` coalesces `pendingBurst` into individual events up to `threshold`, then one
`kind: "list"` summary for the remainder, then clears `pendingBurst`.

**D5. `_injectEvent` is package-internal, not exported**
The registry routes events from external sources into the bus via a closure reference to
an internal `injectEvent(event)` function returned alongside the `DevtoolsEventBus` object
(or stored as a symbol-keyed property). It is not exported from `src/index.ts` and not part
of the `DevtoolsEventBus` interface — only `registerStore` (in the same file) can access it.

Concretely, `createDevtoolsEventBus` returns `{ bus, _inject }` internally, but the public
overload returns only `bus`. `registerStore` lives in the same module and has closure access.

**D6. Registry is module-level (a `Map<string, RegistryEntry>`)**
Module-level state mirrors the singleton pattern of `devtoolsSubscribers` in `engine.ts`.
`__resetStoreRegistry()` is exported for tests and calls `registry.clear()`. It is
intentionally omitted from `src/index.ts`.

**D7. Duplicate name guard applies to active registrations only**
The guard checks `registry.get(name)?.active === true` — not mere existence. Registering
the same name on a second bus after the first is destroyed is permitted (the entry is now
`active: false`). This matches the spec's scenario for re-registration after destroy.

**D8. `bus.destroy()` cascades to registry**
On destroy, the bus iterates all registry entries where `entry.busRef === bus` and calls
their internal unsubscribe + marks `active: false`. This keeps the registry consistent
without requiring callers to manually unregister every store before destroying the bus.

**D9. Commit scope**
```
feat(devtools): event bus + multi-store registry (W5)
```
Files changed: `src/devtools-event-bus.ts` (new), `src/devtools-event-bus.test.ts` (new),
`src/index.ts` (re-exports only).

## Risks

- **Microtask coalescing interacts with fake timers in tests** — the same `vi.useFakeTimers`
  issue surfaced in W3 applies here. Use `flushPromises()` (from `@vitest/utils` or a manual
  `await Promise.resolve()`) in tests that need to observe coalesced output, rather than
  `vi.runAllTimers()`. Document this in a test comment.
- **Circular dependency risk** — `devtools-event-bus.ts` imports `subscribeDevtoolsEvent` from
  `engine.ts`; `engine.ts` does not import `devtools-event-bus.ts`. No cycle. Verify with
  `madge` or `pnpm tsc --noEmit` if uncertain.
- **`process.env.NODE_ENV` guard** — the bus module itself need not be wrapped in a NODE_ENV
  guard (it has no module-level side effects per D6). Only the `createDevtoolsEventBus()` call
  site in the panel (W6) should be guarded. Document this distinction clearly in JSDoc.
