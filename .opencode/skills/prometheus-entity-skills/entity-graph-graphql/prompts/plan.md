# Plan — Descriptor mapping and normalization strategy

## Inputs

- Completed `specify.md` artifact.
- Actual SDL or introspection (for field names).

## Design checklist

### 1. Descriptor graph per operation

For **each** operation the app will call:

- [ ] Root `path` from `data` to the first entity or array (e.g. `posts`, `node`, `viewer.posts`).
- [ ] Primary `EntityDescriptor` for the main entity type.
- [ ] `relations[]` for nested entities (author, categories, …) with paths relative to parent node.
- [ ] `sideDescriptors` list for hooks that need multiple roots (e.g. detail query returns `post` + `viewer`).

### 2. List operations

- [ ] `useGQLList.getItems`: pure function from `TData` → `unknown[]` of nodes.
- [ ] `getPagination`: map connection `pageInfo` / custom totals to `{ total, nextCursor, hasNextPage, page, pageSize }`.
- [ ] Stable `queryKey` array (include filter variables).

### 3. ID extraction

- [ ] Confirm `extractId` for Relay `node { id }` global IDs vs database integers.
- [ ] Document if `normalize` rewrites id to string.

### 4. Subscriptions (if any)

- [ ] Choose **descriptor path** (`useGQLSubscription`) vs **adapter `getPayload`** (`RealtimeManager`).
- [ ] Map subscription event fields to `EntityChange` ops (`insert`/`upsert`/`delete`).

### 5. Error and loading UX

- [ ] `onError` on client for logging/metrics.
- [ ] Hook-level `onError` for toast boundaries.

## Output

Markdown table:

| Operation | Hook/API | Root path | Entity types written | Notes |
|-----------|----------|-----------|----------------------|-------|

## Handoff

Execute in `execute.md` order: client module → descriptors → hooks → subscription wiring.
