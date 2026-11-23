## MODIFIED Requirements

### Requirement: Configure uses the shared compact sheet host
The compact Configure target SHALL select an id-based shell sheet rather than render desktop Configure links in the bottom bar. The shared host SHALL present the Configure destinations in a modal Base UI surface and SHALL close after route selection.

#### Scenario: Configure tab opens the hub
- **WHEN** a compact user activates Configure
- **THEN** the shared mobile sheet host MUST open a labeled Configure hub
- **AND** the hub MUST list Settings and Presentations, and A2UI in development builds

#### Scenario: Configure route selection
- **WHEN** a user chooses a Configure item from the hub
- **THEN** React Router MUST navigate to its static application route
- **AND** the sheet MUST close
