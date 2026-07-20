## ADDED Requirements

### Requirement: Deterministic scoped policy resolution
UAR SHALL resolve chat configuration in global, agent, conversation, and turn order, SHALL apply deny and disabled state before lower-scope selections, and SHALL persist an immutable effective-policy snapshot with provenance for every run.

#### Scenario: Conversation narrows agent defaults
- **WHEN** an agent allows three skills and a conversation selects one of them
- **THEN** the effective run policy contains only that skill and identifies the conversation as its selection source

#### Scenario: Lower scope cannot escape a deny
- **WHEN** a global or agent policy disables a resource selected by a conversation or turn
- **THEN** UAR excludes the resource and emits an actionable policy warning

### Requirement: Resource selections govern execution
UAR MUST apply the effective skills, MCP servers, knowledge bases, memory, context strategy, tool approval, provider, and model before prompt assembly or model invocation.

#### Scenario: Disabled resources have no effect
- **WHEN** a run selects no skills, MCP servers, or knowledge bases
- **THEN** no skill prompt, MCP tool, or knowledge retrieval from those resources enters the model request

#### Scenario: Selected resources affect chat
- **WHEN** a conversation selects an eligible skill, MCP server, and knowledge base
- **THEN** the skill can activate, only tools from the selected server are exposed, and retrieval is restricted to the selected knowledge base

### Requirement: Durable and inspectable conversation policy
UAR SHALL persist conversation policy in its configured storage backend and SHALL expose requested and effective policy through typed APIs without exposing secrets.

#### Scenario: Restart restoration
- **WHEN** UAR restarts after a conversation policy was saved
- **THEN** the same policy is restored and governs the next run

#### Scenario: Deleted resource reference
- **WHEN** a persisted policy references a deleted resource
- **THEN** inspection reports it unavailable and execution does not broaden to all resources

### Requirement: Protected built-in agents
UAR SHALL prevent deletion of the Orchestrator and Default agents at the service boundary.

#### Scenario: Protected deletion rejected
- **WHEN** a client attempts to delete a protected built-in agent
- **THEN** UAR returns a stable conflict error and the agent remains available
