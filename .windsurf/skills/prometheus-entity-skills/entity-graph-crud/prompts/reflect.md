# Reflect — entity-graph-crud

Validate implementation after `execute`.

## 1. Architecture audit (static)

Search violations:

- `useGraphStore` imports under component directories (e.g. `components/`, `pages/` `*.tsx`) → **must be absent** except in dedicated `hooks/` or `stores/` files that are not components.
- `fetch(` inside `*.tsx` → **must be absent**.

## 2. Typecheck

```bash
pnpm run typecheck
```

If the consumer is an example app:

```bash
pnpm run typecheck:vite
# or
pnpm run typecheck:next
```

## 3. Behavioral smoke tests (manual)

- [ ] List loads; pagination/cursor if enabled
- [ ] Selecting row opens detail with correct data
- [ ] Edit → save updates graph and list row without full reload
- [ ] Edit error rolls back optimistic state
- [ ] Create adds entity; optional `selectAfterCreate` behavior
- [ ] Delete removes row and clears selection appropriately
- [ ] Related lists refresh when FK changes (if schema registered)

## 4. Graph invariants

- Confirm **IDs in lists** are strings/keys consistent with `EntityId` usage.
- Confirm **no duplicate entity copies** stored outside the graph (no parallel `useState` mirroring server rows).

## 5. Documentation strings

Public hooks (`useXxxCRUD`) should have a one-line JSDoc describing which entity and API surface they wrap.

## 6. Persist decision

If reflect passes, proceed to `prompts/persist.md`. If not, open a minimal fix list prioritized by user impact.
