## 1. Reads
- [ ] 1.1 Replace `useSettings()` (legacy store hook) read sites with `useSettingsEntity()` in `settings-page.tsx`
- [ ] 1.2 Hydrate via `loadSettingsIntoGraph()` on mount

## 2. Keep mutations untouched
- [ ] 2.1 Confirm all save/update paths still go through `settings-store`
- [ ] 2.2 No optimistic helper calls yet (next change)

## 3. Smoke
- [ ] 3.1 Manual: edit a settings field → save → hard refresh → value persists
- [ ] 3.2 Two-tab smoke: edit in tab A, observe tab B reflects via SSE

## 4. Verification
- [ ] 4.1 36/36; clean build
