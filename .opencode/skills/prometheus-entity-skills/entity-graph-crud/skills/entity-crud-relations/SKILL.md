---
name: entity-crud-relations
description: >
  Register EntitySchema graphs with registerSchema: belongsTo, hasMany, manyToMany, listKeyPrefix alignment,
  invalidateTargetLists, globalListKeys, and verification that cascadeInvalidation matches list query keys.
---

# `/entity-crud-relations` — Schema registration and cascade invalidation

## When to use

- Entities reference other entities via FKs or id arrays
- Mutations should **automatically** refresh related lists and detail joins
- You use **`readRelations`** in detail panels (populated via `useEntityCRUD`)

## API surface (`src/crud/relations.ts`)

- **`registerSchema(schema: EntitySchema)`** — idempotent replace per `type`
- **`EntitySchema`**: `type`, optional `relations`, optional `globalListKeys`
- **`BelongsToRelation`**: `foreignKey`, `targetType`, optional `invalidateTargetLists` (prefix matching)
- **`HasManyRelation`**: `targetType`, `foreignKey` on child, **`listKeyPrefix(parentId)`** → same shape as child list’s `baseQueryKey` / serialized key
- **`ManyToManyRelation`**: `targetType`, `localArrayField`, `listKeyPrefix(thisId)`

## **`cascadeInvalidation`**

Called automatically after successful **create/update/delete** inside **`useEntityCRUD`**. Your job is to **register accurate schemas** so it can:

- Invalidate parent/target entities when FKs move
- Invalidate child lists when parent id or membership changes
- Invalidate reverse **`hasMany`** edges registered on other types

## **`readRelations(type, entity)`**

Returns a plain object of joined data for detail UI. **Does not fetch**; reads the graph and list slot for ids. Ensure child lists are loaded elsewhere when you expect non-empty arrays.

## Checklist

- [ ] **`listKeyPrefix`** return value serializes to the **same** key `useEntityView` / `useEntityList` uses (compare with **`serializeKey`** mentally or in dev logs)
- [ ] **`globalListKeys`** used sparingly for “any list of this type” invalidation
- [ ] Registration runs **once** at app startup before screens mount

## Playbook

**`agents/relation-wirer.md`**

## Parent skill

**`prometheus-entity-skills/entity-graph-crud/SKILL.md`**
