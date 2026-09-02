## ADDED Requirements

### Requirement: Every child agent is a persisted thread through the same kernel
The runtime SHALL execute every child agent, whether reached through an actor, a graph node, or an A2A request, as a thread through the same turn kernel, and SHALL persist a thread record and a parent-to-child edge with owner, root, parent, canonical path, artifact id, status, and history revision in every persistence provider with stable ordering.

#### Scenario: Graph child uses the kernel
- **WHEN** a graph node delegates to a child agent
- **THEN** the child runs with its artifact, skills, tools, policy, and history mode, and its thread and edge are persisted

### Requirement: Child policy is an intersection that only narrows
A child's effective policy SHALL be the intersection of the parent's effective policy and the child artifact's policy; skills, MCP servers, tools, credentials, sandbox permissions, and budgets SHALL only narrow; unsupported policy shapes SHALL fail closed; and a child SHALL NOT widen approval or supply user authorization.

#### Scenario: Child artifact requests a denied tool
- **WHEN** the child artifact allows a tool the parent policy denies
- **THEN** the child's effective policy excludes the tool

#### Scenario: Approval originates from the root
- **WHEN** a child's tool call requires approval
- **THEN** the approval request is raised on the root run and the child's own text cannot satisfy it

### Requirement: Inter-agent messages are typed
Messages between agents SHALL be typed records carrying sender and recipient identity as metadata and a flag stating whether the message triggers a turn; the runtime SHALL NOT convey identity by prepending text to a user message.

#### Scenario: Parent sends a note
- **WHEN** a parent sends a message with `trigger_turn: false`
- **THEN** the child's mailbox holds it and no turn starts until a triggering message arrives

### Requirement: Tree-wide limits, budgets, and cancellation
The runtime SHALL enforce per root run at most four concurrent children, depth three, and sixteen total children, SHALL record every child's usage against the root run's budget and refuse new spawns and model calls when it is exceeded, and SHALL cancel every child, including remote A2A tasks, when the root is cancelled.

#### Scenario: Concurrency limit
- **WHEN** four children are running and a fifth spawn is requested
- **THEN** the spawn is refused with a typed limit error

#### Scenario: Root cancelled with a remote child
- **WHEN** the root run is cancelled while an A2A child task is running
- **THEN** the runtime sends `tasks/cancel` for that task and the child thread ends as cancelled

### Requirement: Agent operations are model tools with explicit authorization
The runtime SHALL expose `spawn_agent`, `send_agent_message`, `wait_agents`, `list_agents`, and `interrupt_agent` as descriptor-registered tools whose descriptions state that spawning requires explicit user or artifact authorization.

#### Scenario: Spawn without authorization
- **WHEN** neither the user nor the artifact authorizes delegation
- **THEN** the spawn tool is not exposed to the model

### Requirement: Lifecycle is observable without leaking content
The runtime SHALL emit additive lifecycle events with parent id, child id, canonical path, status, and terminal outcome, SHALL derive AG-UI subagent events from them, and SHALL NOT include child prompts or hidden reasoning.

#### Scenario: Child completes
- **WHEN** a child thread finishes
- **THEN** clients receive a finished event with ids, path, and outcome and no prompt text

### Requirement: The inbound A2A endpoint runs agents
The A2A `message/send`, `tasks/get`, and `tasks/cancel` operations SHALL map onto the thread service for the named agent artifact with the existing wire contract unchanged.

#### Scenario: External client sends a message
- **WHEN** an A2A client calls `message/send` for a registered agent
- **THEN** a run starts on that agent's artifact and the returned task reflects the thread's status
