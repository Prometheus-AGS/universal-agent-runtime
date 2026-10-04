## Purpose

Control external connector effects through durable, scoped UAR intent records and a trusted host credential broker.

## ADDED Requirements

### Requirement: Exact connector authority
An effect SHALL be authorized by authenticated owner/workspace, current connector binding revision, exact provider and target, action and egress label. Read, draft, write, send and publish SHALL be separate capabilities. External content SHALL never expand authority.

#### Scenario: Injected external instruction
- **WHEN** a retrieved issue or channel message asks for another target or action
- **THEN** the stored binding and requested action remain authoritative and the expansion is denied.

### Requirement: Durable intent and uncertain outcome
UAR SHALL commit the intent and one dispatch identity before a trusted host performs an external effect. Repeated commands SHALL not create another dispatch. An unknown response SHALL remain unresolved without blind retry; reconciliation SHALL preserve the original identity and evidence.

#### Scenario: Lost issue response
- **WHEN** GitHub accepts an issue creation but the HTTP response is lost
- **THEN** restart exposes the original uncertain effect and never posts it again without independently verified reconciliation.

### Requirement: Host credential boundary
UAR SHALL persist only an opaque credential reference and SHALL refuse dispatch when no trusted host broker can resolve it. Credential values SHALL never appear in catalog records, ordinary API responses or logs.

#### Scenario: Missing broker
- **WHEN** a configured connector has no trusted credential broker
- **THEN** its prepared effect remains inspectable as unavailable and no network request occurs.
