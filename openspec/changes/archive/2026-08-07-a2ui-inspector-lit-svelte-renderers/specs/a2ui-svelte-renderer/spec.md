## ADDED Requirements

### Requirement: Svelte renderer uses web_core state
The Svelte renderer SHALL render UAR A2UI surfaces from `@prometheus-ags/a2ui-core` `SurfaceModel` state and SHALL react to component and data-model updates.

#### Scenario: Bound text changes
- **WHEN** `web_core` updates a bound text value
- **THEN** the Svelte surface SHALL update the corresponding semantic text without recreating protocol state

### Requirement: Svelte renderer fails closed
The Svelte renderer MUST reject component types absent from its approved catalog and expose an actionable diagnostic.

#### Scenario: Unknown component is selected
- **WHEN** a surface references an unregistered component type
- **THEN** the Svelte renderer SHALL render no unsafe widget and SHALL expose the component type and id in its error
