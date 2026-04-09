## ADDED Requirements

### Requirement: Streaming State Machine
ToolCallExtractor SHALL implement a streaming state machine with states: Text, InToolCallTag, and Done.

#### Scenario: Pure text with no tool calls
- **WHEN** the extractor receives a stream of `MessageDelta` events containing only plain text with no `<tool_call>` tags
- **THEN** it MUST remain in the Text state and forward all `MessageDelta` events unmodified.

#### Scenario: State transitions through a complete tool call
- **WHEN** the extractor receives text containing `<tool_call>`, then JSON content, then `</tool_call>`
- **THEN** it MUST transition from Text to InToolCallTag on detecting `<tool_call>`, buffer JSON content while in InToolCallTag, and transition back to Text (or Done) on detecting `</tool_call>`.

### Requirement: Tool Call Tag Detection
ToolCallExtractor SHALL detect `<tool_call>` opening tags in streamed text fragments.

#### Scenario: Tag split across multiple deltas
- **WHEN** the text `"<tool_call>"` arrives split across two `MessageDelta` events (e.g., `"<tool_"` and `"call>"`)
- **THEN** the extractor MUST correctly detect the opening tag by buffering partial tag matches.

#### Scenario: Tag appears mid-text
- **WHEN** a `MessageDelta` contains `"Here is the result: <tool_call>{..."`
- **THEN** the extractor MUST emit a `MessageDelta` with `"Here is the result: "` and then transition to InToolCallTag for the JSON content.

### Requirement: JSON Content Buffering and Delta Emission
ToolCallExtractor SHALL buffer JSON content within tool_call tags and emit ToolCallDelta fragments as they arrive.

#### Scenario: Streaming JSON argument fragments
- **WHEN** the extractor is in InToolCallTag state and receives `MessageDelta` events containing JSON fragments like `'{"name": "search", "argu'` and `'ments": {"query": "rust"}}'`
- **THEN** it MUST emit `NormalizedEvent::ToolCallDelta` events with each fragment as `arguments_delta`.

### Requirement: ToolCallComplete on Close Tag
ToolCallExtractor SHALL emit a ToolCallComplete event when the `</tool_call>` closing tag is detected.

#### Scenario: Complete tool call parsed
- **WHEN** the closing `</tool_call>` tag is detected and the buffered JSON is `{"name": "search", "arguments": {"query": "rust"}}`
- **THEN** the extractor MUST emit a `NormalizedEvent::ToolCallComplete` with `name: "search"`, `arguments_json: '{"query": "rust"}'`, and the generated tool call id.

#### Scenario: Multiple sequential tool calls
- **WHEN** the stream contains two consecutive `<tool_call>...</tool_call>` blocks
- **THEN** the extractor MUST emit separate `ToolCallDelta`/`ToolCallComplete` event sequences for each, with incrementing `call_index` values.

### Requirement: Synthetic UUID Generation
ToolCallExtractor SHALL generate synthetic UUIDs for tool call IDs since instruction-tuned models do not produce them.

#### Scenario: Tool call ID assignment
- **WHEN** a `<tool_call>` opening tag is detected
- **THEN** the extractor MUST generate a unique UUID (v4) and use it as the `id` field in all subsequent `ToolCallDelta` and `ToolCallComplete` events for that tool call.

#### Scenario: Multiple tool calls get unique IDs
- **WHEN** two tool calls are extracted from the same response
- **THEN** each MUST receive a distinct UUID as its tool call id.

### Requirement: Malformed JSON Handling
ToolCallExtractor SHALL handle malformed JSON within tool_call tags gracefully.

#### Scenario: Invalid JSON detected on close tag
- **WHEN** the closing `</tool_call>` tag is detected but the buffered content is not valid JSON
- **THEN** the extractor MUST emit a `NormalizedEvent::Error` indicating a JSON parse failure and include the raw buffered content in the error message for debugging.

#### Scenario: Retry with error feedback
- **WHEN** malformed JSON is detected and retry budget allows (up to 2 retries)
- **THEN** the extractor MUST signal that the tool call failed to parse, allowing the orchestrator to re-prompt the model with error feedback for correction.
