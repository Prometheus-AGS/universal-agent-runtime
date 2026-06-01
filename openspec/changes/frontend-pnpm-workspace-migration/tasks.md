## 1. Workspace setup

- [x] 1.1 `frontend/pnpm-workspace.yaml` (`.`, `packages/*`) created.
- [x] 1.2 `packageManager: pnpm@10.33.0` added to `frontend/package.json`.
- [x] 1.3 `bun.lock` removed; `pnpm install` run; `pnpm-lock.yaml` committed.

## 2. Build system

- [x] 2.1 `build.rs::build_frontend` prefers pnpm → bun → npm.
- [x] 2.2 `cargo build` verified end-to-end via pnpm.

## 3. Scripts

- [ ] 3.1 Script audit — no `bun` references found in scripts; nothing to update.
- [ ] 3.2 Doc updates — deferred to integration-tests-and-docs.

## 4. CI

- [ ] 4.1 GitHub Actions pnpm setup — deferred.
- [ ] 4.2 Cache directories — deferred.

## 5. Smoke

- [x] 5.1 `pnpm --filter ./frontend build` produces new bundle hash (`index-C4EUbi3J.js`).
- [ ] 5.2 SPA serves correctly — verified after restart in the live system test.
