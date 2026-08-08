## REMOVED Requirements

### Requirement: Admin page for A2UI artifact testing
**Reason**: The dedicated A2UI schema-testing playground is a developer-time tool, not an operational surface. It is being retired from the production admin navigation as the frontend converges on a compact live runtime operations console.
**Migration**: A2UI artifact schemas are still available through the A2UI schema store, the A2UI API service, and chat artifact rendering. Operators observe live A2UI surfaces through the runtime console rather than a standalone testing page. No production data or persisted state is affected by the page's removal.

### Requirement: A2UI testing surface is responsive in the runtime console
**Reason**: This requirement describes reaching the dedicated A2UI testing page from the runtime console navigation across desktop and mobile viewports. With the testing page retired, there is no such navigable surface to make responsive.
**Migration**: The A2UI testing navigation entry is removed from the admin/runtime-console shell. Live A2UI protocol state continues to be reachable through the runtime console's own A2UI surfaces view, whose responsive behavior is governed by the runtime-console capability.

### Requirement: A2UI visual coverage preserves schema testing behavior
**Reason**: This requirement preserved the retired testing page's schema listing, preview, test submission, and custom-schema validation behavior under responsive visual coverage. Those page behaviors no longer exist in production once the page is removed.
**Migration**: Schema listing, preview, and validation capabilities remain available programmatically via the A2UI schema store and API service for use by chat and the runtime console; they are simply no longer surfaced through a dedicated admin testing page.

## MODIFIED Requirements

### Requirement: Replayed A2UI surfaces are visible in protocol testing UI
The runtime console SHALL show replayed A2UI surface events and chunk-style updates as live runtime protocol state.

#### Scenario: Replayed A2UI surface is visible
- **WHEN** a replayed A2UI surface event is ingested
- **THEN** the runtime console protocol view MUST show the A2UI surface title, status, and payload summary without a manual refresh.

#### Scenario: Replayed A2UI update replaces stale surface state
- **WHEN** a later replayed A2UI event targets an existing A2UI surface id
- **THEN** the runtime console MUST show the latest surface status and payload instead of stale state.
