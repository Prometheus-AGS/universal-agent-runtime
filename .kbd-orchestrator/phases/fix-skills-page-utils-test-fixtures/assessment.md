# Assessment — `fix-skills-page-utils-test-fixtures`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `vitest-contract-test-suite` (reflect_complete, 90%)

---

## 1. Phase goal

Bring the frontend test suite to **36/36 green** by fixing the 2 remaining failures in `frontend/src/admin/pages/skills-page.utils.test.ts`. These are pre-existing test-fixture bugs that surfaced once Vitest actually started running the previously-dead `bun:test` files.

---

## 2. Current state

### 2.1 The failure

```
TypeError: Cannot read properties of undefined (reading 'trim')
 ❯ buildExecutionConfig src/admin/pages/skills-page.utils.ts:69:37
     69|   const model = form.preferredModel.trim() || null;
 ❯ buildCreateSkillRequest src/admin/pages/skills-page.utils.ts:84:23
 ❯ src/admin/pages/skills-page.utils.test.ts:15:21
```

`buildExecutionConfig` reads `form.preferredModel.trim()`. The 2 failing test cases construct form payloads as object literals that **omit `preferredModel`**, even though `SkillEditorFormState` declares it as a required `string` (with default `""` in `DEFAULT_SKILL_FORM`).

### 2.2 The two failing test bodies (from `skills-page.utils.test.ts`)

Both tests do:

```ts
const payload = buildCreateSkillRequest({
  title: "Skill One",
  version: "1.2.3",
  description: "...",
  promptOverlay: "...",
  keywords: "...",
  preferredTools: "...",
  enabled: true,
  // ← preferredModel intentionally absent
});
```

The third test (`parseCommaSeparated`) passes because it doesn't touch the form shape.

### 2.3 Why TypeScript doesn't catch this

The test file passes object literals straight into `buildCreateSkillRequest`. The compiler should catch the missing required field, but the test was originally written under `bun:test` and the call sites have been silently failing only at *runtime*. The TS check probably reports the error on `pnpm typecheck` — let me note this as a follow-up (the typecheck script is separate and may also be unhappy with these fixtures).

### 2.4 Fix options

| Option | Diff size | Verdict |
|--------|-----------|---------|
| A. Add `preferredModel: ""` (and any other missing field) to each fixture | 2 lines | **Recommended.** Fixtures should be valid `SkillEditorFormState`s. |
| B. Make `buildExecutionConfig` tolerate missing field: `(form.preferredModel ?? "").trim()` | 1 line | Hides the test-author's mistake; defensive coding the wrong direction. |
| C. Make `preferredModel` optional in `SkillEditorFormState` | wider | Spreads the optionality across every consumer (form components, defaults). Overkill. |

Default = **A**.

### 2.5 Other phase outputs

- The skills-page test file (which we already migrated to `vitest`) is otherwise sound — only the form-fixture object literals need the missing field.
- No documentation, infrastructure, or backend change required.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | The 2 failing tests in `skills-page.utils.test.ts` pass. | `pnpm --filter ./frontend test` shows 36 passed / 36 |
| A2 | The fixtures match the actual `SkillEditorFormState` shape — including `preferredModel: ""`. | TypeScript no longer complains via `tsc --noEmit` for these test bodies (if it had been complaining). |
| A3 | Suite total stays at 36 tests (no tests added or removed). | runner output |
| A4 | No production code changed (utils stay untouched). | git diff scoped to `*.test.ts` |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | Two test fixtures missing `preferredModel: ""`. | Low | One-line addition each. |
| G2 | If any *other* required field on `SkillEditorFormState` was added later and the fixtures were never updated, the same fix applies. Need to inspect the full type vs. fixtures. | Low | Audit at fix time. |
| G3 | `pnpm typecheck` may also currently fail on these fixtures (TS would refuse missing required props). | Low | Run typecheck after the fix to confirm. |
| G4 | No regression test for the helper itself — only happy-path assertions. | Low | Out of scope; covered by the contract tests this phase doesn't add to. |

---

## 5. Sequencing recommendation

Single change, single PR:
1. Add `preferredModel: ""` to both failing fixtures.
2. Audit `SkillEditorFormState` vs. each fixture for any other missing required field.
3. Run `pnpm test` — verify 36/36.
4. Optionally run `pnpm typecheck` — verify TS is happy too.

No further changes needed.

---

## 6. Open questions for the user before planning

None. The fix is mechanical and the only viable option (A) is clearly the right one. Defaults apply.

---

## 7. Progress signal

Completed kbd-assess — fix-skills-page-utils-test-fixtures
