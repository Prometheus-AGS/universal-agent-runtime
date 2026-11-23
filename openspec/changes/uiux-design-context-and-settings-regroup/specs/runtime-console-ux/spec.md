## ADDED Requirements

### Requirement: Settings are organized into seven areas with deep links
The console SHALL present Settings as the areas Models & providers, Tools & MCP, Skills, Knowledge & memory, Governance, Appearance and Desktop, each addressable as `/settings/<area>/<section>`. One route table SHALL feed the settings navigation, the command palette and breadcrumbs. Every retained setting SHALL appear in exactly one section or as a link to its owning page. Unknown areas and sections SHALL redirect to a defined fallback.

#### Scenario: A section URL is opened cold
- **WHEN** an operator opens `/settings/models/routing` in a new tab
- **THEN** the Routing section of Models & providers renders
- **AND** reload, back and forward preserve the section

#### Scenario: A moved page is requested by its old URL
- **WHEN** the console opens `/admin/cost`
- **THEN** it replaces the location with `/settings/governance/budget` and renders the Cost page

#### Scenario: Unknown locations
- **WHEN** the console opens `/settings/nope`, `/settings/models/nope`, or `/settings/desktop` outside the desktop host
- **THEN** it replaces the location with `/settings/models`, the first section of `models`, and `/settings/models` respectively

#### Scenario: Completeness
- **WHEN** the navigation is compared with the namespaces the instance lists
- **THEN** every listed namespace not removed from the console is shown exactly once or linked to its owning page

### Requirement: Settings sections disclose advanced controls on request
Each settings section SHALL render the controls needed for its main job first and group the remaining controls under one Advanced disclosure that shows its control count. A section SHALL render no more than 8 controls before Advanced is opened. Raw JSON and comma-separated editors, where they remain, SHALL appear only inside Advanced.

#### Scenario: A section is rendered
- **WHEN** a settings section first renders
- **THEN** at most 8 controls are visible
- **AND** the Advanced disclosure shows how many controls it holds

#### Scenario: Advanced is operated by keyboard
- **WHEN** an operator opens Advanced with the keyboard
- **THEN** the hidden controls appear in logical focus order
- **AND** the disclosure state is exposed to assistive technology

### Requirement: Primary navigation has one Settings destination
The shell destination inventory SHALL contain one Settings destination for `/settings` in place of separate Providers, MCP & tools, Skills and Runtime settings destinations. The Runs destination SHALL match `/admin/runtime`, `/admin/approvals` and `/admin/protocols`. The command palette SHALL include one entry per settings section.

#### Scenario: A settings route is active
- **WHEN** the current path begins with `/settings`
- **THEN** the Settings destination is active and the breadcrumb shows its area and section

#### Scenario: The runtime cockpit is active
- **WHEN** the current path is `/admin/runtime`
- **THEN** the Runs destination is active and Settings is not

#### Scenario: A section is searched
- **WHEN** an operator searches the command palette for "Routing"
- **THEN** the palette offers `/settings/models/routing`
