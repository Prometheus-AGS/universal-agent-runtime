> **ABANDONED 2026-07-09 — SUPERSEDED, NOT IMPLEMENTED.**
> While this change was being applied, a background `git pull` moved HEAD
> (`2243ea4` → `cf37653`) and brought in commit `d37abfd`
> _"feat(a2ui): upgrade A2uiTestingPage to a real live round-trip tester"_.
> That commit upgrades the exact page this change proposed to retire, turning it
> from a static schema-preview playground into a real, operationally-useful live
> tester — which directly contradicts this change's premise. Per operator
> decision, the retirement is abandoned. No code changes from this change were
> committed (the in-flight edits were discarded by the pull). Checkboxes below
> are reset to reflect that nothing was actually applied.

## 1. Remove the A2UI testing page component

- [ ] 1.1 Delete `frontend/src/admin/A2uiTestingPage.tsx`

## 2. Unwire the A2UI testing page from navigation and routing

- [ ] 2.1 In `frontend/src/admin/admin-shell.tsx`, remove the `"a2ui-testing"` member from the nav-id union type and delete its nav item entry (`{ id: "a2ui-testing", label: "A2UI Testing", ... }`)
- [ ] 2.2 In `frontend/src/pages/admin-page.tsx`, remove the `A2uiTestingPage` import and the `"a2ui-testing"` route-map entry

## 3. Verify no regressions

- [ ] 3.1 Grep `frontend/src` for `A2uiTestingPage` and `"a2ui-testing"` to confirm no dangling references remain (real A2UI features — `a2ui-api.ts`, `a2ui-schemas-store.ts`, `use-a2ui-schemas.ts`, `a2ui-artifact-block.tsx`, runtime-console surfaces — must still be present)
- [ ] 3.2 Run the frontend build and typecheck (`pnpm run build`) and confirm both pass clean
