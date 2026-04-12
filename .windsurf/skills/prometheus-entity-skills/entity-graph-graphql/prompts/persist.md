# Persist — Save outcomes

## Repository artifacts

- [ ] Client + descriptors + hook modules merged.
- [ ] `.env.example` updated with `VITE_GRAPHQL_URL` / `NEXT_PUBLIC_GRAPHQL_URL` (or server-only vars if proxied).
- [ ] If introspection JSON is committed: strip sensitive comments; prefer SDL.

## Team knowledge

- [ ] Short ADR or wiki: “GraphQL uses @prometheus-ags/prometheus-entity-management `GQLClient`; descriptors live in `…`.”
- [ ] On-call note: where `onError` logs go.

## Optional automation

- [ ] npm/pnpm script: `graphql:schema` to pull SDL.
- [ ] CI step: `graphql-codegen` or custom script to **validate** descriptor paths against schema (recommended for large teams).

## State

- Store final spec + plan in project docs or KBD phase folder if orchestrator is active.
