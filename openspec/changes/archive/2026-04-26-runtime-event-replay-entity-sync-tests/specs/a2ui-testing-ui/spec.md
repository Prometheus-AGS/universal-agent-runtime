## ADDED Requirements

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
