# Execute — Apply optimizations

## Pattern fixes

### Narrow selectors

Before:

```typescript
const tasks = useStore(useGraphStore, (s) => s.entities.Task);
```

After (example — fetch one id):

```typescript
const task = useStore(
  useGraphStore,
  useCallback((s) => (id ? s.entities.Task?.[id] : undefined), [id])
);
```

Prefer library hooks (`useEntity`, `useGQLEntity`) which already encode subscriber + selector logic.

### Stable query keys

```typescript
const queryKey = useMemo(() => ["Task", "list", filter], [filter]);
```

Ensure `filter` is serializable/stable.

### Realtime batching

Restore default manager flush unless latency-sensitive:

```typescript
new RealtimeManager({ flushInterval: 16 });
```

### Eviction on route leave

```typescript
// Example: app hook on unmount of heavy explorer
useEffect(() => () => {
  for (const id of transientIds) useGraphStore.getState().removeEntity("PreviewRow", id);
}, []);
```

(Use only with clear product rules — do not evict shared canonical entities still visible elsewhere.)

## Process

1. Land architectural fixes before micro-optimizations.
2. Keep diffs small per PR.
3. Run `pnpm run typecheck` after each batch.

## Next

`reflect.md`
