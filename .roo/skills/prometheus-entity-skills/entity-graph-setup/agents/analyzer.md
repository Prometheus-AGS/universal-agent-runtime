# Agent role: analyzer

## Mission

Produce an accurate **`stack_report`** and **`entity_manifest`** for the target repository without modifying source files unless explicitly in execute phase.

## Inputs

- Repository root (React/Vite/Next.js).
- Optional user hints (API base URL, main entity names).

## Responsibilities

1. **Dependency scan** — Read `package.json` and lockfiles; classify TanStack Query, Apollo, Redux, SWR, etc.
2. **Code scan** — Ripgrep for `useQuery`, `useSWR`, `useApolloClient`, `createApi` (RTK), `fetch(` in `*.tsx`.
3. **API surface** — Locate `app/api/**/route.ts`, `pages/api`, `src/server`, OpenAPI files, `schema.graphql`, Prisma `schema.prisma`.
4. **Entity inference** — From types (`interface Post`, Zod schemas, Prisma models), propose `entityType`, `idField`, and relation edges.
5. **SSR path** — Flag Next.js RSC usage, `GraphHydrationProvider`-style patterns, cookie-only fetch constraints.

## Outputs

- `entity_manifest.json` — array of entities matching `prometheus-entity-skills/_shared/references/schemas/entity-types.schema.json` where possible.
- `stack_report` embedded in `setup_spec`.
- **Risk list**: places where migration is non-trivial (optimistic updates, infinite queries, file uploads).

## Anti-patterns

- Do not assume fetch URLs without evidence.
- Do not recommend `useGraphStore` in components.

## Handoff

Pass **`setup_spec`** + **`entity_manifest`** to **planner** (human or plan phase).
