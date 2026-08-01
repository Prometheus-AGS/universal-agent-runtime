# run-policy-resolution

## ADDED Requirements

### Requirement: Embedded resolution honors the global run policy

The embedded runtime SHALL resolve run policy using the full precedence
Global → Agent → Conversation → Turn, reading the Global scope from the
`run_policy.global` setting via a `SettingsManager` built from the runtime's
persistence. When no settings manager is available, the runtime SHALL fall back
to the legacy agent+conversation resolution without error.

#### Scenario: Global default model is applied on the embedded runtime

- **WHEN** `run_policy.global` sets a `ModelRoute` and the running agent and the
  conversation set no model
- **THEN** the resolved `EffectiveRunPolicy.model` equals the global `ModelRoute`
- **AND** the emitted `effective_run_policy` artifact reports that model with
  provenance attributing the model to the Global scope

#### Scenario: Agent and conversation still override the global default

- **WHEN** `run_policy.global` sets model A, the agent sets model B, and the
  conversation sets model C
- **THEN** the resolved `EffectiveRunPolicy.model` equals model C (conversation
  wins), matching the service path's precedence exactly

#### Scenario: Missing settings manager falls back without error

- **WHEN** the embedded runtime has no settings manager available
- **THEN** resolution succeeds using agent + conversation scopes only (legacy
  behavior) and does not error

### Requirement: Service and embedded resolvers produce identical results

The service (`AppState`) path and the embedded (`RunManager`) path SHALL resolve
run policy through one shared transport-free core, so that identical inputs
(global setting, agent artifact, conversation policy, turn override) yield an
identical `EffectiveRunPolicy`.

#### Scenario: Same inputs yield the same effective policy on both paths

- **WHEN** the same global setting, agent artifact, conversation policy, and turn
  override are supplied to the service resolver and to the embedded resolver
- **THEN** both return an `EffectiveRunPolicy` equal in model, agent_id,
  chat_mode, and provenance
