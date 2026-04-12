# Specify — entity-graph-realtime

Capture `realtime_spec` before implementation.

## 1. Source system

- **Vendor**: raw WebSocket, Supabase Realtime, Convex, GraphQL subscriptions, ElectricSQL+PGlite, other (justify)
- **Auth**: cookies, bearer tokens, API keys, per-channel JWT
- **Transport constraints**: browser-only, SSR safe?, corporate proxies

## 2. Entity coverage

List each **`EntityType`** receiving push updates and whether events are **full rows** or **patches**.

## 3. Event shape

Provide **example JSON** payloads for:

- insert / create
- update (full vs partial)
- delete

## 4. Ordering & idempotency

- Must clients tolerate duplicate deliveries?
- Is ordering per-entity guaranteed?
- Need CRDT / version vectors? (If yes, flag for custom normalize)

## 5. Latency goals

- Target UI freshness (ms) vs cost of batching
- If **<16ms** visual requirements are mandatory, justify **`flushInterval: 0`**

## 6. List invalidation

- Do events include hints for which **list query keys** should refresh? Map to `affectedListKeys` in `ChangeSet`.
- Otherwise rely on graph entity updates + existing hook staleness.

## 7. Lifecycle

- When to connect (post-auth, workspace select, route enter)
- When to disconnect (logout, route leave)
- Hot reload behavior in dev

## 8. Failure modes

- Reconnect backoff expectations
- User-visible status UI (banner, dot)

## Output

```json
{
  "source": "supabase|websocket|convex|graphql-ws|electricsql",
  "entities": [],
  "samples": { "insert": {}, "update": {}, "delete": {} },
  "flushIntervalMs": 16,
  "affectedListKeys": [],
  "lifecycle": {}
}
```

## Done when

Example payloads + entity mapping agreed; flush policy chosen with rationale.
