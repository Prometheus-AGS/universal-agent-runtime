## ADDED Requirements

### Requirement: Exact zero-length Ping continuation is one-shot

After the header-only frame classification is independently accepted and separate Execute authorization is given, the diagnostic workflow SHALL make at most one WebSocket upgrade and one authentication RPC to the confirmed existing database target. It SHALL accept either one final, RSV-clear, server-unmasked, minimally encoded Text response, or one final, RSV-clear, server-unmasked, minimally encoded zero-length Ping followed by one such Text response.

For the exact zero-length Ping, the workflow SHALL send one final client-masked zero-length Pong with a fresh four-byte masking key and no application payload. It SHALL observe at most two response frames, send at most one Pong and accept at most one sign-in response. It SHALL NOT add a general frame loop, retry, or accept any different or repeated control frame.

#### Scenario: Sign-in returns Text directly

- **WHEN** the first response is an accepted Text frame
- **THEN** the workflow processes that frame as the sole sign-in response and sends no Pong
- **AND** it ends the connection after determining sign-in success or error

#### Scenario: Sign-in returns an exact empty Ping before Text

- **WHEN** the first response is the exact accepted zero-length Ping
- **THEN** the workflow completely writes one masked empty Pong and waits for at most one accepted Text response
- **AND** it ends the connection after that response without accepting another control frame

#### Scenario: A different frame or sequence is received

- **WHEN** the workflow receives a repeated or nonzero Ping, unexpected Pong, Close, continuation, fragmented frame, Binary frame, reserved opcode, masked server frame, nonzero RSV bit, non-minimal length encoding, or any other unsupported structure
- **THEN** it stops the attempt with a sanitized structural reason
- **AND** it does not send a fallback response, reconnect, retry, or broaden the accepted sequence

### Requirement: Ping continuation uses one non-resetting sign-in deadline

The workflow SHALL start one monotonic 15-second sign-in-exchange deadline immediately before the sign-in write. That same deadline SHALL cover sign-in write completion, first-frame receipt, the optional Pong write and the optional following Text response, and SHALL NOT reset after any progress. The existing 15-second upgrade limit and 30-second total active-observation limit SHALL remain unchanged.

The workflow SHALL NOT inspect a response until the sign-in write settles. A response frame already buffered while that write is pending SHALL be eligible as the first response only after the write settles and only while the original sign-in-exchange deadline has time remaining.

#### Scenario: First response is buffered during sign-in write

- **WHEN** response bytes arrive before the sign-in write has settled and the write then settles before the shared deadline
- **THEN** the workflow treats the first complete buffered frame as the first response under the remaining original budget
- **AND** it does not restart or extend the deadline

#### Scenario: Sign-in or Pong write exhausts the deadline

- **WHEN** the sign-in write or optional Pong write does not completely settle before the shared deadline
- **THEN** the workflow destroys the active socket and records a sanitized deadline stop reason
- **AND** it does not inspect another response, send another frame, or retry

### Requirement: Ping-continuation evidence is bounded and content-free

The workflow SHALL accept a Text sign-in response only when its declared length is between 1 and 262,144 bytes. It MAY transiently parse the bounded response to match the expected RPC identifier and determine sign-in success or error, but it SHALL clear response, credential, frame and masking-key buffers and SHALL NOT log, hash or serialize their content.

Retained evidence SHALL be no larger than 65,536 bytes and SHALL contain only allowlisted identity digests and comparisons, sanitized handshake and frame classes, declared byte counts, monotonic timings, result or stop categories, and request, privacy and cardinality counters. It SHALL NOT contain raw framing bytes, masking keys, payload bytes or hashes, response text, tokens, credentials, raw identity values, record identifiers, environment dumps, process arguments or raw configuration.

#### Scenario: Sign-in result is available

- **WHEN** an accepted Text response matches the expected RPC identifier and contains a sign-in success or error result
- **THEN** retained evidence records only the sanitized result category, sizes, timings and counters
- **AND** the workflow sends no namespace/database selection, query, health, readiness, inference or mutation request

#### Scenario: Text or evidence exceeds its cap

- **WHEN** a Text response declares more than 262,144 bytes or sanitized evidence would exceed 65,536 bytes
- **THEN** the workflow stops with a sanitized size reason
- **AND** it retains no response content and does not retry

#### Scenario: Operator interrupts the attempt

- **WHEN** the process receives `SIGINT` or `SIGTERM`
- **THEN** it destroys the active socket, clears sensitive buffers and records only an allowlisted interrupt stop code
- **AND** it performs no retry or continuation

### Requirement: Exact-frame handling receives phase-boundary acceptance

The diagnostic workflow SHALL defer its synthetic behavioral suite until the end of the child Execute phase. That suite SHALL cover the two accepted response sequences, exact masked-Pong framing, every named unsupported frame class, write failure and deadline behavior, buffered-first-frame behavior, interrupt cleanup, evidence privacy, size limits and cardinality limits. A fresh-context artifact-only reviewer SHALL evaluate the source digest, test output, sanitized evidence and result classification before unqualified acceptance.

Successful sign-in SHALL NOT be reported as database-readiness causation, recovery, sustained readiness or release certification. Unresolved critical findings SHALL prevent unqualified acceptance.

#### Scenario: Phase-boundary suite or review finds a critical defect

- **WHEN** a required synthetic case fails or independent review retains a critical finding
- **THEN** the diagnostic change remains blocked or explicitly qualified
- **AND** it does not claim the Ping boundary or database readiness is resolved

#### Scenario: Exact-frame acceptance passes

- **WHEN** all required synthetic cases, privacy checks, strict specification validation and independent review pass
- **THEN** the workflow may accept only the exact Ping/Pong diagnostic behavior and its observed sign-in result
- **AND** database-readiness attribution and recovery remain separate unresolved work
