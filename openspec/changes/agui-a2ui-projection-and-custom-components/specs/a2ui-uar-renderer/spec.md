## MODIFIED Requirements

### Requirement: The catalog implements exactly the 9 `uar.a2ui/1` protocol-standard components

The renderer SHALL provide a `Catalog<UarComponentImplementation>`
(`uarBasicCatalog`, id `urn:uar:a2ui:catalog:1`) containing exactly the 9
components named in `docs/protocols/a2ui-profile.md`: `Text`, `Button`,
`TextField`, `CheckBox`, `ChoicePicker`, `Row`, `Column`, `Card`,
`Divider`. Each component's `ComponentApi` schema SHALL be the same Zod
schema `@a2ui/web_core`'s `basic_catalog` defines for that component name
(imported via `@prometheus-ags/a2ui-core/v0_9/basic_catalog`), so wire
payloads valid against the upstream catalog are valid against this one.
This certified baseline SHALL NOT gain components; host components are
available only through extension catalogs a surface explicitly declares, and
template components never reach the renderer because the server expands them.

#### Scenario: A server sends a message using an unapproved component
- **WHEN** an `updateComponents` message for a surface in catalog
  `urn:uar:a2ui:catalog:1` references a component type not in the
  9-component catalog (e.g. `Image`, a host component, or any component
  outside the `uar.a2ui/1` profile)
- **THEN** the renderer throws `UnknownUarComponentError` when asked to
  render that component, and does not render arbitrary/unknown markup —
  failing closed per the security boundary in
  `docs/protocols/a2ui-profile.md`

#### Scenario: A developer inspects which components are approved
- **WHEN** a developer reads `frontend/packages/a2ui-uar/src/catalog/uar-basic-catalog.ts`
- **THEN** they find exactly the 9 components above, each documented
  with a citation to `docs/protocols/a2ui-profile.md` as the source of
  the approved list

## ADDED Requirements

### Requirement: Host components resolve through registered implementations per extension catalog

The React renderer SHALL resolve a surface's declared extension catalog
(`urn:uar:a2ui:catalog:1+<name>`) to the 9 certified built-ins plus the host
implementations registered for that catalog with
`registerA2uiComponent(catalogId, name, implementation)`. The existing entity
components SHALL be registered for `urn:uar:a2ui:catalog:1+entities`. When a
surface uses a component the client has no implementation for, the chat
application SHALL report the surface invalid, naming the component and
catalog, with the bounded source disclosure the chat display requirement
already defines. The Lit and Svelte renderers SHALL fail closed the same way.

#### Scenario: A host component renders its registered implementation
- **WHEN** a surface declares `urn:uar:a2ui:catalog:1+entities` and uses `EntityCard`
- **THEN** the registered EntityCard implementation renders, and its actions dispatch through `SurfaceModel.onAction`

#### Scenario: A host component without an implementation fails closed
- **WHEN** a surface uses a host component of a catalog this client has no implementation for
- **THEN** the chat application reports the surface invalid and names the component and catalog, and renders no arbitrary markup

#### Scenario: Template output renders as built-ins
- **WHEN** a surface published from a template component arrives
- **THEN** it renders through the baseline catalog with no registry lookup
