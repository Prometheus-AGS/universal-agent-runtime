## MODIFIED Requirements

### Requirement: Per-user data is isolated by authenticated identity
Threads, memories, knowledge bases, documents and chunks SHALL be scoped to the
authenticated user identity: the JWT-derived identity, or, on a request that
passed the sidecar launch-token check, the host-asserted principal. No endpoint
SHALL accept a caller-supplied user id in a body or query parameter for
authorization decisions, and no run SHALL read or change memories without a
verified owner.

#### Scenario: Cross-user thread access
- **WHEN** user B requests a thread created by user A
- **THEN** the API returns 404/403 and no content from user A leaks

#### Scenario: Thread-adjacent state follows the verified owner
- **WHEN** user B uses the direct API or ACP to request, stream, approve,
  cancel, or resume a run or
  reads agent configuration or conversation policy for user A's session
- **THEN** the operation returns 404/403 or an empty owner-scoped result and
  does not expose or mutate user A's state

#### Scenario: Legacy sessions remain compatible without becoming claimable
- **WHEN** a pre-ownership session is encountered after migration
- **THEN** it remains available only in the anonymous compatibility scope and
  an authenticated caller cannot claim it by presenting its former logical ID

#### Scenario: Memory identity from token
- **WHEN** a request to any memory endpoint supplies a user_id in body or query
- **THEN** the supplied id is ignored and the JWT subject is used

#### Scenario: Unresolved KB names fail closed
- **WHEN** an agent's configured knowledge-base names resolve to no accessible KB
- **THEN** retrieval returns no chunks rather than searching all knowledge bases

#### Scenario: Owners may reuse durable knowledge identifiers
- **WHEN** two verified owners create knowledge bases, documents, or chunks
  with identical logical identifiers
- **THEN** both durable record graphs coexist, parent relationships remain
  owner-qualified, and either owner can read or delete only their own graph

#### Scenario: Sidecar principals are isolated like users
- **WHEN** two host-asserted sidecar principals use the same session id,
  memory tools, knowledge-base names and approval routes
- **THEN** each principal sees and changes only its own state, exactly as two
  JWT users would

#### Scenario: Anonymous run cannot enumerate memories
- **WHEN** a run with no verified owner calls a memory tool that lists or
  addresses memories by id
- **THEN** the tool is refused and no memory is read or changed
