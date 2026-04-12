# Execute — Generate GQLClient, descriptors, hooks

## Order of work

1. **`lib/graphql-client.ts`** (or app convention)
   - `export const gqlClient = createGQLClient({ url, headers, onError })`.
   - Optional: export `GQL_DOCUMENTS` as const strings or use `graphql-tag` if already in project.

2. **`lib/graphql/descriptors/*.ts`**
   - One file per domain or per operation group.
   - Export arrays: `postListDescriptors`, `postDetailDescriptors`, etc.
   - Use shared `normalizeX` pure functions for testability.

3. **Hook wrappers** (thin layer in `hooks/` or feature folders)
   - Wrap `useGQLEntity` / `useGQLList` / `useGQLMutation` with concrete `document`, `descriptor`, `queryKey` factories.
   - Keep components free of GraphQL strings where possible.

4. **Subscriptions**
   - Instantiate `graphql-ws` `createClient` (or existing Apollo Link) implementing the `wsClient` shape expected by `GQLClient.subscribe`.
   - Or register `createGraphQLSubscriptionAdapter` with `RealtimeManager.register`.

5. **Exports**
   - Re-export only what pages need; avoid circular imports with client singleton.

## Code patterns (concise)

### Entity hook wrapper

```typescript
export function usePost(id: string | undefined) {
  return useGQLEntity<PostQueryData, Post>({
    client: gqlClient,
    document: POST_QUERY,
    variables: {},
    type: "Post",
    id,
    descriptor: postDescriptor,
    sideDescriptors: [userDescriptor],
  });
}
```

### List hook wrapper

```typescript
export function usePosts(filter: PostsFilter) {
  return useGQLList<PostsQueryData, Post>({
    client: gqlClient,
    document: POSTS_QUERY,
    variables: { filter },
    type: "Post",
    queryKey: ["Post", "list", filter],
    descriptor: postDescriptor,
    getItems: (d) => d.posts?.nodes ?? [],
    getPagination: (d) => ({
      hasNextPage: d.posts?.pageInfo?.hasNextPage,
      nextCursor: d.posts?.pageInfo?.endCursor ?? undefined,
    }),
  });
}
```

## Constraints

- Match **actual** operation names from schema.
- After edits, run typecheck and fix `TData` typings against real response shapes.

## Handoff

Go to `reflect.md` for verification steps.
