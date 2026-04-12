---
name: entity-gql-setup
description: >
  Slash command /entity-gql-setup — scaffold createGQLClient (GQLClientConfig), authentication headers,
  and EntityDescriptor sets from GraphQL SDL or introspection. Aligns paths and extractId with
  normalizeGQLResponse behavior in `@prometheus-ags/prometheus-entity-management`.
---

# /entity-gql-setup

## When invoked

User wants a **new** or **refactored** GraphQL client module plus descriptor scaffolding for this library.

## Steps

1. Read `prompts/specify.md` and capture endpoint + schema source.
2. Run **schema-analyzer** (`agents/schema-analyzer.md`) on SDL/introspection.
3. Run **descriptor-generator** (`agents/descriptor-generator.md`) for each operation group.
4. Emit:
   - `createGQLClient({ url, headers, onError })` singleton
   - `descriptors/*.ts` exports
   - Minimal example query proving `normalizeGQLResponse` writes `entities[type][id]`

## Checklist

- [ ] `url` from env with documented name
- [ ] `headers()` safe on SSR (no `window` without guard)
- [ ] Every graph `type` consistent with CRUD `registerSchema`
- [ ] `pnpm run typecheck` (or app equivalent)

## References

- `../CLAUDE.md` — descriptor rules
- `../references/gql-normalization.md`
- Library: `src/graphql/client.ts`
