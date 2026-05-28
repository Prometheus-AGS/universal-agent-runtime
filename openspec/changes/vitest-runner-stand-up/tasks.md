## 1. Config

- [ ] 1.1 `frontend/vitest.config.ts` with `environment: "happy-dom"`, React plugin, alias, glob.
- [ ] 1.2 `frontend/src/test/setup.ts` with jest-dom matchers + graph reset hook.

## 2. Dependencies

- [ ] 2.1 `pnpm add -D --filter ./frontend vitest@4.1.7 @vitest/ui @vitejs/plugin-react @testing-library/react @testing-library/user-event @testing-library/jest-dom happy-dom`.

## 3. Scripts

- [ ] 3.1 Add `test`, `test:watch`, `test:ui` to `frontend/package.json`.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend test` exits 0 with "0 tests" before any test files migrate.
- [ ] 4.2 `pnpm install` succeeds.
