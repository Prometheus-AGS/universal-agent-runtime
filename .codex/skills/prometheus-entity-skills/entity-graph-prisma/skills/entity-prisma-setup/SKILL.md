---
name: entity-prisma-setup
description: >
  Slash command /entity-prisma-setup — analyze schema.prisma, generate TypeScript entity types,
  createPrismaEntityConfig modules, registerSchema bootstrap, and align REST list/detail shapes
  with @prometheus-ags/prometheus-entity-management adapters.
---

# /entity-prisma-setup

## Flow

1. **schema-parser** → YAML inventory
2. **type-generator** → `*Entity` interfaces extending `Record<string, unknown>`
3. **api-scaffolder** → minimal Next.js routes (or your server)
4. **Client** → `createPrismaEntityConfig` + `registerSchema` in app init

## Deliverables

- Server: working list + detail endpoints
- Client: `taskEntity.entity` / `taskEntity.list` / `taskEntity.crud` wired in hooks
- Docs: query param contract

## References

- `../references/prisma-mapping.md`
- `../references/transport-adapters.md`
- Library: `src/adapters/prisma.ts`
