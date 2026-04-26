## ADDED Requirements

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
