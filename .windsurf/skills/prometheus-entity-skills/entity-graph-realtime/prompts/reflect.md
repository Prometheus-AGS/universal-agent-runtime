# Reflect — entity-graph-realtime

## 1. Static audit

- Search for `useGraphStore` writes outside `src/adapters` / approved hooks — should not appear in new adapter code.
- Ensure **every `register` has a matching cleanup**.

## 2. Behavioral tests

- [ ] Insert appears in graph and any mounted lists using those IDs
- [ ] Update merges visible in all views subscribed to entity
- [ ] Delete removes entity and list rows
- [ ] Burst updates → **≤1 flush per frame** (default 16ms window)
- [ ] Reconnect replays or refetches as designed (`replayOnConnect`)

## 3. Performance spot check

- Open React profiler or log flush counts during 50 rapid events.
- If frames still spike, revisit normalization or `flushInterval`.

## 4. Typecheck

```bash
pnpm run typecheck
```

## 5. Security

- Secrets not logged with payloads
- Auth tokens not committed in repo

## 6. Exit criteria

Pass audit + manual smoke; document known limitations (e.g. no multi-tenant isolation on shared channel).
