## Context

The W2-provisioned worktree at `~/.claude/worktrees/seim-entity-management` already carries one W3 commit (the surreal-live adapter at `9f314ba`). This change adds the second W3 commit: an in-process pub/sub for devtools events, plus call-site instrumentation in `src/graph-actions.ts`.

Files in play (verified at proposal-drafting time):

- `src/engine.ts` — owns `subscribers` (the existing per-key list-subscription registry), `dedupe`, `startGarbageCollector`, `configureEngine`, plus the existing `subscribeSubscriberStats` / `getActiveSubscriberCount` pair that `devtools.ts` already consumes. Natural home for the new `subscribeDevtoolsEvent` / `notifyDevtools` pair.
- `src/graph-actions.ts` — three mutating ops: `upsertEntity`, `patchEntity`, `clearPatch`. Each one calls `useGraphStore.getState().<op>(...)`. Adding a `notifyDevtools` call after each store call is mechanical.
- `src/index.ts` — has an existing "Subscriber stats (low-level)" or similar export block (around the `subscribeSubscriberStats` re-export, ~line 200). The new exports slot in alongside.

The change does not touch `src/devtools.ts` (the existing snapshot helper). Snapshot vs. push-stream stay independent surfaces.

## Goals / Non-Goals

**Goals**
- Single-file implementation of the pub/sub primitive in `engine.ts` (≤ 60 LOC including type union).
- Three call-site additions in `graph-actions.ts` (≤ 30 LOC including the `NODE_ENV` guards).
- Re-entrancy safe (snapshot iteration).
- Hot-path zero-cost when no subscribers.
- Tests: a new file `src/engine-devtools-tap.test.ts` (~120 LOC) with at least 12 assertions, mapping to every spec requirement.
- Build passes; full vitest suite stays green (current baseline 104 tests).

**Non-Goals**
- No event-buffer ring. That's change 7 (`seim-em-explorer-event-bus-registry`).
- No instrumentation of `setListResult`, `setListError`, `unpatchEntity`, etc. — deferred to change 7 when the bus actually consumes them.
- No React hook. Also change 7.
- No telemetry/metrics fanout. Future enhancement.

## Decisions

### D1. `Set<DevtoolsListener>` not `Array`

`Set` gives O(1) `add`/`delete`/`has` and idempotent `add` (the same function added twice is still one entry). The spec mandates the latter (Subscriber Lifecycle — "subscribe of the same function twice is no-op").

Array would require manual dedupe + `indexOf`-based removal. No upside.

### D2. Snapshot-of-subscribers iteration for re-entrancy

Pattern:

```ts
function notifyDevtools(event: DevtoolsEvent): void {
  if (subscribers.size === 0) return;
  // Snapshot at the top — re-entrant subscribers that subscribe/unsubscribe
  // during iteration affect future events, not this one.
  const listeners = [...subscribers];
  for (const cb of listeners) {
    try {
      cb(event);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn("[engine] devtools subscriber threw:", err);
    }
  }
}
```

The `[...subscribers]` spread copies the set's current members into a new array. A subscriber that calls `subscribeDevtoolsEvent(...)` or its returned `UnsubscribeFn` during dispatch mutates the underlying `Set` but does NOT change `listeners`, so iteration is safe.

Why not `subscribers.forEach`? `Set.prototype.forEach` *does* observe in-flight mutations in some edge cases (V8 / SpiderMonkey behavior varies on subset of ECMAScript spec edge cases). Snapshot is the portable safe path.

### D3. `try/catch` around each subscriber

Spec mandates that a throwing subscriber does NOT block delivery to others. The `try/catch` lives inside the iteration, around each invocation. `console.warn` records the error; no swallowing without a trace.

### D4. Hot-path early return is the FIRST statement

The spec calls out an "explicit early-return guard." Implementation puts `if (subscribers.size === 0) return;` as the literal first statement of `notifyDevtools`, before any allocation or timestamp creation. Two reasons:

1. Operations like `new Date().toISOString()` allocate a `Date` plus a string. If we built the event object at the call site (we don't — see D6) and that included a `Date.now()`, an empty-subscriber check at the *end* would already have paid the allocation cost.
2. Code review readability: the guard's intent is unmissable when it's the first line.

### D5. NODE_ENV guard at call sites, not at the dispatcher

The spec requires the call sites in `graph-actions.ts` to be tree-shake-elidable. The dispatcher in `engine.ts` stays unguarded (it's exported as a public symbol; consumers may install dev tooling that calls it directly outside `NODE_ENV`).

Wrap each call-site like:

```ts
upsertEntity(type, id, data) {
  useGraphStore.getState().upsertEntity(type, id, data);
  if (process.env.NODE_ENV !== "production") {
    notifyDevtools({ kind: "upsert", type, id, data, at: new Date().toISOString() });
  }
  return transaction;
}
```

`tsup` + esbuild perform `process.env.NODE_ENV` dead-code elimination when given `NODE_ENV=production` at build time. The whole `if`-block disappears from the prod bundle. This means the payload literal `kind: "upsert"` doesn't appear in the prod bundle either — exactly what the W7 tree-shake gate (change 9) needs to assert.

### D6. Build the event at the call site, not in `notifyDevtools`

Per D5, the `kind: "..."` literal lives at the call site. That keeps the literal under the `NODE_ENV` gate (and thus elidable). If `notifyDevtools` accepted a builder function instead (`notifyDevtools((): DevtoolsEvent => ({...}))`), the builder would execute even in prod under the guard — pointless complexity for no gain.

### D7. `DevtoolsEvent` union shape and discriminator-first

The union is plain `kind` discriminator + per-kind payload, no class wrappers, no factory. Members:

```ts
export type DevtoolsEvent =
  | { kind: "upsert";     type: EntityType; id: EntityId; data: Record<string, unknown>; at: string }
  | { kind: "patch";      type: EntityType; id: EntityId; patch: Record<string, unknown>; at: string }
  | { kind: "unpatch";    type: EntityType; id: EntityId; keys: string[]; at: string }
  | { kind: "clearPatch"; type: EntityType; id: EntityId; at: string }
  | { kind: "list";       key: string; idCount: number; at: string };
```

The two forward-compatible kinds (`unpatch`, `list`) are part of the type but NOT instrumented in this change. Their inclusion lets W5/W6 consume the union as-is without a type-only follow-up change.

`at: string` is ISO-8601 UTC. Consumers parse with `new Date(at)`. The string form is human-debuggable in the Events tab (change 8); a `number` (epoch ms) would force a render-time conversion.

### D8. Single internal `subscribers` Set, not per-kind Sets

Per-kind Sets would let consumers filter at registration time (e.g. "only `upsert` events"). Rejected — premature optimisation. Consumers filter on the discriminator in their callback. Easy to retrofit if a real use case appears.

### D9. No public `notifyDevtools` export

`notifyDevtools` is exported FROM `engine.ts` so `graph-actions.ts` can import it, but it is NOT re-exported through `src/index.ts`. External callers should not be calling it directly — they observe via `subscribeDevtoolsEvent`. Making it internal-only keeps the public API surface minimal.

This is a minor convention choice; if a consumer has a legitimate need to fire synthetic events (e.g. integration testing tools), we can promote `notifyDevtools` to public in a follow-up.

### D10. Test infrastructure

A new file `src/engine-devtools-tap.test.ts` (parallel to `electricsql-tenant.test.ts` etc.). Uses:

- The real `useGraphStore` (no fake) — the test exercises the actual chain: `graph-actions.upsertEntity` → store mutation → `notifyDevtools` → subscriber.
- Resets the store between tests via the existing reset utility if any, or by explicit `clearPatch` cleanup.
- Asserts both subscriber receipt AND store state (the events are *complement* to the store; one without the other is incomplete observability).

Tests cover the 8 spec requirements via 12+ `it` blocks. Per spec scenario "Tests file exists" — file at `src/engine-devtools-tap.test.ts`.

## Implementation Sketch

### `src/engine.ts` (additions)

```ts
import type { EntityType, EntityId } from "./graph";

// ── DevtoolsEvent + tap (W3 — entity-engine-devtools-tap capability) ────────

export type DevtoolsEvent =
  | { kind: "upsert";     type: EntityType; id: EntityId; data: Record<string, unknown>; at: string }
  | { kind: "patch";      type: EntityType; id: EntityId; patch: Record<string, unknown>; at: string }
  | { kind: "unpatch";    type: EntityType; id: EntityId; keys: string[]; at: string }
  | { kind: "clearPatch"; type: EntityType; id: EntityId; at: string }
  | { kind: "list";       key: string; idCount: number; at: string };

type DevtoolsListener = (event: DevtoolsEvent) => void;

const devtoolsSubscribers = new Set<DevtoolsListener>();

/**
 * Subscribe to DevtoolsEvent stream. Returns an UnsubscribeFn that
 * removes this listener. Idempotent: subscribing the same function
 * twice tracks it once. Throwing listeners do NOT block delivery to
 * other listeners; the error is logged via console.warn.
 */
export function subscribeDevtoolsEvent(cb: DevtoolsListener): () => void {
  devtoolsSubscribers.add(cb);
  return () => {
    devtoolsSubscribers.delete(cb);
  };
}

/**
 * Internal — called from graph-actions.ts at every mutating op site.
 * Hot-path no-op when no subscribers registered.
 */
export function notifyDevtools(event: DevtoolsEvent): void {
  if (devtoolsSubscribers.size === 0) return;
  const listeners = [...devtoolsSubscribers]; // snapshot for re-entrancy safety
  for (const cb of listeners) {
    try {
      cb(event);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn("[engine] devtools subscriber threw:", err);
    }
  }
}
```

### `src/graph-actions.ts` (op-site additions)

```ts
import { notifyDevtools } from "./engine";

// inside the transaction factory:

upsertEntity(type, id, data) {
  useGraphStore.getState().upsertEntity(type, id, data);
  if (process.env.NODE_ENV !== "production") {
    notifyDevtools({ kind: "upsert", type, id, data, at: new Date().toISOString() });
  }
  return /* transaction */;
},

patchEntity(type, id, patch) {
  useGraphStore.getState().patchEntity(type, id, patch);
  if (process.env.NODE_ENV !== "production") {
    notifyDevtools({ kind: "patch", type, id, patch, at: new Date().toISOString() });
  }
  return /* transaction */;
},

clearPatch(type, id) {
  useGraphStore.getState().clearPatch(type, id);
  if (process.env.NODE_ENV !== "production") {
    notifyDevtools({ kind: "clearPatch", type, id, at: new Date().toISOString() });
  }
  return /* transaction */;
},
```

### `src/index.ts` (re-export)

Slot alongside the existing `subscribeSubscriberStats` re-export (devtools group):

```ts
// In whatever block re-exports devtools helpers
export { subscribeDevtoolsEvent } from "./engine";
export type { DevtoolsEvent } from "./engine";
```

### `src/engine-devtools-tap.test.ts` (skeleton)

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { subscribeDevtoolsEvent, type DevtoolsEvent } from "./engine";
import { createEntityGraphActions } from "./graph-actions";   // verify exact export name during apply

describe("entity-engine-devtools-tap — public observation surface", () => { … 4 it() … });
describe("entity-engine-devtools-tap — event payload shape", () => { … 4 it() … });
describe("entity-engine-devtools-tap — op-site instrumentation", () => { … 4 it() … });
describe("entity-engine-devtools-tap — hot-path no-op", () => { … 2 it() … });
describe("entity-engine-devtools-tap — production tree-shake gate", () => { … 1 it.todo() — verified by W7 gate … });
describe("entity-engine-devtools-tap — subscriber lifecycle", () => { … 3 it() … });
describe("entity-engine-devtools-tap — re-entrancy safety", () => { … 2 it() … });
// Total: ~20 it() across 7 describes (one is .todo for W7).
```

## Risks

1. **`graph-actions.ts` export shape unknown until apply** — the proposal references the transactional API, but the exact factory name / interface needs confirmation during apply. Mitigation: read the file first, adapt to the actual shape; the tests verify the externally observable behavior regardless of internal naming.
2. **`process.env.NODE_ENV` in `vitest`** — vitest sets `NODE_ENV=test` by default, so all the `notifyDevtools` calls execute in tests. That's the correct behavior for testing the tap. The W7 tree-shake gate validates the prod path separately.
3. **Re-entrancy snapshot allocates per dispatch** — `[...subscribers]` creates a new array each call. For zero subscribers this never runs (D4 guard); for n subscribers the cost is O(n) allocation. Acceptable — the alternative (mutable iteration) is unsound.
4. **`unpatch` and `list` kinds in the union but not emitted** — TypeScript narrowing on a discriminator means consumers MUST handle all 5 variants in exhaustive `switch`. Documented in the spec; W5+ consumers will emit those kinds when the bus is ready.
5. **`graph-actions.ts` might be wrapped by an outer transaction layer** — adding `notifyDevtools` calls *inside* the action methods means the notification fires regardless of whether the outer transaction is later rolled back. If transaction semantics matter for observability (rollback should also emit an event), this is wrong. Mitigation: investigate transaction model during apply; if a "transaction committed" hook exists, prefer that as the notification point.

## Alternatives Considered

- **Per-kind Sets** (`upsertSubscribers`, `patchSubscribers`, ...) for registration-time filter. Rejected (D8).
- **Instrument the Zustand store itself** instead of `graph-actions.ts`. Rejected — the store's mutating functions are called by code paths other than `graph-actions.ts` (Zustand allows direct `useGraphStore.setState`), and instrumenting at the store means `notifyDevtools` fires from places that aren't part of the documented public API. `graph-actions.ts` is the API boundary; instrument there.
- **Single `notifyDevtools(kind, payload)` signature** to centralize the timestamp creation. Rejected — would force `at` field allocation in the dispatcher rather than at the call site, breaking the NODE_ENV gate (D5/D6).
- **EventEmitter from `node:events`**. Rejected — adds a Node-only API to a package that ships browser-targetable builds.
- **RxJS / observable**. Rejected — dependency. Callback-with-unsubscribe is the package's existing convention (see `subscribeSubscriberStats`).
