## Why

Change 16 (`a2ui-vendor-google-core-react`) made Google's A2UI libraries
available as pinned dependencies (`@prometheus-ags/a2ui-core` wrapping
`@a2ui/web_core`, `@prometheus-ags/a2ui-react` wrapping `@a2ui/react` as a
reference-implementation-only package) but deliberately shipped no product
code — "nothing in `frontend/src/` imports either new package yet." UAR has
no renderer for the `uar.a2ui/1` protocol it already documents
(`docs/protocols/a2ui-profile.md`) and already exposes Rust-side
(`src/uar/a2ui/`). This change builds that renderer.

## What Changes

- Add `frontend/packages/a2ui-uar/` — a new `pnpm` workspace package,
  `@prometheus-ags/a2ui-uar`, the UAR-owned React renderer for A2UI,
  built directly on `@prometheus-ags/a2ui-core`'s `v0_9` surface
  (`GenericBinder`, `MessageProcessor`, `Catalog`, `ComponentContext`,
  `SurfaceModel`) rather than reimplementing any of that framework-agnostic
  machinery.
- Implement all 9 `uar.a2ui/1` protocol-standard components named in
  `docs/protocols/a2ui-profile.md` — Text, Button, TextField, CheckBox,
  ChoicePicker, Row, Column, Card, Divider — using shadcn/ui (this repo's
  existing design-system convention, `frontend/components.json`) as the
  visual baseline and `react-aria-components` for the one accessibility
  primitive shadcn's Base UI components don't cover (`ChoicePicker`'s
  multi-select listbox semantics).
- Implement 1 of the 7 planned UAR-specific `Entity*` components
  (`EntityCard`) as a genuinely new, A2UI-protocol-native component whose
  schema mirrors `@prometheus-ags/prometheus-entity-management`'s
  established naming (`EntityType`/`EntityId`/`EntitySyncMetadata.origin`),
  without migrating any of that package's rendering logic — that migration
  is Change 18 (`a2ui-entity-component-migration`).
- Extend `@prometheus-ags/a2ui-react` (Change 16) with a `./v0_9` export
  subpath so this change can cross-test against `@a2ui/react`'s v0.9
  surface — the package's default export is `@a2ui/react`'s v0_8 surface,
  which uses a different component set (`MultipleChoice`, not
  `ChoicePicker`) and API shape, and isn't comparable to a v0.9-targeting
  renderer.
- Add a performance-measurement harness (`src/perf/measure.ts`,
  `test/perf/`) for the phase plan's stated budget (initial render < 16ms,
  streaming chunk < 8ms), run as its own `pnpm --filter
  @prometheus-ags/a2ui-uar run perf` script — see "Out of scope" for the
  gap between this harness and an actual CI-enforced gate.

### Plan correction: "9 components from `uar.a2ui/1`" resolved against `docs/protocols/a2ui-profile.md`, not `web_core`'s full basic_catalog

The phase plan's line item ("14+ components: the 9 from `uar.a2ui/1` +
EntityCard, EntityDiff, ...") does not enumerate the 9 by name.
`@a2ui/web_core`'s own `basic_catalog` ships **18** components (Text,
Image, Icon, Video, AudioPlayer, Row, Column, List, Card, Tabs, Modal,
Divider, Button, TextField, CheckBox, ChoicePicker, Slider,
DateTimeInput), so "the 9" is ambiguous without an external source of
truth. `docs/protocols/a2ui-profile.md` (already checked into this repo,
authored for the Rust-side `A2uiRegistry`) resolves the ambiguity
explicitly: "The initial approved catalog is `Text`, `Button`,
`TextField`, `CheckBox`, `ChoicePicker`, `Row`, `Column`, `Card`, and
`Divider`." This change implements exactly that list, under catalog id
`urn:uar:a2ui:catalog:1` (matching
`.kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/analysis.md`'s
citation of the Rust-side catalog id), rather than a different
9-component subset chosen ad hoc from `web_core`'s 18.

### Architecture note: `GenericBinder`'s type-level resolution is shallower than its runtime resolution

`web_core`'s `ResolveA2uiProps<T>` type utility maps `DynamicString`/
`Action`/`ChildList` fields to their resolved runtime type
(`string`/`() => void`/child references) only **one schema level deep**.
At runtime, `GenericBinder` recursively resolves those same field kinds
when they're nested inside `OBJECT`/`ARRAY` schema fields (e.g.
`accessibility.label`, `ChoicePicker.options[].label`,
`EntityCard.actions[].action`) — the type utility just doesn't model that
recursion. This is documented and localized in
`frontend/packages/a2ui-uar/src/lib/resolved.ts` (`resolvedText`/
`resolvedAction` helpers with a comment explaining the gap) rather than
worked around with unexplained casts scattered through component code.

## Capabilities

### New Capabilities

- `a2ui-uar-renderer`: the UAR-owned React renderer for the `uar.a2ui/1`
  protocol — component catalog, structural surface renderer, two-way data
  binding, action dispatch, and the fail-closed unknown-component
  behavior required by the protocol's security boundary.

## Impact

- **New workspace package:** `frontend/packages/a2ui-uar/`
  (`@prometheus-ags/a2ui-uar`), a `pnpm` workspace member (matched by the
  existing `packages/*` glob, no config change needed).
- **Modified package:** `frontend/packages/a2ui-react/` gains a `./v0_9`
  export subpath (`package.json` `exports` + new `src/v0_9.ts` re-export
  module) — additive only, the existing default export and `./styles`
  subpath are unchanged.
- **New dependencies** (all scoped to the new package):
  `@base-ui/react` (already used elsewhere in `frontend/src/`, pinned to
  the same `1.6.0`), `react-aria-components`, `class-variance-authority`,
  `clsx`, `tailwind-merge`, `lucide-react`, `zod@^3.25.76` (matching
  `@a2ui/web_core`'s own peer, deliberately **not** the `zod@4.4.3` the
  rest of `frontend/` uses — see tasks.md 2.1 for why mixing them broke
  typecheck).
- **No changes to `frontend/src/`.** Nothing in UAR's existing app code
  imports the new renderer yet; wiring it into the actual A2UI-testing
  surface (`/admin/a2ui-testing`, `src/uar/a2ui/` on the Rust side) is
  out of scope for this change (see below).
- **Lint scope:** `frontend/eslint.config.js` already ignores
  `packages/**`; the new package has its own `eslint.config.js` (mirrors
  the root config's rule set minus `react-refresh`, which doesn't apply
  to a library package) and `lint` script, run and verified
  independently.

## Out of scope

- **Wiring the renderer into UAR product code** (`/admin/a2ui-testing`
  or any other surface). This change delivers the renderer package
  itself; integrating it into an actual UI surface is a separate,
  follow-up decision — not part of Change 17's done condition, which is
  scoped to the renderer package.
- **The remaining 9 `Image`/`Icon`/`Video`/`AudioPlayer`/`List`/`Tabs`/
  `Modal`/`Slider`/`DateTimeInput` components** from `web_core`'s
  `basic_catalog` — outside the certified `uar.a2ui/1` catalog per
  `docs/protocols/a2ui-profile.md`, regardless of remaining effort
  budget.
- **6 of 7 `Entity*` components** (`EntityDiff`, `EntityStream`,
  `EntityApproval`, `EntityToolProvider`, `EntityChat`, `EntityCopilot`).
  Each has materially different requirements from `EntityCard`
  (diffing, live-stream subscription via the binderless component
  pattern, multi-step approval flows, near-mini-application scope for
  chat/copilot) — see `frontend/packages/a2ui-uar/README.md`'s
  "Deferred" section for the per-component reasoning.
- **A full cross-testing matrix** against `@a2ui/react` covering all 9
  protocol components and every prop variant. This change cross-tests a
  representative subset (Text, Button, CheckBox, and a Row/Column/Divider
  structural tree) — real, executable tests, not a stub, but not
  exhaustive.
- **A CI-enforced performance gate.** This change delivers the
  measurement harness and a real, currently-passing regression-style
  test suite; wiring a dedicated CI job with environment-calibrated
  budgets and a trend baseline is documented as follow-up work in
  `frontend/packages/a2ui-uar/README.md`'s "Performance budget" section,
  not attempted here.
- **Theming** (`createSurface`'s `theme` payload). Accepted by
  `web_core`/`MessageProcessor` already; not yet threaded into any
  component in this package. Change 21 (`a2ui-theming` per the phase
  plan) owns making it apply.
- **Change 18** (`a2ui-entity-component-migration`): migrating
  `prometheus-entity-management`'s existing rendering logic into
  `Entity*` components. This change only aligns `EntityCard`'s schema
  naming with that package's conventions so Change 18 has less renaming
  to do later — it does not perform that migration.
