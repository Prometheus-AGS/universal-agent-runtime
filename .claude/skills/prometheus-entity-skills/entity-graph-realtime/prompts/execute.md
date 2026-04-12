# Execute — entity-graph-realtime

## Implementation order

1. **Adapter module** with vendor SDK / WebSocket code; export factory matching library signature.
2. **Normalization function** translating vendor payloads → `EntityChange[]` inside adapter or `normalize` callback.
3. **Manager bootstrap** using `getRealtimeManager({ flushInterval, onStatusChange, onChangeReceived })`.
4. **`register(adapter, channelConfigs[, normalize])`** with `replayOnConnect` respected via adapter options.
5. **React integration** via `useEffect` cleanup storing `UnsubscribeFn`.
6. **Dev-only logging** toggle.

## Rules (re-verify)

- No graph writes in adapter—only call provided handler.
- Deletes must produce `op: "delete"` with stable `id`.
- Prefer **`affectedListKeys`** when server indicates list drift without row payload.

## Electric / PGlite extras

- Wire `createElectricAdapter({ pglite, tables: [...] })` with `ElectricTableConfig` per synced table.
- Use `useLocalFirst` / `usePGliteQuery` only where local reads are intentional.

## Testing hooks

- Simulate burst events in devtools console to observe single coalesced flush (default settings).
- Toggle `flushInterval: 0` temporarily to compare behavior—do not ship without justification.

## Output

Provide code + **runbook snippet** (env vars + how to verify connection status in UI).
