# Tasks: base-ui-composition-patterns

## Task 1 — Audit remaining asChild usages post-regen
- [ ] Run: `grep -rn "asChild" frontend/src/ --include="*.tsx"` and record full list
- [ ] Categorize: (a) UI component files, (b) app-level files
- [ ] Note: shadcn CLI regen may have already removed asChild from some ui/ files — only migrate what remains

## Task 2 — Migrate button.tsx Slot → useRender
- [ ] Replace `import { Slot } from '@radix-ui/react-slot'` with `import { useRender } from '@base-ui/react/use-render'`
- [ ] Rewrite `ButtonProps.asChild` → `ButtonProps.render`
- [ ] Rewrite internal `const Comp = asChild ? Slot : 'button'` → `useRender` pattern
- [ ] Verify button renders correctly as both `<button>` and polymorphic element

## Task 3 — Migrate breadcrumb.tsx Slot → useRender
- [ ] Replace `Slot` with `useRender` in `BreadcrumbLink` component
- [ ] `BreadcrumbLink` must accept `render` prop for `<a>` wrapping
- [ ] Verify breadcrumb with link renders correctly

## Task 4 — Migrate form.tsx to Base UI Field.* pattern
- [ ] Import `Field` from `@base-ui/react/field`
- [ ] Replace `FormItem` → `Field.Root`
- [ ] Replace `FormLabel` → `Field.Label`
- [ ] Replace `FormControl` (Slot-based) → `Field.Control render={...}`
- [ ] Replace `FormMessage` → `Field.Error`
- [ ] Replace `FormDescription` → `Field.Description` or a styled `<p>`
- [ ] Keep `FormField` (react-hook-form Controller wrapper) — it does not use Radix
- [ ] Verify re-exports match what admin pages expect: `{ Form, FormField, FormItem, FormLabel, FormControl, FormDescription, FormMessage }`

## Task 5 — Migrate app-level asChild usages
For each of the 9 app-level files:
- [ ] `model-selector.tsx`: Replace `<Button asChild><...>` → `<Button render={<.../>}>`
- [ ] `agent-selector.tsx`: Same pattern
- [ ] `capability-toggles.tsx`: Same pattern
- [ ] `agent-editor.tsx`: Same pattern
- [ ] `settings-page.tsx`: Same pattern
- [ ] `tooltip-icon-button.tsx`: Same pattern
- [ ] `enhanced-thread.tsx`: Same pattern
- [ ] `top-nav.tsx`: Same pattern
- [ ] `left-sidebar.tsx`: Same pattern

## Task 6 — Migrate ui/sidebar.tsx asChild usages
- [ ] `sidebar.tsx` is the largest file — process all `asChild` usages systematically
- [ ] Verify sidebar open/close, sheet animation, and collapsible behavior still works
- [ ] Check mobile sidebar behavior

## Task 7 — Migrate ui/select.tsx asChild usages
- [ ] Process any remaining `asChild` in select trigger or content
- [ ] Verify select opens/closes correctly and keyboard nav works

## Task 8 — Full grep audit
- [ ] Run: `grep -r "asChild" frontend/src/` — must return zero
- [ ] Run: `grep -r "@radix-ui/react-slot" frontend/src/` — must return zero
- [ ] Run: `grep -r "from 'radix-ui'" frontend/src/` — must return zero

## Task 9 — TypeScript + lint
- [ ] Run: `pnpm typecheck` — zero errors
- [ ] Run: `pnpm lint` — zero errors

## Task 10 — Manual smoke test: forms
- [ ] Open admin settings page — form renders correctly
- [ ] Submit a form with validation errors — error messages appear via Field.Error
- [ ] Submit valid form — succeeds

## Completion Gate
All tasks checked. Zero asChild/Slot/radix-slot in own code. Forms work. Commit: `feat: migrate asChild to render prop and form.tsx to Base UI Field pattern`
