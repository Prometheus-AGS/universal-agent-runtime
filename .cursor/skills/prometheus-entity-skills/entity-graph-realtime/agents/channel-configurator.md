# Agent: channel-configurator

Design `ChannelConfig[]` passed to `RealtimeManager.register`.

## Inputs

- `EntityType` strings used in graph
- Server-side topic/table names (vendor specific)
- Optional row filters (`filter`, `id`)

## Steps

1. **One channel per independent subscription** when the adapter reuses one socket but multiplexes—follow the concrete adapter’s pattern (some adapters ignore `filter` and rely on server-side setup instead).
2. **Labeling:** `SubscriptionConfig.label` defaults to `${adapter.name}/${channel.type}`—ensure readable logs.
3. **Replay:** Keep `replayOnConnect: true` unless duplicates are harmful and server guarantees idempotent merges.
4. **Operations filter:** When adapter supports `operations`, restrict to `insert|update|delete` subset if needed.
5. **List invalidation:** If a server message implies “full list unknown”, attach `affectedListKeys` in emitted `ChangeSet`.

## Checklist

- [ ] Every `EntityType` in configs matches `normalize` output
- [ ] IDs align with REST `normalize` functions
- [ ] Cleanup path removes all internal vendor subscriptions

## Example skeleton

```ts
const channels: ChannelConfig[] = [
  { type: "Task" },
  { type: "Project", id: workspaceId },
];
```

Adjust to vendor capabilities—some sources need one channel with broader stream + client-side filtering inside `normalize`.
