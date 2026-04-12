# Coalescing guide — RealtimeManager

## Why coalesce?

Realtime vendors may emit **many row events in one frame** (e.g. bulk edit, chatty triggers). Writing each event to Zustand immediately causes **React render storms**.

`RealtimeManager` buffers `EntityChange` objects and flushes them after a short delay, merging duplicates.

## Default window

- **`flushInterval` default: 16ms** (~one animation frame)
- `scheduleFlush` coalesces multiple `handleChangeset` calls into one timeout

## `flushInterval: 0`

- **Semantics:** flush **synchronously** on every changeset—no timer batching
- **When:** debugging, deterministic tests, or ultra-low latency requirements that outweigh render cost
- **Caution:** can devastate performance if source is noisy

## Merge algorithm (`coalesceChanges`)

Per `(type, id)` key:

- Latest **`delete`** wins
- Multiple non-delete ops collapse toward **`upsert`** with merged `patch` objects
- Prevents intermediate states from flashing when only the final snapshot matters

## `affectedListKeys`

- Collected in a `Set` across buffered changesets
- Applied after entity mutations during flush
- Use when the server signals pagination totals or ordering changes without sending every row

## Manual flush

- **`forceFlush()`** clears the timer and applies immediately—useful before navigation teardown or imperative tests

## Observability

```ts
getRealtimeManager({
  onChangeReceived: (adapter, change) => {
    // dev log: pre-coalesce volume
  },
});
```

Compare counts **before** vs **after** flush in tests by mocking store.

## Failure modes

| Symptom | Likely cause |
|---------|--------------|
| UI lag with low event rate | `flushInterval` too high *and* extra work in render—profile first |
| Stale rows after burst | Missing `delete` op or wrong `id` |
| Lists not updating | No `affectedListKeys` + list query not tied to graph ids |

## Recommended defaults

- Start with **16ms**
- Move to `0` only with evidence; prefer optimizing normalization or reducing event frequency first

## Manager construction note

`getRealtimeManager(options)` only applies **`ManagerOptions`** on the **first** call. To change `flushInterval` or callbacks after startup, use **`resetRealtimeManager()`** deliberately (drops all registrations) or centralize options in one bootstrap module.
