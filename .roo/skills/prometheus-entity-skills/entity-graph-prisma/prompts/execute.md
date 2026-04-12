# Execute — Generate integration

## 1. Prisma singleton (server)

```typescript
// lib/prisma.ts
import { PrismaClient } from "@prisma/client";

const globalForPrisma = globalThis as unknown as { prisma?: PrismaClient };
export const prisma = globalForPrisma.prisma ?? new PrismaClient();
if (process.env.NODE_ENV !== "production") globalForPrisma.prisma = prisma;
```

## 2. List route (Next.js App Router sketch)

```typescript
// app/api/tasks/route.ts
import { prisma } from "@/lib/prisma";
import { NextRequest, NextResponse } from "next/server";

export async function GET(req: NextRequest) {
  const sp = req.nextUrl.searchParams;
  const where = sp.get("where");
  const orderBy = sp.get("orderBy");
  const page = sp.get("page");
  const pageSize = sp.get("pageSize");
  const parsedWhere = safeParseWhere(where); // implement allowlist
  const parsedOrderBy = safeParseOrderBy(orderBy);
  const skip = page && pageSize ? (Number(page) - 1) * Number(pageSize) : undefined;
  const take = pageSize ? Number(pageSize) : undefined;
  const [items, total] = await Promise.all([
    prisma.task.findMany({ where: parsedWhere, orderBy: parsedOrderBy, skip, take }),
    prisma.task.count({ where: parsedWhere }),
  ]);
  return NextResponse.json({ items, total, page: page ? Number(page) : 1, pageSize: take ?? items.length });
}
```

Tune to your pagination contract (`nextCursor` if using cursor).

## 3. Client config

```typescript
import { createPrismaEntityConfig } from "@prometheus-ags/prometheus-entity-management";

export const taskEntity = createPrismaEntityConfig<TaskEntity>({
  type: "Task",
  endpoint: "/api/tasks",
  relations: {
    project: { type: "Project", foreignKey: "projectId", relation: "belongsTo" },
  },
});
```

## 4. Bootstrap schemas

```typescript
import { registerSchema } from "@prometheus-ags/prometheus-entity-management";

for (const s of taskEntity.schemas()) registerSchema(s);
```

## 5. Hooks usage

- `useEntity(taskEntity.entity(id))`
- `useEntityList(taskEntity.list({ queryKey: ["Task"], initialPageSize: 20 }))`
- `useEntityCRUD({ ...taskEntity.crud(), onCreate, onUpdate, onDelete })`

## Order

Implement server routes first (so fetch URLs work), then client config, then UI.

## Next

`reflect.md`
