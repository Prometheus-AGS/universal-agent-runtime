## ADDED Requirements

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
