## ADDED Requirements

### Requirement: Stable Rust SDK package

The project SHALL ship a publishable Rust SDK at version 1.0.0 with explicit
migration guidance and documented public APIs.

#### Scenario: Consumer builds the SDK

- **WHEN** a consumer builds the default HTTP client feature
- **THEN** the SDK compiles without requiring the embedded runtime

### Requirement: Complete typed API surface

The SDK SHALL expose typed APIs for chat completion and streaming, tool calls,
structured outputs, embeddings, runs, checkpoints, knowledge bases, documents,
search, and ingestion.

#### Scenario: Run lifecycle

- **WHEN** a consumer creates a run
- **THEN** the consumer can stream it, cancel it, list checkpoints, and resume
  from either the latest or a selected checkpoint

#### Scenario: Knowledge lifecycle

- **WHEN** a consumer manages retrieval content
- **THEN** the consumer can create, read, update, delete, search, upload, list,
  inspect, and delete knowledge resources through typed operations

### Requirement: Reconnectable typed streaming

The SDK SHALL decode server-sent events into typed events and SHALL support
resuming with a last-event identifier.

#### Scenario: Stream reconnect

- **WHEN** a consumer supplies the last received event identifier
- **THEN** the SDK sends the identifier to the runtime and yields later events

### Requirement: Actionable diagnostics

SDK failures SHALL implement `miette::Diagnostic` and preserve structured
runtime error codes and messages when returned by the server.

#### Scenario: Runtime rejects a request

- **WHEN** the server returns a structured UAR error response
- **THEN** the SDK error contains the HTTP status, runtime code, message, and
  optional request identifier

### Requirement: Runnable examples

The SDK SHALL include runnable examples for chat, streaming chat, tool calls,
structured outputs, an agent run, and a RAG query.

#### Scenario: Examples compile

- **WHEN** documentation verification builds all examples
- **THEN** every example compiles against the public version-1 API
