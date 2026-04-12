# Specify — entity-graph-setup interview

Run before planning or code generation. Capture answers as structured JSON (`setup_spec`) plus a short freeform `constraints` section.

## 1. Target application

- **Framework**: Vite SPA, Next.js (App Router / Pages), Remix, other?
- **TypeScript**: strict? path aliases for `@/`?
- **Package manager**: pnpm / npm / yarn (confirm from lockfile).

## 2. Current data layer (`stack_report`)

For each, note **where** it appears (deps, sample files):

| Layer | Present? | Notes |
|-------|----------|--------|
| TanStack Query | | Query keys, `useQuery` usage |
| Apollo Client | | `ApolloProvider`, `useQuery` |
| Redux / RTK | | Normalized or denormalized entities? |
| SWR | | Key strategy |
| Zustand (app) | | Server data or UI-only? |
| Raw `fetch` in components | | Anti-pattern count |

## 3. APIs and auth

- **REST** base URL, **OpenAPI** URL or file path, or **GraphQL** endpoint + schema location.
- **Auth**: cookies, `Authorization` header, Next.js server-only fetch, tenant header?
- **SSR**: Does the app need **hydration** of server-fetched entities into the client graph?

## 4. Entity inventory (initial)

For each resource the UI cares about:

- **entityType** string (PascalCase convention)
- **idField** on wire
- **List endpoints** vs **detail** endpoints (partial list rows?)
- **Relations** you already know (FK fields, nested arrays)

## 5. Risk and non-goals

- Features that **must not regress** during migration.
- Deadlines / whether **parallel run** behind a flag is required.
- **Out of scope**: realtime, offline, CRUD sheets (defer to other skills).

## 6. Design / examples

- Should generated demo UI follow **travisjames.ai** tokens (see `prometheus-entity-skills/_shared/references/branding.md`)?
- Existing design system: shadcn, MUI, Chakra?

## Output artifact (`setup_spec`)

```json
{
  "workspace_root": ".",
  "framework": "next-app-router|vite|other",
  "package_manager": "pnpm|npm|yarn",
  "strict_typescript": true,
  "stack_report": {
    "tanstack_query": { "present": false, "notes": "" },
    "apollo": { "present": false, "notes": "" },
    "redux": { "present": false, "notes": "" },
    "swr": { "present": false, "notes": "" },
    "raw_fetch_in_components": { "present": false, "notes": "" }
  },
  "api": {
    "style": "rest|graphql|mixed",
    "auth": "cookie|bearer|server-only|other"
  },
  "entities": [],
  "risks": [],
  "design": { "brand_tokens": "travisjames|custom|none" }
}
```

**Done when:** `setup_spec` is complete enough to choose between **init-only**, **migrate**, and **detect** emphasis for the next phase.
