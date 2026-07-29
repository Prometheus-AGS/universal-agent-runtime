# embedded-admin-surface

## ADDED Requirements

### Requirement: Embedded runtime exposes per-conversation policy administration

The embedded SDK `Runtime` SHALL let a host read, write, and delete a
conversation-scoped run policy without an HTTP service: `save_conversation_policy`,
`get_conversation_policy`, and `delete_conversation_policy`, delegating to the
persistence layer. A model set on a conversation policy is the Conversation scope
of the Global → Agent → Conversation → Turn precedence and SHALL override the
agent and global defaults for that conversation only.

#### Scenario: A conversation-scoped model overrides the lower scopes

- **WHEN** a host saves a conversation policy whose `model` is provider `openai`,
  model `gpt-4o`, for a conversation whose agent and global scopes set no model
- **THEN** `get_conversation_policy` returns that policy
- **AND** the conversation's effective policy reports `openai`/`gpt-4o`

#### Scenario: Deleting the conversation policy reverts to the lower scopes

- **WHEN** the conversation policy is deleted
- **THEN** the conversation's effective policy no longer reports the override and
  falls back to the agent/global scopes and the registry-default backfill

### Requirement: Embedded runtime resolves a conversation's effective configuration

The embedded SDK `Runtime` SHALL expose `effective_config(conversation_id)`
returning the resolved agent, the stored requested policy (if any), and the
effective run policy after full-precedence resolution and model backfill —
mirroring the service path's effective-config endpoint.

#### Scenario: Effective config with no stored policy resolves the default agent and model

- **WHEN** `effective_config` is called for a conversation with no stored policy
- **THEN** the result's `requested_policy` is absent
- **AND** the result's `effective_policy.model` is the registry-default route
  (the on-device provider on embedded hosts)
