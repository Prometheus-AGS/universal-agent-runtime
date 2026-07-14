## ADDED Requirements

### Requirement: Stable typed Python package

The project SHALL provide an MIT-licensed Python SDK versioned 1.0.0 whose public request and response values are validated by Pydantic v2.

#### Scenario: Package metadata identifies the stable release
- **WHEN** a consumer inspects the built package metadata
- **THEN** the version is 1.0.0 and the license is MIT

### Requirement: Complete core SDK capability surface

The SDK SHALL expose async methods for chat, streaming chat, tool calls, structured outputs, embeddings, run creation/streaming/cancellation/resumption/checkpoints, knowledge-base CRUD/search, document upload, and ingestion.

#### Scenario: Consumer uses each core capability
- **WHEN** a consumer invokes any core SDK operation
- **THEN** the client sends the documented UAR HTTP request and returns a typed model or async stream

### Requirement: Standards-compliant streaming

The SDK SHALL consume server-sent events through `httpx-sse`, preserve event IDs and event names, and decode JSON payloads where possible.

#### Scenario: Run emits SSE events
- **WHEN** a consumer iterates a chat or run stream
- **THEN** each SSE message is yielded as a typed event without buffering the entire response

### Requirement: Runnable guidance and generated reference docs

The SDK SHALL include six runnable examples and a Sphinx autodoc project that can build on ReadTheDocs or GitHub Pages.

#### Scenario: User learns a supported workflow
- **WHEN** a user opens the examples or builds the documentation
- **THEN** all required 1.0 workflows and the public API are represented
