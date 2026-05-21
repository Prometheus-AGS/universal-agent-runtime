# Plan: ui-base-ui-migration

**Phase:** `ui-base-ui-migration`  
**Created:** 2026-05-20  
**Change Backend:** OpenSpec  
**Assessment:** `.kbd-orchestrator/phases/ui-base-ui-migration/assessment.md`

---

## Ordered Change List

| Order | Change ID | Scope | Agent | Dependency |
|---|---|---|---|---|
| 1 | `base-ui-foundation` | Package swap + shadcn CLI regen | `gsd-executor` | none |
| 2 | `base-ui-icon-migration` | Replace @radix-ui/react-icons → lucide-react | `gsd-executor` | base-ui-foundation |
| 3 | `base-ui-composition-patterns` | asChild → render, Slot → useRender, form.tsx → Field.* | `gsd-executor` | base-ui-icon-migration |
| 4 | `base-ui-verification` | cmdk audit, assistant-ui audit, E2E smoke tests | `gsd-executor` + `e2e-runner` | base-ui-composition-patterns |

---

## Change Summaries

### Change 1: base-ui-foundation
**Goal:** Establish the new primitive foundation.  
Remove 27 `@radix-ui/*` packages, install `@base-ui/react` + `@floating-ui/react`, update
`components.json` to `"style": "base-vega"`, and run `pnpm dlx shadcn@latest add --all --overwrite`
to regenerate all 46 UI component files from the Base UI registry.

**Files:** `frontend/package.json`, `frontend/components.json`, `frontend/src/components/ui/*.tsx` (all 46)  
**Risk:** Medium — shadcn CLI regen overwrites custom modifications; backup required first.  
**Tasks:** `openspec/changes/base-ui-foundation/tasks.md`

---

### Change 2: base-ui-icon-migration
**Goal:** Eliminate the last `@radix-ui` package (`react-icons`) from own code.  
Replace all 16 files using `@radix-ui/react-icons` with `lucide-react` equivalents (already installed).
Full icon mapping documented in proposal.  

**Files:** 16 UI component files (see proposal.md for list)  
**Risk:** Low — mechanical find-replace with already-installed package.  
**Tasks:** `openspec/changes/base-ui-icon-migration/tasks.md`

---

### Change 3: base-ui-composition-patterns
**Goal:** Eliminate all Radix composition patterns.  
Rewrite 44 `asChild` usages → `render` prop across 13 files. Replace `Slot` imports with
`useRender` in 3 UI files. Migrate `form.tsx` from Radix Label/Slot to Base UI `Field.*` pattern
while keeping react-hook-form + Zod wiring unchanged.

**Files:** 9 app-level files + `button.tsx`, `breadcrumb.tsx`, `form.tsx`, `sidebar.tsx`, `select.tsx`  
**Risk:** Medium-High — `form.tsx` migration requires care; `sidebar.tsx` is the most complex file.  
**Tasks:** `openspec/changes/base-ui-composition-patterns/tasks.md`

---

### Change 4: base-ui-verification
**Goal:** Confirm zero own-code Radix footprint; handle edge cases; run E2E.  
Audit `cmdk` transitive deps — if still Radix-backed, replace `command.tsx` with Base UI native
`Combobox` while preserving public API. Audit `assistant-ui` transitive deps (accept if
uncontrollable). Run Playwright E2E + manual smoke tests for all critical flows.

**Files:** `frontend/src/components/ui/command.tsx` (conditional), `frontend/docs/` (audit notes)  
**Risk:** Medium — cmdk replacement is conditional and requires API-compatible rewrite.  
**Tasks:** `openspec/changes/base-ui-verification/tasks.md`

---

## Implementation Guidance

### Always work in change order
Each change gates the next. Do not start `base-ui-icon-migration` until `base-ui-foundation`
TypeScript check passes (with only icon errors).

### Commit strategy
One commit per change. Use conventional commits:
- `feat: swap radix-ui for @base-ui/react and regenerate shadcn components`
- `feat: replace @radix-ui/react-icons with lucide-react across all ui components`
- `feat: migrate asChild to render prop and form.tsx to Base UI Field pattern`
- `feat: complete base-ui migration — verification and cleanup`

### TypeScript is the progress gate
After each change, `pnpm typecheck` must pass (or only have expected errors from the next
change's scope). Do not move to the next change with unexpected type errors.

### Do not touch Rust backend
This phase is strictly `frontend/` only. No Cargo.toml or src/ changes.

---

## Success Criteria (Phase-Level)

From assessment:
- [ ] `grep -r "@radix-ui" frontend/src/` returns zero results in **own code**
- [ ] `grep -r "asChild" frontend/src/` returns zero results
- [ ] `grep -r "from '@radix-ui/react-slot'" frontend/src/` returns zero results
- [ ] `pnpm tsc --noEmit` passes
- [ ] `pnpm lint` passes
- [ ] All shadcn UI components render correctly
- [ ] Dialog, dropdown-menu, select work without flicker or infinite refresh
- [ ] E2E smoke tests pass for chat, admin, model-selector flows
