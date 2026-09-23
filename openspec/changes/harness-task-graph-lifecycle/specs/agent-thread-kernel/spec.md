## MODIFIED Requirements

### Requirement: The inbound A2A endpoint runs agents
The A2A message/send, tasks/get and tasks/cancel operations SHALL map onto the existing thread service for the named agent artifact without changing supported wire contracts. A2A task/context and AG-UI thread/run identities SHALL resolve through a durable owner-scoped mapping, with persisted status, approvals, parent/child relationships and artifacts. Both protocols SHALL observe the same committed lifecycle state.

#### Scenario: External client sends a message
- **WHEN** an A2A client calls message/send for a registered agent
- **THEN** a run starts on that agent's artifact and the returned task reflects the thread's status

#### Scenario: Cross-protocol lookup and cancellation
- **WHEN** an authorized client follows or cancels the same task through either supported adapter
- **THEN** both views resolve to the same run and converge on its persisted status, cancellation and artifacts

#### Scenario: Cross-tenant task reference
- **WHEN** a client supplies another tenant's task/context/run identifier
- **THEN** access and mutation are denied without leaking task content or existence details beyond the existing authorization contract

## ADDED Requirements

### Requirement: Task creation and recovery are idempotent
The trusted host SHALL durably correlate task creation, enqueue and publication so retries and crashes yield at most one logical run per owner-scoped accepted request identity. Recovery SHALL expose committed status and artifacts, reconcile outstanding execution and SHALL NOT blindly repeat side effects whose completion is unknown. Exactly-once logical identity SHALL NOT be presented as a guarantee of exactly-once external effects.

#### Scenario: Crash at each creation boundary
- **WHEN** injected crashes occur between identity persistence, enqueue, execution acceptance and response publication followed by the same accepted request retry
- **THEN** recovery yields one logical task/run mapping and no duplicate governed tool invocation; ambiguous external completion produces an explicit recovery state

#### Scenario: Restart during approval
- **WHEN** a host restarts with a pending approval or retained artifact
- **THEN** authorized lookup restores that approval/artifact and does not treat the pending call as cancelled or completed

