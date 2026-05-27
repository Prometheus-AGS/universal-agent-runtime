## Why

Two tests in `frontend/src/admin/pages/skills-page.utils.test.ts` (`buildCreateSkillRequest maps markdown-capable fields` and `buildUpdateSkillRequest preserves id and partial updates`) construct `SkillEditorFormState` object literals that omit the required `preferredModel: string` field. `buildExecutionConfig` then calls `form.preferredModel.trim()` → `TypeError: Cannot read properties of undefined (reading 'trim')`. These failures only became visible after `vitest` started running the previously-dead `bun:test` files.

## What Changes

Add `preferredModel: ""` to each of the two failing fixtures. Eye-audit `SkillEditorFormState` to confirm no other required field is missing. No production code change.

## Acceptance

- `pnpm --filter ./frontend test` reports `Tests  36 passed (36)`.
- Diff scoped to `skills-page.utils.test.ts`.
- Optional: `pnpm --filter ./frontend typecheck` clean for these test bodies.
