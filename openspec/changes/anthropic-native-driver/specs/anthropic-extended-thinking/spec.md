## ADDED Requirements

### Requirement: Thinking Parameter in API Request
When thinking configuration is provided, the driver SHALL send the `thinking` parameter with `budget_tokens` to the Anthropic Messages API.

#### Scenario: Thinking config present on LlmRequest
- **WHEN** the LlmRequest contains a `thinking_config` with `budget_tokens: 10000`
- **THEN** the driver MUST include `"thinking": {"type": "enabled", "budget_tokens": 10000}` in the API request body.

#### Scenario: Thinking config absent
- **WHEN** the LlmRequest does not contain a `thinking_config`
- **THEN** the driver MUST NOT include the `thinking` parameter in the API request body.

### Requirement: Thinking Content Block Streaming
Thinking content blocks SHALL stream as ThinkingDelta events to downstream consumers.

#### Scenario: Thinking block starts streaming
- **WHEN** the SSE stream emits a `content_block_start` with `type: thinking` followed by `content_block_delta` events with `type: thinking_delta`
- **THEN** the driver MUST emit `NormalizedEvent::ThinkingDelta` events containing each thinking text fragment in order.

#### Scenario: Thinking block completes
- **WHEN** the SSE stream emits a `content_block_stop` for a thinking block
- **THEN** the driver MUST transition its internal state to accept subsequent content blocks (text or tool_use) without error.

### Requirement: Transparent to Downstream Consumers
Thinking events SHALL be transparent to downstream consumers via the existing NormalizedEvent enum without requiring protocol changes.

#### Scenario: Consumer receives ThinkingDelta
- **WHEN** a downstream consumer (SSE serializer, UI handler) iterates the NormalizedEvent stream and encounters a `ThinkingDelta` event
- **THEN** the event MUST be serializable using the same SSE format as all other NormalizedEvent variants.

#### Scenario: Consumer ignores thinking events
- **WHEN** a downstream consumer does not handle `ThinkingDelta` events explicitly
- **THEN** the unhandled events MUST NOT cause errors or interrupt processing of subsequent events in the stream.

### Requirement: Thinking Budget Enforcement
The driver SHALL pass the budget_tokens value to the API to cap thinking output length.

#### Scenario: Budget tokens limit respected
- **WHEN** the thinking config specifies `budget_tokens: 5000`
- **THEN** the API request MUST include the budget constraint and the model's thinking output MUST be bounded by that token budget.

#### Scenario: Extended thinking produces interleaved blocks
- **WHEN** the model produces a thinking block followed by a text block followed by a tool_use block
- **THEN** the driver MUST emit `ThinkingDelta` events, then `MessageDelta` events, then `ToolCallDelta`/`ToolCallComplete` events in the correct order without mixing content from different blocks.
