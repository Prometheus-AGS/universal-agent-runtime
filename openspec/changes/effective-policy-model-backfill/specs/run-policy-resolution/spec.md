# run-policy-resolution

## ADDED Requirements

### Requirement: The effective-run-policy model route reports the executing model

The runtime SHALL ensure the `effective_run_policy` artifact reports the model
that will actually execute the run. When the resolved `EffectiveRunPolicy.model`
is absent or has an empty `provider_id` or `model_id` — as happens when the
agent, conversation, and global scopes all defer to the registry default — the
runtime SHALL backfill the route from the resolved default model
(`resolve_default_model()`: the provider-registry default, or the configured
`llm_config.model`) before emitting the artifact. A route that already specifies
a non-empty provider and model SHALL NOT be overwritten, preserving
Global → Agent → Conversation → Turn precedence.

#### Scenario: Built-in agent with an empty provider default reports the registry model

- **WHEN** a run starts for an agent whose `policy.provider.default` is empty and
  neither the conversation nor the global scope sets a model
- **THEN** the emitted `effective_run_policy` artifact's `model.provider_id` and
  `model.model_id` equal the registry-default provider and model
- **AND** on an embedded host that default is the on-device local provider, so
  the provenance surface can render `agent · provider/model` rather than a blank
  model

#### Scenario: A fully resolved route is not overwritten by the default

- **WHEN** the agent, conversation, or global scope resolves a non-empty
  `ModelRoute`
- **THEN** the emitted `effective_run_policy` artifact reports that resolved route
  unchanged, and the registry default is not applied
