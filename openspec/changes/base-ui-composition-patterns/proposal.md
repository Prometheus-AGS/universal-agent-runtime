# Change: Base UI Composition Patterns — asChild → render, Slot → useRender, form.tsx → Field.*

## Why

Base UI replaces Radix's `asChild` + `Slot` composition model with an explicit `render` prop
pattern and `useRender` hook. After the package swap and icon migration, the remaining Radix
footprint is 44 `asChild` usages across 13 files, plus direct `Slot` imports in 3 UI components,
plus the `form.tsx` which wraps Radix `Label` and `Slot` for react-hook-form integration.

This change makes all composition patterns Base UI–native.

## What Changes

### 1. asChild → render prop (9 app-level files + 4 ui files)

**Pattern:**
```tsx
// BEFORE (Radix)
<Button asChild>
  <a href="/path">Link</a>
</Button>

// AFTER (Base UI)
<Button render={<a href="/path" />}>
  Link
</Button>
```

**Files (app-level):**
- `frontend/src/components/model-selector.tsx`
- `frontend/src/features/chat/agent-selector.tsx`
- `frontend/src/features/chat/capability-toggles.tsx`
- `frontend/src/admin/components/agent-editor.tsx`
- `frontend/src/admin/pages/settings-page.tsx`
- `frontend/src/components/assistant-ui/tooltip-icon-button.tsx`
- `frontend/src/components/assistant-ui/enhanced-thread.tsx`
- `frontend/src/components/layout/top-nav.tsx`
- `frontend/src/components/layout/left-sidebar.tsx`

**Files (ui layer — if shadcn CLI regen left asChild in custom overrides):**
- `frontend/src/components/ui/button.tsx`
- `frontend/src/components/ui/breadcrumb.tsx`
- `frontend/src/components/ui/sidebar.tsx`
- `frontend/src/components/ui/select.tsx`

### 2. Slot → useRender (3 ui files)

**Pattern:**
```tsx
// BEFORE (Radix)
import { Slot } from '@radix-ui/react-slot';
function MyComp({ asChild, ...props }) {
  const Comp = asChild ? Slot : 'div';
  return <Comp {...props} />;
}

// AFTER (Base UI)
import { useRender } from '@base-ui/react/use-render';
function MyComp({ render: renderProp = <div />, ...props }) {
  const { renderElement } = useRender({ render: renderProp, props });
  return renderElement();
}
```

**Files:**
- `frontend/src/components/ui/button.tsx` — `Slot` used for `asChild` pattern
- `frontend/src/components/ui/breadcrumb.tsx` — `Slot` used for `BreadcrumbLink asChild`
- `frontend/src/components/ui/form.tsx` — `Slot` used for form control wrapping

### 3. form.tsx → Base UI Field.* pattern

**Pattern:**
```tsx
// BEFORE (Radix shadcn Form)
<FormItem>
  <FormLabel>Email</FormLabel>
  <FormControl><Input /></FormControl>
  <FormMessage />
</FormItem>

// AFTER (Base UI Field.*)
<Field.Root>
  <Field.Label>Email</Field.Label>
  <Field.Control render={<Input />} />
  <Field.Error />
</Field.Root>
```

`react-hook-form` Controller wiring and `zodResolver` are **unchanged**.

## Acceptance Criteria

- [ ] `grep -r "asChild" frontend/src/` returns zero results
- [ ] `grep -r "from '@radix-ui/react-slot'" frontend/src/` returns zero results
- [ ] `grep -r "from 'radix-ui'" frontend/src/` returns zero results
- [ ] `frontend/src/components/ui/form.tsx` uses `Field.Root`, `Field.Label`, `Field.Control`, `Field.Error`
- [ ] All forms in `frontend/src/admin/` still submit correctly with react-hook-form + Zod validation
- [ ] `pnpm typecheck` passes
- [ ] `pnpm lint` passes
