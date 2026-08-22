# skill-governance Specification

## Purpose
TBD - created by archiving change skill-scoped-governance. Update Purpose after archive.
## Requirements
### Requirement: Every skill is configurable at global, agent and conversation scope
The runtime SHALL support enabling and disabling any skill at global, per-agent
and per-conversation scope, regardless of origin. Where several scopes apply, the
most specific SHALL win: conversation over agent over global.

#### Scenario: Conversation scope overrides agent scope
- **WHEN** a skill is enabled for an agent but disabled for one conversation
- **THEN** it is not activated in that conversation and remains available in the agent's other conversations

#### Scenario: Agent scope overrides global
- **WHEN** a skill is enabled globally but disabled for one agent
- **THEN** it is not activated for that agent and remains available to other agents

#### Scenario: Built-in skills are configurable
- **WHEN** a built-in skill is disabled at any scope
- **THEN** the disable takes effect exactly as it would for a user-created skill

### Requirement: Scoped configuration is durable and survives restart re-registration
Scoped configuration SHALL be persisted, and startup registration of built-in
skills SHALL NOT overwrite stored configuration.

#### Scenario: Disable survives restart
- **WHEN** a built-in skill is disabled globally and the runtime restarts
- **THEN** the skill remains disabled after startup re-registration

#### Scenario: Per-agent state survives restart
- **WHEN** a skill is disabled for one agent and the runtime restarts
- **THEN** the disable still applies to that agent only

### Requirement: Configuration changes take effect without a restart
A scoped configuration change SHALL affect subsequent skill matching without
requiring a restart. Runs already in flight SHALL retain the binding established
at run start. A persisted conversation policy SHALL constrain the skills eligible
for the next run in that conversation.

#### Scenario: Disable takes effect on the next request
- **WHEN** a skill is disabled and a new request is made that would otherwise match it
- **THEN** the skill is not activated, with no restart in between

#### Scenario: In-flight run is unaffected
- **WHEN** a skill is disabled while a run using it is in progress
- **THEN** that run continues with the binding it started with

#### Scenario: Session skill selection gates activation
- **WHEN** a persisted session agent-config excludes a skill for one conversation
- **THEN** the next run in that conversation emits no activation event and injects no prompt overlay for that skill

### Requirement: Deletability is determined by origin, and origin is visible to clients
Built-in skills, including skills loaded from the installed pack, SHALL NOT be
editable or deletable. User-created skills SHALL remain editable and deletable.
The skills API SHALL expose each skill's origin.

#### Scenario: Built-in delete is refused
- **WHEN** a delete is requested for a skill whose origin is built-in
- **THEN** the request is refused and the skill remains present and configurable

#### Scenario: User skill delete succeeds
- **WHEN** a delete is requested for a user-created skill
- **THEN** the skill is removed

#### Scenario: Origin is exposed
- **WHEN** a client lists skills
- **THEN** each entry carries its origin, so a client can offer disable rather than delete for built-ins

#### Scenario: Built-in edit is refused
- **WHEN** a client submits an edit for a built-in skill
- **THEN** the API refuses the edit and leaves the skill unchanged

#### Scenario: Built-in delete control offers disable instead
- **WHEN** a client renders a built-in or installed-pack skill
- **THEN** Edit and Delete are unavailable while the global enable/disable control remains available
