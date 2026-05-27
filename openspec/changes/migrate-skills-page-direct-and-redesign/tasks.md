## 1. Reads
- [ ] 1.1 Swap `useSkillsAdmin()` → `useSkills()`

## 2. Mutations
- [ ] 2.1 Toggle: `optimisticUpsert("Skill", id, { enabled }, () => patchSkillApi(...))`
- [ ] 2.2 Delete: `optimisticRemove("Skill", id, () => deleteSkillApi(...))`

## 3. Store retire
- [ ] 3.1 `git rm frontend/src/stores/skills-admin-store.ts`
- [ ] 3.2 `git grep useSkillsAdmin frontend/` → empty

## 4. Aesthetic
- [ ] 4.1 Apply terminal tokens + shared aesthetic components
- [ ] 4.2 Verify skills-page.utils.test.ts still 2/2

## 5. Screenshot + audit
- [ ] 5.1 `screenshots/skills-page.png`
- [ ] 5.2 Flip `Skill` row to `direct`

## 6. Verification
- [ ] 6.1 36/36; clean build; `useGraphStore.getState` grep empty
