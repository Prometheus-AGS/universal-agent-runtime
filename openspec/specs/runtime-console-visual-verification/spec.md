## Purpose

Define the runtime console visual and navigation verification requirements for desktop and mobile operator workflows.
## Requirements
### Requirement: Runtime console desktop shell is visually stable
The runtime console SHALL provide a Playwright-verified desktop layout where the persistent navigation, active content surface, and contextual side panels remain visible and non-overlapping.

#### Scenario: Desktop cockpit shell renders
- **WHEN** the operator opens `/admin/runtime` in a desktop viewport
- **THEN** the page MUST show the runtime console navigation
- **AND** the cockpit content MUST be visible
- **AND** the provider health or workflow context panel MUST remain visible without overlapping the primary content.

#### Scenario: Desktop runtime navigation reaches key surfaces
- **WHEN** the operator uses the desktop runtime console navigation
- **THEN** the operator MUST be able to reach runs, approvals, protocols, providers, memory, and A2UI testing surfaces
- **AND** each surface MUST show an operator-facing heading or empty state instead of an error boundary.

### Requirement: Runtime console mobile shell is navigable
The runtime console SHALL provide a Playwright-verified mobile layout where navigation can open, route to runtime surfaces, and close without hiding the selected content.

#### Scenario: Mobile navigation opens and routes
- **WHEN** the operator opens `/admin/runtime` in a mobile viewport
- **THEN** a mobile navigation control MUST be visible
- **WHEN** the operator opens navigation and selects a runtime surface
- **THEN** the selected surface MUST become visible
- **AND** the navigation overlay MUST no longer block the selected content.

#### Scenario: Mobile content avoids incoherent overlap
- **WHEN** the operator views runtime console pages in a mobile viewport
- **THEN** primary headings, navigation controls, and visible action controls MUST have non-overlapping bounding boxes.

### Requirement: Runtime command palette routes to console surfaces
The runtime console SHALL expose command palette navigation that can be verified by Playwright without depending on implementation-specific CSS classes.

#### Scenario: Command palette opens with keyboard shortcut
- **WHEN** the operator presses the configured command palette keyboard shortcut
- **THEN** the command palette MUST become visible with runtime console destinations.

#### Scenario: Command palette routes to provider diagnostics
- **WHEN** the operator selects the providers destination from the command palette
- **THEN** the browser MUST navigate to the providers surface
- **AND** provider diagnostics content MUST be visible.

### Requirement: Visual tests avoid external runtime dependencies
The runtime console visual verification SHALL pass against deterministic local UI state without requiring live provider credentials, live model calls, or seeded runtime events.

#### Scenario: Empty runtime state is acceptable
- **WHEN** no runtime entities are present in the frontend entity graph
- **THEN** the visual verification MUST accept the intended empty-state content as a valid console state.

#### Scenario: Provider credentials are not required
- **WHEN** targeted visual tests navigate to provider surfaces
- **THEN** the tests MUST NOT require real provider API keys or live model responses to pass.

### Requirement: Runtime console visual replay coverage
The runtime console visual verification SHALL include replay-driven visible state checks for live runtime surfaces.

#### Scenario: Replayed cockpit state is visible
- **WHEN** replayed runtime entities are ingested while `/admin/runtime` is open
- **THEN** the runtime cockpit MUST show the replayed live runs, execution timeline, provider health, approval count, tool call count, and memory event count.

#### Scenario: Replayed protocol state is visible
- **WHEN** replayed AG-UI, A2UI, and model route decision entities are ingested before or while the protocol console is open
- **THEN** the compatibility console MUST show updated AG-UI event, A2UI surface, and liter-llm route decision summaries.

#### Scenario: Replay coverage preserves empty-state coverage
- **WHEN** replay-driven visual tests are added
- **THEN** the existing empty-state visual checks MUST remain valid for unseeded runtime console sessions.

