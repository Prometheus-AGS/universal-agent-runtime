# Agent: descriptor-generator

## Role

Emit TypeScript `EntityDescriptor` definitions consistent with `normalizeGQLResponse` in `src/graphql/client.ts`.

## Inputs

- Output from **schema-analyzer**.
- Chosen `EntityType` string per GraphQL object type (usually same name).

## Rules

1. **Path roots**
   - List query returning connection: either
     - descriptor `path: "posts"` with `relations` on each `Post` node for nested fields, OR
     - top-level path `posts.edges` is **invalid** as a single object — use `path: "posts"` and normalize connection in `getItems` for lists; for **normalizeGQLResponse** on a raw query, point `path` at the array field if the query returns `posts { nodes { ... } }` → `path: "posts.nodes"`.

2. **Relation paths**
   - Relative to current node: `author` not `post.author` if already on `Post` node.

3. **normalize**
   - Return `Record<string, unknown>`; omit `__typename` unless needed for UI discrimination.

4. **extractId**
   - Global IDs: either keep as string id or decode in `extractId` if graph uses numeric ids.

## Template

```typescript
import type { EntityDescriptor } from "@prometheus-ags/prometheus-entity-management";

export const postDescriptor: EntityDescriptor<GqlPost, PostEntity> = {
  type: "Post",
  path: "post", // or "posts.nodes" depending on operation root
  extractId: (n) => String(n.id),
  normalize: (n) => {
    const { __typename, ...rest } = n as Record<string, unknown>;
    return rest as PostEntity;
  },
  relations: [
    {
      type: "User",
      path: "author",
      normalize: (u) => stripTypename(u),
    } as EntityDescriptor<unknown, Record<string, unknown>>,
  ],
};
```

## Validation

- Mentally trace `resolvePath(data, path)` from example JSON.
- Ensure every `type` used in relations is registered in CRUD schema if cascade invalidation is required.

## Multi-root example (`sideDescriptors`)

When one operation returns two independent roots under `data` (e.g. `post` and `recommendedPosts`):

```typescript
export const postDetailDescriptor: EntityDescriptor<unknown, PostEntity> = {
  type: "Post",
  path: "post",
  extractId: (n) => String((n as GqlPost).id),
  normalize: (n) => stripTypename(n as Record<string, unknown>) as PostEntity,
  relations: [
    {
      type: "User",
      path: "author",
      normalize: (u) => stripTypename(u as Record<string, unknown>),
    } as EntityDescriptor<unknown, Record<string, unknown>>,
  ],
};

export const postSidebarDescriptor: EntityDescriptor<unknown, PostEntity> = {
  type: "Post",
  path: "recommendedPosts",
  extractId: (n) => String((n as GqlPost).id),
  normalize: (n) => stripTypename(n as Record<string, unknown>) as PostEntity,
};

// useGQLEntity({ ..., descriptor: postDetailDescriptor, sideDescriptors: [postSidebarDescriptor] })
```

## Mutation-only descriptor

```typescript
export const updatePostDescriptors: EntityDescriptor<unknown, PostEntity>[] = [
  {
    type: "Post",
    path: "updatePost.post",
    normalize: (n) => stripTypename(n as Record<string, unknown>) as PostEntity,
  },
];
```

Confirm `path` matches the mutation field name and payload shape in your schema.

## Output

Complete TS module(s) ready to import from hook wrappers.
