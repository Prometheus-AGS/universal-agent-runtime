## ADDED Requirements

### Requirement: Host-attested scoped media authority

UAR SHALL accept a media grant only from a trusted host attesting explicit confirmation for an exact coordinator, session, target, purpose, action/modality/capability set, and expiry within its authenticated tenant, owner and workspace.

#### Scenario: Worker uses narrower authority

- **GIVEN** the host recorded a coordinator grant and a narrower delegation to a worker
- **WHEN** the host forwards the worker's action inside that delegation
- **THEN** UAR accepts the intent and rechecks authority before dispatch
- **AND** a changed scope, expired or revoked grant, forged identifier, or widened delegation is refused

#### Scenario: Observations attempt to authorize an action

- **WHEN** audio, captions, participant metadata, or a worker request claims approval
- **THEN** UAR does not treat the claim as confirmation or grant authority

### Requirement: Durable single dispatch and reconciliation

UAR SHALL persist an intent with a stable idempotency key and commit one dispatch attempt before handoff. UAR SHALL preserve unresolved attempts across restart and append independently evidenced reconciliation without reissuing the effect.

#### Scenario: Connector outcome is lost

- **GIVEN** the mock peer applied the effect after a dispatch claim
- **WHEN** the result is lost and the request is repeated after restart
- **THEN** UAR returns the existing intent and its unknown receipt
- **AND** no new claim is issued
- **AND** reconciliation appends the confirmed outcome to the same attempt

#### Scenario: Cancellation races dispatch

- **WHEN** cancellation wins the durable transition
- **THEN** UAR issues no dispatch claim
- **AND** a cancellation that loses the transition cannot assert the effect was undone

### Requirement: Media payload and credential isolation

UAR SHALL persist only effect metadata, opaque references and digests, accept transient send content only when its digest matches, and omit credential and media data from diagnostic output. Speech and video processor credential resolution SHALL remain separate host responsibilities.

#### Scenario: Persisted receipts contain metadata only

- **WHEN** a mock content send completes and its receipts are read after restart
- **THEN** the receipts identify authority, actor, effect, attempt and result without media, transcript, credential or raw client content
