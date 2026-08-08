# Change: Base UI Composition Patterns — render elements and Field semantics

## Why

The Base UI foundation regeneration already moved Button, Breadcrumb, Sidebar,
and Select onto Base UI-native composition. A live 2026-08-07 audit found ten
remaining `asChild` calls, all in the assistant-ui enhanced thread, plus a legacy
React Hook Form facade that still owns label/error semantics instead of delegating
them to Base UI Field.

This change finishes the application-owned composition migration against the
post-regeneration tree without rewriting already-correct wrappers.

## What Changes

### 1. assistant-ui `asChild` calls use supported render elements

Installed `@assistant-ui/react` 0.14.26 types and implements a `render` prop for
the affected ScrollToBottom, Composer, ActionBar, and BranchPicker primitives.
The ten call sites in
`frontend/src/components/assistant-ui/enhanced-thread.tsx` will compose the same
Button or TooltipIconButton elements through that API.

### 2. The stable form facade delegates to Base UI Field

`frontend/src/components/ui/form.tsx` keeps its React Hook Form Controller wiring
and existing exports while implementing FormItem, FormLabel, FormControl,
FormDescription, and FormMessage with `Field.Root`, `Field.Label`,
`Field.Control`, `Field.Description`, and `Field.Error`.

### 3. Regenerated wrappers are verified, not regenerated again

Button, Breadcrumb, Sidebar, and Select already use Base UI primitives,
`useRender`, or Base UI render elements and contain no legacy composition syntax.
They remain in the source audit and behavior verification but require no source
rewrite in this change.

## Capabilities

### Modified Capabilities

- `frontend-component-primitives`: require application-owned render-element
  composition, Base UI Field-backed form semantics, and stable regenerated local
  wrappers.

## Impact

- Modified runtime source is limited to the enhanced assistant thread and the
  local form facade.
- A focused form integration test replaces the obsolete admin-settings smoke
  instruction because no production page currently consumes this facade.
- No dependency, persistence, service, store, hook, API, or visual-token change is
  introduced.

## Acceptance Criteria

- [ ] `frontend/src` contains no `asChild`, `@radix-ui/react-slot`, or direct
  `radix-ui` import syntax.
- [ ] `form.tsx` uses the corresponding Base UI Field parts while preserving its
  existing public exports and React Hook Form Controller wiring.
- [ ] Focused tests prove label/control association, description exposure,
  external validation errors, and valid submission.
- [ ] Frontend typecheck, lint, architecture boundaries, composition source gates,
  and strict OpenSpec validation pass.
