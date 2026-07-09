# runtime-console-ux Specification

## Purpose
TBD - created by archiving change resolve-runtime-protocols-page-facade. Update Purpose after archive.
## Requirements
### Requirement: Unbuilt Runtime Console Panels Disclose Their Status Honestly

Runtime Console panels backed by entity data the backend does not yet emit SHALL disclose that plainly, distinguishable from a panel that is simply quiet because no matching activity has occurred recently. This covers `RuntimeProtocolsPage`'s panels, `RuntimeCockpitPage`'s Provider Health and Memory Activity panels, and `RuntimeRunsPage`'s Artifacts panel.

#### Scenario: A panel's backing entity type has never been populated

- **Given** a Runtime Console panel renders from an entity type the backend
  has no emission path for (e.g. `RuntimeAgUiEvent`, `RuntimeModelRouteDecision`,
  `RuntimeA2uiSurface`, `RuntimeProviderHealth`, `RuntimeMemoryEvent`,
  `RuntimeArtifact`)
- **When** the panel has zero entities to render
- **Then** it MUST show a disclosure stating the panel is not yet wired to
  live backend data, not a generic "no activity yet" empty state

### Requirement: Operators Can Inspect a Specific Run's Detail

The Runtime Console SHALL let an operator select any listed run and view that specific run's detail, rather than always displaying a fixed run regardless of selection.

#### Scenario: Inspecting a run from the Runs page list

- **Given** the Runs page lists multiple runs
- **When** the operator clicks "Inspect" on a run row
- **Then** the Run Detail column MUST show that run's steps, not always the first run in the list

#### Scenario: Inspecting a run from the Cockpit's Live Runs panel

- **Given** the operator is on the Cockpit page, which has no run-detail column
- **When** the operator clicks "Inspect" on a run row
- **Then** the system MUST navigate to the Runs page with that run preselected

