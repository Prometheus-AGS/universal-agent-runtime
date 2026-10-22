## ADDED Requirements

### Requirement: Client declarations do not grant host execution
The trusted host SHALL distinguish caller-owned schemas from executable host descriptors in every frozen model-call snapshot.

#### Scenario: Mixed tool batch
- **WHEN** a model returns caller and host calls together
- **THEN** only existing admitted host descriptors execute and caller calls are returned after durable handoff
- **AND** collisions, unsupported choices and invalid arguments never become host authority

### Requirement: Durable exact continuation
Caller handoffs SHALL persist owner/tenant/agent/protocol/session identity, visible and canonical histories, graph position and budget continuity before public handoff.

#### Scenario: Authenticated result batch after restart
- **WHEN** the exact owner submits every pending result with matching history and declarations
- **THEN** one durable CAS claim admits continuation with retained graph/budget state and fresh policy/credentials
- **AND** identical completed retries replay the saved projection without repeated effects

#### Scenario: Forged history or another principal
- **WHEN** submitted tool IDs, history, results, schema or authority do not match durable evidence
- **THEN** continuation is refused before executable effects, including when a new user message follows the forged exchange
