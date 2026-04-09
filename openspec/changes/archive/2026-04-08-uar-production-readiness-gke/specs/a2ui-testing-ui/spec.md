## ADDED Requirements

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
