# Common EntityDescriptor patterns

Patterns for **@prometheus-ags/prometheus-entity-management** GraphQL integration.

## 1. Relay connection with `edges`

Query:

```graphql
query Posts { posts(first: 20) { edges { node { id title } } } }
```

**List hook**: `getItems: (d) => d.posts.edges.map(e => e.node)`.

**Normalization** on the same query: use a descriptor with `path: "posts.edges"` and `normalize` on each **edge** that forwards to `node`, or use two-step:

- Descriptor A: `path: "posts.edges"`, `normalize` on edge — usually wrong because entity is `node`.
- Prefer **flattening** in `normalize` by making `path: "posts.edges"` and `normalize: (edge) => normalizePost(edge.node)` — but then `extractId` must read from `edge.node`.

Cleaner: `path: "posts.edges"`, `extractId: (e) => String(e.node.id)`, `normalize: (e) => stripTypename(e.node)`.

## 2. Connection with `nodes` shortcut

```graphql
query { posts { nodes { id } pageInfo { hasNextPage endCursor } } }
```

- `path: "posts.nodes"` for array walk.
- `getItems: (d) => d.posts.nodes ?? []`.

## 3. Nested author on post

```typescript
relations: [
  {
    type: "User",
    path: "author",
    extractId: (u) => String(u.id),
    normalize: (u) => stripTypename(u as Record<string, unknown>),
  } as EntityDescriptor<unknown, Record<string, unknown>>,
]
```

## 4. Union / interface

- Use `normalize` to return a discriminated object including `__typename` if UI needs it.
- `extractId` must work for every variant; sometimes each type uses `id`.

## 5. Mutation payload envelope

```graphql
mutation { updatePost(input: $in) { post { id title } userErrors { message } } }
```

- Descriptor `path: "updatePost.post"` (depends on mutation name).
- Do not upsert `userErrors` as entities.

## 6. Multiple roots

Pass additional top-level descriptors via `sideDescriptors` in `useGQLEntity` / `useGQLList` so multiple paths under `data` normalize in one response.

## 7. Subscription adapter payloads

For `createGraphQLSubscriptionAdapter`, descriptors are **not** used. Implement `getPayload(data)` so it returns `null`, a single payload, or an array. Each payload matches `GQLPayload` in `src/adapters/realtime-adapters.ts`:

- **`type`** (optional): `"created"` → graph `insert`; `"deleted"` → `delete`; anything else (including `"updated"` or omitted) → `upsert`.
- **`node`**: entity body for insert/upsert (required for those ops).
- **`id`**: for deletes when there is no `node`.

Example mapping a Hasura-style payload:

```typescript
getPayload: (data) => {
  const row = data?.task_updated?.returning?.[0];
  if (!row) return null;
  return { type: "updated", node: row as Record<string, unknown> };
},
```

Keep shapes JSON-serializable; strip or normalize `__typename` in `normalize` adapter option if you attach one.
