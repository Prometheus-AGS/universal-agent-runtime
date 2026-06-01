## 1. Types

- [x] 1.1 Added `ProviderMetaEntity` to `frontend/src/entities/types.ts`.

## 2. Schema

- [x] 2.1 `registerSchema({ type: "ProviderMeta" })` added to `frontend/src/entities/schemas.ts`.

## 3. Fetcher

- [x] 3.1 `loadProvidersIntoGraph()` upserts the singleton with `default_id` from the configured-providers response.

## 4. Hook

- [x] 4.1 Authored `frontend/src/entities/hooks/use-provider-default.ts`.

## 5. Verification

- [x] 5.1 `pnpm --filter ./frontend build` clean.
- [ ] 5.2 Browser check of `useGraphStore.getState().entities["ProviderMeta"]` — pending manual verification.
