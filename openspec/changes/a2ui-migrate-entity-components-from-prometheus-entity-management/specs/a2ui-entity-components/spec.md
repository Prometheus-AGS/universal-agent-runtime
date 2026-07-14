# A2UI entity components

## Purpose

Extend the UAR entity-component catalog (started in Change 17 with
`EntityCard`) with `EntityDiff` and `EntityStream`, demonstrating both the
bound (`GenericBinder`) and binderless (imperative subscription) component
patterns `@prometheus-ags/a2ui-uar` supports.

## ADDED Requirements

### Requirement: EntityDiff renders a before/after comparison
`EntityDiff` MUST be registered in `uarEntityCatalogComponents` under
`urn:uar:a2ui:catalog:1+entities`. It MUST render a label/before/after row
for each entry in its `fields` array, and MUST visually distinguish a row
whose resolved `before` and `after` values differ from a row where they
are equal.

#### Scenario: A field's value changed
- **WHEN** a server sends an `EntityDiff` component with a field whose
  `before` and `after` resolve to different strings
- **THEN** that row is rendered with `data-a2ui-diff-changed="true"`
- **AND** a row whose `before`/`after` resolve to the same string is
  rendered with `data-a2ui-diff-changed="false"`

### Requirement: EntityStream subscribes to a live data-model path
`EntityStream` MUST be registered via
`createBinderlessUarComponentImplementation`, reading its `source: {path}`
declaration directly from `context.componentModel.properties` (not
through `GenericBinder`'s declarative resolution) and subscribing to that
path via `context.dataContext.subscribeDynamicValue`. It MUST render the
current array of items at that path and MUST update reactively when the
data model at that path changes after mount, without requiring the
component to be re-created.

#### Scenario: Items exist at the source path on mount
- **WHEN** a surface's data model already has an array at
  `EntityStream.source.path` when the component mounts
- **THEN** every item in that array is rendered

#### Scenario: A new item is pushed to the source path after mount
- **WHEN** an `updateDataModel` message updates the array at
  `EntityStream.source.path` after the component has already mounted
- **THEN** the newly added item appears in the rendered list without a
  full component remount

#### Scenario: No items exist yet at the source path
- **WHEN** the data model has no array (or an empty array) at
  `EntityStream.source.path`
- **THEN** the component renders an empty-state message instead of an
  empty list
