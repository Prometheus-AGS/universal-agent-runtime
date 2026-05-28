## 1. Scaffold `src/devtools-event-bus.ts`

- [x] 1.1 Create `~/.claude/worktrees/seim-entity-management/src/devtools-event-bus.ts`
- [x] 1.2 Import `subscribeDevtoolsEvent` and `DevtoolsEvent` from `"./engine"`
- [x] 1.3 Define and export `DevtoolsEventBusOptions`, `DevtoolsEventBus`, `DevtoolsSourceFn`, `RegisteredStore` types
- [x] 1.4 Define internal `RegistryEntry` type: `{ name: string; active: boolean; busRef: symbol; unsubscribeSrc: () => void }`

## 2. Implement ring buffer

- [x] 2.1 Implement `createDevtoolsEventBus(opts?)` factory with circular buffer (`buffer[]`, `head`, `count`, `bufferSize`)
- [x] 2.2 Implement `getBuffer(): readonly DevtoolsEvent[]` — returns chronological snapshot (oldest→newest)
- [x] 2.3 Wire the bus's internal `handleEngineEvent` to `subscribeDevtoolsEvent` immediately on construction (one engine tap per bus)

## 3. Implement fan-out with replay

- [x] 3.1 Implement `subscribe(cb)` — replay `getBuffer()` synchronously to `cb` before adding to live-subscriber set; return `UnsubscribeFn`
- [x] 3.2 Ensure `dispatchToSubscribers(event)` uses a snapshot of the subscriber set (re-entrancy safety, mirrors W3 pattern)
- [x] 3.3 Wrap each subscriber call in `try/catch`; log errors via `console.warn("[devtools-event-bus]", err)`

## 4. Implement burst coalescing

- [x] 4.1 Add `pendingBurst: DevtoolsEvent[]` and `flushScheduled: boolean` to bus closure state
- [x] 4.2 In `handleEngineEvent`: below threshold → dispatch immediately + clear burst; at/above → accumulate + schedule `Promise.resolve().then(flush)` once
- [x] 4.3 Implement `flush()` — dispatches individual events up to threshold, then one `{ kind: "list", key: "burst", idCount: N, at: ISO }` for the rest; resets `pendingBurst` and `flushScheduled`
- [x] 4.4 When `coalesceBurstThreshold === 0`, skip accumulation entirely — dispatch every event immediately

## 5. Implement `destroy()`

- [x] 5.1 `destroy()` calls the engine-tap unsubscribe fn, clears the subscriber set, marks a `destroyed` flag
- [x] 5.2 After `destroy()`, calls to `subscribe()` and `flush()` are no-ops (guard on `destroyed` flag)
- [x] 5.3 `destroy()` iterates the module-level registry and marks all entries with `busRef === this bus's symbol` as `active: false` and calls their `unsubscribeSrc()`

## 6. Implement multi-store registry

- [x] 6.1 Declare module-level `registry = new Map<string, RegistryEntry>()` and a unique `busSymbol` per bus instance (use `Symbol()`)
- [x] 6.2 Implement `registerStore(bus, source, name)`:
  - Throw if `registry.get(name)?.active === true` (duplicate active name)
  - Call `source(event => injectIntobus(event))` to get `unsubscribeSrc`
  - Store `{ name, active: true, busRef: bus._symbol, unsubscribeSrc }` in registry
  - Return an `UnsubscribeFn` that calls `unsubscribeSrc()`, sets `entry.active = false`
- [x] 6.3 Implement `getRegisteredStores()` — returns a new array of `{ name, active }` for all entries
- [x] 6.4 Implement `__resetStoreRegistry()` — calls `registry.clear()` (test-only; NOT exported from `index.ts`)
- [x] 6.5 Internal `injectIntobus(event)` routes event into the bus's `handleEngineEvent` path (same ring buffer + coalescing + fan-out as engine events)

## 7. Update `src/index.ts`

- [x] 7.1 Add re-exports: `createDevtoolsEventBus`, `registerStore`, `getRegisteredStores`
- [x] 7.2 Add type re-exports: `DevtoolsEventBusOptions`, `DevtoolsEventBus`, `DevtoolsSourceFn`, `RegisteredStore`
- [x] 7.3 Do NOT export `__resetStoreRegistry` or any internal symbol

## 8. Write `src/devtools-event-bus.test.ts`

- [x] 8.1 Create test file; import bus + registry + `__resetStoreRegistry`; add `beforeEach(() => __resetStoreRegistry())`
- [x] 8.2 **Bus — ring buffer:** empty start, populates in order, evicts oldest on overflow, `getBuffer()` snapshot stability
- [x] 8.3 **Bus — replay:** new subscriber receives buffer synchronously before `subscribe()` returns; live events arrive after
- [x] 8.4 **Bus — multi-subscriber:** two subscribers both get replay + live; unsubscribe stops one without affecting the other
- [x] 8.5 **Bus — error isolation:** throwing subscriber doesn't block sibling
- [x] 8.6 **Bus — coalescing below threshold:** 9 events → 9 individual dispatches, no `kind: "list"`
- [x] 8.7 **Bus — coalescing above threshold:** 12 events → 10 individual + 1 `kind: "list"` (after `await flush()` or `flushPromises()`)
- [x] 8.8 **Bus — `flush()` forces coalesced output synchronously**
- [x] 8.9 **Bus — `destroy()`:** no more events, idempotent, clears subscribers
- [x] 8.10 **Registry — routing:** registered source events reach bus subscribers
- [x] 8.11 **Registry — multi-source fan-in:** two sources, both reach subscribers in arrival order
- [x] 8.12 **Registry — duplicate name throws**
- [x] 8.13 **Registry — unsubscribe:** stops routing, marks `active: false`, idempotent
- [x] 8.14 **Registry — snapshot:** immutable, reflects active/inactive state correctly
- [x] 8.15 **Registry — bus destroy cascades:** `bus.destroy()` → registered source marked `active: false`
- [x] 8.16 **Registry — re-registration after destroy:** allowed when prior entry is `active: false`

## 9. Run tests and verify

- [x] 9.1 `cd ~/.claude/worktrees/seim-entity-management && pnpm test` — confirm all new tests pass
- [x] 9.2 Confirm total suite count ≥ 149 (124 baseline + ≥25 new); zero regressions
- [x] 9.3 `pnpm tsc --noEmit` — no TypeScript errors

## 10. Commit

- [x] 10.1 `git add src/devtools-event-bus.ts src/devtools-event-bus.test.ts src/index.ts`
- [x] 10.2 `git diff --cached --name-only` — confirm exactly 3 files staged
- [x] 10.3 Commit:
  ```
  feat(devtools): event bus + multi-store registry (W5)
  ```
- [x] 10.4 `git diff --name-only HEAD~1 HEAD` — verify exactly 3 files

## 11. Progress update

- [x] 11.1 Update `progress.json`: move `seim-em-explorer-event-bus-registry` to `completed_changes`, set `changes_completed: 7`, set `active_change: "seim-em-explorer-panel-components"`, add commit sha to `entity_mgmt_worktree_local_commits[]`, update `updatedAt`
