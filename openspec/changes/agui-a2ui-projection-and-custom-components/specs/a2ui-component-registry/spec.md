## ADDED Requirements

### Requirement: Operators define template components that expand to built-ins on the server

UAR SHALL let an operator define a template component: a PascalCase name that
does not collide with a built-in, any host component or another template, a
props schema, and a template built only from the 9 certified built-ins whose
values reference the template's props. Template writes SHALL require the same
authority as settings mutation. Before validation, every A2UI admission path
SHALL replace each template use with its built-ins: the template's root keeps
the use's id, inner components get ids `<useId>__<nodeId>`, and references
inside the template are rewritten to match. Published surfaces SHALL contain
only certified built-ins. Definitions SHALL persist across restart. No agent
tool or model-produced A2UI message SHALL create, change or delete a
definition.

#### Scenario: An operator defines a template component
- **WHEN** an operator creates `StatusBadge` with a `label` prop and a template of a `Card` containing a `Text` bound to `label`
- **THEN** the definition is listed and survives a UAR restart

#### Scenario: An agent uses a template component as the surface root
- **WHEN** an agent publishes a surface whose `root` component is `StatusBadge` with `label` "Deployed"
- **THEN** the published surface has a `Card` with id `root`, a `Text` with id `root__<nodeId>` showing "Deployed", valid child references, and no `StatusBadge` component

#### Scenario: A template that uses a non-built-in component is refused
- **WHEN** a template definition contains a component that is not one of the 9 certified built-ins
- **THEN** it is refused and nothing is stored

#### Scenario: A name that collides is refused
- **WHEN** a template is named `Button`, `EntityCard`, or the name of an existing template
- **THEN** it is refused and nothing is stored

#### Scenario: Deleting a template does not change published surfaces
- **WHEN** an operator deletes a template after surfaces using it were published
- **THEN** those surfaces still render, and later uses of the template are refused as unknown components

### Requirement: Hosts register host components in declared extension catalogs

A host SHALL declare host component definitions in seed files read at
startup, placing them in extension catalogs `urn:uar:a2ui:catalog:1+<name>`.
Seed files SHALL be authoritative for host catalogs: each definition carries a
version, startup SHALL replace a stored definition whose version differs, and
startup SHALL remove host definitions that no seed file declares. A seed entry
whose name collides with a built-in or an existing template SHALL be skipped
and logged. No REST endpoint SHALL create, change or delete host definitions. Each extension
catalog contains the 9 built-ins plus its definitions. The
`urn:uar:a2ui:catalog:1+entities` catalog SHALL be seeded with all 7 client
entity components. Host and template definitions SHALL be stored apart from
the generic component library, whose list, get, update and delete endpoints
SHALL NOT reach them.

#### Scenario: The entity catalog is available from the server
- **WHEN** UAR starts
- **THEN** listing `urn:uar:a2ui:catalog:1+entities` shows EntityCard, EntityDiff, EntityStream, EntityApproval, EntityToolProvider, EntityChat and EntityCopilot with their props schemas

#### Scenario: A host schema upgrade lands on restart
- **WHEN** a host ships a newer version of a seeded definition and UAR restarts
- **THEN** the stored definition is replaced with the new version

#### Scenario: A removed seed entry is removed on restart
- **WHEN** a host removes a seed file's catalog and UAR restarts
- **THEN** that catalog and its components are no longer listed and surfaces using them are refused

#### Scenario: A seed entry that collides with a template is skipped
- **WHEN** a seed file declares a host component with the name of an existing template
- **THEN** the entry is skipped with a logged error and the template is unchanged

#### Scenario: The generic library cannot reach a definition
- **WHEN** a caller lists the component library or uses its get, update or delete endpoint on a definition's id
- **THEN** the definition is not listed or found there and stays unchanged

### Requirement: Custom component props are limited to a non-executable schema subset

Template and host props schemas SHALL allow only strings, numbers, booleans,
string enums, arrays, A2UI data-model bindings, action props whose value is an
event-only action `{event:{name, context?}}`, and closed objects (explicit
properties, no additional properties) nested at most 3 levels. They SHALL
reject URL-formatted strings, open objects, deeper nesting, props whose names
start with `on`, and `functionCall` actions. Validation of component props
against a definition SHALL apply the same rules to values.

#### Scenario: Every entity component fits the subset
- **WHEN** the 7 entity definitions are seeded
- **THEN** each passes the subset check, including EntityCard's `accessibility` object and `actions[].action`, and EntityApproval's `approve` and `reject` actions

#### Scenario: A props schema with a URL field is refused
- **WHEN** a definition declares a string with `format: uri`
- **THEN** it is refused with a diagnostic naming that prop

#### Scenario: An open object is refused
- **WHEN** a definition declares an object prop without closed properties
- **THEN** it is refused

#### Scenario: A functionCall action value is refused
- **WHEN** a surface gives a host component's action prop a `functionCall` action
- **THEN** the message is rejected

### Requirement: A2UI validation accepts host components only in their surface's declared catalog

Every path that admits A2UI messages SHALL expand templates and then
validate; no path SHALL publish unvalidated A2UI messages. For the
`a2ui_render` tool and submitted messages, validation SHALL use a surface
catalog index keyed by owner, session and surface, populated from validated
`createSurface` messages: a surface found in the index uses its catalog, and a
surface not found is validated as the baseline catalog. When the owner or the
session is unknown, extension catalogs SHALL be refused. Saved surface
templates SHALL be validated against the catalog named by their own batch's
`createSurface`, or as baseline when it has none. A non-built-in component
SHALL be accepted only in a catalog that contains it, with props valid against
the definition. Presentations and `presentation_render` output SHALL remain
baseline-only. The surface test trigger SHALL validate its payload the same
way as submitted messages.

#### Scenario: An agent uses a host component in its catalog
- **WHEN** an agent publishes a surface declaring `urn:uar:a2ui:catalog:1+entities` that uses `EntityCard` with valid props
- **THEN** the surface is accepted and published

#### Scenario: A later update in a separate call keeps its catalog
- **WHEN** a later submission in the same session updates that surface with another `EntityCard`, without repeating `createSurface`
- **THEN** the catalog is taken from the index and the update is accepted

#### Scenario: A continuation updates a built-in surface from an earlier run
- **WHEN** a continuation run updates a baseline surface created by its source run, using only built-ins
- **THEN** the update is accepted as it is today

#### Scenario: A host component on an unindexed surface is refused
- **WHEN** an update uses `EntityCard` for a surface with no indexed `createSurface`
- **THEN** it is validated as baseline and rejected as an unknown component

#### Scenario: A host component in the baseline catalog is refused
- **WHEN** a surface declaring `urn:uar:a2ui:catalog:1` uses `EntityCard`
- **THEN** the message is rejected as an unknown component

#### Scenario: An unregistered component is refused
- **WHEN** an agent uses a component that is neither a built-in, a template, nor a host component of its surface's catalog
- **THEN** the message is rejected with a non-executable diagnostic

#### Scenario: Presentations reject host components
- **WHEN** a presentation is saved, or `presentation_render` outputs a surface, with an `EntityCard` component
- **THEN** it is rejected

#### Scenario: The surface test trigger cannot publish an unvalidated component
- **WHEN** the surface test trigger is called with a component that is neither a built-in nor in its surface's catalog
- **THEN** it is rejected and nothing is published to the run's replay

#### Scenario: A tool call without a session cannot use an extension catalog
- **WHEN** `a2ui_render` runs in a context with no session or no verified owner and declares `urn:uar:a2ui:catalog:1+entities`
- **THEN** the surface is rejected

### Requirement: Agents can discover registered components at call time

UAR SHALL provide a read-only `a2ui_catalog` tool that returns the current
template components and each extension catalog's host components with their
props, read from the registry when called. The `a2ui_render` tool's schema
SHALL accept extension catalog ids and extension component items generically,
so registry changes need no tool re-registration.

#### Scenario: A new template is discoverable without re-registration
- **WHEN** an operator defines a template and an agent then calls `a2ui_catalog`
- **THEN** the result names the template and its props, and an `a2ui_render` call using it is accepted
