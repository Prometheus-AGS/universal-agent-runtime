## Context

This change implements `createSurrealLiveAdapter` against the corrected spec at `openspec/specs/entity-surreal-live-adapter/spec.md`. All TS edits land in the W2-provisioned worktree at `~/.claude/worktrees/seim-entity-management` on branch `feat/seim-entity-management-impl`.

Reference patterns in the codebase:

- **`src/adapters/electricsql.ts`** — sibling `SyncAdapter` implementing `subscribe(config, handler)`. The shape of the export, the import of `{ RealtimeManager / SyncAdapter / SubscriptionConfig / ChangeSet / EntityChange / AdapterStatus / UnsubscribeFn }` from `./types`, the use of `getRealtimeManager()`, and the minimal-surface-types pattern (declaring `interface PGlite { … }` instead of taking a hard dep on `@electric-sql/pglite`) all carry over.
- **`src/adapters/electricsql-tenant.test.ts`** — sets the test pattern: `fakeShapeStream()` and `fakePGlite()` helpers produce a fully-controlled fake that the test can drive synchronously. Our `fakeSurreal()` follows the same shape: an object with `.live`, `.kill`, `.query` methods plus driver-side controls (`.emit(notification)`, `.simulateDisconnect()`, `.simulateReconnect()`).
- **`src/adapters/types.ts`** — the binding interface: `RealtimeAdapter { name, subscribe, onStatusChange? }`. Our adapter implements exactly this shape; nothing else.

The adapter does NOT extend `SyncAdapter` (spec D1) — the consumer keeps full control of their SurrealDB client; the adapter only orchestrates live queries.

## Goals / Non-Goals

**Goals**
- Conform to every requirement and scenario in the corrected spec (10 requirements, 33 scenarios).
- Code surface ≤ 350 LOC for the implementation file; ≤ 600 LOC including the test file.
- Tests run synchronously where possible (no real timers); use `vi.useFakeTimers()` for backoff verification.
- Per-channel state machine is encapsulated in a single internal class `SurrealChannel` so the public factory stays declarative.
- Single-source-of-truth for status aggregation: a private `_status` field updated atomically through one `_setChannelStatus(channel, status)` method.

**Non-Goals**
- No real SurrealDB integration tests. The vitest suite is hermetic.
- No React hooks. Wiring example in the companion skill (already shipped) covers the consumer pattern.
- No telemetry / metrics emission. Future enhancement.
- No connection pooling. One `SurrealLike` instance per adapter; consumers manage the client lifecycle.

## Decisions

### D1. Client injection — caller passes a `SurrealLike` instance

```ts
export interface SurrealLike {
  query<T = unknown>(sql: string, vars?: Record<string, unknown>): Promise<T[]>;
  live<T = unknown>(table: string, callback: (action: SurrealLiveAction<T>) => void, diff?: boolean): Promise<string>;
  kill(uuid: string): Promise<void>;
}
```

Why injection: the consumer owns the client connection (authentication, namespace/database selection, WebSocket lifecycle). The adapter has no business owning a client it didn't authenticate. The `SurrealLike` interface is the minimal subset the adapter needs — works against `surrealdb` 1.x (`new Surreal()`) and 2.x with no version pinning. Type-only import; no runtime dep on `surrealdb`.

Alternative considered: factory function `clientFactory(): SurrealLike`. Rejected — consumers already have an established client instance in their app bootstrapping; passing it directly is simpler than re-instantiating per adapter.

### D2. Per-channel state machine — `SurrealChannel` internal class

Encapsulates the per-channel concerns spec mandates:

```ts
class SurrealChannel {
  readonly key: string;                  // adapter.name/label-or-typehash — checkpoint keying (spec)
  readonly config: SubscriptionConfig;
  readonly channelConfig: ChannelConfig;
  readonly handler: (cs: ChangeSet) => void;

  private liveUuid: string | null = null;
  private status: AdapterStatus = "connecting";
  private buffer: SurrealLiveAction<Row>[] = [];
  private seedComplete = false;
  private attempt = 0;
  private connectedAt: number | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private aborted = false;

  async start(): Promise<void> { … }      // run initial select, open live, prime buffer flush
  async stop(): Promise<void> { … }       // kill live, clear timer, set aborted
  private async _seed(): Promise<void> { … }
  private async _openLive(): Promise<void> { … }
  private _onAction(action: SurrealLiveAction<Row>): void { … }
  private _scheduleReconnect(reason?: Error): void { … }
  private _setStatus(s: AdapterStatus): void { … }
}
```

The adapter holds a `Map<string, SurrealChannel>` keyed by `SubscriptionConfig.label` (which `RealtimeManager.register` populates as `${adapter.name}/${channel.type}`). Each `subscribe(config, handler)` call constructs a channel, starts it, and returns an `UnsubscribeFn` that calls `channel.stop()` and removes the entry.

### D3. Seed-then-live ordering — Promise chain, not a queue

The spec mandates that live notifications arriving *during* the initial `SELECT` must be buffered and flushed after the seed `ChangeSet`. Implementation:

```ts
async start(): Promise<void> {
  this._setStatus("connecting");

  // Open live FIRST so we don't lose notifications between SELECT-end and LIVE-start
  await this._openLive();
  // notifications now arrive into _onAction → pushed into this.buffer

  // Then seed
  if (this.config.replayOnConnect !== false && this.opts.initialQueryStrategy !== "live-only") {
    const rows = await this.surreal.query<Row>(this._buildSeedSql(), this._buildVars());
    const seedChangeset: ChangeSet = {
      changes: rows.map(r => this._rowToChange("insert", r)),
      timestamp: new Date().toISOString(),
      affectedListKeys: this._resolveListKeys(rows.map(r => this._rowToChange("insert", r))),
    };
    this.handler(seedChangeset);
  }

  // Flush any buffered live actions in arrival order
  this.seedComplete = true;
  const buffered = this.buffer.splice(0);
  for (const action of buffered) this._onAction(action);  // re-entry; emits straight through

  this._setStatus("connected");
  this.connectedAt = Date.now();
}
```

`_onAction` checks `seedComplete`: if false, push to buffer; if true, map+emit immediately. This is simpler than maintaining a separate queue object — the buffer is just a `Row[]` field on the channel.

### D4. Reconnect backoff with jitter + cancellation

```ts
private _scheduleReconnect(reason?: Error): void {
  if (this.aborted) return;

  // Detect permanent errors first (spec: auth, schema → no reconnect)
  if (this._isPermanent(reason)) {
    this._setStatus("error");
    return;
  }

  this._setStatus("disconnected");
  this.attempt += 1;

  const base = Math.min(
    this.opts.initialDelayMs * (2 ** Math.min(this.attempt - 1, 6)),
    this.opts.maxDelayMs,
  );
  const jitter = base * 0.25 * (Math.random() * 2 - 1);  // ±25%
  const delay = Math.max(0, Math.round(base + jitter));

  this._setStatus("connecting");
  this.reconnectTimer = setTimeout(async () => {
    if (this.aborted) return;
    try {
      await this._openLive();          // re-issue LIVE SELECT
      if (this.opts.checkpointStore) await this._replayFromCheckpoint();
      this._setStatus("connected");
      this.connectedAt = Date.now();
    } catch (e) {
      this._scheduleReconnect(e as Error);
    }
  }, delay);
}
```

Cancellation: `stop()` sets `aborted = true` and `clearTimeout(this.reconnectTimer)`. The "aborted" check at the top of the timer callback prevents a late-firing reconnect from re-opening a killed channel.

Permanent-error detection (D4b): inspect `reason.message` for the canonical SurrealDB markers — `"authentication"`, `"namespace"`, `"unknown table"`, `"permission"`. Hard-coded list; not exhaustive but covers the documented auth + schema cases. Anything else is treated as transient.

### D4c. Attempt-counter reset after `connectedSettleMs`

A `setTimeout(connectedSettleMs)` armed inside `_setStatus("connected")` resets `this.attempt = 0`. Cleared on next status change.

### D5. Checkpoint store interface

```ts
export interface SurrealCheckpointStore {
  get(channelKey: string): Promise<string | undefined>;
  set(channelKey: string, value: string): Promise<void>;
}
```

Two-method async interface. Consumers can back it with localStorage / IndexedDB / a remote KV — all out of scope here. Per-channel keying via `channel.key` (computed in D2). Default checkpoint field: `updated_at`. Configurable via `opts.checkpointField` (default `"updated_at"`).

The replay query, run on reconnect when `checkpointStore` is supplied:

```sql
SELECT * FROM <table> WHERE <filter> AND <checkpointField> > $stored ORDER BY <checkpointField> ASC
```

`stored` is bound as a vars parameter — never interpolated.

### D6. Action normalization

The adapter receives `{ action: "CREATE" | "UPDATE" | "DELETE" | "CLOSE" | string, result: Row, ... }` from SurrealDB's live callback. Mapping:

```ts
function mapAction(a: string): ChangeOperation | "close" | null {
  switch (a.toUpperCase()) {
    case "CREATE": return "insert";
    case "UPDATE": return "update";
    case "DELETE": return "delete";
    case "CLOSE":  return "close";
    default:       return null;
  }
}
```

`close` → reconnect path. `null` → warn + skip. Spec D4 mandates that `update` always carries the full row (SurrealDB live notifications do), so `EntityChange.patch` is left undefined.

### D7. `listKeyResolver` opt-in

```ts
opts.listKeyResolver?: (change: EntityChange) => string[] | undefined
```

Applied at `_rowToChange` time. For `update` changes specifically, called only if the resolver opted into updates (no special flag — the resolver is the only signal); for `insert` and `delete`, always called when present. Dedup via `Set` at `ChangeSet` assembly time.

When `listKeyResolver` is absent → `ChangeSet.affectedListKeys = undefined`. Consumer's `RealtimeManager` already coalesces via 16ms flush; no list-key info means no targeted list refresh, which is fine.

### D8. `ChannelConfig.operations` filter — honor it

The W1 design risks §3 flagged this as undocumented. Decision: **honor**. After mapping the SurrealDB action to a `ChangeOperation`, check membership against `channelConfig.operations` if present; drop the notification (no handler call) if the op isn't allowed.

```ts
const op = mapAction(action.action);
if (op === null) { warn; return; }
if (op === "close") { _scheduleReconnect(); return; }
if (channelConfig.operations && !channelConfig.operations.includes(op)) return;
const change = this._rowToChange(op, action.result);
…
```

Documented in tasks.md §X as part of the implementation contract.

### D9. Status aggregation — single source of truth

The adapter owns one `Map<string, AdapterStatus>` keyed by `channel.key`. Every `channel._setStatus` invocation calls back into `adapter._recomputeStatus()`:

```ts
private _recomputeStatus(): void {
  const states = [...this.channelStates.values()];
  const order: AdapterStatus[] = ["error", "disconnected", "connecting", "connected"];
  const aggregate = order.find(s => states.includes(s)) ?? "connecting";
  if (aggregate !== this._status) {
    this._status = aggregate;
    for (const cb of this.statusCallbacks) cb(aggregate);
  }
}
```

Only fires callbacks on actual transitions (spec D7 in W1 design).

### D10. Test infrastructure

`fakeSurreal()`:

```ts
function fakeSurreal() {
  const liveCallbacks = new Map<string, (a: SurrealLiveAction<any>) => void>();
  const queryResponses: Array<unknown[]> = [];   // FIFO; tests prime this before triggering a query
  let nextUuid = 0;
  return {
    // SurrealLike implementation
    query: vi.fn(async () => queryResponses.shift() ?? []),
    live: vi.fn(async (_table: string, cb: (a: any) => void) => {
      const uuid = `live-${++nextUuid}`;
      liveCallbacks.set(uuid, cb);
      return uuid;
    }),
    kill: vi.fn(async (uuid: string) => { liveCallbacks.delete(uuid); }),
    // Test controls
    _emit: (uuid: string, action: any) => liveCallbacks.get(uuid)?.(action),
    _disconnect: (uuid: string) => liveCallbacks.get(uuid)?.({ action: "CLOSE", result: null }),
    _pendingQueries: () => queryResponses.length,
    _liveCount: () => liveCallbacks.size,
    _primeQuery: (rows: unknown[]) => queryResponses.push(rows),
  };
}
```

Tests use `vi.useFakeTimers()` to control backoff schedules deterministically. Async assertions use `await flushPromises()` (a small helper that does `await new Promise(r => setImmediate(r))`).

## Implementation Sketch

```ts
// src/adapters/surreal-live.ts (~280 LOC target)

import type {
  RealtimeAdapter, SubscriptionConfig, ChannelConfig,
  ChangeSet, EntityChange, ChangeOperation,
  AdapterStatus, UnsubscribeFn,
} from "./types";
import type { EntityType, EntityId } from "../graph";

// ── Public surface ────────────────────────────────────────────────────────
export interface SurrealLike {
  query<T = unknown>(sql: string, vars?: Record<string, unknown>): Promise<T[]>;
  live<T = unknown>(table: string, callback: (action: SurrealLiveAction<T>) => void, diff?: boolean): Promise<string>;
  kill(uuid: string): Promise<void>;
}

export interface SurrealLiveAction<T = unknown> {
  action: "CREATE" | "UPDATE" | "DELETE" | "CLOSE" | string;
  result: T;
}

export interface SurrealCheckpointStore {
  get(channelKey: string): Promise<string | undefined>;
  set(channelKey: string, value: string): Promise<void>;
}

export interface SurrealTableConfig {
  type: EntityType;
  table?: string;                                       // defaults to `type`
  idField?: string;                                     // defaults to "id"
}

export interface SurrealLiveAdapterOptions {
  name?: string;                                        // default "surreal-live"
  surreal: SurrealLike;
  tables: SurrealTableConfig[];
  initialQueryStrategy?: "select-then-live" | "live-only";  // default "select-then-live"
  initialDelayMs?: number;                              // default 500
  maxDelayMs?: number;                                  // default 30000
  connectedSettleMs?: number;                           // default 30000
  checkpointStore?: SurrealCheckpointStore;
  checkpointField?: string;                             // default "updated_at"
  listKeyResolver?: (change: EntityChange) => string[] | undefined;
  permanentErrorPatterns?: RegExp[];                    // default [/authentication/i, /namespace/i, /unknown table/i, /permission/i]
}

export function createSurrealLiveAdapter(opts: SurrealLiveAdapterOptions): RealtimeAdapter {
  const adapter = new SurrealLiveAdapter(opts);
  return adapter;
}

// Private class — not exported. Holds the per-channel Map + status aggregation.
class SurrealLiveAdapter implements RealtimeAdapter {
  readonly name: string;
  private channels = new Map<string, SurrealChannel>();
  private channelStates = new Map<string, AdapterStatus>();
  private statusCallbacks = new Set<(s: AdapterStatus) => void>();
  private _status: AdapterStatus = "connecting";
  …
  subscribe(config, handler) { … construct channel, start, return UnsubscribeFn … }
  onStatusChange?(cb) { this.statusCallbacks.add(cb); return () => this.statusCallbacks.delete(cb); }
  _recomputeStatus() { … }
}

class SurrealChannel { … per D2 … }
```

```ts
// src/adapters/surreal-live.test.ts (~320 LOC target)

describe("createSurrealLiveAdapter", () => {
  describe("RealtimeAdapter contract", () => { … 4 scenarios … });
  describe("per-channel subscription", () => { … 4 scenarios … });
  describe("initial seed", () => { … 4 scenarios — incl. ordering guarantee … });
  describe("action mapping", () => { … 5 scenarios … });
  describe("onStatusChange aggregation", () => { … 3 scenarios … });
  describe("reconnect with backoff", () => { … 4 scenarios — uses vi.useFakeTimers … });
  describe("checkpoint replay", () => { … 3 scenarios … });
  describe("listKeyResolver", () => { … 2 scenarios … });
  describe("ChannelConfig.operations filter", () => { … 2 scenarios … });
  describe("test infrastructure asserts", () => { … fake-client no-leaked-subs check … });
});
```

## Risks

1. **Async race between `_openLive` and `_seed`**. Mitigated by D3 ordering: live opens BEFORE seed; notifications buffer; flush after seed delivery. If the spec interpretation is wrong (live arrives only after `select` resolves), the buffer is harmlessly empty.
2. **`vi.useFakeTimers()` interaction with async/await**. Need explicit `vi.runAllTimersAsync()` between reconnect schedule and assertion. Captured in test plan.
3. **Permanent-error detection brittleness** (D4b). Hard-coded regex list. False-negative (transient classified as permanent) means failed reconnect when one would have succeeded; false-positive (permanent classified as transient) means infinite backoff loop until `maxDelayMs` saturation. Mitigation: `opts.permanentErrorPatterns` override + test covers a "unknown error" case explicitly.
4. **SurrealDB driver behavior on `kill` of an already-closed live query**. Spec doesn't address; defensive impl wraps `kill` in `try/catch` and treats failures as no-ops.
5. **Type drift if `RealtimeAdapter` types.ts changes**. The adapter's `import type` will catch breaking changes at compile time; tests will catch behavioral ones.
6. **Bundle size of the test file**. Test file isn't shipped in production builds (tsup config excludes `*.test.ts`); no concern.

## Alternatives Considered

- **Factory returns `SyncAdapter`**. Rejected per W1 spec D1.
- **Per-channel separate `Surreal` connections** (one WebSocket each). Rejected — overhead; consumer should already have a single client; multiple live queries on one client is the supported pattern.
- **Use `RxJS Observable` for live streams**. Rejected — adds dep; existing `subscribe(handler)` callback shape is sufficient.
- **Backoff via `p-retry`**. Rejected — dep; the bespoke backoff is ~15 LOC.
- **Defer per-channel buffer**. Implement a global buffer for the adapter, demultiplex by table. Rejected — couples channels; per-channel buffer is simpler and matches spec scenario language.
