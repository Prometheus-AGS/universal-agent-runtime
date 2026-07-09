# a2ui-live-testing Specification

## Purpose
TBD - created by archiving change upgrade-a2ui-testing-live-round-trip. Update Purpose after archive.
## Requirements
### Requirement: Trigger a Real Artifact Input Request Against an Active Run

The system SHALL provide a way to trigger a real `ArtifactInputRequest` event against a currently active run, using the same event type and delivery mechanism a live agent tool call would use — not a mocked or parallel code path.

#### Scenario: Triggering against an active run

- **Given** a run exists and is in `running` or `waiting` state
- **When** an operator triggers a test artifact with a valid `artifact_type`, `title`, and `content`
- **Then** the backend MUST emit a real `ArtifactInputRequest` event onto that run's SSE stream, with a freshly generated `artifact_id`
- **AND** any client currently or subsequently connected to that run's stream MUST receive it exactly as it would receive an agent-originated one

#### Scenario: Triggering against a nonexistent or inactive run

- **Given** a run ID that doesn't exist or is not currently tracked as active
- **When** a test-trigger is attempted
- **Then** the system MUST reject it with a clear error identifying the run as not found/active, not silently succeed or fabricate a run

### Requirement: Test-Triggered Artifacts Complete Through Real Chat Components

A test-triggered artifact input request SHALL be observable and completable entirely through the production chat rendering and submission path — no separate rendering or submission logic specific to testing.

#### Scenario: Operator completes a test-triggered artifact

- **Given** a test-triggered `ArtifactInputRequest` has been emitted against a run whose thread is open in the chat UI
- **When** the operator interacts with the resulting input block and submits a response
- **Then** the submission MUST go through the same `POST /api/uar/runs/{run_id}/artifact-response` endpoint and `A2uiInputBlock` component real agent-originated artifacts use

