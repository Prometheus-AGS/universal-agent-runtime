## ADDED Requirements

### Requirement: Skill availability is governed by durable scoped disables
Skills from installed packs SHALL be non-deletable and disableable at global,
per-agent and per-conversation scope; disable state SHALL survive restart.

#### Scenario: Pack skill delete attempt
- **WHEN** a delete is requested for a pack- or builtin-origin skill via API or UI
- **THEN** the operation is rejected and the UI offers disable instead

#### Scenario: Builtin disable survives restart
- **WHEN** a builtin skill is disabled globally and the runtime restarts
- **THEN** the skill remains disabled after startup re-registration

#### Scenario: Conversation-scope disable gates activation
- **WHEN** a skill is disabled for a conversation via session config
- **THEN** intent matching never activates it for that conversation
