# Spec Delta

## MODIFIED Requirements

### Requirement: Supported streams require protocol completion evidence

**Stable task:** R2. Every supported streaming route SHALL preserve incremental framing and require the selected protocol's terminal evidence before reporting successful completion. An ordinary non-cancelled run SHALL settle successfully only after orchestration explicitly completes; transport EOF, a provider terminal marker, a stream-close envelope or a tool result SHALL NOT substitute for completed orchestration.

#### Scenario: EOF without terminal

- **WHEN** a supported provider connection closes without its required terminal event
- **THEN** the runtime reports an incomplete/error outcome rather than successful completion

#### Scenario: Arbitrary byte boundaries

- **WHEN** valid events including multibyte characters and tool arguments cross transport chunks
- **THEN** the selected production route reconstructs the same events and succeeds only after valid terminal evidence

#### Scenario: Ordinary manager stream exhausts before completion

- **WHEN** the ordinary manager's orchestrator stream exhausts without explicit orchestration completion and the run was not cancelled
- **THEN** the manager persists and projects the typed missing-completion failure before normal success settlement
- **AND** the run does not become successful, execute a pending tool, publish a synthetic successful completion, or rewrite a tool receipt

#### Scenario: Ordinary manager observes completion

- **WHEN** the ordinary manager observes explicit orchestration completion after a valid completed text turn or completed tool loop
- **THEN** the run follows the existing successful completion path

#### Scenario: Cancellation and cleanup retain their existing ownership

- **WHEN** cancellation occurs while the manager joins the captured execution and performs terminal cleanup
- **THEN** cancellation and cleanup retain their existing ordering, checkpoint/history behavior, and reservation ownership
- **AND** an unconfirmed cleanup error remains an error rather than a successful completion

## ADDED Requirements

### Requirement: Native Anthropic streams preserve framing and bounded completion

**Stable task:** R3. Native Anthropic streams SHALL accept one optional initial UTF-8 BOM and CR, LF or CRLF line endings independently of transport chunk boundaries. They SHALL retain strict UTF-8 and a 1 MiB incomplete-frame bound, reject incomplete EOF frames, and preserve payload, tool, usage and completion meaning.

#### Scenario: Equivalent newline encodings cross arbitrary chunks

- **WHEN** equivalent valid native Anthropic event streams use LF, CRLF, or lone CR and split CRLF, UTF-8, or JSON escapes across chunks
- **THEN** the driver emits equivalent normalized event sequences after accepting at most one initial BOM
- **AND** a later BOM remains payload data

#### Scenario: Incomplete or oversized framing remains fatal

- **WHEN** framing ends with an incomplete event, exceeds the decoder's effective incomplete-frame limit, or lacks `message_stop`
- **THEN** the native driver reports the existing stream failure path and does not report successful completion

#### Scenario: Well-formed future event type is ignored

- **WHEN** valid Anthropic JSON carries an unknown top-level string `type` and its explicit SSE event name is empty or agrees with that type
- **THEN** the driver ignores that frame without a normalized event or state mutation

#### Scenario: Malformed known event remains fatal

- **WHEN** a known event has an invalid expected shape, JSON is invalid, `type` is absent or non-string, the event and payload type disagree, or data follows `message_stop`
- **THEN** the native driver reports its malformed-stream failure and does not treat the frame as an ignorable future type
