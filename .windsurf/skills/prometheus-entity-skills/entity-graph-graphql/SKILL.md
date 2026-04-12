---
name: entity-graph-graphql
description: >
  Set up the GraphQL layer for @prometheus-ags/prometheus-entity-management: GQLClient, EntityDescriptor trees,
  response normalization into the Zustand entity graph, and typed hooks (useGQLEntity, useGQLList,
  useGQLMutation, useGQLSubscription). Supports schema-driven descriptor generation from .graphql
  files or introspection of a running endpoint. Sub-skills cover setup, hook codegen patterns,
  and wiring subscriptions through graphql-ws plus RealtimeManager.
---

# Entity Graph — GraphQL

This skill guides integration of **@prometheus-ags/prometheus-entity-management** GraphQL APIs with the shared entity graph. Queries and mutations normalize through `EntityDescriptor` definitions; results coexist with REST-loaded entities so `Post:123` updates every subscriber regardless of transport.

## When to use

- Adding or refactoring GraphQL data loading in a React app that already uses (or will use) this library.
- Mapping Apollo/Hasura/Postgraphile/schema-first `.graphql` types into `EntityDescriptor` configs.
- Connecting **graphql-ws** (or compatible) subscriptions to either `GQLClient.subscribe` / `useGQLSubscription` or `createGraphQLSubscriptionAdapter` + `RealtimeManager`.

## Architecture (library facts)

| Layer | Responsibility |
|--------|----------------|
| `GQLClient` | `query` / `mutate` with dedupe; calls `normalizeGQLResponse` on success |
| `normalizeGQLResponse` | Walks descriptor paths, `upsertEntity`, `setEntityFetched` |
| Hooks | `useGQLEntity`, `useGQLList`, `useGQLMutation`, `useGQLSubscription` — same graph as REST |
| Realtime | Optional: `createGraphQLSubscriptionAdapter` emits `ChangeSet` for `RealtimeManager` |

## Sub-skills (slash commands)

| Command | Purpose |
|---------|---------|
| `/entity-gql-setup` | Scaffold `createGQLClient`, auth headers, and descriptor set from schema |
| `/entity-gql-hooks` | Produce typed hook usage for specific operations (entity, list, mutation) |
| `/entity-gql-subscription` | Wire WebSocket client + descriptors or adapter + manager |

## PMPO workflow

1. **Specify** — `prompts/specify.md`: endpoint, SDL vs introspection, auth.
2. **Plan** — `prompts/plan.md`: entity types, paths, relation nesting, side descriptors.
3. **Execute** — `prompts/execute.md`: generate client module, descriptors, hook wrappers.
4. **Reflect** — `prompts/reflect.md`: verify graph writes, list keys, error handling.
5. **Persist** — `prompts/persist.md`: document conventions for the repo.

## Agents

- `agents/schema-analyzer.md` — Parse SDL / introspection JSON for types, connections, edges.
- `agents/descriptor-generator.md` — Emit `EntityDescriptor` trees and `extractId` rules.

## References

- `references/gql-normalization.md` — How `path`, `walk`, and `relations` behave.
- `references/descriptor-patterns.md` — Relay connections, unions, interfaces.

## Non-negotiables (from repo `CLAUDE.md`)

- **Components** must not call `useGraphStore` directly; use hooks.
- **Hooks** must not `fetch` the GraphQL HTTP endpoint themselves — delegate to `GQLClient` (or a thin wrapper module the hook calls). The library’s hooks already encapsulate this pattern.
- **Stores** own external I/O; for GraphQL, the canonical pattern is `GQLClient` in a module imported by hooks.

## Schema sources

| Source | When to use | Agent |
|--------|-------------|--------|
| Committed `.graphql` / `.gql` | Schema-first repos, codegen already in CI | `schema-analyzer` on SDL |
| Introspection JSON | No SDL in repo; need field-accurate types from a server | Run introspection once (dev/staging), store redacted `schema.json`, then analyzer |

**Introspection (illustrative)** — adjust headers and URL for your gateway:

```bash
curl -sS -X POST "$GRAPHQL_HTTP_URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query":"query Introspect { __schema { types { name kind } } }"}' | jq . > schema-introspection.json
```

For full schema, use your stack’s standard tool (`graphql-inspector`, Apollo CLI, or `get-introspection-query` from `graphql`). Never commit production secrets embedded in schema description fields.

## Package manager

Monorepo examples use **pnpm** only.
