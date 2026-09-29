## MODIFIED Requirements

### Requirement: Host run credentials supply model-specific context limits
For a host-supplied run credential with a selected `default_model`, the runtime SHALL accept a bounded optional `context_window` and SHALL use it only when the run's selected provider and model exactly match that credential. A mismatched or absent hint SHALL leave configured and embedded catalog behavior unchanged. The hint SHALL remain run-local and SHALL NOT be written into durable provider configuration.

#### Scenario: External liter alias absent from UAR's model catalog
- **WHEN** a host starts a run with a credential for `the-boss-gateway/kimi-for-coding` and its known context window
- **THEN** the run sizes its context using that model-specific window rather than UAR's unknown-model fallback

#### Scenario: Another model is selected
- **WHEN** the run selects a different provider or model than the credential's `default_model`
- **THEN** the credential's context-window hint does not affect that run's context sizing

### Requirement: Host gateways own provider-specific request translation
For a host-supplied OpenAI-compatible connection, the runtime SHALL send the selected model alias unchanged and SHALL NOT add model-dialect-specific fields to the request body. A slash within a gateway alias SHALL NOT be treated as a UAR provider separator unless it matches the selected credential's exact provider prefix.

#### Scenario: liter-llm Kimi alias
- **WHEN** a host selects `kimi-for-coding` through its liter-llm gateway
- **THEN** the gateway receives that alias without UAR's Kimi-specific `thinking` field

#### Scenario: Slash-containing gateway alias
- **WHEN** a host selects `custom/kimi` through provider `the-boss-gateway`
- **THEN** UAR sends `custom/kimi` and retains that exact model for the run's context-window hint
