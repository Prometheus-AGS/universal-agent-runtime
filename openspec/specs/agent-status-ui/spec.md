## Purpose

Define how agent processing status appears in user-facing chat and runtime console surfaces.
## Requirements
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

### Requirement: Runtime console status affordances are visually covered
The agent status UI SHALL have targeted visual coverage when status affordances appear inside runtime console or chat-adjacent operator surfaces.

#### Scenario: Status surface renders without console layout regression
- **WHEN** Playwright opens a runtime console surface that displays agent processing status or status empty states
- **THEN** the status content MUST be visible
- **AND** it MUST NOT overlap primary navigation or page headings.

#### Scenario: Status coverage preserves existing chat status behavior
- **WHEN** runtime console visual tests are added
- **THEN** the existing chat status requirements for thinking, tool execution, searching, completion, and transition behavior MUST remain unchanged.

### Requirement: Replayed run status transitions update status surfaces
The agent status UI SHALL reflect replayed runtime run and run step state transitions in chat-adjacent and runtime console status surfaces.

#### Scenario: Replayed running status is visible
- **WHEN** a replayed run or run step event sets status to `running` or `waiting`
- **THEN** the relevant runtime console status surface MUST display the active status without a manual refresh.

#### Scenario: Replayed terminal status is visible
- **WHEN** a replayed run or run step event updates status to `completed`, `failed`, or `cancelled`
- **THEN** the relevant runtime console status surface MUST display the terminal status without leaving stale active-state indicators.

#### Scenario: Status replay preserves chat status behavior
- **WHEN** runtime run status replay tests are added
- **THEN** the existing chat status requirements for thinking, tool execution, searching, completion, and transition behavior MUST remain unchanged.
