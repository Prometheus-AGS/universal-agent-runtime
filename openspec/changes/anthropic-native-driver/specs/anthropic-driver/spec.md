## ADDED Requirements

### Requirement: LlmDriver Trait Implementation
AnthropicDriver SHALL implement the LlmDriver trait, producing a NormalizedEvent stream from Anthropic Messages API responses via reqwest.

#### Scenario: Successful streaming chat completion
- **WHEN** the Orchestrator calls `AnthropicDriver::stream()` with a valid LlmRequest containing an `anthropic/*` model
- **THEN** the driver MUST send a POST request to `https://api.anthropic.com/v1/messages` with `stream: true` and return a `Pin<Box<dyn Stream<Item = Result<NormalizedEvent>>>>`.

#### Scenario: API key authentication
- **WHEN** the driver constructs the HTTP request
- **THEN** it MUST include the `x-api-key` header with the configured API key and the `anthropic-version` header set to the pinned API version string.

### Requirement: Content Block Type Handling
AnthropicDriver SHALL handle all Anthropic content block types: text, tool_use, and thinking.

#### Scenario: Text content block received
- **WHEN** the SSE stream contains a `content_block_delta` event with `type: text_delta`
- **THEN** the driver MUST emit a `NormalizedEvent::MessageDelta` with the text fragment.

#### Scenario: Tool use content block received
- **WHEN** the SSE stream contains a `content_block_start` with `type: tool_use` followed by `content_block_delta` events with `type: input_json_delta`
- **THEN** the driver MUST emit `NormalizedEvent::ToolCallDelta` events with the tool call id, name, and JSON argument fragments.

#### Scenario: Thinking content block received
- **WHEN** the SSE stream contains a `content_block_delta` event with `type: thinking_delta`
- **THEN** the driver MUST emit a `NormalizedEvent::ThinkingDelta` with the thinking text fragment.

### Requirement: Anthropic Version Header
AnthropicDriver SHALL send the `anthropic-version` header on every API request.

#### Scenario: Version header present on request
- **WHEN** any request is sent to the Anthropic Messages API
- **THEN** the request MUST include `anthropic-version: 2023-06-01` (or the configured pinned version).

### Requirement: Rate Limit Handling with Backoff
AnthropicDriver SHALL handle HTTP 429 rate limit responses with exponential backoff and retry.

#### Scenario: Rate limited by API
- **WHEN** the Anthropic API returns HTTP 429 with a `retry-after` header
- **THEN** the driver MUST wait for at least the duration specified in `retry-after` before retrying, up to a maximum of 3 retry attempts.

#### Scenario: Rate limit retries exhausted
- **WHEN** the driver has exhausted all retry attempts and still receives HTTP 429
- **THEN** the driver MUST emit a `NormalizedEvent::Error` with a descriptive rate limit error message and terminate the stream.

### Requirement: HTTP Error Handling
AnthropicDriver SHALL map non-2xx HTTP responses to NormalizedEvent::Error.

#### Scenario: Authentication failure
- **WHEN** the Anthropic API returns HTTP 401
- **THEN** the driver MUST emit a `NormalizedEvent::Error` indicating invalid API key and terminate the stream.

#### Scenario: Server error
- **WHEN** the Anthropic API returns HTTP 500 or 503
- **THEN** the driver MUST retry with exponential backoff up to 3 attempts, then emit `NormalizedEvent::Error` if all retries fail.
