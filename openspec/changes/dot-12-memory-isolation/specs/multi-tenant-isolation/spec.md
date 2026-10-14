# Spec Delta

## ADDED Requirements

### Requirement: Assistant memory is scoped by verified complete owner

All assistant memory operations and retrieval SHALL enforce complete verified owner identity, including direct IDs, history and background capture, without granting authority from retrieved content.

#### Scenario: Cross-surface direct ID

- **WHEN** a principal supplies another owner's memory ID through HTTP, MCP, native tools or history
- **THEN** the operation is denied without revealing content or affecting the owner's record

#### Scenario: Scoped context retrieval

- **WHEN** memory is retrieved for prompt context or background processing
- **THEN** only currently authorized owner-scoped content is returned and its text cannot grant execution authority

## MODIFIED Requirements

### Requirement: Per-user data is isolated by authenticated identity
Threads, memories, knowledge bases, documents and chunks SHALL be scoped to the
host-verified issuer/tenant/subject owner identity; no endpoint SHALL accept a caller-supplied user id
for authorization decisions.

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
- **THEN** the supplied id is ignored and the host-verified issuer/tenant/subject owner identity is used

#### Scenario: Unresolved KB names fail closed
- **WHEN** an agent's configured knowledge-base names resolve to no accessible KB
- **THEN** retrieval returns no chunks rather than searching all knowledge bases

#### Scenario: Owners may reuse durable knowledge identifiers
- **WHEN** two verified owners create knowledge bases, documents, or chunks
  with identical logical identifiers
- **THEN** both durable record graphs coexist, parent relationships remain
  owner-qualified, and either owner can read or delete only their own graph


#### Scenario: Canonical-looking legacy subjects cannot claim private history
- **WHEN** a legacy subject string is byte-identical to a new complete principal key
- **THEN** authenticated history and knowledge access use disjoint storage namespaces
  and do not read, overwrite, migrate or delete the legacy record

#### Scenario: Session erasure preserves independent authority records
- **WHEN** an authenticated owner erases a conversation session
- **THEN** only that owner's session and conversation policy are removed, while
  accounting and tool admission evidence remain controlled by their host lifecycle
