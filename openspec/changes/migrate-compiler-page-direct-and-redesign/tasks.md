## 1. Shared aesthetic components
- [ ] 1.1 Author `frontend/src/components/admin/empty-frame.tsx` (ASCII border + slot + action)
- [ ] 1.2 Author `frontend/src/components/admin/loading-cursor.tsx` (flicker ▍ at 600ms)
- [ ] 1.3 Author `frontend/src/components/admin/error-bar.tsx` (mono code prefix, red pin)

## 2. Page migration
- [ ] 2.1 Replace `useCompilerSessionsStore` reads with `useEntityList("CompilerSession")`
- [ ] 2.2 Hydrate via fetcher on mount (add `entities/fetchers/compiler-sessions.ts` if missing)
- [ ] 2.3 `git rm frontend/src/stores/compiler-sessions-store.ts`

## 3. Aesthetic
- [ ] 3.1 Apply terminal tokens; banned-font grep clean

## 4. Screenshot
- [ ] 4.1 Playwright snapshot at `screenshots/compiler-page.png`

## 5. Audit
- [ ] 5.1 Flip `CompilerSession` row to `direct` in audit doc

## 6. Verification
- [ ] 6.1 36/36 tests; clean build; `useGraphStore.getState` grep empty on the page
