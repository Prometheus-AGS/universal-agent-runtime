## MODIFIED Requirements

### Requirement: Three-way model resolution status in the Admin Agents list
The Admin Agents list SHALL distinguish an agent's resolved model-routing status as one of three states — fully configured, deferring to a working system default, or genuinely unresolved — rather than a single binary warning condition. It SHALL explain every displayed non-explicit status on hover and keyboard focus, and SHALL NOT report an unresolved route until the provider registry needed to resolve that status has loaded successfully.

#### Scenario: Agent with an explicit per-agent override shows no icon
- **WHEN** an agent's `policy.provider.default` has both `provider` and `model` set
- **THEN** the Admin Agents list shows no status icon next to that agent

#### Scenario: Agent deferring to a working system default shows a neutral indicator
- **WHEN** an agent's `policy.provider.default` has no `provider` or no `model` set, and the loaded system-wide provider registry has both a default provider id and that provider has a default model configured
- **THEN** the Admin Agents list shows a neutral non-warning indicator next to that agent
- **AND** hovering or focusing the agent row exposes a tooltip that names the effective default provider and model

#### Scenario: Agent with no resolution path shows the warning indicator
- **WHEN** an agent's `policy.provider.default` has no `provider` or no `model` set, and the loaded system-wide provider registry confirms that either no default provider is configured or the default provider has no default model
- **THEN** the Admin Agents list shows the amber warning indicator next to that agent
- **AND** hovering or focusing the agent row exposes a tooltip that explains the missing route and tells the operator to assign an agent model or configure a system default

#### Scenario: Provider registry is still loading
- **WHEN** an agent has no explicit provider and model and the provider registry has not completed its initial load
- **THEN** the Admin Agents list indicates that model configuration is being checked
- **AND** it does not show the confirmed unresolved-route warning

#### Scenario: Provider registry cannot be verified
- **WHEN** an agent has no explicit provider and model and the initial provider-registry load fails
- **THEN** the Admin Agents list shows an amber status indicator distinct from a confirmed missing route
- **AND** hovering or focusing the agent row exposes a tooltip that says model availability could not be verified
