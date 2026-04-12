# Transport: toPrismaWhere / toPrismaOrderBy

Source: `src/view/prisma-compile.ts` (re-exported from `src/adapters/prisma.ts`).

## toPrismaWhere(FilterSpec)

Accepts:

- Array of `FilterClause` (combined with implicit `AND`), or
- `FilterGroup` with nested `AND` / `OR`.

### Clause → Prisma leaf (selected ops)

| Filter op | Prisma shape |
|-----------|----------------|
| `eq` | `{ equals: value }` |
| `neq` | `{ not: value }` |
| `gt`, `gte`, `lt`, `lte` | `{ gt: ... }` etc. |
| `contains`, `startsWith`, `endsWith` | string op + `mode: "insensitive"` |
| `in` / `nin` | `{ in: [...] }` / `{ notIn: [...] }` |
| `arrayContains` | `{ has: value }` |
| `isNull` | `null` or `{ not: null }` depending on `value` |
| `isNotNull` | `{ not: null }` |

**Omitted**: `between`, `arrayOverlaps`, `matches`, `custom` — extend server or pre-process filters.

### Nested fields

Field `author.name` becomes nested `{ author: { name: { … } } }`.

## toPrismaOrderBy(SortSpec)

Produces Prisma `orderBy` array shape used by list serialization.

## Wire-up

`useEntityView` + `createPrismaEntityConfig().crud()` list fetch receives `ViewFetchParams` with `view.filter` / `view.sort`; the adapter calls `toPrismaWhere` / `toPrismaOrderBy` and JSON-stringifies into query params.

## Server decode

Always validate shape before passing to Prisma:

- Reject unknown top-level keys
- Cap array lengths for `in` / `notIn`
- Enforce max `take`
