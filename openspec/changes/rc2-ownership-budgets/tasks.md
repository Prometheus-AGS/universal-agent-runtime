## 1. Implementation

- [x] 1.1 `CompilerOwner` and owner-filtered spec, report and session storage on SurrealDB and PostgreSQL (#387)
- [x] 1.2 Migration `20261031010000_compiler_owner_scope.sql` (#387)
- [x] 1.3 Compiler REST handlers require a verified owner; another owner's record answers 404 (#387)
- [x] 1.4 Per-run fallback windows and `fit_fallback_request` before each fallback attempt; `context_fit` in the attempt manifest (#388)
- [x] 1.5 Version 1.4.0-rc.2, CHANGELOG entry, `docs/releases/v1.4.0-rc.2.md`
- [x] 1.7 Concurrent first turns share one session (`src/session/thread.rs`)
- [x] 1.6 BDD: `compiler-owner-isolation.feature`, `fallback-destination-budget.feature`; fixture `stubFixtures`, async config, unavailable fault mode

## 2. Verification

- [x] 2.1 server-full and postgres-backend builds with zero warnings
- [x] 2.2 `openspec validate rc2-ownership-budgets`
- [x] 2.3 `pnpm sdk-runtime-pin:validate`
- [x] 2.4 `compiler-owner-isolation.cucumber.mjs` (three scenarios)
- [x] 2.5 `fallback-destination-budget.cucumber.mjs` (two scenarios)
- [x] 2.6 Mac BDD catalog
