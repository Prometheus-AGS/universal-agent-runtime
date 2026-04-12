---
name: entity-prisma-api
description: >
  Slash command /entity-prisma-api — scaffold Next.js App Router API routes with Prisma Client
  for list/detail/create/update/delete, decoding where/orderBy JSON from createPrismaEntityConfig
  list fetchers with allowlisted filters.
---

# /entity-prisma-api

## Prerequisites

- `lib/prisma.ts` singleton
- Shared validation helpers: `safeParseWhere`, `safeParseOrderBy`, `parseBody`

## Route templates

### Collection (`app/api/tasks/route.ts`)

- `GET` → `findMany` + `count`
- `POST` → `create`

### Item (`app/api/tasks/[id]/route.ts`)

- `GET` → `findUnique`
- `PATCH` → `update`
- `DELETE` → `delete`

### Example: list + count (sketch)

```typescript
import { NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { allowlistedWhere, allowlistedOrderBy } from "@/lib/prisma-query";

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url);
  const whereRaw = searchParams.get("where");
  const orderRaw = searchParams.get("orderBy");
  const page = Number(searchParams.get("page") ?? "1");
  const pageSize = Math.min(Number(searchParams.get("pageSize") ?? "20"), 100);

  const where = allowlistedWhere("Task", whereRaw);
  const orderBy = allowlistedOrderBy("Task", orderRaw);

  const [items, total] = await Promise.all([
    prisma.task.findMany({
      where,
      orderBy,
      skip: (page - 1) * pageSize,
      take: pageSize,
    }),
    prisma.task.count({ where }),
  ]);

  return NextResponse.json({ items, total, currentPage: page, pageSize });
}
```

Implement `allowlistedWhere` / `allowlistedOrderBy` per model (parse JSON safely, reject unknown keys and deep nesting). Never pass arbitrary client JSON to Prisma.

## Query params

Mirror `listSearchParams` from library:

- `where` — JSON string
- `orderBy` — JSON string
- `page`, `pageSize`, `cursor` as needed

## Response shape

Return JSON compatible with `readListResponse`:

```json
{ "items": [...], "total": 42, "nextCursor": "..." }
```

## Auth

- Protect mutating methods
- Inject tenant scope into `where` merge **server-side**

## Agent

See `agents/api-scaffolder.md`.
