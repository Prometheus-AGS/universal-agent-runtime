# Codebase analysis — entity-graph-setup

Ordered checklist for scanning a consumer repository before integration.

## 1. Tooling and boundaries

| Signal | Where to look | Why it matters |
|--------|---------------|----------------|
| Package manager | `pnpm-lock.yaml`, `package-lock.json`, `yarn.lock` | Correct install commands |
| TS strict | `tsconfig.json` | Strictness of generated code |
| Aliases | `tsconfig` `paths`, Vite `resolve.alias` | Import paths for new modules |
| ESLint / Biome | config files | Avoid patterns CI will reject |

## 2. Framework-specific

### Vite + React SPA

- Entry: `src/main.tsx`
- Router: TanStack Router, React Router — where data loaders live
- Env: `import.meta.env.VITE_*`

### Next.js App Router

- Server: `app/**/page.tsx`, `layout.tsx`, `route.ts`
- Client: `'use client'` modules — **hooks and graph live here**
- **Rule**: do not import `useGraphStore` in Server Components; hydrate on client

### Next.js Pages Router

- `getServerSideProps` / `getStaticProps` — candidates for passing **serializable** props to a client hydrator

## 3. Data layer fingerprints

Search (ripgrep) patterns:

```text
@tanstack/react-query
useQuery(
useMutation(
@apollo/client
useQuery(
gql`
useSWR
createApi
redux
useSelector
```

For each hit file, classify:

- **Read path** — list vs detail
- **Cache key** — query key, SWR key, Apollo variables
- **Normalization** — already normalized or nested JSON blobs

## 4. API discovery

- REST: `fetch('/api`, axios baseURL, `NEXT_PUBLIC_API_URL`
- GraphQL: `.graphql`, `codegen.yml`, `apollo.config.js`
- Prisma: `prisma/schema.prisma` — model names → suggested `EntityType` strings
- OpenAPI: `openapi.json`, `swagger` — operationId → resource grouping

## 5. Entity manifest fields

For each candidate resource, record:

- Wire **JSON sample** or **TypeScript interface** (redact secrets)
- **id** stability (UUID, numeric string, composite)
- **List payload** completeness (partial vs full)

Validate against `prometheus-entity-skills/_shared/references/schemas/entity-types.schema.json` when emitting JSON.

## 6. Outputs

- `stack_report` (markdown table)
- `entity_manifest.json` (array of entity definitions)
- **Top 5 migration risks** (optimistic UI, pagination cursors, auth refresh, file uploads, websockets)
