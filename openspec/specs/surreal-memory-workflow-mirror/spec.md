# surreal-memory-workflow-mirror Specification

## Purpose
Define the deterministic KBD/OpenSpec workflow mirror contract for Surreal Memory MCP as a secondary recovery and query store.

## Requirements
### Requirement: Workflow mirror records use deterministic metadata
The system SHALL mirror KBD/OpenSpec workflow state into Surreal Memory records with deterministic workflow metadata.

#### Scenario: Workflow entity types are representable
- **WHEN** workflow state is prepared for mirroring
- **THEN** the mirror record metadata MUST identify the workflow kind as one of project, phase, openspec_change, task, waypoint, assessment, plan, blocker, or verification_result.

#### Scenario: Workflow identity fields are preserved
- **WHEN** a workflow record is mirrored
- **THEN** the mirror record metadata MUST include workflow_id, source_tool, updated_at, and source_path fields.

#### Scenario: Change-scoped workflow records include routing metadata
- **WHEN** a workflow record belongs to a KBD phase or OpenSpec change
- **THEN** the mirror record metadata MUST include the phase and change identifiers that are available from the source workflow state.

### Requirement: Workflow mirror writes do not replace KBD source of truth
The system SHALL treat `.kbd-orchestrator/` files as authoritative and Surreal Memory as a secondary recovery and query store.

#### Scenario: Mirror write leaves source files authoritative
- **WHEN** workflow state is written to Surreal Memory
- **THEN** the mirror operation MUST NOT silently overwrite `.kbd-orchestrator/` files.

#### Scenario: Recovery produces candidate state
- **WHEN** mirrored workflow state is queried for recovery
- **THEN** the system MUST return recovery candidates without mutating authoritative KBD files automatically.

### Requirement: Workflow mirror supports deterministic round trip validation
The system SHALL validate create, retrieve, and update behavior for mirrored workflow records without requiring live provider credentials or model calls.

#### Scenario: Workflow record can be created and retrieved
- **WHEN** a workflow mirror record is created through the memory service or `/mcp/memory` boundary
- **THEN** the same workflow_id, workflow_kind, source_tool, updated_at, and content MUST be retrievable.

#### Scenario: Workflow record can be updated
- **WHEN** an existing mirrored workflow record is updated with new workflow metadata or content
- **THEN** retrieval MUST return the updated workflow metadata or content for that record.

#### Scenario: Validation avoids provider dependencies
- **WHEN** workflow mirror validation runs
- **THEN** it MUST NOT require live LLM provider credentials, live model calls, or external provider network access.

### Requirement: Workflow mirror conflict resolution preserves auditability
The system SHALL resolve mirrored workflow conflicts by newest updated_at while preserving source_tool audit metadata.

#### Scenario: Newer workflow record wins recovery selection
- **WHEN** two mirrored workflow records share the same workflow_kind and workflow_id but have different updated_at values
- **THEN** recovery selection MUST choose the record with the newest updated_at value.

#### Scenario: Source tool remains auditable
- **WHEN** conflict resolution selects a winning mirrored workflow record
- **THEN** the selected record MUST preserve the source_tool value from the winning write.

#### Scenario: Older candidates are not erased by conflict resolution
- **WHEN** conflict resolution evaluates older mirrored workflow records
- **THEN** the system MUST keep older candidate records available for audit unless an explicit deletion operation is requested.

### Requirement: Workflow mirror verification records KBD evidence
The workflow mirror validation SHALL record verification evidence in the KBD phase state.

#### Scenario: Workflow mirror validation passes
- **WHEN** workflow mirror validation passes
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST record `surreal_memory_workflow_mirror` as verified or complete with the verification timestamp.

#### Scenario: Workflow mirror validation fails
- **WHEN** workflow mirror validation fails
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST keep `surreal_memory_workflow_mirror` in a non-complete state and record the blocking command or failure summary.
