## 1. Submodule

- [x] 1.1 `git submodule add … frontend/packages/prometheus-entity-management`.
- [x] 1.2 `git submodule update --init --recursive` confirmed clone.
- [ ] 1.3 Pin to known-good commit — leaving on `main` HEAD for now; pin later.

## 2. Workspace wiring

- [x] 2.1 `frontend/package.json` dep set to `"workspace:*"`.
- [x] 2.2 `pnpm install` resolves the workspace package.
- [x] 2.3 `pnpm --filter @prometheus-ags/prometheus-entity-management build` produces `dist/`.

## 3. Vite

- [x] 3.1 Workspace import resolves with no alias change — confirmed via SPA build.

## 4. Type-check

- [ ] 4.1 Standalone tsc pass — deferred (existing peer-dep warning unrelated to this change).

## 5. Smoke

- [x] 5.1 SPA rebuilds with new bundle hash (`index-mptpqUVW.js`) after workspace dep swap.
