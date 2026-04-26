## ADDED Requirements

### Requirement: Runtime replay exposes workflow mirror memory activity
The runtime event replay/entity-sync validation SHALL make workflow mirror memory activity observable as runtime entity graph state without making the frontend responsible for workflow persistence.

#### Scenario: Workflow mirror memory event is ingested
- **WHEN** a replayed runtime memory event represents a workflow mirror operation
- **THEN** the entity graph MUST contain a runtime memory event entity with action `workflow_mirror`, the associated workflow metadata, and the provided updated timestamp.

#### Scenario: Workflow mirror memory event appears without refresh
- **WHEN** a workflow mirror memory event is ingested while the runtime console is open
- **THEN** the runtime console MUST expose the memory activity through existing live runtime state without requiring a browser refresh.

#### Scenario: Frontend does not own mirror persistence
- **WHEN** workflow mirror memory activity is displayed or replayed in the frontend
- **THEN** the frontend MUST NOT write authoritative workflow mirror records directly to Surreal Memory.
