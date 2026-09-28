# Channel-source observers

## ADDED Requirements

### Requirement: Explicit channel source profile

UAR MUST negotiate `uar.channel-source/1` separately from C07 logical-instance observation. An ordinary run or token SSE MUST NOT be advertised as a durable channel source.

#### Scenario: Fabric delivery

- **WHEN** an authenticated host submits a `frf.routed-observer/1` metadata payload or an exact policy-filtered text projection for a matching subscriber
- **THEN** UAR persists one exact receipt keyed by owner, workspace, subscription and delivery ID, and independent subscribers retain independent cursors.

#### Scenario: Authorized text projection

- **WHEN** BossFang has disclosed a text-only source projection and UAR receives the matching `policy_filtered` `{ "text": "..." }` Fabric envelope
- **THEN** UAR hashes the exact UTF-8 text, obtains a fresh Gate recipient-delivery release for that digest and classification, and only then persists the text for that subscriber.
- **AND** metadata-only deliveries never persist text; media, attachments, chunks, or arbitrary projection keys are rejected.

### Requirement: Current authority at each effect

UAR MUST obtain current Gate recipient-delivery authority before admitting a subscriber copy and current handler-execution authority before submitting the selected handler's C06 turn. A caller-supplied permit or transport admission MUST NOT substitute for Gate's authenticated release.

#### Scenario: Revoked queued grant

- **WHEN** a grant is revoked before recipient delivery or selected handler execution
- **THEN** UAR withholds that effect and preserves the occurrence and recovery status without executing a different handler.

### Requirement: Replay and uncertain outcome

UAR MUST use immutable delivery identity and a deterministic selected-handler command ID. A repeated Gate release with an uncertain outcome MUST NOT blindly resubmit a handler turn.

#### Scenario: Restart between release and turn admission

- **WHEN** UAR restarts after Gate releases handler execution but before C06 admits the command
- **THEN** it reports an uncertain effect for operator reconciliation rather than issuing a second execution.

### Requirement: Remote durability is attested, not inferred

UAR MUST NOT advertise a remote SurrealDB instance as durable from its URL alone. A trusted process attestation and the completed remote integration gate are required before cross-host support is claimed.

#### Scenario: External in-memory SurrealDB endpoint

- **WHEN** UAR connects to a remote SurrealDB endpoint without a trusted durability attestation
- **THEN** durable channel admission remains unsupported with an explicit reason, while the local SurrealKV profile remains available.
