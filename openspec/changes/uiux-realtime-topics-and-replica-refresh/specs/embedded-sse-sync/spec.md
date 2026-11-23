## ADDED Requirements

### Requirement: The embedded invalidation stream serves every public topic with its snapshot declaration
`/api/uar/sync/stream` SHALL serve the same public topics as the shared live stream, SHALL declare each topic's snapshot requirement in its frames, and the embedded client SHALL treat a frame that declares no snapshot as an invalidation and not as an error.

#### Scenario: New topic appears on the embedded stream
- **WHEN** a conversation policy is saved on an embedded Surreal deployment
- **THEN** the stream emits a `reset` frame for `conversation_policies` with `requires_snapshot: false`

#### Scenario: Snapshotless frame is not an error
- **WHEN** the embedded client receives a frame that declares no snapshot
- **THEN** its connection status stays connected and the topic's reset handler runs
