## Context

`prometheus-entity-management/src/adapters/types.ts` defines the actual adapter surface:

```ts
export interface RealtimeAdapter {
  readonly name: string;
  subscribe(config: SubscriptionConfig, handler: (changeset: ChangeSet) => void): UnsubscribeFn;
  onStatusChange?: (cb: (status: AdapterStatus) => void) => UnsubscribeFn;
}

export interface SyncAdapter extends RealtimeAdapter {
  query<T>(sql, params?): Promise<SyncQueryResult<T>>;
  execute(sql, params?): Promise<void>;
  isSynced(): boolean;
  onSyncComplete(cb: () => void): UnsubscribeFn;
}
```

And `realtime-manager.ts` consumes it:

```ts
register(adapter: RealtimeAdapter, channels: ChannelConfig[], normalize?: …) {
  for (const channel of channels) {
    const unsub = adapter.subscribe({ label: `${adapter.name}/${channel.type}`, replayOnConnect: true },
      (cs) => this.handleChangeset(adapter.name, cs, normalize));
    …
  }
}
```

Two facts emerge that the original spec missed:

1. There is **no `start/stop`**. Lifecycle is per-subscription via the returned `UnsubscribeFn`.
2. **Each channel gets its own `subscribe(...)` call**. The adapter does not control the channel set; the manager hands it one channel at a time.

This change rewrites the spec to match. No code is involved — this is the gating documentation correction before change 4 (`seim-em-surreal-live-adapter-impl`) implements anything.

## Goals / Non-Goals

**Goals**
- The corrected spec is implementable as-written against `types.ts` with zero further reconciliation.
- Per-channel state-machine semantics are made explicit (reconnect counter, replay checkpoint, status aggregation).
- Capability ID is preserved.
- A reconciliation preamble makes the supersession discoverable from the spec itself.
- Companion skill alignment is required as a verification scenario.

**Non-Goals**
- **No TypeScript work.** That's change 4.
- **No edit to the archived original spec.** It stays as history; the link from the preamble preserves traceability.
- **No new capability.** Same capability ID. The capability `entity-surreal-live-adapter` is *modified*, not replaced.
- **No companion skill edit in this change.** The shipped skill (from PR #3) already documents the corrected shape; change 4's `/opsx:verify` will catch any drift.

## Decisions

### D1. Return `RealtimeAdapter`, not `SyncAdapter`

`SyncAdapter` adds `query` / `execute` / `isSynced` / `onSyncComplete`. SurrealDB's driver already exposes query/execute via its own client; the adapter has no value-add for those methods. Implementing `SyncAdapter` would mean the adapter must own (or proxy) a SurrealDB client connection, which entangles its lifecycle with the consumer's. By returning `RealtimeAdapter`, the consumer keeps full control of the SurrealDB client (passed in via `opts.surreal`) and the adapter restricts itself to live-query orchestration.

The PGlite (`SyncAdapter`) case is different — PGlite *is* both the data source and the runtime, so the adapter owns the connection. SurrealDB isn't that.

### D2. Per-channel state machine, single status stream

Each channel maintains:
- A SurrealDB live query handle (the `uuid` returned by `LIVE SELECT`)
- A `connectionState` ∈ `connecting | connected | disconnected | error`
- A reconnect attempt counter + active backoff timer (if any)
- A connected-settle timestamp (for resetting the attempt counter)
- An in-flight reconnect promise (so a second disconnect during reconnect doesn't double-fire)
- Per-channel checkpoint value (when `opts.checkpointStore` is supplied)

The adapter aggregates per-channel states into a single `AdapterStatus` for the `onStatusChange` callback. Aggregation rule: worst-of (`error` > `disconnected` > `connecting` > `connected`).

Why not one status callback per channel? Because the `RealtimeAdapter` interface declares one `onStatusChange` per adapter. Per-channel state can be introspected via debug logs but isn't part of the public contract.

### D3. Strict seed-then-live ordering with a per-channel buffer

SurrealDB allows `LIVE SELECT` to start immediately, before the initial `SELECT` completes. A naïve implementation could deliver a live `UPDATE` before the seed `ChangeSet`, causing the consumer to apply an update to a graph that doesn't yet contain the row.

Solution: per-channel buffer. The flow is:

```
subscribe(config, handler)
 ├─ async: SELECT * FROM <table>[ WHERE …]          → rows
 ├─ async: LIVE SELECT * FROM <table>[ WHERE …]     → uuid + start receiving notifications
 │       receiving notifications BEFORE seed completes → push into buffer[]
 ├─ when SELECT resolves:
 │       handler({ changes: rows.map(insert), … })
 │       flush buffer[] (in arrival order) → handler(each)
 │       clear buffer state; future notifications go straight to handler
```

This guarantees the seed is the first ChangeSet delivered, with zero risk of reordering.

### D4. Map SurrealDB action enum to `ChangeOperation`

| SurrealDB live action | `EntityChange.op` |
|---|---|
| `CREATE` | `insert` |
| `UPDATE` | `update` |
| `DELETE` | `delete` |
| `CLOSE` | (no emission — channel disconnects, enters reconnect) |
| `*` (unknown) | (no emission — warn + skip) |

The `upsert` operation defined in `ChangeOperation` is reserved for adapters that can't distinguish insert from update; SurrealDB can, so we use the precise op.

### D5. Per-channel checkpoint keying

When multiple channels target the same table with different filters, a single checkpoint would interfere. Keying:

```
checkpointKey(channel) = `${adapter.name}/${channel.label || channel.type + "?" + filterHash}`
```

where `filterHash` is a stable JSON serialisation of `channel.filter`. The hashing detail is left to implementation; the spec requires only "distinct channels do not overwrite each other's checkpoints."

### D6. Replay on reconnect is opt-in

Two reasons:
1. Many SurrealDB schemas don't have an `updated_at` column to filter against. Forcing a checkpoint mechanism would require schema enforcement.
2. Consumers may prefer "missed deltas are lost; sync on next live event" semantics for ephemeral data.

The opt-in is `opts.checkpointStore` being present. When absent, reconnect re-runs only the live subscription (no replay query).

### D7. Status aggregation runs on every per-channel transition

Implementation note (informs change 4): every channel state change calls a private `recomputeStatus()` that aggregates per-channel states and, only if the aggregate has actually changed, fires the registered `onStatusChange` callbacks. This avoids spamming status callbacks with no-op transitions.

### D8. `ChannelConfig.id` scopes the SurrealDB query to a single record

SurrealDB record IDs are of the form `<table>:<id>`. When `ChannelConfig.id` is supplied:

- Initial seed: `SELECT * FROM <table>:<id>`
- Live: `LIVE SELECT * FROM <table>:<id>`

This is materially cheaper than a table-wide LIVE SELECT plus client-side filtering. The original spec did not call this out; the corrected spec does.

### D9. `affectedListKeys` derivation is consumer-supplied

`opts.listKeyResolver?: (change: EntityChange) => string[] | undefined` is optional. When present, the adapter calls it for each `insert`/`delete` change and aggregates the returned strings (deduplicated) into `ChangeSet.affectedListKeys`. When absent, the field is left `undefined` and the `RealtimeManager`'s 16ms coalesce handles list-refresh decisions on its own.

For `update` changes specifically, list keys are *not* populated by default because most apps don't re-render lists on per-row updates. Consumers that need it can opt in via `listKeyResolver` returning a non-empty array on update.

### D10. Archive-overwrite mechanics

The spec correction lives in `openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md`. On `/opsx:archive`:

- The change directory moves to `openspec/changes/archive/<date>-seim-surreal-live-spec-correction/`.
- The spec file gets promoted to `openspec/specs/entity-surreal-live-adapter/spec.md`, **overwriting** the existing file.
- The originally-archived spec at `openspec/changes/archive/2026-05-27-ssed-entity-surreal-live-adapter/specs/entity-surreal-live-adapter/spec.md` is **untouched** — it remains the historical record.

The reconciliation preamble in the new promoted spec points back to the historical archive so anyone reading the live spec can follow the trail.

## Implementation Sketch

(There is no code in this change. The "implementation" is the rewritten spec already drafted in `/opsx:continue`. This section captures the implementation *of the spec correction itself* — the file moves and verification touchpoints.)

### Archive operation

```sh
# 1. Verify the corrected spec parses (markdown only — sanity scan)
grep -c '^### Requirement:' \
  openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
# Expected: 10

grep -c '^#### Scenario:' \
  openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
# Expected: 33

# 2. Confirm the reconciliation preamble exists in the corrected spec
grep -q '^> \*\*Reconciliation note\.' \
  openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md

# 3. Confirm the companion skill on disk (already shipped via PR #3) still references this capability
grep -l 'entity-surreal-live-adapter' \
  ~/.claude/skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md
```

### Verification touchpoints (for `/opsx:verify` post-archive)

- Capability count == modified-spec count == 1. ✓
- Every requirement in the new spec has ≥1 scenario. ✓ (smallest is 2 — Companion Skill / Test Coverage have 2 each)
- The reconciliation preamble names the supersession path. ✓
- The companion skill's documented surface (`createSurrealLiveAdapter` return type, options, manager registration sequence) matches the new spec. → verification scenario calls this out explicitly.

### Companion-skill alignment check

After the spec is promoted, run a manual diff (or a future automated check) between:

- `openspec/specs/entity-surreal-live-adapter/spec.md` (the new, promoted spec)
- `~/.claude/skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md` (the skill on disk, shipped via PR #3)

If the skill describes a `SyncAdapter` shape, that's a bug in the skill that landed via PR #3 — file a corrective change immediately. (Inspection during proposal-drafting suggests the skill already uses the `RealtimeAdapter` shape, but verification confirms.)

## Risks

1. **Skill drift undetected.** D-above mitigation is manual; an automated check would be better. Filed as a follow-up.
2. **Future SurrealDB driver changes.** SurrealDB live-query event shape has changed at least once historically. The spec pins the current shape (`CREATE | UPDATE | DELETE | CLOSE`); a future driver update may require a spec amendment. Acceptable risk — driver-version pinning is in `package.json`, not in the spec.
3. **`ChannelConfig.operations` field undocumented in the corrected spec.** `types.ts` allows `ChannelConfig.operations?: ChangeOperation[]` to restrict which actions a channel emits (e.g. "deletes only"). The corrected spec does not mandate the adapter honor this — left as an enhancement for change 4. *Flag for review.*
4. **Change 4 may discover further drift.** When change 4 implements against this spec, additional gaps may surface (e.g. error-message format requirements). The plan accommodates that by making change 4's design re-read both the spec AND `types.ts` before writing code.

## Alternatives Considered

- **Edit the originally-archived spec in place.** Rejected — archives are read-only by convention; future audits need the original to understand the correction.
- **Keep the original spec, add a "Corrigenda" section.** Rejected — readers would have to mentally merge the original + corrigenda. Cleaner to ship a corrected spec and reference the original.
- **Wait for change 4 to surface every gap before correcting.** Rejected — sequencing concern. Change 4's spec-conformance check fails immediately against the broken original; the implementor would block waiting for the correction. Easier to do the correction first.
- **Drop the capability entirely and redefine it under a new name.** Rejected — the capability *intent* is unchanged. Renaming would orphan the companion skill's spec reference and burn the prior phase's archive linkage.
