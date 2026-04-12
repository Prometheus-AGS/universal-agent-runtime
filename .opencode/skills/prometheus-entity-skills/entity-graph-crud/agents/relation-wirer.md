# Agent: relation-wirer

**Goal:** Register `EntitySchema` entries so `cascadeInvalidation` and `readRelations` behave correctly.

## Inputs

- All entity types participating in the feature
- Foreign keys and child collections
- Actual `listQueryKey` constructors used in hooks

## Steps

1. **Per entity, draft `EntitySchema`**

   ```ts
   registerSchema({
     type: "Task",
     relations: { /* ... */ },
     globalListKeys: optional,
   });
   ```

2. **`belongsTo`**

   - `foreignKey` column on this entity pointing to parent
   - `targetType` parent entity
   - `invalidateTargetLists` when parent aggregates (e.g. task counts per project) need refresh—use serialized key prefixes that match `invalidateLists` filtering logic in `cascadeInvalidation`

3. **`hasMany`**

   - Child type and `foreignKey` on child pointing back
   - `listKeyPrefix(parentId)` must return the **same array shape** the child list hook uses (then serialized by the engine)

4. **`manyToMany`**

   - `localArrayField` on this entity (id array)
   - `listKeyPrefix` for partner entity lists that should refresh when membership changes

5. **Reverse invalidation**

   - Library walks other schemas to invalidate `hasMany` back-links—ensure `targetType` matches the entity being mutated

6. **Bootstrap location**

   - Single module `registerRelationSchemas()` called from app entry before routes render

## Output

- TypeScript `registerSchema` calls with comments tying each relation to UI features
- Table of example serialized keys for sanity checking

## Checklist

- [ ] Every `listKeyPrefix` matches a real `baseQueryKey` / filter combination
- [ ] No circular imports between API and schema modules
- [ ] IDs used in keys are `EntityId`-compatible
