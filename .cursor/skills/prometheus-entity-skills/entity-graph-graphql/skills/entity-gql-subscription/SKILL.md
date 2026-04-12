---
name: entity-gql-subscription
description: >
  Slash command /entity-gql-subscription — connect GraphQL subscriptions via graphql-ws (or compatible)
  to either GQLClient.subscribe/useGQLSubscription with EntityDescriptors, or createGraphQLSubscriptionAdapter
  + RealtimeManager for coalesced ChangeSet delivery.
---

# /entity-gql-subscription

## When invoked

User needs **live** GraphQL updates integrated with the entity graph.

## Path A — Direct normalization (descriptors)

1. Create `wsClient` implementing:

```typescript
type WsClient = {
  subscribe: (
    payload: { query: string; variables?: Record<string, unknown> },
    sink: { next: (v: { data: T }) => void; error: (e: unknown) => void; complete: () => void }
  ) => () => void;
};
```

2. Use `useGQLSubscription` with same `descriptors` as query/mutation so incoming payloads upsert entities.

**Pros**: Simple; identical normalization to queries.  
**Cons**: No 16ms coalescing; high-frequency events may cause extra renders.

## Path B — RealtimeManager adapter

1. Import `createGraphQLSubscriptionAdapter` from `@prometheus-ags/prometheus-entity-management`.
2. Configure `subscriptions: [{ type, document, variables, getPayload }]`.
3. `getPayload(data)` returns `GQLPayload | GQLPayload[] | null` where each payload has:
   - `type`: `"created" | "updated" | "deleted"` (or default upsert behavior per adapter)
   - `node` / `id` as documented in `src/adapters/realtime-adapters.ts`

4. `manager.register(adapter, [{ type: "Post", ... }])` with channel config per your app.

**Pros**: Coalesced writes; consistent with WebSocket/Supabase adapters.  
**Cons**: Must map subscription shape manually.

## Operational concerns

- Reconnect + auth refresh: ensure `client` is recreated or subscription resubscribes when token changes.
- Unsubscribe on route change: `useGQLSubscription` effect cleanup handles local scope; adapter registration should mirror app lifecycle (singleton vs provider).

## References

- Library: `src/graphql/client.ts` (`subscribe`)
- Library: `src/graphql/hooks.ts` (`useGQLSubscription`)
- Library: `src/adapters/realtime-adapters.ts` (`createGraphQLSubscriptionAdapter`)
- Library: `src/adapters/realtime-manager.ts`
