# `@prometheus-ags/a2ui-uar` — the UAR-owned A2UI renderer

This is the renderer UAR product code should import. It is built directly
on [`@prometheus-ags/a2ui-core`](../a2ui-core) (the vendored
`@a2ui/web_core`), using shadcn/ui as the visual baseline and
`react-aria-components` for accessibility primitives shadcn's Base UI
primitives don't cover. `@prometheus-ags/a2ui-react` (Google's official
React renderer, vendored in `../a2ui-react`) is a **reference
implementation only** — this package cross-tests against it but nothing
in UAR product code should import it.

## Architecture: what "built on `web_core`" actually means

`@a2ui/web_core` is **framework-agnostic core**, not a component library.
Reading its actual exported API (`@a2ui/web_core/v0_9`) rather than
guessing:

- **`Catalog<ComponentApi>`** — a named collection of `{ name, schema
  (Zod) }` component definitions. A `ComponentApi`'s schema is the single
  source of truth for both wire validation and (via `zod-to-json-schema`)
  the client-capabilities payload a server sees.
- **`MessageProcessor`** — the wire-protocol engine. Feed it
  `createSurface` / `updateComponents` / `updateDataModel` /
  `deleteSurface` messages (the exact 4 message types
  `docs/protocols/a2ui-profile.md` documents for `uar.a2ui/1`) and it
  maintains a `SurfaceGroupModel` → `SurfaceModel` → `SurfaceComponentsModel`
  → `ComponentModel` tree, plus a `DataModel` per surface.
- **`GenericBinder`** — scrapes a component's Zod schema into a
  `BehaviorNode` tree (`DYNAMIC` / `ACTION` / `STRUCTURAL` / `CHECKABLE` /
  `STATIC` / `OBJECT` / `ARRAY`) and turns a raw JSON component payload
  into fully-resolved, reactive props: `DynamicString`/`DynamicBoolean`/…
  fields become subscribed primitives (plus a generated `setXxx` two-way
  setter), `Action` fields become `() => void` callables, `ChildList`
  fields become renderable child references. **This is the framework
  boundary** — everything above it is framework-agnostic; a renderer for
  any framework (React, Lit, Svelte, ...) is "just" a thin adapter over
  `GenericBinder` plus a per-component visual implementation.

`@a2ui/react` (Google's reference) is exactly that: a React adapter
(`createComponentImplementation`) plus its own `basic_catalog` visual
components. This package is the same shape — `src/react/create-component.tsx`
is our `createComponentImplementation` equivalent, `src/react/UarSurface.tsx`
is our `A2uiSurface`/`DeferredChild` equivalent — but the visual layer is
shadcn/ui + `react-aria-components` instead of `@a2ui/react`'s own styles,
because that's UAR's established design system (see
`frontend/components.json`).

### One real gap found in `web_core`'s type utilities

`ResolveA2uiProps<T>` (the type-level prop resolver) only maps **one
level deep**. At runtime, `GenericBinder` recursively resolves `DYNAMIC`/
`ACTION` fields nested inside `OBJECT`/`ARRAY` schema fields (e.g.
`accessibility.label`, `ChoicePicker.options[].label`,
`EntityCard.actions[].action`) — but the TypeScript type utility doesn't
model that recursion, so nested fields keep their raw
`DynamicString | DataBinding | FunctionCall` union type even though the
runtime value is always the resolved primitive/callable. `src/lib/resolved.ts`
documents and localizes this one-line gap (`resolvedText`/`resolvedAction`)
rather than sprinkling unexplained casts through every component.

## The `uar.a2ui/1` catalog: which 9 components, and why

`docs/protocols/a2ui-profile.md` names the UAR-approved catalog
explicitly: **Text, Button, TextField, CheckBox, ChoicePicker, Row,
Column, Card, Divider** — a curated subset of `web_core`'s 18-component
`basic_catalog` (it excludes `Image`, `Icon`, `Video`, `AudioPlayer`,
`List`, `Tabs`, `Modal`, `Slider`, `DateTimeInput` until their
URL/content/privacy/accessibility policies are separately certified).
`src/catalog/uar-basic-catalog.ts` builds exactly that 9-component
catalog under catalog id `urn:uar:a2ui:catalog:1`, matching the id the
Rust-side `A2uiRegistry` (`src/uar/a2ui/`) advertises.

`docs/protocols/a2ui-profile.md`'s "root" convention (the entry-point
component id) isn't encoded in `web_core`'s types — `ComponentModel`
carries no "this is the root" flag. `src/react/UarSurface.tsx`'s
`getRootComponentId` follows the a2ui.org convention of id `"root"`, with
a fallback to the surface's first component so a malformed/legacy payload
still renders instead of going blank. This is a documented judgment call,
not something `web_core` specifies.

## `Entity*` components

`EntityCard` (`src/entities/`) is implemented as a proof of the pattern:
a genuinely new, A2UI-protocol-native component (Zod schema + React
render function, same shape as the 9 protocol components), served from a
**separate** catalog (`uarEntityCatalog`, id
`urn:uar:a2ui:catalog:1+entities`) since `Entity*` components are a UAR
extension, not part of the certified `uar.a2ui/1` baseline.

Its schema deliberately mirrors `@prometheus-ags/prometheus-entity-management`'s
established naming (`frontend/packages/prometheus-entity-management/src/graph.ts`):
`entityType`/`entityId` (matching `EntityType`/`EntityId`), and a
`syncOrigin` field mirroring `EntitySyncMetadata.origin`
(`"server" | "client" | "optimistic"`). This is **not** a migration of
that package's rendering logic — that's Change 18
(`a2ui-entity-component-migration`)'s job. The point of aligning naming
now is that Change 18 can re-home rendering logic into components like
this one without a field-renaming exercise on top of everything else it
has to do.

### Deferred: `EntityDiff`, `EntityStream`, `EntityApproval`, `EntityToolProvider`, `EntityChat`, `EntityCopilot`

Not implemented in this pass. Each has materially different requirements
from `EntityCard` (`EntityDiff` needs a diffing data shape and probably
two data-model snapshots to compare; `EntityStream` needs the
binderless/imperative pattern — see `createBinderlessUarComponentImplementation`
— to subscribe to a live stream rather than a single bound value;
`EntityApproval`/`EntityChat`/`EntityCopilot` are closer to full
mini-applications than single components). Building all 7 well, plus a
full cross-testing matrix for each, is out of scope for a single pass —
see "Scope of this change" below.

## Scope of this change (Change 17, `a2ui-uar-renderer-on-webcore`)

The phase plan estimates this at ~40 hours; this pass delivers a real,
tested vertical slice, not a complete 14+ component catalog:

**Done:**
- Package skeleton correctly wired to `@prometheus-ags/a2ui-core`
  (`GenericBinder`, `MessageProcessor`, `Catalog`, `ComponentContext`).
- All 9 `uar.a2ui/1` protocol-standard components: Text, Button,
  TextField, CheckBox, ChoicePicker, Row, Column, Card, Divider —
  including the two-way-bound inputs (TextField/CheckBox/ChoicePicker via
  `GenericBinder`'s generated setters) and action dispatch (Button).
- 1 of 7 `Entity*` components (`EntityCard`), demonstrating the pattern
  and the naming alignment with `prometheus-entity-management`.
- `UarSurface`/`UarDeferredChild`: the full recursive surface renderer,
  including structural (`ChildList`) traversal, reactive re-render on
  data-model changes, and fail-closed behavior for unknown component
  types (`UnknownUarComponentError`) per the security boundary in
  `docs/protocols/a2ui-profile.md`.
- Cross-testing against `@prometheus-ags/a2ui-react` (the vendored
  Google reference) for a representative subset: Text, Button, CheckBox,
  and a Row/Column/Divider structural tree (`test/cross-reference.test.tsx`).
  This required adding a `./v0_9` export to `@prometheus-ags/a2ui-react`
  (it previously only exposed `@a2ui/react`'s v0_8 surface, which uses a
  different component set and API shape) — a small, justified extension
  to Change 16's package, not a full re-scope of it.
- A performance-measurement harness (`src/perf/measure.ts`) and a real
  test suite exercising it (`test/perf/`) — see "Performance budget"
  below for the gap between this and a CI-enforced gate.
- 16 passing tests total (`pnpm test`), zero ESLint warnings/errors
  (`pnpm lint`), zero TypeScript errors (`pnpm typecheck`).

**Deferred (tracked, not silently dropped):**
- `Image`, `Icon`, `Video`, `AudioPlayer`, `List`, `Tabs`, `Modal`,
  `Slider`, `DateTimeInput` — outside the certified `uar.a2ui/1` catalog
  per `docs/protocols/a2ui-profile.md`; not part of this change's scope
  regardless of remaining budget.
- `EntityDiff`, `EntityStream`, `EntityApproval`, `EntityToolProvider`,
  `EntityChat`, `EntityCopilot` — see above.
- Full cross-testing matrix (all 9 protocol components × every prop
  variant) against `@a2ui/react` — only a representative subset is
  covered.
- A real CI-enforced performance gate (see below).
- Theming (`theme` payload on `createSurface`) is accepted by
  `web_core`/`MessageProcessor` but not yet threaded into any of this
  package's components — every component currently uses UAR's static
  Tailwind design tokens. Change 21 (`a2ui-theming`, per the phase plan)
  owns making surface-level theming actually apply.

## Performance budget

Change 17's stated budget: **initial render < 16ms**, **streaming chunk
< 8ms**. `src/perf/measure.ts` is the measurement primitive
(`performance.now()` around a synchronous render/update), and
`test/perf/render-budget.test.tsx` exercises it against a moderately
complex surface (a `Column` of `Row`s containing `Card`, `TextField`,
`CheckBox`, `Text`) for both an initial mount and a streaming
`dataModel.set()` update, using `percentile()` for p95 over repeated
runs rather than a single noisy sample.

**What this is not yet:** a CI-enforced gate. Concretely, turning this
into one needs:

1. **A dedicated CI job** running `pnpm --filter @prometheus-ags/a2ui-uar
   run perf` (already wired as its own script/`vitest.perf.config.ts`,
   separate from `pnpm test`, specifically so a perf regression can fail
   independently without being bundled into functional-test noise).
2. **Environment-realistic budgets.** The current test asserts against
   `CI_INITIAL_RENDER_BUDGET_MS`/`CI_STREAMING_UPDATE_BUDGET_MS`
   (200ms/100ms), not the literal 16ms/8ms product budget — `happy-dom` +
   a cold JS engine in a CI runner is measurably slower than the
   Chromium frame the 16ms/8ms figures describe. A real gate needs either
   (a) a headless-Chromium-based benchmark (Playwright + the CDP
   `Performance` domain, or `web-vitals`-style real-frame timing) to
   assert the literal budget, or (b) an explicitly documented,
   calibrated CI-environment multiplier with a baseline captured once and
   tracked over time (regression-relative, not absolute).
3. **A trend baseline**, so "did this PR regress perf" is answered by
   comparing against the last N runs, not a single hard threshold that's
   either too loose to catch real regressions or too tight to survive
   CI noise.

None of that CI wiring exists yet — this pass delivers the harness and a
working, currently-passing regression-style check, explicitly scoped
short of the "real CI gate" infrastructure work.

## Verification run in this pass

```bash
pnpm -C frontend install
pnpm --filter @prometheus-ags/a2ui-uar typecheck   # 0 errors
pnpm --filter @prometheus-ags/a2ui-uar lint         # 0 errors/warnings
pnpm --filter @prometheus-ags/a2ui-uar test         # 16 passed
pnpm --filter @prometheus-ags/a2ui-uar run perf     # 2 passed (measurement harness, not a CI gate — see above)
```
