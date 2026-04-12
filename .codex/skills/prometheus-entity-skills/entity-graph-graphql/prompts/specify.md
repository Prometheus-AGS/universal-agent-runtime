# Specify — GraphQL entity graph integration

## Goal

Capture everything needed to design `GQLClient` configuration and `EntityDescriptor` sets.

## Questions (ask / document)

1. **Endpoint**
   - HTTP URL for queries/mutations (same origin or absolute).
   - WebSocket URL for subscriptions (if any); subprotocol (`graphql-transport-ws` vs legacy).

2. **Schema source**
   - Committed SDL: path(s) to `*.graphql` / `*.gql`.
   - Or introspection: how to run it (CI script, one-off `curl`, Apollo CLI). Store **redacted** introspection JSON if secrets appear in descriptions.

3. **Authentication**
   - Bearer, cookie, custom header, or Next.js server proxy.
   - Whether `headers` must be async (refresh token).

4. **Entity inventory**
   - Which GraphQL object types map to graph `EntityType` keys (PascalCase strings)?
   - Primary key field per type (`id`, `uuid`, …).

5. **Operations**
   - List query name + shape (connection vs array).
   - Detail query name + variable name for id.
   - Mutations that return payloads suitable for normalization.

6. **Constraints**
   - File size / pagination (cursor vs offset).
   - Nullable handling and error policy (`onError` on `GQLClientConfig`).

## Output (spec artifact)

```yaml
endpoint_http: string
endpoint_ws: string | null
schema_paths: string[]
auth:
  style: bearer | cookie | header | proxy
  header_name: string | null
entity_types:
  - gql_type: string
    graph_type: string
    id_field: string
operations:
  list: { name: string, root_path: string }
  detail: { name: string, id_variable: string }
  mutations: [{ name: string, returns_entities: string[] }]
```

## Handoff

Proceed to `plan.md` with this spec filled.
