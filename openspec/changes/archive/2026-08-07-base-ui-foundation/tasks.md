# Tasks: base-ui-foundation

## Task 1 — Back up existing UI components
- [x] Copy `frontend/src/components/ui/` → `frontend/src/components/ui-radix-backup/`
- [x] Commit backup with message: `chore: backup radix-based ui components before base-ui migration`

## Task 2 — Remove Radix from production primitive source
- [x] Migrate production wrappers away from direct `@radix-ui/*` imports.
- [x] Perform the original package-removal pass and verify `pnpm install` succeeds.
  - HISTORICAL NOTE: `@radix-ui/react-icons` was deferred to `base-ui-icon-migration`.
  - ARCHIVE NOTE (2026-08-07): unused Radix declarations later reappeared in the
    manifest. Production source still has zero direct Radix imports; C-14c owns the
    required transitive audit and final declaration prune.

## Task 3 — Install Base UI packages
- [x] Run: `pnpm add @base-ui/react @floating-ui/react`
- [x] Original implementation installed `@base-ui/react` v1.5.0; the current tree
  pins v1.6.0.

## Task 4 — Update components.json
- [x] `"style": "new-york"` → `"style": "base-vega"`
- [x] `"baseColor": "slate"` → `"baseColor": "neutral"`

## Task 5 — Regenerate all shadcn UI components
- [x] Run from `frontend/`: `pnpm dlx shadcn@latest add --all --overwrite`
- [x] 45 files updated + 11 new files created:
  - New: `field.tsx`, `combobox.tsx`, `spinner.tsx`, `kbd.tsx`, `native-select.tsx`,
    `empty.tsx`, `direction.tsx`, `button-group.tsx`, `item.tsx`, `input-group.tsx`, `use-mobile.ts`
- [x] `button.tsx` imports from `@base-ui/react/button` ✅
- [x] `dialog.tsx` imports from `@base-ui/react/dialog` + uses `lucide-react` XIcon ✅

## Task 5b — Fix Base UI API surface changes (discovered during Task 6)
- [x] `main.tsx`: `delayDuration` → `delay` on `TooltipProvider`
- [x] `form.tsx`: removed `@radix-ui/react-label` + `@radix-ui/react-slot` imports
- [x] `input.tsx`: use `InputPrimitive.Props` instead of `React.ComponentProps<"input">`
- [x] `input-group.tsx`: same fix for `InputGroupInput`
- [x] `calendar.tsx`: remove `table` from `ClassNames` (no longer valid in react-day-picker)
- [x] `capability-toggles.tsx`: remove `onOpenAutoFocus` (not in Base UI `PopoverPopup.Props`)
- [x] `session-config-panel.tsx`: wrap `Select` `onValueChange` to handle `string | null`
- [x] `a2ui-artifact-block.tsx`: same Select fix
- [x] `settings-page.tsx`: same Select fix

## Task 6 — TypeScript check (partial pass expected)
- [x] Run: `pnpm typecheck`
- [x] Remaining errors are EXPECTED scope only:
  - `asChild` in 13 files → handled in `base-ui-composition-patterns`
  - `enhanced-markdown-text.tsx` → pre-existing `@types/react` duplicate-ref from `assistant-ui`
  - `agent-editor.tsx` `string | null` narrowing → pre-existing
  - `agents-page.tsx`, `knowledge-page.tsx` → pre-existing
  - `ui-radix-backup/` → expected (backup files still import Radix)
- [x] No unexpected Base UI primitive errors remain

## Completion Gate
All tasks checked. TypeScript errors limited to asChild/pre-existing scope. 
Commit: `feat: swap radix-ui for @base-ui/react and regenerate shadcn components` ✅
SHA: e92670e

Archive readiness was re-verified against the current tree in `verification.md`.
