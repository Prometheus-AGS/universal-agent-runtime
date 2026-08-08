# Tasks: base-ui-composition-patterns

## Task 1 — Audit remaining asChild usages post-regen
- [x] Run the application-source `asChild` census and record the full live list
- [x] Categorize the results: zero UI-wrapper uses and ten app-level uses in `enhanced-thread.tsx`
- [x] Confirm the foundation regeneration already removed stale composition from the UI wrappers

## Task 2 — Verify regenerated button composition
- [x] Confirm `button.tsx` imports the Base UI Button primitive rather than Radix Slot
- [x] Confirm Button exposes Base UI's `render` prop through `ButtonPrimitive.Props`
- [x] Confirm Button has no internal `asChild`/Slot branch
- [x] Verify the installed Base UI Button contract supports native and composed render elements

## Task 3 — Verify regenerated breadcrumb composition
- [x] Confirm `BreadcrumbLink` uses the current `useRender` return-element API
- [x] Confirm `BreadcrumbLink` accepts a `render` prop for link composition
- [x] Verify the wrapper retains anchor semantics and merged class names

## Task 4 — Migrate form.tsx to Base UI Field.* pattern
- [x] Import `Field` from `@base-ui/react/field`
- [x] Implement `FormItem` with `Field.Root` and external React Hook Form state
- [x] Implement `FormLabel` with `Field.Label`
- [x] Implement `FormControl` with `Field.Control render={...}`
- [x] Implement `FormMessage` with `Field.Error`
- [x] Implement `FormDescription` with `Field.Description`
- [x] Keep the React Hook Form Controller wiring in `FormField`
- [x] Preserve `{ Form, FormField, FormItem, FormLabel, FormControl, FormDescription, FormMessage }` exports

## Task 5 — Migrate app-level asChild usages
- [x] Confirm `model-selector.tsx` has no remaining `asChild`
- [x] Confirm `agent-selector.tsx` has no remaining `asChild`
- [x] Confirm `capability-toggles.tsx` has no remaining `asChild`
- [x] Confirm `agent-editor.tsx` has no remaining `asChild`
- [x] Confirm `settings-page.tsx` has no remaining `asChild`
- [x] Confirm `tooltip-icon-button.tsx` already uses Base UI TooltipTrigger `render`
- [x] Replace all ten `enhanced-thread.tsx` `asChild` calls with supported render elements
- [x] Confirm `top-nav.tsx` has no remaining `asChild`
- [x] Confirm `left-sidebar.tsx` has no remaining `asChild`

## Task 6 — Verify ui/sidebar.tsx composition
- [x] Confirm every composable Sidebar part uses the current `useRender` API
- [x] Verify desktop sidebar open/close and collapsible behavior remains intact
- [x] Verify mobile sidebar behavior remains intact

## Task 7 — Verify ui/select.tsx composition
- [x] Confirm Select uses Base UI render elements and has no `asChild`
- [x] Verify Select open/close and keyboard navigation remains intact

## Task 8 — Full source audit
- [x] Verify `asChild` is absent from `frontend/src`
- [x] Verify `@radix-ui/react-slot` is absent from `frontend/src`
- [x] Verify direct `radix-ui` imports are absent from `frontend/src`

## Task 9 — Cheap validation gates
- [x] Run frontend typecheck with zero errors
- [x] Run frontend lint with zero errors

## Task 10 — Focused form integration proof
- [x] Render the public facade and verify label/control plus description semantics
- [x] Submit invalid data and verify the React Hook Form error appears through Field.Error
- [x] Submit valid data and verify the Controller value reaches the submit handler

## Completion Gate
All 40 tasks checked. Application source has zero asChild/Slot/direct-radix syntax,
the form facade is Base UI Field-backed, and scoped interaction evidence passes.
