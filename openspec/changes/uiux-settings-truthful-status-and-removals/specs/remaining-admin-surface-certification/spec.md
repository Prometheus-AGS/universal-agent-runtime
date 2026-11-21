## ADDED Requirements

### Requirement: Every stable admin page is reachable from the console
Each stable admin section that has a rendering page SHALL be reachable by navigation from `/threads` without typing a URL: through the sidebar, a parent page's secondary navigation, or the command palette. A section whose backing service is absent SHALL say so on its page or be removed from stable UI; it SHALL NOT remain unreachable.

#### Scenario: Previously orphaned pages are opened
- **WHEN** an operator starts at `/threads` and navigates by clicks or the command palette
- **THEN** the `models`, `credentials`, `mcp-health`, `compiler`, `memory`, `cost`, `auth` and `protocols` sections each render their heading

#### Scenario: A parent page offers its children
- **WHEN** an operator opens the Providers page
- **THEN** its secondary navigation offers Models and Credentials
- **AND** the active-navigation highlight matches the page shown

#### Scenario: A page has no backing service
- **WHEN** a section's data request fails because its service is absent
- **THEN** the page shows an actionable state and does not present an empty success

#### Scenario: An unknown section is requested
- **WHEN** the console opens `/admin/nonexistent`
- **THEN** it renders the runtime cockpit as before

### Requirement: The Providers default banner shows the configured default model
The Providers page SHALL show the default provider's configured `default_model` in its default banner. It SHALL NOT infer the default model from the first entry of a model list.

#### Scenario: The default model is not the first model
- **WHEN** the default provider's `default_model` differs from its first listed model
- **THEN** the banner shows `provider/default_model`

#### Scenario: The default provider has no default model
- **WHEN** the default provider has no `default_model`
- **THEN** the banner shows that no default model is set
- **AND** it prints no model identifier

#### Scenario: The provider registry has not loaded
- **WHEN** the registry load is in progress or has failed
- **THEN** the banner shows a loading or error state, not a model identifier
