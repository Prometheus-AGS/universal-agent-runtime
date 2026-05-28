## Why

`providers-page.tsx` reads its `catalog`, `configured`, `defaultId`, and `loading` state from `useProvidersAdmin()`, which wraps `useProvidersAdminStore`, which fetches REST and caches in Zustand. The bridge from the prior phase keeps the store fresh via SSE, but the page still reads through the store. To retire the bridge for this entity, the page must read directly from the entity graph.

This change swaps **reads only** — mutations stay on the legacy hook so we can verify reads in isolation before touching writes. Bailing out after this change leaves a fully functional page.

## What Changes

- Page imports `useProviders()` from `@/entities/hooks/use-providers` and `useProviderDefault()` from the singleton hook (shipped in `provider-meta-singleton`).
- `const providers = useProviders();` replaces the `catalog`/`configured`/`defaultId`/`loading` reads.
- Derive `catalog` (`providers.items`), `configured` (`providers.items.filter(p => p.configured)`), and the subtitle counts from the view.
- `defaultId` from `useProviderDefault()`.
- `loading` derived from `providers.loading` (or fallback when the field doesn't exist on the view).
- Mutations (`configureProvider`, `setDefault`, `removeProvider`) still come from `useProvidersAdmin()` — untouched in this PR.

## Acceptance

- Page renders pixel-equivalent to today.
- Filter chips (all/configured/unconfigured) still work.
- Subtitle counts match (`N available · M configured`).
- Sort order unchanged (configured-first, then alphabetical).
- Two-tab smoke: configure a provider in tab A → tab B reflects the new row (via SSE → graph → `useProviders` re-render).
