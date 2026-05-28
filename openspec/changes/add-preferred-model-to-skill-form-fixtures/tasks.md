## 1. Edit

- [x] 1.1 Added `preferredModel: ""` to the `buildCreateSkillRequest({...})` fixture in `skills-page.utils.test.ts`.
- [x] 1.2 Added `preferredModel: ""` to the `buildUpdateSkillRequest({...})` fixture in the same file.

## 2. Audit

- [x] 2.1 Audited each fixture against `SkillEditorFormState`. All 8 required fields (`title`, `version`, `description`, `promptOverlay`, `keywords`, `preferredTools`, `enabled`, `preferredModel`) present.

## 3. Verification

- [x] 3.1 `pnpm --filter ./frontend test` → **`Tests  36 passed (36)`**.
- [ ] 3.2 (Optional) `pnpm --filter ./frontend typecheck` clean — not run; can verify later if desired.
