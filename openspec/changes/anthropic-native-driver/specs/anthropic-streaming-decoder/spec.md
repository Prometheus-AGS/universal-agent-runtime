## ADDED Requirements

### Requirement: SSE Event Ordering State Machine
The streaming decoder SHALL implement a state machine that processes Anthropic SSE events in the correct order: message_start, content_block_start, content_block_delta (repeated), content_block_stop, message_delta, message_stop.

#### Scenario: Normal message lifecycle
- **WHEN** the SSE stream emits events in order: `message_start` then `content_block_start` then one or more `content_block_delta` then `content_block_stop` then `message_delta` then `message_stop`
- **THEN** the decoder MUST emit `StreamStart`, then the appropriate content events, then `Usage` (if present), then `Done`, in that order.

#### Scenario: Unexpected event ordering
- **WHEN** the SSE stream emits a `content_block_delta` before any `content_block_start`
- **THEN** the decoder MUST emit a `NormalizedEvent::Error` indicating an unexpected event ordering and terminate the stream gracefully.

### Requirement: Text Block Decoding
The decoder SHALL map text content blocks to MessageDelta events.

#### Scenario: Single text block with multiple deltas
- **WHEN** the stream emits `content_block_start(type: text)` followed by three `content_block_delta(type: text_delta)` events with texts "Hello", " ", "world"
- **THEN** the decoder MUST emit three `NormalizedEvent::MessageDelta` events with the corresponding text fragments.

### Requirement: Tool Use Block Decoding
The decoder SHALL map tool_use content blocks to ToolCallDelta and ToolCallComplete events.

#### Scenario: Tool use block with streamed JSON arguments
- **WHEN** the stream emits `content_block_start(type: tool_use, id: "toolu_123", name: "search")` followed by `content_block_delta(type: input_json_delta)` events with partial JSON fragments, then `content_block_stop`
- **THEN** the decoder MUST emit `NormalizedEvent::ToolCallDelta` events with each JSON fragment, then a `NormalizedEvent::ToolCallComplete` with the full accumulated JSON arguments, tool call id, and name.

#### Scenario: Tool use block with empty arguments
- **WHEN** the stream emits a tool_use block with no `input_json_delta` events before `content_block_stop`
- **THEN** the decoder MUST emit a `NormalizedEvent::ToolCallComplete` with `arguments_json` set to `"{}"`.

### Requirement: Interleaved Block Handling
The decoder SHALL handle interleaved text and tool_use content blocks within a single message.

#### Scenario: Text block followed by tool_use block
- **WHEN** the stream emits a text content block (index 0), then a tool_use content block (index 1)
- **THEN** the decoder MUST track each block by its index and emit the correct event types for each block without cross-contamination.

#### Scenario: Multiple tool_use blocks in sequence
- **WHEN** the stream emits two tool_use content blocks (index 0 and index 1) with different tool names and arguments
- **THEN** the decoder MUST emit separate `ToolCallDelta`/`ToolCallComplete` event sequences for each block, with the correct `call_index` corresponding to each block's index.

### Requirement: Thinking Block Decoding
The decoder SHALL map thinking content blocks to ThinkingDelta events.

#### Scenario: Thinking block streamed
- **WHEN** the stream emits `content_block_start(type: thinking)` followed by `content_block_delta(type: thinking_delta)` events
- **THEN** the decoder MUST emit `NormalizedEvent::ThinkingDelta` events with each thinking text fragment.

### Requirement: Message Delta and Usage Handling
The decoder SHALL extract usage information from message_delta events.

#### Scenario: message_delta with usage
- **WHEN** the stream emits a `message_delta` event containing `usage: {output_tokens: 150}`
- **THEN** the decoder MUST emit a `NormalizedEvent::Usage` event with the output token count and any cache token fields present.

### Requirement: Error Event Handling
The decoder SHALL map Anthropic error events to NormalizedEvent::Error.

#### Scenario: API error event in stream
- **WHEN** the stream emits an `error` event with `type: overloaded_error` and a message
- **THEN** the decoder MUST emit a `NormalizedEvent::Error` with the error type and message, then terminate the stream.

#### Scenario: Malformed SSE line
- **WHEN** the stream contains an SSE line that cannot be parsed as valid JSON
- **THEN** the decoder MUST emit a `NormalizedEvent::Error` with a descriptive parse error message.
