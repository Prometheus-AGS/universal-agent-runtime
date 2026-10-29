## ADDED Requirements

### Requirement: Published A2UI surfaces reach AG-UI clients as activities
The dedicated AG-UI agent route SHALL send every A2UI surface the agent publishes as an `ACTIVITY_SNAPSHOT` event with `activityType` `"a2ui-surface"`, `replace: true`, and `content.a2ui_operations` holding all of that surface's operations published so far in the conversation. The `messageId` SHALL be stable for a given conversation and surface. Forwarded A2UI messages SHALL NOT contain UAR's top-level `profile` field.

#### Scenario: Agent publishes a surface
- **WHEN** the agent publishes a surface with `a2ui_render` during a run on the route
- **THEN** the run's stream contains an `ACTIVITY_SNAPSHOT` for that surface whose operations include its `createSurface` and `updateComponents` messages

#### Scenario: Deleted surface
- **WHEN** the agent publishes `deleteSurface` for a surface
- **THEN** the stream sends a final snapshot for that surface whose operations end with the delete, and later runs do not restate it

### Requirement: Surfaces belong to the conversation
Each run's stream on the route SHALL restate, right after `RUN_STARTED`, the live surfaces published by the conversation's earlier runs, read from durable run history, using the same stable `messageId`s.

#### Scenario: Reconnect shows the surface
- **WHEN** a fresh client attaches to the conversation's latest run without sending a message
- **THEN** it receives snapshots for the conversation's live surfaces and no new run is admitted

### Requirement: A2UI actions become same-conversation runs
A request on the route carrying `forwardedProps.a2uiAction` SHALL start a new run on the same conversation whose input is a JSON `a2ui.user_action` naming the action, its surface, its source component and its context. The route SHALL accept the request without a new user message, SHALL drop `role: "activity"` history messages, and SHALL accept the action only when a live surface of the conversation declares a `Button` with that component id and action name.

#### Scenario: User presses a declared button
- **WHEN** a client sends the action for a declared Button, resending the conversation's messages with nothing new from the user
- **THEN** a new run starts on the same conversation and its input carries the action's name and context

#### Scenario: Button clicked again
- **WHEN** the user presses the same button after the agent has answered
- **THEN** the action is accepted again and starts another run

#### Scenario: Undeclared action
- **WHEN** a client sends an action naming a button the conversation's surfaces never declared
- **THEN** the route responds `422 a2ui_action_not_declared` and no run is admitted

#### Scenario: Orchestrator conversation
- **WHEN** a client sends an action to an agent that runs on the agent graph
- **THEN** the route responds `422 a2ui_action_unsupported_for_graph_agent` before any run is reserved
