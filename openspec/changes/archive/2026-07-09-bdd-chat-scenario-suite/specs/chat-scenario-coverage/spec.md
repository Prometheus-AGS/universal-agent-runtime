## ADDED Requirements

### Requirement: Single-turn chat completion is covered by a BDD scenario
The BDD suite SHALL include a scenario asserting that a single-turn, non-streaming request to the OpenAI-compatible completion endpoint returns a well-formed `chat.completion` with the stub-configured content.

#### Scenario: Single-turn non-streaming completion
- **WHEN** a bare OpenAI-shaped chat completion request with a single user message is sent to the completion endpoint, backed by a stub LLM configured to answer that message
- **THEN** the response status MUST be successful
- **AND** the response body MUST be an OpenAI `chat.completion` whose assistant content matches the stub-configured answer

### Requirement: Multi-turn conversation is covered by a BDD scenario
The BDD suite SHALL include a scenario asserting that a request carrying prior conversation turns in `messages` is answered against the latest user turn.

#### Scenario: Multi-turn conversation answers the latest user turn
- **WHEN** a chat completion request is sent whose `messages` include a prior user/assistant exchange followed by a new user message, backed by a stub LLM configured to answer that new message
- **THEN** the response status MUST be successful
- **AND** the response body MUST be an OpenAI `chat.completion` whose assistant content matches the answer configured for the latest user turn

### Requirement: Tool-call round trip is covered by a BDD scenario
The BDD suite SHALL include a scenario asserting that when the model requests a tool, the server runs the tool loop and the final assistant content reflects the tool result.

#### Scenario: Tool-call round trip produces final content
- **WHEN** a chat completion request is sent, backed by a stub LLM configured to first request a tool call and then return final content after the tool result
- **THEN** the response status MUST be successful
- **AND** the final assistant content MUST match the stub-configured post-tool content

### Requirement: Streaming chat is covered by a BDD scenario
The BDD suite SHALL include a scenario asserting that a streaming chat request emits streamed output and a terminal completion signal.

#### Scenario: Streaming request emits deltas and completion
- **WHEN** a streaming chat completion request is sent, backed by a stub LLM configured to answer the message
- **THEN** the response status MUST be successful
- **AND** the streamed response MUST contain the stub-configured content
- **AND** the streamed response MUST contain a terminal completion signal

### Requirement: Malformed chat request is handled gracefully
The BDD suite SHALL include a scenario asserting that a structurally invalid chat request is rejected with a client error rather than causing a server error or panic.

#### Scenario: Missing messages field yields a client error
- **WHEN** a chat completion request missing the required `messages` field is sent
- **THEN** the response status MUST be a 4xx client error
- **AND** the response status MUST NOT be a 5xx server error
