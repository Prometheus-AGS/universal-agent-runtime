# Plan — `fix-skills-page-utils-test-fixtures`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/fix-skills-page-utils-test-fixtures/assessment.md`

---

## Single change

| # | Change ID | Title | Size |
|---|-----------|-------|------|
| 1 | `add-preferred-model-to-skill-form-fixtures` | Add `preferredModel: ""` to the two failing fixtures; audit `SkillEditorFormState` for any other missing required field | ~2-line diff |

No multi-step decomposition warranted — this is a single mechanical fix.

---

## Synopsis

`SkillEditorFormState` declares `preferredModel: string` (required). The two failing tests construct form payloads as object literals **omitting** that field. `buildExecutionConfig` then runs `form.preferredModel.trim()` and crashes at runtime.

Fix:

1. In `frontend/src/admin/pages/skills-page.utils.test.ts`, add `preferredModel: ""` to both `buildCreateSkillRequest({...})` and `buildUpdateSkillRequest({...})` fixture object literals.
2. Eye-audit the full set of required keys on `SkillEditorFormState` vs. each fixture — confirm no other missing fields.
3. Run `pnpm --filter ./frontend test` to confirm 36/36 green.
4. (Optional) Run `pnpm --filter ./frontend typecheck` to confirm TS is happy.

No production code changes.

---

## Acceptance gate

1. `pnpm --filter ./frontend test` → `Tests  36 passed (36)`.
2. Diff is scoped to `skills-page.utils.test.ts`; production source untouched.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Another required field also missing | Audit step 2 catches it; same single-line fix per field |
| `parseCommaSeparated` test starts failing | Untouched in this change; should stay green |
| `pnpm typecheck` reveals additional unrelated errors | Out of scope — note them as carry-over, don't address in this phase |

---

## Progress signal

Completed kbd-plan — fix-skills-page-utils-test-fixtures
