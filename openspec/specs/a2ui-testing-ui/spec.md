## Purpose

Define the admin A2UI artifact testing surface and its responsive runtime-console verification requirements.
## Requirements
### Requirement: Admin page for A2UI artifact testing
The admin dashboard SHALL include a page for previewing and testing all 5 built-in A2UI artifact types (form, confirm, select, text-input, display).

#### Scenario: List available schemas
- **WHEN** the user navigates to the A2UI testing admin page
- **THEN** all registered A2UI schemas are displayed with their IDs and descriptions

#### Scenario: Preview form artifact
- **WHEN** the user selects the `a2ui/form` schema and provides sample JSON data
- **THEN** the page renders a live preview of the form as the agent would present it to users

#### Scenario: Submit test response
- **WHEN** the user fills in a previewed form and clicks "Submit Test"
- **THEN** the page displays the JSON response payload that would be sent to the agent

#### Scenario: Custom schema testing
- **WHEN** the user pastes a custom A2UI schema JSON
- **THEN** the page validates the schema and renders a preview if valid, or shows validation errors

### Requirement: A2UI testing surface is responsive in the runtime console
The A2UI testing UI SHALL be reachable and visually usable from the runtime console navigation across desktop and mobile viewports.

#### Scenario: Desktop A2UI testing surface is reachable
- **WHEN** the operator navigates to the A2UI testing surface from the desktop runtime console shell
- **THEN** the page MUST show the A2UI testing content
- **AND** schema preview or schema selection controls MUST be visible without being obscured by navigation.

#### Scenario: Mobile A2UI testing surface is reachable
- **WHEN** the operator navigates to the A2UI testing surface from the mobile runtime console shell
- **THEN** the page MUST show the A2UI testing content
- **AND** the mobile navigation overlay MUST NOT obscure the selected surface after navigation completes.

### Requirement: A2UI visual coverage preserves schema testing behavior
The A2UI testing UI SHALL keep its existing schema listing, preview, test submission, and custom schema validation behavior while adding responsive visual coverage.

#### Scenario: Existing A2UI behavior remains in scope
- **WHEN** the responsive visual tests are implemented
- **THEN** the existing scenarios for listing schemas, previewing form artifacts, submitting test responses, and testing custom schemas MUST remain valid.

### Requirement: Replayed A2UI surfaces are visible in protocol testing UI
The A2UI testing UI SHALL show replayed A2UI surface events and chunk-style updates as live runtime protocol state.

#### Scenario: Replayed A2UI surface is visible
- **WHEN** a replayed A2UI surface event is ingested
- **THEN** the runtime protocol or A2UI testing surface MUST show the A2UI surface title, status, and payload summary without a manual refresh.

#### Scenario: Replayed A2UI update replaces stale surface state
- **WHEN** a later replayed A2UI event targets an existing A2UI surface id
- **THEN** the UI MUST show the latest surface status and payload instead of stale state.

#### Scenario: A2UI replay preserves schema testing behavior
- **WHEN** A2UI replay validation is added
- **THEN** the existing schema listing, preview, test submission, and custom schema validation requirements MUST remain valid.
