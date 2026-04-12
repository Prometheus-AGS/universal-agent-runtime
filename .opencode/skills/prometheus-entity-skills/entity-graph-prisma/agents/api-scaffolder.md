# Agent: api-scaffolder

## Role

Scaffold Next.js App Router (or chosen server) CRUD routes backed by `PrismaClient`.

## Conventions

### List — `GET /api/<modelPlural>`

- Query: `where`, `orderBy` as JSON strings (match `createPrismaEntityConfig` list fetch).
- Response: `{ items: T[], total?: number, nextCursor?: string, ... }` per `readListResponse` in `src/adapters/prisma.ts`.

### Detail — `GET /api/<modelPlural>/[id]`

- `findUnique` with allowlisted `where` (id only unless composite documented).

### Create — `POST`

- Parse JSON body; `create({ data: validated })`.

### Update — `PATCH` or `PUT`

- Parse body; `update({ where: { id }, data })`.

### Delete — `DELETE`

- `delete({ where: { id } })` or soft-delete if schema uses `deletedAt`.

## Safety

- **Allowlist** fields on create/update (no mass assignment from raw JSON).
- Map Prisma errors to HTTP status (409 unique, 404 not found).

## Files

- `app/api/tasks/route.ts`
- `app/api/tasks/[id]/route.ts`

## Testing

- `curl` or REST client collection checked into `docs/` (optional).
