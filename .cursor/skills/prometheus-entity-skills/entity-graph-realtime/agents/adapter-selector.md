# Agent: adapter-selector

Choose the correct realtime integration path.

## Decision matrix

| Backend | Adapter factory | Notes |
|---------|-----------------|-------|
| Custom WS JSON | `createWebSocketAdapter` | Provide `parseMessage` mapping to `EntityChange[]` |
| Supabase Postgres changes | `createSupabaseRealtimeAdapter` | Row-level filters, channel per table |
| Convex reactive backend | `createConvexAdapter` | Map Convex subscription results |
| GraphQL live queries / subs | `createGraphQLSubscriptionAdapter` | Bridge subscription payloads |
| Electric + PGlite | `createElectricAdapter` | Local-first; implements `SyncAdapter` |

## Questions

1. **Do we already have a REST CRUD path?** Realtime should **augment**, not replace, mutation responses unless offline-first.
2. **Payload fidelity:** Full row vs patch → determines `op: "update"` with `data` vs `patch`.
3. **Multi-tenant:** Need channel per tenant? If yes, encode tenant in `ChannelConfig.filter` + server-side ACLs.
4. **SSR:** WebSocket/subscriptions usually **client-only**—guard with `typeof window` or dynamic import.

## Outputs

- Named adapter choice + rationale paragraph
- List of required npm packages (if any beyond library)
- Risk: vendor lock-in vs generic WS

## Anti-patterns

- Using Electric adapter without actual ShapeStream + PGlite wired (placeholder config)
- Piping GraphQL through WebSocket adapter manually when `createGraphQLSubscriptionAdapter` fits
