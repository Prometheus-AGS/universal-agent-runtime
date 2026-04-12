# Reflect — entity-graph-setup verification

Run after **execute** (or after user applies patches).

## Automated checks

From the **application** package root (not necessarily repo root):

```bash
# TypeScript
pnpm exec tsc --noEmit || pnpm run typecheck || npx tsc --noEmit

# Lint (if configured)
pnpm run lint || true
```

For this monorepo’s library development:

```bash
pnpm run typecheck
pnpm run typecheck:vite
pnpm run typecheck:next
```

## Behavioral checks

- [ ] Navigate to migrated screen: **initial load** shows data; **loading** states behave.
- [ ] Mutate an entity elsewhere (or via devtools): **all views** showing that id update without duplicate fetch storms (SWR may still refetch—watch counts).
- [ ] List pagination: **ids** append or replace per hook options; no duplicated rows in UI.
- [ ] **SSR page** (if any): no hydration mismatch; server HTML matches first client render.

## Architecture audit (quick)

- [ ] No `useGraphStore` in **`.tsx` component files** (allow in `hooks/` / `lib/` if justified).
- [ ] No raw `fetch` introduced **new** in components as part of this migration.
- [ ] `registerSchema` **listKeyPrefix** matches actual `useEntityList` query keys.

## Output artifact

`reflect_report.md`:

- Commands run + exit codes
- Pass/fail per checklist item
- Follow-ups filed (e.g. “enable entity-graph-migrate phase 2 for Comments”)
