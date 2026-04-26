## ADDED Requirements

### Requirement: Runtime replay fixtures cover observable runtime entities
The frontend validation suite SHALL provide deterministic replay fixtures for runtime events that map to every runtime entity surfaced by the runtime console.

#### Scenario: Replay fixture set covers runtime entities
- **WHEN** the runtime event replay fixtures are loaded
- **THEN** the fixture set MUST include events for runs, run steps, tool calls, approvals, artifacts, memory events, AG-UI events, A2UI surfaces, model route decisions, and provider health.

#### Scenario: Replay fixtures avoid live provider dependencies
- **WHEN** replay fixtures are executed during frontend validation
- **THEN** the fixtures MUST NOT require live provider credentials, live model calls, or external network access.

### Requirement: Runtime event ingestion normalizes replayed events into the entity graph
The frontend replay tests SHALL verify that replayed runtime events are normalized through the same ingest boundary used by live runtime event streams.

#### Scenario: Runtime events upsert expected entity types
- **WHEN** replayed runtime event envelopes are passed to the runtime ingest boundary
- **THEN** the Prometheus entity graph MUST contain matching entities for the expected runtime entity types.

#### Scenario: Replayed updates merge with existing entities
- **WHEN** a later replayed event targets the same runtime entity id as an earlier event
- **THEN** the entity graph MUST update the existing entity instead of creating a duplicate logical entity.

#### Scenario: AG-UI events use the AG-UI ingest path
- **WHEN** replayed AG-UI event envelopes are passed with a run id
- **THEN** the entity graph MUST contain `RuntimeAgUiEvent` entities with the provided run id, event type, sequence, payload, and updated timestamp.

### Requirement: Runtime replay updates are visible without refresh
The runtime console SHALL display replayed entity graph updates without requiring a browser refresh or route reload.

#### Scenario: Cockpit reflects replayed operational state
- **WHEN** replayed run, tool, approval, memory, provider health, and route decision events are ingested while the runtime cockpit is open
- **THEN** the cockpit MUST update its visible runtime summaries and panels without a manual refresh.

#### Scenario: Detail surfaces reflect replayed operational state
- **WHEN** replayed run step, tool call, approval, artifact, AG-UI, and A2UI events are ingested while their runtime console surfaces are open
- **THEN** those surfaces MUST show the replayed state without a manual refresh.

### Requirement: Replay validation records workflow evidence
The runtime event replay/entity-sync workflow SHALL record validation evidence in the KBD phase state.

#### Scenario: Replay validation completes
- **WHEN** runtime replay/entity-sync validation passes
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST record `runtime_event_replay_tests` as verified or complete.

#### Scenario: Replay validation fails
- **WHEN** runtime replay/entity-sync validation fails
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST keep `runtime_event_replay_tests` in a non-complete state and record the blocking command or failure summary.
