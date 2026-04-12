# AGENTS.md — entity-graph-graphql

Guidance for coding agents using this skill in **prometheus/prometheus-entity-management** or consuming apps.

## Scope

This skill targets the library exports under `graphql/client`, `graphql/hooks`, and `adapters/realtime-adapters` (`createGraphQLSubscriptionAdapter`).

## Do

- Read the app’s `.graphql` files or introspection JSON before inventing field names.
- Align `EntityType` strings with `registerSchema` / CRUD relation types elsewhere in the app.
- Reuse one `GQLClient` instance per endpoint (module singleton).
- Add `sideDescriptors` when a query returns nested users, tags, etc. that should be normalized as first-class entities.
- Run `pnpm run typecheck` (or the app’s equivalent) after adding descriptors.

## Do not

- Call `useGraphStore` from components.
- Perform raw `fetch` to the GraphQL URL inside components; keep HTTP in `GQLClient` (or a store module the hook uses).
- Duplicate entity data in React state when the graph already holds it — read via hooks.

## Orchestrators

`scripts/detect-orchestrators.sh` emits JSON when run from the **workspace root**. Use KBD / OpenSpec / Evolver flows if present; otherwise follow `prompts/*.md` linearly.

## Verification

- Manual: exercise list + detail + mutation in the example app pattern (Vite or Next).
- Confirm a mutation updates a REST-only view of the same `type:id` without refetch (proves shared graph).
