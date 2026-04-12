# Agent: type-generator

## Role

Generate TypeScript interfaces for entities stored in the graph, compatible with `Record<string, unknown>`.

## Pattern

```typescript
/** Normalized Task as returned by /api/tasks */
export interface TaskEntity extends Record<string, unknown> {
  id: string;
  title: string;
  status: TaskStatus;
  projectId: string;
  createdAt: string; // ISO from JSON
}
```

## Rules

1. Use `string` for `DateTime` in JSON APIs unless you standardize `Date` parsing in normalize.
2. Optional Prisma fields → optional TS properties **or** explicit `null` in API contract.
3. Relations:
   - **Do not** embed full nested objects in entity type if graph stores **only IDs** (normalized); use `projectId` not `project` on `TaskEntity` unless you explicitly denormalize.
4. Enums: `export type TaskStatus = "TODO" | "DONE"` mirroring Prisma enum values.

## File layout

- `src/types/entities/task.ts` or colocated with config `src/entity/task.ts`

## Handoff

Import these types into `createPrismaEntityConfig<TaskEntity>(...)`.
