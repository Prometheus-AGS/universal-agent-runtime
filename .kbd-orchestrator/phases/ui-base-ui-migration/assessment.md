# Assessment: Radix UI → Base UI Migration

**Phase:** `ui-base-ui-migration`  
**Date:** 2026-05-20  
**Project:** Universal Agent Runtime (`frontend/`)  
**Assessed by:** kbd-assess + radix-to-base-ui-migration skill + live web research

---

## Executive Summary

The `frontend/` package ships **27 individual `@radix-ui/*` packages** plus `@radix-ui/react-icons` and `@radix-ui/react-slot`, across 46 shadcn/ui component files. The current style is `new-york` backed by Radix. Migrating to Base UI (`@base-ui/react` v1.5.0) with style `base-vega` is well-supported by the shadcn CLI as of December 2025 and will eliminate known Radix bugs (combobox infinite refresh, dropdown flicker, Slot/asChild composition edge cases).

**Migration complexity:** Medium-High  
**Estimated scope:** ~9 migration steps, phased over ~4 implementation batches  
**Risk areas:** `@radix-ui/react-icons` replacement, `assistant-ui` compatibility, `cmdk` combobox, `sidebar.tsx` complexity

---

## Current State Inventory

### Dependency Count

| Category | Count |
|---|---|
| `@radix-ui/*` primitive packages | 27 |
| `@radix-ui/react-icons` (icon library) | 1 |
| `@radix-ui/react-slot` (Slot primitive) | 1 |
| Files importing any `@radix-ui/*` | 40 (UI components) + 3 (app-level) |
| Files with `asChild` prop usage | 13 |
| Total `asChild` occurrences | 44 |
| `data-[state=*]` Tailwind selectors | 40 (UI components) |

### UI Components Inventory (46 total in `src/components/ui/`)

**Atomic (low migration risk):**
- `badge.tsx`, `skeleton.tsx`, `alert.tsx`, `card.tsx`, `input.tsx`, `textarea.tsx`, `table.tsx`, `button.tsx`, `separator.tsx`, `avatar.tsx`

**Overlay/Positioning (medium risk — Positioner swap needed):**
- `tooltip.tsx`, `popover.tsx`, `hover-card.tsx`, `dropdown-menu.tsx`, `context-menu.tsx`, `menubar.tsx`, `navigation-menu.tsx`, `sheet.tsx` (Dialog-based drawer)

**Composite Form Controls (medium risk):**
- `checkbox.tsx`, `switch.tsx`, `slider.tsx`, `radio-group.tsx`, `select.tsx`, `toggle.tsx`, `toggle-group.tsx`, `tabs.tsx`, `accordion.tsx`, `collapsible.tsx`, `progress.tsx`, `scroll-area.tsx`, `calendar.tsx`

**High Complexity (high risk — careful manual migration):**
- `command.tsx` — uses `cmdk` which wraps Radix; must verify `cmdk` Base UI compatibility or replace with Base UI `Combobox`
- `sidebar.tsx` — large composite, many `asChild` usages, sheet + collapsible internals
- `dialog.tsx` / `alert-dialog.tsx` — currently uses `@radix-ui/react-icons` Cross2Icon inline
- `form.tsx` — wraps `@radix-ui/react-label` + `@radix-ui/react-slot`; must port to `Field.*` pattern
- `breadcrumb.tsx` — uses `Slot` from `@radix-ui/react-slot` directly
- `input-otp.tsx` — third-party `input-otp` package; verify it doesn't pull in Radix

### App-Level Files Using Radix Directly

| File | Radix Usage |
|---|---|
| `src/components/model-selector.tsx` | `asChild` on buttons |
| `src/features/chat/agent-selector.tsx` | `asChild` on interactive elements |
| `src/features/chat/capability-toggles.tsx` | `asChild` |
| `src/admin/components/agent-editor.tsx` | `asChild` |
| `src/admin/pages/settings-page.tsx` | `asChild` |
| `src/components/assistant-ui/tooltip-icon-button.tsx` | `asChild` |
| `src/components/assistant-ui/enhanced-thread.tsx` | `asChild` |
| `src/components/layout/top-nav.tsx` | `asChild` |
| `src/components/layout/left-sidebar.tsx` | `asChild` |

### components.json (Current)

```json
{
  "style": "new-york",
  "baseColor": "slate",
  "rsc": false,
  "tsx": true
}
```

**Target:**
```json
{
  "style": "base-vega",
  "baseColor": "neutral",
  "rsc": false,
  "tsx": true
}
```

---

## Gap Analysis Against Target State

### Gap 1 — Package Layer (CRITICAL)
- **Current:** 27+ `@radix-ui/*` packages + `@radix-ui/react-slot`
- **Target:** `@base-ui/react` (v1.5.0, released May 2026) as sole primitive
- **Action:** `pnpm remove` all `@radix-ui/*`, `pnpm add @base-ui/react @floating-ui/react`

### Gap 2 — Icon Library (HIGH)
- **Current:** `@radix-ui/react-icons` used in 16 files (Cross2Icon, ChevronRightIcon, DotFilledIcon, CheckIcon, ChevronDownIcon, etc.)
- **Target:** `lucide-react` (already installed) — replace all icon imports
- **Note:** `lucide-react` is already in `package.json`; this is a mechanical find-replace
- **Action:** Global replacement of `@radix-ui/react-icons` imports with `lucide-react` equivalents

### Gap 3 — components.json Style (HIGH)
- **Current:** `"style": "new-york"` (Radix-backed)
- **Target:** `"style": "base-vega"` (Base UI-backed, classic shadcn look)
- **Action:** Update `components.json`, then `pnpm dlx shadcn@latest add --all --overwrite` to regenerate all UI components

### Gap 4 — `asChild` → `render` Prop (HIGH)
- **Scope:** 44 occurrences across 13 files
- **Pattern:** `<Button asChild><a href="...">` → `<Button render={<a href="..." />}>`
- **Action:** Per-component mechanical rewrite after CLI regenerates base components

### Gap 5 — `Slot` Direct Usage (MEDIUM)
- **Files:** `breadcrumb.tsx`, `button.tsx`, `form.tsx` import `Slot` from `@radix-ui/react-slot`
- **Action:** Replace with `useRender` from `@base-ui/react/use-render`

### Gap 6 — `data-[state=*]` Tailwind Selectors (MEDIUM)
- **Scope:** 40 occurrences across UI components
- **Note:** Base UI largely preserves the same `data-[state=open/closed/checked]` conventions; verify per-component after CLI regeneration — most will be handled automatically by `shadcn add --overwrite`

### Gap 7 — `cmdk` Combobox (MEDIUM-HIGH)
- **Current:** `command.tsx` wraps `cmdk` which has Radix peer dependency
- **Risk:** `cmdk` v1.x may still pull in `@radix-ui/*` internally even after migration
- **Action:** After migration, audit `cmdk`'s dependency tree. If it still imports Radix, replace `command.tsx` with Base UI native `Combobox` component.

### Gap 8 — `form.tsx` / `Field.*` Migration (MEDIUM)
- **Current:** shadcn `<Form>` wrapping `react-hook-form` + Radix `Label` + Slot
- **Target:** Base UI `Field.Root`, `Field.Label`, `Field.Error`, `Field.Control`
- **Note:** `react-hook-form` + `zodResolver` are unchanged; only the primitive layer changes

### Gap 9 — `assistant-ui` Compatibility (LOW-MEDIUM)
- **Current:** `@assistant-ui/react` v0.12.10 wraps its own Radix internals
- **Risk:** `assistant-ui` may pull in Radix regardless of our migration (peer dep)
- **Action:** After migration, run `pnpm why @radix-ui/react-dialog` etc. to check if `assistant-ui` reintroduces Radix. If so, the audit passes — we cannot control transitive deps of third-party packages, only our own code. File an issue with `assistant-ui` upstream.

---

## Migration Plan (9 Steps, 4 Batches)

### Batch 1 — Foundation (Steps 1–2)
**Goal:** Swap packages, update config, regenerate all UI primitives from shadcn CLI

1. **Back up** `src/components/ui/` to `src/components/ui-radix-backup/`
2. **Remove** all `@radix-ui/*` packages from `package.json`; **add** `@base-ui/react` + `@floating-ui/react`
3. **Update** `components.json` → `"style": "base-vega"`, `"baseColor": "neutral"`
4. **Run** `pnpm dlx shadcn@latest add --all --overwrite` to regenerate all 46 UI components

### Batch 2 — Icon Replacement (Step 3)
**Goal:** Eliminate all `@radix-ui/react-icons` usage

5. **Global replace** all 16 files using `@radix-ui/react-icons` with `lucide-react` equivalents:
   - `Cross2Icon` → `X`
   - `ChevronRightIcon` → `ChevronRight`
   - `ChevronDownIcon` → `ChevronDown`
   - `DotFilledIcon` → `Circle` (filled)
   - `CheckIcon` → `Check`
   - `DotsHorizontalIcon` → `MoreHorizontal`
   - (full mapping to be established during implementation)

### Batch 3 — asChild / Slot Removal (Steps 4–6)
**Goal:** Remove all Radix composition patterns from app-level code

6. **Rewrite** `asChild` → `render` prop in 9 app-level files
7. **Replace** `Slot` direct imports with `useRender` in `breadcrumb.tsx`, `button.tsx`, `form.tsx`
8. **Port** `form.tsx` to Base UI `Field.*` pattern

### Batch 4 — Verification + Edge Cases (Steps 7–9)
**Goal:** Confirm zero Radix footprint in own code; handle `cmdk` and `assistant-ui`

9. **Audit** `cmdk`: run `pnpm why @radix-ui/react-dialog` — if `cmdk` still imports Radix, replace `command.tsx` with Base UI native `Combobox`
10. **Audit** `assistant-ui`: accept transitive Radix if present (third-party), or replace if Base UI–compatible version is available
11. **Run** full TypeScript check + lint + E2E smoke tests

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `cmdk` retains Radix dep | High | Medium | Replace with Base UI Combobox |
| `assistant-ui` pulls Radix transitively | High | Low | Accept; outside our control |
| `data-[state=*]` selector breakage after regen | Medium | Medium | Verify visually + E2E after Batch 1 |
| `sidebar.tsx` complexity post-regen | Medium | High | Manual review after CLI overwrite |
| `input-otp` Radix dep | Low | Low | Run `pnpm why` to confirm |
| `react-day-picker` Radix dep | Low | Low | Run `pnpm why` to confirm |

---

## Success Criteria

- [ ] `grep -r "@radix-ui" src/` returns zero results in **own code** (transitive from third-party packages is acceptable)
- [ ] `grep -r "asChild" src/` returns zero results
- [ ] `grep -r "from '@radix-ui/react-slot'" src/` returns zero results
- [ ] `pnpm tsc --noEmit` passes
- [ ] `pnpm lint` passes  
- [ ] All shadcn UI components render correctly (visual smoke test)
- [ ] `dialog.tsx`, `dropdown-menu.tsx`, `select.tsx`, `combobox` work without flicker or infinite refresh
- [ ] E2E smoke tests pass for chat, admin, model-selector flows

---

## Sources

- [shadcn/ui Changelog — January 2026 Base UI](https://ui.shadcn.com/docs/changelog/2026-01-base-ui)
- [Migrate from Radix UI to Base UI in 9 Easy Steps — shadcnstudio](https://shadcnstudio.com/blog/migrate-from-radix-ui-to-base-ui)
- [Migrating from Radix UI — basecn.dev](https://basecn.dev/docs/get-started/migrating-from-radix-ui)
- [@base-ui/react on npm](https://www.npmjs.com/package/@base-ui/react) (v1.5.0, May 2026)
- [Base UI v1 announcement — InfoQ](https://www.infoq.com/news/2026/02/baseui-v1-accessible/)
- [shadcn/ui Discussion #9562 — Radix to Base UI Migration Guide](https://github.com/shadcn-ui/ui/discussions/9562)
- [shadcnblocks.com — Base UI Support](https://www.shadcnblocks.com/blog/introducing-base-ui-and-component-styles)
