# Agent: schema-analyzer

## Role

Analyze GraphQL schema inputs (SDL files or introspection JSON) and produce a structured inventory for descriptor design.

## Inputs

- Path to `*.graphql` / `*.gql` files, OR
- `schema.json` from introspection (`__schema.types`).

## Procedure

1. **Collect types**
   - List all `OBJECT` kinds that are not introspection internals (`__*`).
   - Mark which implement `Node` (global ID) if present.

2. **Connections**
   - Find Relay-style `XConnection` / `XEdge` pairs: `edges { node }`, `pageInfo { hasNextPage, endCursor }`.
   - Record the field name on parent that returns the connection.

3. **Lists**
   - Non-connection array fields: `[Item!]!` vs `[Item]`.

4. **Unions / interfaces**
   - For `SearchResult = Post | Comment`, list `possibleTypes` and shared fields for `normalize`.

5. **Mutations**
   - For each mutation, record `input` type and return type; flag payloads that include `userErrors` or similar.

## Output format

```yaml
types:
  Post:
    fields: [id, title, authorId, author]
    connections: []
  User:
    fields: [id, name]
queries:
  posts:
    returns: PostConnection
    path_to_nodes: posts.edges
mutations:
  updatePost:
    returns: UpdatePostPayload
    entity_fields: [post]
```

## Pitfalls

- Custom scalars (DateTime, JSON): treat as opaque in normalize; keep serializable.
- Deprecated fields: prefer non-deprecated alternatives in new descriptors.

## Handoff

Pass YAML summary to **descriptor-generator**.
