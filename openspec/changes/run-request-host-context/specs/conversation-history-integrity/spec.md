## ADDED Requirements

### Requirement: Host-supplied history seeds an empty session
`POST /api/uar/runs` SHALL accept an optional `history` object with a `session_id` and an ordered list of messages. Each message SHALL have role `user`, `assistant` or `tool`; an `assistant` message MAY carry tool calls with an id, a name and arguments; a `tool` message SHALL carry a `tool_call_id` that answers a tool call of an earlier assistant message in the list. When the resolved session holds no messages, UAR SHALL place the supplied messages in the session, in order, before the current input, and they SHALL be the session's history for this and later turns. The create-run response SHALL report `history: "seeded"` and the number of messages seeded.

#### Scenario: Cold session after a restart
- **WHEN** a restarted UAR receives a run for session S with history of two prior user turns and two assistant replies
- **THEN** the first model request contains those four messages in order followed by the new input, and the response reports `history: "seeded"` and `seeded_messages: 4`

#### Scenario: Assistant tool call and result
- **WHEN** history contains an assistant message with tool call `c1` followed by a tool message answering `c1`
- **THEN** the model request contains the tool call and its result as a pair

### Requirement: In-memory session history takes precedence over host history
When the resolved session already holds messages, UAR SHALL NOT replace, merge or append the supplied history, and the create-run response SHALL report `history: "ignored_warm_session"`. When no history is supplied the response SHALL report `history: "none"`. Resume routes (`POST /api/uar/runs/{id}/resume` and `POST /api/uar/runs/{id}/resume/{checkpoint_id}`) SHALL reject a body that carries `history` with HTTP 422 and code `history_invalid`, because a checkpoint's protected history is authoritative and a resume continues a live session.

#### Scenario: Warm session
- **WHEN** session S already holds messages in memory and a run for S carries history
- **THEN** the model request contains the in-memory history once, the supplied history is not added, and the response reports `ignored_warm_session`

#### Scenario: History on a checkpoint resume
- **WHEN** a `POST /api/uar/runs/{id}/resume/{checkpoint_id}` body carries `history`
- **THEN** the response is 422 with `history_invalid` and no run starts

### Requirement: Host history binds only to its own session
A request with `history` SHALL carry a `session_id`, and `history.session_id` SHALL equal it; otherwise the request SHALL be rejected with HTTP 422 and code `history_session_mismatch`. Seeded history SHALL be stored only in the session resolved for the request's verified owner and `session_id`, and SHALL NOT be visible to any other session or owner.

#### Scenario: Mismatched session
- **WHEN** a run for session S1 carries history labeled S2
- **THEN** the response is 422 with `history_session_mismatch` and neither session changes

#### Scenario: History without a session id
- **WHEN** a run carries history and no `session_id`
- **THEN** the response is 422 with `history_session_mismatch`

### Requirement: Malformed or oversized host history is rejected, not repaired
UAR SHALL reject with HTTP 422 and code `history_invalid` host history that contains a `system` message, an unknown role, a tool message without a matching earlier tool call, a duplicate tool call id, or empty required fields. It SHALL reject with HTTP 422 and code `history_too_large` history above the configured message-count or byte limit. Rejection SHALL leave the session unchanged and start no run.

#### Scenario: Orphaned tool result
- **WHEN** history contains a tool message whose `tool_call_id` matches no earlier assistant tool call
- **THEN** the response is 422 with `history_invalid` and the session holds no messages

#### Scenario: System message in history
- **WHEN** history contains a `system` message
- **THEN** the response is 422 with `history_invalid`
