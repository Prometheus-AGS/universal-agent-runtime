## ADDED Requirements

### Requirement: Stable TypeScript SDK 1.0 contract

The TypeScript SDK SHALL publish an MIT-licensed `1.0.0` package whose typed,
runtime-validated client covers chat and streaming, tool calls, structured
outputs, embeddings, the complete run lifecycle, knowledge-base CRUD and
ingest with errors preserving HTTP status and server detail.

#### Scenario: A consumer invokes the supported runtime surface

- **Given** a configured SDK client and a conforming UAR server
- **When** the consumer invokes any supported chat, tool, structured output,
  embedding, run lifecycle, knowledge, document, search, or ingest method
- **Then** the SDK sends the documented HTTP request and validates the response
  before returning a strongly typed value

#### Scenario: A consumer streams chat or run events

- **Given** an SSE endpoint and an optional last event identifier
- **When** the consumer starts a stream with an abort signal
- **Then** the SDK uses fetch-based SSE, yields validated events in order,
  forwards resume metadata, and honors cancellation

### Requirement: TypeScript SDK examples and API documentation

The SDK SHALL provide six typechecked runnable examples, including a Next.js
example, and SHALL generate TypeDoc API documentation deployable to GitHub
Pages.

#### Scenario: A maintainer verifies release readiness

- **Given** a clean SDK checkout with dependencies installed
- **When** focused typecheck, lint, test, build, documentation, and example
  checks execute
- **Then** every check passes without requiring a live UAR server
