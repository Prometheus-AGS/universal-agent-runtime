## ADDED Requirements

### Requirement: Display agent processing status in chat UI
The chat interface SHALL display real-time status labels indicating what the agent is currently doing during message processing.

#### Scenario: Thinking status
- **WHEN** the agent is generating a response (message deltas streaming)
- **THEN** the UI displays a "Thinking..." status indicator below the last message

#### Scenario: Tool execution status
- **WHEN** the agent invokes a tool call
- **THEN** the UI displays "Executing [tool_name]..." with the specific tool name

#### Scenario: Searching status
- **WHEN** the agent invokes a search-related tool (e.g., `tavily__search`)
- **THEN** the UI displays "Searching..." as the status

#### Scenario: Status clears on completion
- **WHEN** the agent finishes processing and the `done` event is received
- **THEN** the status indicator is removed

### Requirement: Status transitions are smooth
The UI SHALL animate status transitions to avoid jarring visual changes.

#### Scenario: Fade transition
- **WHEN** the status changes from "Thinking..." to "Executing search..."
- **THEN** the transition uses a fade animation (not an abrupt swap)
