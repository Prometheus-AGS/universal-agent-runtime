## ADDED Requirements

### Requirement: Unbuilt Runtime Console Panels Disclose Their Status Honestly

Runtime Console panels backed by entity data the backend does not yet emit SHALL disclose that plainly, distinguishable from a panel that is simply quiet because no matching activity has occurred recently.

#### Scenario: A panel's backing entity type has never been populated

- **Given** a Runtime Console panel renders from an entity type the backend
  has no emission path for (e.g. `RuntimeAgUiEvent`, `RuntimeModelRouteDecision`,
  `RuntimeA2uiSurface`)
- **When** the panel has zero entities to render
- **Then** it MUST show a disclosure stating the panel is not yet wired to
  live backend data, not a generic "no activity yet" empty state
