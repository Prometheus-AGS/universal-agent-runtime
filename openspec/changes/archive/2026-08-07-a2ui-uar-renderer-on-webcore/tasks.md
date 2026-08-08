## 1. Audit `@a2ui/web_core`'s actual API surface
- [x] 1.1 Read `@a2ui/web_core`'s `v0_9` exports (`node_modules/.pnpm/@a2ui+web_core@0.10.4/.../src/v0_9/index.d.ts`) to determine the real renderer-building architecture: `Catalog`, `MessageProcessor`, `GenericBinder`, `ComponentContext`, `SurfaceModel`/`SurfaceGroupModel`/`SurfaceComponentsModel`/`ComponentModel`, `DataContext`. Result: framework-agnostic core; a renderer is a thin per-framework adapter over `GenericBinder` plus visual components, not a from-scratch implementation.
- [x] 1.2 Read `@a2ui/react`'s `v0_9` `.d.ts` (`createComponentImplementation`, `A2uiSurface`, `DeferredChild`, `basicCatalog`) to confirm the reference shape a "renderer on web_core" takes. Result: `ReactComponentImplementation = { name, schema, render: FC<{context, buildChild}> }`, built via `createComponentImplementation(api, RenderComponent)`.
- [x] 1.3 Read `docs/protocols/a2ui-profile.md` to resolve which 9 components `uar.a2ui/1` actually approves (the plan doesn't name them, and `web_core`'s own basic_catalog has 18). Result: Text, Button, TextField, CheckBox, ChoicePicker, Row, Column, Card, Divider.
- [x] 1.4 Read each of those 9 components' Zod schemas in `basic_components.js` (compiled JS is more readable than the expanded `.d.ts`) to get exact prop shapes, defaults, and behavior hints (`DynamicString`, `Action`, `ChildList`, `CheckableSchema`).
- [x] 1.5 Read `generic-binder.d.ts`/`.js` to understand `BehaviorNode` scraping (`DYNAMIC`/`ACTION`/`STRUCTURAL`/`CHECKABLE`/`STATIC`/`OBJECT`/`ARRAY`) and confirm (via a failing test, see 6.3) that `STRUCTURAL` resolution differs for a static id array vs. a templated `{componentId, path}` list.
- [x] 1.6 Confirm `shadcn/ui` is already set up in this frontend (`frontend/components.json`, `src/components/ui/`) and read a handful of existing primitives (`button.tsx`, `checkbox.tsx`, `input.tsx`, `card.tsx`, `separator.tsx`) to match conventions (Base UI primitives + `class-variance-authority` + `cn()`).
- [x] 1.7 Read `frontend/packages/prometheus-entity-management/src/graph.ts` for its `EntityType`/`EntityId`/`EntitySyncMetadata` naming, to align `EntityCard`'s schema with it per the task's Change-18-smoothing instruction.

## 2. `frontend/packages/a2ui-uar/` package skeleton
- [x] 2.1 `package.json` — name `@prometheus-ags/a2ui-uar`, `@prometheus-ags/a2ui-core` workspace dependency, `@base-ui/react`/`react-aria-components`/`class-variance-authority`/`clsx`/`tailwind-merge`/`lucide-react` deps, `zod@^3.25.76` (matching `@a2ui/web_core`'s own peer — pinning `zod@^4` as the rest of `frontend/` uses broke typecheck with duplicate/incompatible `ZodTypeAny` types; documented in package.json's description and README).
- [x] 2.2 `tsconfig.json` — standalone strict config (JSX enabled), matching `a2ui-core`/`a2ui-react`'s pattern.
- [x] 2.3 `vitest.config.ts` (functional tests) and `vitest.perf.config.ts` (perf harness, separate script/config so a perf regression can fail independently of functional tests) + `test/setup.ts`.
- [x] 2.4 `eslint.config.js` — own lint lifecycle (root config ignores `packages/**`), mirrors root rules minus `react-refresh`.

## 3. React adapter over `GenericBinder`
- [x] 3.1 `src/react/use-a2ui-props.ts` — `useA2uiProps` hook: creates one `GenericBinder` per `(context, schema)` identity via `useMemo`, disposes on teardown, bridges `binder.subscribe`/`binder.snapshot` into React via `useSyncExternalStore`.
- [x] 3.2 `src/react/create-component.tsx` — `createUarComponentImplementation`/`createBinderlessUarComponentImplementation`, the UAR-owned equivalent of `@a2ui/react`'s `createComponentImplementation`/`createBinderlessComponentImplementation`.
- [x] 3.3 `src/react/UarSurface.tsx` — `UarSurface`/`UarDeferredChild`: recursive surface renderer over `SurfaceModel`/`SurfaceComponentsModel`, `getRootComponentId` (documented `"root"`-id convention per a2ui.org, since `ComponentModel` carries no explicit root flag), `UnknownUarComponentError` (fail-closed per the protocol's security boundary in `docs/protocols/a2ui-profile.md`).
- [x] 3.4 `src/lib/resolved.ts` — `resolvedText`/`resolvedAction`, documenting the `ResolveA2uiProps` one-level-deep type gap (see proposal.md).
- [x] 3.5 `src/lib/child-refs.ts` — `resolveChildRefs`, normalizing `ChildList`'s two runtime shapes (bare string-id array vs. resolved `{id, basePath}[]` template expansion) — found via a failing `Row` test (task 6.3), not assumed upfront.

## 4. The 9 `uar.a2ui/1` protocol components
- [x] 4.1 `src/components/ui/{button,input,checkbox,card,separator,label}.tsx` — vendored subset of `frontend/src/components/ui/*` (this package is self-contained, matching `a2ui-core`/`a2ui-react`'s own-lifecycle convention; no cross-package `@/` alias reach into `frontend/src`).
- [x] 4.2 `Text` — typographic primitive, variant→tag/class map, Markdown-free per the protocol.
- [x] 4.3 `Row`/`Column` — structural layout via `resolveChildRefs`+`buildChild`, `justify`/`align`/`weight` mapped to Tailwind classes.
- [x] 4.4 `Card` — single required `child`.
- [x] 4.5 `Divider` — `axis` → `Separator` orientation.
- [x] 4.6 `Button` — `variant`→shadcn button variant, `action()` dispatch on click, `isValid`/`validationErrors` (from `CheckableSchema`) surfaced via `aria-invalid` (advisory, not a client-side hard gate — the agent/server remains the action's source of truth).
- [x] 4.7 `TextField` — two-way bound via the generated `setValue` setter; `variant`→HTML input type/textarea; validation errors rendered as an alert.
- [x] 4.8 `CheckBox` — two-way bound boolean via `setValue`.
- [x] 4.9 `ChoicePicker` — `react-aria-components` `ListBox`/`ListBoxItem` (not shadcn's Radix/Base-UI `Select`, which is single-value only) for real multi-select listbox semantics per `variant: multipleSelection`; two-way bound via `setValue`.
- [x] 4.10 `src/catalog/uar-basic-catalog.ts` — assembles the 9 into a `Catalog` under id `urn:uar:a2ui:catalog:1`.

## 5. Complete `Entity*` catalog
- [x] 5.1 `src/entities/entity-card-api.ts` — Zod schema aligned with `prometheus-entity-management` naming (`entityType`, `entityId`, `syncOrigin` mirroring `EntitySyncMetadata.origin`), `fields`/`actions` arrays.
- [x] 5.2 `src/entities/EntityCard.tsx` — render implementation: title/subtitle, sync-origin badge, field list, action buttons dispatching bound `Action`s.
- [x] 5.3 `src/catalog/uar-entity-catalog.ts` — separate catalog id `urn:uar:a2ui:catalog:1+entities` (extension catalog, not merged into the certified baseline catalog).
- [x] 5.4 Implement `EntityDiff`, `EntityStream`, `EntityApproval`, `EntityToolProvider`, `EntityChat`, and `EntityCopilot` as protocol-native schemas and renderers with explicit state and recovery semantics.
- [x] 5.5 Assemble all 7 entity components into the extension catalog (16 total components including the baseline).

## 6. Tests
- [x] 6.1 `test/helpers.ts` — `buildSurface`: `MessageProcessor` + `createSurface`/`updateComponents`/`updateDataModel` message sequence, matching real wire traffic.
- [x] 6.2 `test/surface.test.tsx` — Text data binding + reactive re-render (`act()`-wrapped `dataModel.set`), Row/Column/Card/Divider structural tree, Button action dispatch via `surface.onAction`, `UnknownUarComponentError` fail-closed behavior.
- [x] 6.3 `test/inputs.test.tsx` — TextField/CheckBox/ChoicePicker two-way binding via `userEvent`, asserting the underlying `dataModel` value changed (not just DOM state). Found and fixed two real bugs while writing these: `ChildList`'s static-array shape (task 3.5) and a `getByLabelText` test-only ambiguity (Base UI's hidden native `<input>` also matches the label).
- [x] 6.4 `test/entity-card.test.tsx` — `EntityCard` full-field render + action dispatch, and a bare-minimum (no fields/actions/subtitle) render without crashing.
- [x] 6.5 `test/cross-reference.test.tsx` — cross-tests against `@prometheus-ags/a2ui-react/v0_9` (the vendored Google reference) for Text, Button, CheckBox, and a Row/Column/Divider structural tree, asserting semantic equivalence (roles/accessible names/text content), not DOM/CSS equality.
- [x] 6.6 `test/perf/render-budget.test.tsx` — exercises `src/perf/measure.ts` against a moderately complex surface for initial render and a streaming `dataModel.set()` update, using p95 over repeated runs.
- [x] 6.7 Extend `@prometheus-ags/a2ui-react`'s `package.json` `exports` with `./v0_9` + `src/v0_9.ts`, required for 6.5 (the package previously only exposed the non-comparable v0_8 surface). Verified `pnpm --filter @prometheus-ags/a2ui-react typecheck` still passes after the addition.
- [x] 6.8 Assert the catalog contains the 9 baseline and all 7 entity components.

## 7. Verification
- [x] 7.1 `pnpm -C frontend install` — pass (after re-confirming the `prometheus-entity-management` submodule init/build precondition Change 16 already documented hitting).
- [x] 7.2 `pnpm --filter @prometheus-ags/a2ui-uar typecheck` — pass, 0 errors.
- [x] 7.3 `pnpm --filter @prometheus-ags/a2ui-uar lint` — pass, 0 errors/warnings.
- [x] 7.4 `pnpm --filter @prometheus-ags/a2ui-uar test` — pass, 17/17 tests (5 files).
- [x] 7.5 `pnpm --filter @prometheus-ags/a2ui-uar run perf` — literal <16ms initial and <8ms streaming budgets, enforced by `.github/workflows/a2ui-renderer-performance.yml`.
- [x] 7.6 `pnpm --filter @prometheus-ags/a2ui-react typecheck` — pass (confirms the `./v0_9` addition didn't regress Change 16's package).
- [x] 7.7 `pnpm -C frontend lint` (root, full workspace) — pass (confirms `packages/**` ignore still holds; the new package doesn't leak into root lint scope).
- [x] 7.8 `openspec validate a2ui-uar-renderer-on-webcore --strict` — pass.

## 8. Operator follow-up
- [x] 8.1 Reviewed: retain `zod@^3.25.76` for upstream `web_core` type compatibility and retain the separate `+entities` catalog so the certified baseline remains stable.
- [x] 8.2 Merge this complete renderer package now; actual product-surface wiring remains separately scoped.
