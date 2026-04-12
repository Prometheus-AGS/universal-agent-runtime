# Specify — Prisma + entity graph

## Discover

1. Find `schema.prisma` path(s); note multi-file schemas if `prismaSchemaFolder` is used.
2. Inventory:
   - Models (including `@@map` / `@map` for DB naming)
   - Enums
   - Relations: 1:1, 1:N, M:N (implicit or explicit join model)
   - Primary keys (`@id`, composite)

3. Runtime target:
   - Next.js App Router, Pages Router, Remix, Vite + Express, etc.

4. API style:
   - JSON body vs query params for `where` / `orderBy`
   - Pagination: cursor vs page/pageSize (library list fetch supports both knobs via URL params)

## Security

- Auth model (session, JWT, API key) for mutating routes.
- Row-level scope (tenant id, user id) — must be enforced **server-side**, not only in client filters.

## Output

```yaml
prisma_schema_path: string
models: [{ name: string, pk: string[], relations: [...] }]
enums: [string]
api:
  framework: next-app-router | express | other
  base_path: string # e.g. /api
transport:
  list_query_params: [where, orderBy, page, pageSize, cursor]
```

## Next

`plan.md`
