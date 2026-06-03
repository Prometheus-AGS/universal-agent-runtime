## ADDED Requirements

### Requirement: Client tracks the last received event id

While consuming a chat SSE stream, the client SHALL parse the SSE `id:` field of
each event and retain the highest id seen as the resume cursor for the run.

#### Scenario: Event id captured
- **WHEN** the client receives an SSE event carrying `id: 42`
- **THEN** the client records `42` as the last received event id for the run

#### Scenario: Server run id captured
- **WHEN** the chat completion response returns the `x-uar-run-id` header
- **THEN** the client retains that run id for use as the resume target

### Requirement: Mid-stream drop resumes instead of truncating

The client SHALL reconnect via the server resume endpoint
(`GET /api/uar/runs/{run_id}/stream` with `Last-Event-ID` set to the last
received id) and continue consuming events — rather than ending the response or
re-issuing the original POST — when the stream drops after at least one event has
been received and both the server run id and a last-received event id are known.

#### Scenario: Resume after a transport drop
- **WHEN** a chat stream drops mid-response (after the first event) with a known run id and last event id
- **THEN** the client issues a resume GET with `Last-Event-ID` = last received id and continues rendering the remaining events to the same message

#### Scenario: No duplicate run
- **WHEN** the client resumes after a mid-stream drop
- **THEN** it uses the read-only resume GET and never starts a second run via POST

#### Scenario: No duplicate event application
- **WHEN** the resume stream replays events at or below the last received id
- **THEN** the client does not double-apply already-handled events

### Requirement: Reconnection is bounded

The client SHALL bound reconnection by a maximum reconnect-attempt count and the
existing retry budget. When reconnection is exhausted, the stream SHALL end
cleanly using the existing terminal handling (no infinite reconnect loop).

#### Scenario: Reconnect budget exhausted
- **WHEN** repeated reconnect attempts keep failing beyond the configured cap/budget
- **THEN** the client stops reconnecting and finalizes the stream without error spam

### Requirement: Clean and pre-first-chunk paths are unchanged

A response that completes without a drop SHALL behave exactly as before, and a
failure before the first event SHALL still use the existing initial-POST retry
logic (not the resume path).

#### Scenario: Uninterrupted response
- **WHEN** a chat stream completes normally
- **THEN** no resume request is made and behavior is unchanged

#### Scenario: Failure before first chunk
- **WHEN** the initial POST fails before any event is received
- **THEN** the client uses the existing initial-POST retry policy, not the resume GET
