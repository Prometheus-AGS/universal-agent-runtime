## 1. Mutations
- [ ] 1.1 Replace every settings save with `optimisticUpsert("Setting", id, patch, () => saveApi(...))`
- [ ] 1.2 Replace every settings delete with `optimisticRemove("Setting", id, …)`
- [ ] 1.3 Local error state via `useState` (no store-level `error` flag)

## 2. Store retire
- [ ] 2.1 `git rm frontend/src/stores/settings-store.ts`
- [ ] 2.2 `git grep useSettings frontend/` shows only `useSettingsEntity` (rename if needed)
- [ ] 2.3 Confirm `settings-types-meta-store.ts` is still imported (schemas)

## 3. Aesthetic
- [ ] 3.1 Apply terminal tokens to every settings sub-section
- [ ] 3.2 Replace generic Tailwind cards with terminal-styled cards
- [ ] 3.3 Loading/empty/error states use shared components
- [ ] 3.4 Banned-font grep clean

## 4. Screenshot + audit
- [ ] 4.1 Playwright `screenshots/settings-page.png` (capture each major sub-tab)
- [ ] 4.2 Flip `Setting` row to `direct`

## 5. Verification
- [ ] 5.1 36/36; clean build
- [ ] 5.2 `git grep "useGraphStore.getState" frontend/src/admin/pages/settings-page.tsx` empty
- [ ] 5.3 Manual smoke: edit + save + refresh across each settings tab
