# Reflect — Verify normalization and queries

## Graph integrity

1. **Single source of truth**
   - Load a list via `useGQLList`, open detail via `useGQLEntity` for same `id` — should not flash wrong data if normalization matches.

2. **Cross-transport**
   - If REST also loads `Post`, mutate via GraphQL and confirm REST-bound components update (proves shared graph).

3. **Relations**
   - Inspect `useGraphStore.getState().entities` in DevTools/console (debug only; not in components) to ensure nested types received IDs and upserts.

## List keys

- Change filter variables → new `queryKey` → distinct list bucket; verify no accidental cache bleed.

## Subscriptions

- Trigger a backend event; confirm UI updates within one flush frame (RealtimeManager) or immediately (`useGQLSubscription`).

## Errors

- Force 401 / malformed query; `onError` fires; entity/list error state surfaces in hook returns.

## Performance

- Mount ten identical list hooks → single deduped request (`dedupe` in `GQLClient.query`).

## Documentation

- Add short README section: endpoint env vars, how to regenerate descriptors when schema changes.

## Done when

- [ ] Typecheck passes.
- [ ] Happy path + one error path manually verified.
- [ ] Descriptors documented or codegen script noted.

## Handoff

Archive notes via `persist.md`.
