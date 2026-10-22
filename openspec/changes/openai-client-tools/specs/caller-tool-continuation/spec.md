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

### Requirement: Governed graph child caller handoff
The installed graph's local AgentNode delegation SHALL carry caller schemas through an opaque trusted-host capture, preserve ordinary spawn approval and child policy intersection, and bind public continuation to the root API agent with the actual child producer identity.

#### Scenario: Child requests a caller function
- **WHEN** an admitted graph child returns caller-owned calls
- **THEN** the host persists the child's canonical history, root visible history, graph resume position and root-shared budget before forwarding the exact calls
- **AND** public handoff requires successful physical cleanup of both child and parent producers
- **AND** caller schemas never become executable host bindings or model-supplied delegation authority

#### Scenario: Fresh governed graph continuation
- **WHEN** the authenticated caller submits the exact pending result batch, including after restart
- **THEN** the graph resumes the same AgentNode without rerunning the router or past host effects
- **AND** the newly admitted root resolves current credentials and child policy and obtains any newly required spawn approval
- **AND** a completed identical retry replays without another root, approval, child or provider request

### Requirement: Configurable ingestion startup for local production acceptance
The host SHALL use an operator-configurable ingestion directory while retaining `/data/ingest` as its container-compatible default. It SHALL report directory-creation failure before claiming watcher readiness and SHALL retain owned watcher cleanup.

#### Scenario: Isolated native runtime startup
- **WHEN** a native runtime is configured with a writable isolated ingestion directory
- **THEN** its native watcher registers that directory before readiness
- **AND** it does not attempt to create the container default outside the configured directory.
