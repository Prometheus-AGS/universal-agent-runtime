## Purpose

Gives each host session its own UAR owner inside one shared sidecar process, by letting the launch-token-authenticated host name the principal for each request, so UAR's existing owner checks separate the host's sessions.

## ADDED Requirements

### Requirement: The launch-token-authenticated host may assert a principal
On a request that passed the sidecar launch-token check, UAR SHALL accept an `X-UAR-Principal` header and SHALL treat its value as the verified user identity for that request, with no tenant. The value SHALL match `^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$` and SHALL NOT be `anonymous`; otherwise the request SHALL be rejected with HTTP 400 and code `principal_invalid`. On any other request, including standalone UAR with or without JWT, a request carrying the header SHALL be rejected with HTTP 400 and code `principal_header_not_allowed`. A launch-token-authenticated request without the header SHALL keep the anonymous identity. Error responses SHALL NOT echo the header value.

#### Scenario: Sidecar request with a principal
- **WHEN** the host sends `X-UAR-Principal: boss-session-1` with the launch token
- **THEN** the request is handled as the verified owner `boss-session-1`

#### Scenario: Header on standalone UAR
- **WHEN** a request to a non-sidecar UAR carries `X-UAR-Principal`, with or without a valid JWT
- **THEN** the response is 400 with `principal_header_not_allowed` and no handler runs

#### Scenario: Invalid principal
- **WHEN** the host sends `X-UAR-Principal: anonymous` or a value with a space
- **THEN** the response is 400 with `principal_invalid`

### Requirement: Runs and approvals belong to the asserting principal
A run created under a principal SHALL be visible and controllable only by requests carrying the same principal. Stream, cancel, checkpoint listing, resume, tool approval and A2UI routes for a run owned by another principal SHALL respond 404 and SHALL NOT change the run, its pending approval, or its surfaces.

#### Scenario: Approval from another session
- **WHEN** a run of `boss-session-1` waits for approval and a request with `boss-session-2` posts an approval decision for it
- **THEN** the response is 404 and the approval stays pending until `boss-session-1` decides

#### Scenario: Stream from another session
- **WHEN** `boss-session-2` opens the stream of a run owned by `boss-session-1`
- **THEN** the response is 404 and no event bytes are written

### Requirement: Principal-scoped stores do not cross sessions
Under distinct principals, sessions with the same id, memories, knowledge bases, stored credentials, presentations and MCP bindings SHALL be separate. A run SHALL retrieve only from knowledge bases owned by its principal; a name that resolves only under another principal SHALL yield no chunks. Memory tools in a run with a principal SHALL read and change only that principal's memories, whatever owner fields the model supplies.

#### Scenario: Same session id under two principals
- **WHEN** `boss-session-1` and `boss-session-2` both use session id `s1`
- **THEN** each run sees only its own principal's history for `s1`

#### Scenario: Memory written by another session
- **WHEN** a run of `boss-session-2` lists, searches, updates, deletes or reads the history of memories, including by the id of a memory written by `boss-session-1`
- **THEN** no `boss-session-1` memory is returned or changed

#### Scenario: Knowledge base of another session
- **WHEN** a run of `boss-session-2` names a knowledge base that exists only under `boss-session-1`
- **THEN** retrieval returns no chunks and no citation from it

### Requirement: Memory tools refuse unscoped access without a verified owner
In a run without a verified owner, `memory_list`, `memory_update_by_id`, `memory_delete_by_id` and `memory_history` SHALL return a tool error with code `memory_requires_verified_owner` and SHALL NOT read or change any record.

#### Scenario: Anonymous memory list
- **WHEN** an anonymous run calls `memory_list` with no filter
- **THEN** the tool returns `memory_requires_verified_owner` and no memory content

#### Scenario: Anonymous delete by id
- **WHEN** an anonymous run calls `memory_delete_by_id` with an existing memory id
- **THEN** the tool returns `memory_requires_verified_owner` and the memory still exists
