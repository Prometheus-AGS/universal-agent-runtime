## Context

The Base UI foundation regeneration already converted the local Button,
Breadcrumb, Sidebar, and Select wrappers to Base UI primitives or the current
`useRender` API. A 2026-08-07 live audit found no `asChild`, Radix Slot, or
`radix-ui` imports in those wrappers. The proposal's original 44-use census and
three Slot imports therefore describe the pre-regeneration tree rather than the
implementation surface for this change.

Ten `asChild` uses remain in `components/assistant-ui/enhanced-thread.tsx`.
Installed `@assistant-ui/react` 0.14.26 exposes a `render` prop for every affected
primitive and implements it as the same child-composition path, preserving merged
props, refs, handlers, and children. The local form wrapper has no production
consumer, but it still implements its own label, description, invalid-state, and
error wiring instead of the selected Base UI Field foundation.

## Goals / Non-Goals

**Goals:**

- Replace every remaining application-owned `asChild` call with the installed
  primitive's `render` API without adding DOM wrappers or changing visuals.
- Rebuild the stable React Hook Form wrapper exports on Base UI Field parts.
- Preserve semantic button behavior, merged event handlers, forwarded refs,
  disabled state, label/control association, descriptions, and external errors.
- Reconcile the authored task list with the observed post-regeneration tree.

**Non-Goals:**

- Rework assistant-ui runtime state, chat behavior, or visual styling.
- Regenerate or restyle Button, Breadcrumb, Sidebar, or Select again.
- Migrate admin forms that do not currently consume the local form wrapper.
- Remove transitive Radix internals from third-party packages; this change governs
  application-owned source and direct imports.

## Decisions

### Use each primitive's supported `render` composition API

The ten assistant-ui action primitives will receive the existing Button or
TooltipIconButton as their `render` element, with visible children retained on
the primitive. The installed implementation clones the render element with those
children and delegates prop/ref/event merging to its primitive layer.

Alternative considered: leave assistant-ui `asChild` in place because its public
documentation still presents that spelling. Rejected because the installed
version explicitly types and implements `render`, and the phase acceptance gate
requires application-owned `asChild` to reach zero.

### Treat regenerated Base UI wrappers as verified prerequisites

Button, Breadcrumb, Sidebar, and Select will be audited and covered by the final
source gates, but not rewritten. Their current implementations use Base UI
primitives, `useRender`, or Base UI `render` props and have no matching legacy
syntax.

Alternative considered: edit them only to satisfy the old checklist. Rejected
because it would create unobserved churn and risk regressing already-correct
composition behavior.

### Preserve the public form facade while delegating semantics to Field

`Form`, `FormField`, `FormItem`, `FormLabel`, `FormControl`,
`FormDescription`, `FormMessage`, and `useFormField` remain exported.
`FormItem` supplies React Hook Form's name, invalid, touched, and dirty state to
`Field.Root`; the label, control, description, and external-error wrappers render
their corresponding Field parts. `FormControl` composes its existing child
element through `Field.Control render={...}` so no extra wrapper is introduced.
As a consequence, FormItem is intentionally field-aware and must remain inside
the existing FormProvider → FormField composition; the live tree has no
standalone FormItem consumer.

Alternative considered: export Base UI Field parts directly. Rejected because it
would break the stable local wrapper contract and discard React Hook Form state.

### Prove the form contract with a focused integration test

The form facade currently has no production consumer, so the proposal's admin
settings smoke test cannot exercise it. A focused jsdom test will mount the public
facade with React Hook Form and verify label/control association, description,
external validation error rendering, and valid submission.

Alternative considered: report a manual smoke pass against an unrelated admin
form. Rejected because that would not provide evidence for this component.

## Risks / Trade-offs

- **Risk:** A composed action loses an assistant-ui handler or disabled state. →
  Keep children on the primitive, use its installed `render` prop, and cover the
  source transformation with TypeScript plus targeted rendering checks.
- **Risk:** Base UI Field and React Hook Form disagree about validity. → Supply
  `name`, `invalid`, `touched`, and `dirty` from `getFieldState`, and use
  `Field.Error match` for the external error.
- **Risk:** The form control's element type becomes narrower than the old wrapper
  div. → Preserve the public component name but intentionally type it as the Base
  UI field control it now represents; no live consumer requires the old div ref.
- **Trade-off:** FormItem now subscribes to its field state so Field.Root receives
  external validity, touched, and dirty updates. → This can re-render the field
  subtree on those transitions, which is required for the selected Field/RHF
  integration and is bounded to the individual field context.
- **Trade-off:** Third-party dependencies may still contain Radix internals. → The
  gate is scoped to direct application source imports and composition syntax.

## Migration Plan

1. Reconcile proposal, requirements, and tasks with the live census.
2. Convert the ten assistant-ui compositions to `render` elements.
3. Move the form facade onto Base UI Field while retaining exports and React Hook
   Form state.
4. Add and run the focused form integration test, then run the phase's cheap
   TypeScript, lint, boundary, source-grep, and strict OpenSpec gates.
5. Roll back by restoring the ten `asChild` wrappers and the prior form facade;
   no persisted data or external protocol changes are involved.

## Open Questions

None. The installed package APIs and live source census resolve the stale proposal
assumptions.
