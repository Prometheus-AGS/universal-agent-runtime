## ADDED Requirements

### Requirement: Rejected frame observation is single-attempt and header-only

After separate Execute authorization, the workflow SHALL make at most one WebSocket upgrade and one authentication RPC to the confirmed existing database target. It SHALL use at most one active socket, a 15-second upgrade limit, a 15-second first-frame-header limit, and a 30-second total active-observation limit. It SHALL NOT request readiness, health, namespace/database selection, queries, inference, record mutations, retries, service changes, or recovery actions.

The workflow SHALL confirm source, executable, process, listener, nonsecret target, authentication-form, and configuration continuity before the attempt. A disagreement or unsafe prerequisite SHALL stop the observation and SHALL be recorded without a fallback attempt.

#### Scenario: Identity or configuration continuity fails

- **WHEN** the current executable, target, listener, authentication form, or nonsecret configuration cannot be reconciled with the inherited diagnostic evidence
- **THEN** the workflow records the specific disagreement and skips the WebSocket attempt
- **AND** it does not retry, substitute defaults, or claim a frame classification

#### Scenario: One response header is observed

- **WHEN** continuity checks pass and the authorized authentication RPC produces a WebSocket response frame
- **THEN** the workflow reads only the base header and any extended-length bytes needed to derive structural metadata
- **AND** it immediately ends the connection without intentionally reading or parsing the response payload

### Requirement: Frame evidence is structural and content-free

The retained observation SHALL contain only allowlisted identity digests and comparisons; sanitized handshake status, subprotocol, and extension-presence state; FIN, RSV1–RSV3, numeric opcode and allowlisted opcode class, MASK, length-encoding form, declared length, and framing-byte count; elapsed time, stop reason, and request/privacy counters. It SHALL NOT retain raw framing bytes, mask keys, payload bytes, payload hashes, response text, decoded application messages, credentials, tokens, record identifiers, environment dumps, process arguments, or raw configuration. Each evidence file SHALL be no larger than 64 KiB.

If a network read transiently supplies payload bytes together with the needed framing prefix, the workflow SHALL copy only the required prefix into bounded state and SHALL discard the remainder without parsing, hashing, logging, or serializing it. The evidence SHALL claim zero payload bytes retained, not zero bytes received by the operating system or runtime.

#### Scenario: Header and payload share a network chunk

- **WHEN** a socket callback receives more bytes than are required to complete the framing header
- **THEN** retained state and output contain only the framing prefix and derived allowlisted metadata
- **AND** privacy counters report zero payload bytes retained

#### Scenario: Declared length uses the 64-bit form

- **WHEN** the frame uses a 64-bit extended payload length
- **THEN** the workflow records the declared length as a decimal string without allocating a payload-sized buffer
- **AND** it retains at most ten framing bytes before ending the connection

### Requirement: Classification preserves protocol and historical uncertainty

The report SHALL classify the new observation separately against RFC 6455 header-level constraints and the prior collector's acceptance predicate. It SHALL distinguish header-valid, header-invalid, collector-compatible, collector-incompatible, and indeterminate properties. It SHALL NOT infer payload validity, UTF-8 validity, application-message validity, continuation history, or complete protocol validity from header-only evidence.

The report SHALL identify the observation as new evidence and SHALL NOT claim it recovers or proves the discarded historical frame. A fresh-context artifact-only reviewer SHALL verify the evidence boundary, classification, and minimum recommendation before unqualified acceptance. Any recommendation SHALL be limited to the exact newly observed frame form and SHALL NOT implement broader protocol support.

#### Scenario: Standard-valid frame is rejected by the old collector predicate

- **WHEN** the new frame satisfies observable RFC 6455 header constraints but is not an unmasked, final, uncompressed text frame
- **THEN** the report classifies it as header-valid and collector-incompatible, subject to properties that remain indeterminate without payload or fragment history
- **AND** it does not describe collector incompatibility as proof of a server protocol violation

#### Scenario: New attempt does not reproduce the rejection

- **WHEN** no response header is observed or the new frame matches the old collector's accepted form
- **THEN** the report remains inconclusive about the discarded historical frame
- **AND** it names the smallest next required evidence or authority without retrying or resuming readiness
