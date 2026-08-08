# A2UI UAR renderer

## Purpose

Provide UAR's own React renderer for the `uar.a2ui/1` protocol, built
directly on `@prometheus-ags/a2ui-core` (Google's `@a2ui/web_core`), using
shadcn/ui and `react-aria-components` as UAR's visual/accessibility
baseline instead of `@a2ui/react`'s own styling — so UAR product code has
a renderer it owns and can extend (with `Entity*` components, theming,
etc.) rather than depending on the upstream reference implementation.

## ADDED Requirements

### Requirement: The renderer is built on `@prometheus-ags/a2ui-core`'s framework-agnostic primitives, not reimplemented

`frontend/packages/a2ui-uar/` SHALL implement its React binding as a thin
adapter over `@prometheus-ags/a2ui-core`'s `v0_9` exports —
`GenericBinder` for prop resolution/reactivity, `MessageProcessor` for
wire-protocol handling, `Catalog`/`ComponentApi` for the component
registry, `ComponentContext`/`SurfaceModel` for tree state — rather than
reimplementing schema scraping, data binding, or message processing.

#### Scenario: A component's dynamic props are resolved
- **WHEN** a `uar.a2ui/1` component payload contains a `DynamicString`
  field bound to a data-model path (e.g. `{ path: "/greeting" }`)
- **THEN** the rendered React component receives the resolved primitive
  value (a plain `string`) via `@prometheus-ags/a2ui-core`'s
  `GenericBinder`, not a renderer-specific resolution mechanism

#### Scenario: The bound data model changes after mount
- **WHEN** `SurfaceModel.dataModel.set(path, value)` is called for a path
  a mounted component is bound to
- **THEN** that component re-renders with the new value, without a full
  surface remount, via `GenericBinder`'s subscription mechanism bridged
  into React through `useSyncExternalStore`

### Requirement: The catalog implements exactly the 9 `uar.a2ui/1` protocol-standard components

The renderer SHALL provide a `Catalog<UarComponentImplementation>`
(`uarBasicCatalog`, id `urn:uar:a2ui:catalog:1`) containing exactly the 9
components named in `docs/protocols/a2ui-profile.md`: `Text`, `Button`,
`TextField`, `CheckBox`, `ChoicePicker`, `Row`, `Column`, `Card`,
`Divider`. Each component's `ComponentApi` schema SHALL be the same Zod
schema `@a2ui/web_core`'s `basic_catalog` defines for that component name
(imported via `@prometheus-ags/a2ui-core/v0_9/basic_catalog`), so wire
payloads valid against the upstream catalog are valid against this one.

#### Scenario: A server sends a message using an unapproved component
- **WHEN** an `updateComponents` message references a component type not
  in the 9-component catalog (e.g. `Image`, or any component outside the
  `uar.a2ui/1` profile)
- **THEN** the renderer throws `UnknownUarComponentError` when asked to
  render that component, and does not render arbitrary/unknown markup —
  failing closed per the security boundary in
  `docs/protocols/a2ui-profile.md`

#### Scenario: A developer inspects which components are approved
- **WHEN** a developer reads `frontend/packages/a2ui-uar/src/catalog/uar-basic-catalog.ts`
- **THEN** they find exactly the 9 components above, each documented
  with a citation to `docs/protocols/a2ui-profile.md` as the source of
  the approved list

### Requirement: Two-way bound inputs write through `GenericBinder`'s generated setters

`TextField`, `CheckBox`, and `ChoicePicker` SHALL commit user input back
to the surface's data model exclusively via the `setXxx` setter functions
`GenericBinder` generates for `DynamicString`/`DynamicBoolean`/
`DynamicStringList` fields bound to a data-model path — not via a
renderer-owned state mechanism that bypasses the data model.

#### Scenario: A user types into a path-bound TextField
- **WHEN** a `TextField` component's `value` prop is bound to a data-model
  path (e.g. `{ path: "/name" }`) and a user types into the rendered input
- **THEN** `surface.dataModel.get("/name")` reflects the typed value
  after the input's `onChange` fires

#### Scenario: A user selects an option in a path-bound ChoicePicker
- **WHEN** a `ChoicePicker`'s `value` is bound to a data-model path and a
  user selects an option
- **THEN** the data model at that path is updated to the array of
  selected option values, and the rendered listbox's `aria-selected`
  state reflects the selection

### Requirement: The surface renderer walks structural (`ChildList`) references correctly for both wire shapes

`Row`, `Column`, and `Card` SHALL correctly render children for both
`ChildList` wire shapes: a bare array of component ids (`["a", "b"]`,
passed through unchanged by `GenericBinder`) and a templated reference
(`{ componentId, path }`, expanded by `GenericBinder` into
`{ id, basePath }[]`, one entry per data-model array item).

#### Scenario: A Row references children by a static id array
- **WHEN** a `Row`'s `children` field is `["a", "b"]`
- **THEN** both referenced components render, each looked up by id in
  the surface's `SurfaceComponentsModel`

#### Scenario: A component references children via a templated list
- **WHEN** a structural field's raw value is `{ componentId, path }`
  (the templated-list wire shape)
- **THEN** the renderer treats the resolved value as `{ id, basePath }[]`
  and passes each entry's `basePath` through when building that child, so
  each repeated instance resolves its own relative data bindings against
  its own list item

### Requirement: Button actions dispatch through `SurfaceModel.onAction`

`Button`'s `action` prop SHALL be invoked as a callable
(`GenericBinder`'s `ACTION`-resolved `() => void`) on click, and that
invocation SHALL result in the action payload being emitted on
`SurfaceModel.onAction` — the same event source any UAR code (a store,
a service) subscribes to for handling agent-directed actions.

#### Scenario: A user clicks a Button bound to an event action
- **WHEN** a `Button` component's `action` is `{ event: { name:
  "submit", context: {...} } }` and a user clicks it
- **THEN** a subscriber on `surface.onAction` receives a call with
  `name: "submit"` (and the resolved context payload)

### Requirement: A UAR-specific `EntityCard` component is implemented as a protocol-native extension

The renderer SHALL provide an `EntityCard` component (`ComponentApi` +
React implementation) served from a distinct catalog
(`uarEntityCatalog`, id `urn:uar:a2ui:catalog:1+entities`) separate from
the certified `urn:uar:a2ui:catalog:1` baseline. Its schema fields
`entityType`/`entityId`/`syncOrigin` SHALL use the same names as
`@prometheus-ags/prometheus-entity-management`'s `EntityType`/`EntityId`/
`EntitySyncMetadata.origin` (not a renderer-invented naming scheme), to
minimize field-renaming work for the future entity-component migration.

#### Scenario: An EntityCard is rendered from a full payload
- **WHEN** an `EntityCard` message includes `entityType`, `entityId`,
  `title`, `subtitle`, `syncOrigin`, `fields`, and `actions`
- **THEN** the rendered card shows the title, subtitle, a sync-origin
  badge, each field as a label/value pair, and each action as a button
  that dispatches its bound `Action` on click

#### Scenario: An EntityCard is rendered with only required fields
- **WHEN** an `EntityCard` message omits `subtitle`, `fields`, and
  `actions` (all optional in the schema)
- **THEN** the component renders without error, showing only the
  required title/entityType/entityId-derived content

### Requirement: The renderer is cross-tested against the vendored `@a2ui/react` reference for a representative component subset

At least `Text`, `Button`, `CheckBox`, and a structural `Row`/`Column`/
`Divider` tree SHALL have tests that render the same wire messages
through both this renderer and `@prometheus-ags/a2ui-react`'s `v0_9`
surface (`A2uiSurface`/`basicCatalog`), asserting semantic equivalence
(accessible roles, accessible names, rendered text content) between the
two.

#### Scenario: The same Text message renders equivalently in both renderers
- **WHEN** an identical `createSurface`/`updateComponents`/
  `updateDataModel` message sequence for a `Text` component bound to a
  data path is processed by both `@prometheus-ags/a2ui-uar`'s
  `MessageProcessor`+catalog and `@prometheus-ags/a2ui-react`'s
  `v0_9` `MessageProcessor`+`basicCatalog`
- **THEN** both renderers produce an element containing the same bound
  text content

### Requirement: A performance-measurement harness exists for the render-time budget

`frontend/packages/a2ui-uar/src/perf/measure.ts` SHALL provide
`measure`/`measureMany`/`percentile` utilities for timing render/update
callbacks with `performance.now()`, and
`frontend/packages/a2ui-uar/test/perf/` SHALL exercise them against a
representative surface for both an initial render and a streaming
data-model update, run via a dedicated `pnpm --filter
@prometheus-ags/a2ui-uar run perf` script separate from the functional
test suite.

#### Scenario: A developer runs the performance harness
- **WHEN** a developer runs `pnpm --filter @prometheus-ags/a2ui-uar run perf`
- **THEN** it measures initial-render and streaming-update durations for
  a representative surface and asserts they stay below 16ms and 8ms,
  independent of `pnpm test`'s functional
  suite

#### Scenario: CI enforces the performance budget
- **WHEN** renderer code changes in a pull request or on `main`
- **THEN** a dedicated CI job fails unless initial render is below 16ms
  and the p95 streaming update is below 8ms
