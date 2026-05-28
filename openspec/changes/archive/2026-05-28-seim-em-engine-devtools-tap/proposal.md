## Why

The Entity Explorer (changes 7–8) needs a **push stream** of engine-level events to render its "Events" tab and to keep the "Tree" tab live without polling. `src/devtools.ts` already produces *snapshots* via `collectGraphDevStats`; what's missing is a per-op observation channel.

Looking at the actual code surface in the W2 worktree:

- `src/graph.ts` defines the Zustand store with `upsertEntity`, `patchEntity`, `unpatchEntity`, `clearPatch`, `setListResult` (and more) — these are the mutation primitives.
- `src/graph-actions.ts` provides the transactional API every programmatic write goes through: `upsertEntity`, `patchEntity`, `clearPatch`. It calls `useGraphStore.getState().<op>()` for each.
- `src/engine.ts` owns the subscriber registry, dedupe, garbage collection, configuration — all the cross-cutting infrastructure that any new pub/sub surface belongs alongside.

The devtools tap therefore has two natural insertion points:

1. **`src/engine.ts`** owns the subscriber registry: `subscribeDevtoolsEvent(cb): UnsubscribeFn`, a `notifyDevtools(event)` private export, and the `DevtoolsEvent` type union. This is the public observation surface.
2. **`src/graph-actions.ts`** owns the call-sites: each of `upsertEntity`, `patchEntity`, `clearPatch` (and `unpatchEntity` from the store contract) calls `notifyDevtools(...)` after the store mutation.

This is the W3 parallel change to `seim-em-surreal-live-adapter-impl`. Both edit independent files; both land as separate commits on the same `feat/seim-entity-management-impl` topic branch.

## What Changes

### `src/engine.ts` — observation surface

Add three exports:

```ts
export type DevtoolsEvent =
  | { kind: "upsert";    type: EntityType; id: EntityId; data: Record<string, unknown>; at: string }
  | { kind: "patch";     type: EntityType; id: EntityId; patch: Record<string, unknown>; at: string }
  | { kind: "unpatch";   type: EntityType; id: EntityId; keys: string[]; at: string }
  | { kind: "clearPatch";type: EntityType; id: EntityId; at: string }
  | { kind: "list";      key: string; idCount: number; at: string };

export function subscribeDevtoolsEvent(cb: (event: DevtoolsEvent) => void): () => void;

/** Internal — called by graph-actions and store wrappers. */
export function notifyDevtools(event: DevtoolsEvent): void;
```

The subscriber set lives in module scope (one `Set<DevtoolsListener>`). `notifyDevtools` early-returns when `subscribers.size === 0` (the hot-path no-op).

### Tree-shake gate

Every call to `notifyDevtools` is wrapped in a `process.env.NODE_ENV !== "production"` check so the production bundle can drop the call sites. The implementation in `engine.ts` is exported unconditionally — the change 9 tree-shake gate validates that `notifyDevtools` isn't *reachable* from production code paths, not that it's removed from the bundle as a symbol.

The hot-path no-op (early-return on empty subscriber set) makes the runtime cost zero in production even if the tree-shake gate fails to elide the call sites.

### `src/graph-actions.ts` — call sites

Each of the three transactional ops gets a `notifyDevtools` call immediately after the store mutation:

```ts
upsertEntity(type, id, data) {
  useGraphStore.getState().upsertEntity(type, id, data);
  if (process.env.NODE_ENV !== "production") {
    notifyDevtools({ kind: "upsert", type, id, data, at: new Date().toISOString() });
  }
  return /* transaction */;
}
```

Plus parallel additions in `patchEntity` and `clearPatch`.

### `src/index.ts` — re-export

Add the public surface (`subscribeDevtoolsEvent`, `DevtoolsEvent` type) to the existing devtools export block (or alongside it if no such block exists today).

### `src/engine.test.ts` (or new `src/engine-devtools-tap.test.ts`)

A small vitest file: subscribe, call each `graph-actions` op via the test helper, assert that the listener received the right `DevtoolsEvent` payload in the right order. Cover:

- `subscribeDevtoolsEvent` returns a working `UnsubscribeFn` (callback no longer fires post-unsub).
- Multiple subscribers all receive each event.
- Hot-path no-op: zero subscribers → `notifyDevtools` returns immediately (verify by spying on `Date.now`/timestamp creation — should not be invoked).
- Event ordering: upsert → patch → clearPatch → 3 events delivered in that order.
- Event payload shape matches the `DevtoolsEvent` union.

### What this change does NOT include

- **No `DevtoolsEvent` log buffer** — the bus is fire-and-forget. Buffering is W5's `devtools-event-bus.ts` (change 7), which subscribes to this stream and maintains a 1000-entry ring.
- **No store-side `setListResult` instrumentation** — list events are deferred until W5 when the bus needs them; today only the three graph-actions ops emit. (The `DevtoolsEvent` union *includes* `kind: "list"` so the type is forward-compatible.)
- **No `unpatchEntity` instrumentation** — the public `graph-actions.ts` API doesn't expose it; only direct store callers can trigger it, and dev tooling for those is out of scope here.
- **No React hook** (e.g. `useDevtoolsEvents`). The hook can come from change 7's event bus.
- **No telemetry emission.** Future enhancement.

## Capabilities

### New Capabilities

- `entity-engine-devtools-tap`: A `subscribeDevtoolsEvent(cb): UnsubscribeFn` observation surface plus `notifyDevtools(event)` call-sites at every mutating graph-actions op. Hot-path no-op when no subscribers. Production-tree-shakeable via `NODE_ENV` gate. Forward-compatible `DevtoolsEvent` union including `list` kind reserved for future change 7 integration.

### Modified Capabilities

- None as separate spec entries. `entity-graph-engine` gains the new surface; that capability's existing contract is unchanged.

## Impact

- **Risk**: Low. The hot-path is no-op when no one's listening; the change adds an opt-in observation channel without altering any existing behavior. The tree-shake gate (change 9 in W7) is the safety net that prevents the dev tooling from bloating production bundles.
- **Affected files** (worktree):
  - `src/engine.ts` (modified — adds the public surface + private subscriber Set)
  - `src/graph-actions.ts` (modified — 3 op sites gain `notifyDevtools` calls)
  - `src/engine.test.ts` (modified) or `src/engine-devtools-tap.test.ts` (new) — vitest coverage
  - `src/index.ts` (modified — re-export `subscribeDevtoolsEvent` + `DevtoolsEvent`)
- **Cross-repo**: None — all edits in `prometheus-entity-management`.
- **Reversibility**: `git revert` of the commit.
- **Unblocks**: change 7 (`seim-em-explorer-event-bus-registry`) — the bus subscribes to this stream as its primary input. Change 8 (`seim-em-explorer-panel-components`) — the Events tab renders the buffered events. Change 9 (`seim-em-explorer-production-treeshake-check`) — gives the gate a concrete subject to verify.

### Sequencing note

Same artifact-sequence as W0/W1/W2/W3-sibling: this change declares ONE new capability with one spec file. Full 4-artifact spec-driven sequence: `proposal → specs/entity-engine-devtools-tap/spec.md → design → tasks`.

Worktree convention applies: every `/opsx:apply` Bash invocation MUST start with `cd ~/.claude/worktrees/seim-entity-management` per the `bash-cwd-tracking` lesson.
