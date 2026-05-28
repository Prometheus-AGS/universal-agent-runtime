## Why

Two direct-entity migrations (Providers, Agents) have shipped without any automated regression test for the patterns they introduced (graph propagation, optimistic rollback, bridge refetch, SSE adapter). The reflection on the Agent phase flagged this as the highest-priority follow-up before further migrations. Vitest is already in `node_modules` transitively via the entity-mgmt submodule; we just need a config, deps, and an `npm` script entry to make `pnpm --filter ./frontend test` work.

## What Changes

- Author `frontend/vitest.config.ts` with `environment: "happy-dom"`, React plugin, `@/` alias, and `src/**/*.test.{ts,tsx}` glob.
- Author `frontend/src/test/setup.ts` to install `@testing-library/jest-dom/vitest` matchers and reset `useGraphStore` between tests.
- Install dev dependencies: `vitest@4.1.7`, `@vitest/ui`, `@vitejs/plugin-react`, `@testing-library/react`, `@testing-library/user-event`, `@testing-library/jest-dom`, `happy-dom`.
- Add scripts to `frontend/package.json`:
  - `"test": "vitest run"`
  - `"test:watch": "vitest"`
  - `"test:ui": "vitest --ui"`

No existing test files migrated in this change — that's `migrate-existing-bun-tests-to-vitest`.

## Acceptance

- `pnpm --filter ./frontend test` exits 0 with "0 tests" (or "N tests" once #2 lands).
- `pnpm install` succeeds with the new deps.
- The `vitest.config.ts` resolves the `@/` alias correctly.
