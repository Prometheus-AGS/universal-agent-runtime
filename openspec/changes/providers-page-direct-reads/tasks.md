## 1. Imports

- [x] 1.1 `import { useProviders } from "@/entities/hooks/use-providers";`
- [x] 1.2 `import { useProviderDefault } from "@/entities/hooks/use-provider-default";`

## 2. Read swaps

- [x] 2.1 Replaced `catalog`/`configured`/`defaultId`/`loading` reads with view + hook + derivations.
- [x] 2.2 Subtitle counts derived from filtered length.
- [x] 2.3 Sort + search behaviour preserved (page-side filter+sort still applied; entity-view defaults unchanged).
- [x] 2.4 Added `status` + `status_detail` to `ProviderEntity` so the credential-blocked badge renders correctly.

## 3. Mutations stay on legacy

- [x] 3.1 `configureProvider`/`setDefault`/`removeProvider`/`saving`/`removing`/`error`/`load` still come from `useProvidersAdmin()`.

## 4. Verification

- [x] 4.1 `pnpm --filter ./frontend build` clean.
- [ ] 4.2 Manual: page renders identically; counts match — pending browser smoke.
- [ ] 4.3 Two-tab smoke — pending browser smoke.
