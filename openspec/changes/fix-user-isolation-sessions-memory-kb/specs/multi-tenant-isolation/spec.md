## ADDED Requirements

### Requirement: Per-user data is isolated by authenticated identity
Threads, memories, knowledge bases, documents and chunks SHALL be scoped to the
JWT-derived user identity; no endpoint SHALL accept a caller-supplied user id
for authorization decisions.

#### Scenario: Cross-user thread access
- **WHEN** user B requests a thread created by user A
- **THEN** the API returns 404/403 and no content from user A leaks

#### Scenario: Memory identity from token
- **WHEN** a request to any memory endpoint supplies a user_id in body or query
- **THEN** the supplied id is ignored and the JWT subject is used

#### Scenario: Unresolved KB names fail closed
- **WHEN** an agent's configured knowledge-base names resolve to no accessible KB
- **THEN** retrieval returns no chunks rather than searching all knowledge bases
