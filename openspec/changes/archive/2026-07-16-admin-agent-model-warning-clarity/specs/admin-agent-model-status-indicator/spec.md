## ADDED Requirements

### Requirement: Three-way model resolution status in the Admin Agents list
The Admin Agents list SHALL distinguish an agent's model-resolution
status as one of three states — fully configured, deferring to a working
system default, or genuinely unresolved — rather than a single binary
warning condition.

#### Scenario: Agent with an explicit per-agent override shows no icon
- **WHEN** an agent's `policy.provider.default` has both `provider` and
  `model` set
- **THEN** the Admin Agents list shows no status icon next to that agent

#### Scenario: Agent deferring to a working system default shows a neutral indicator
- **WHEN** an agent's `policy.provider.default` has no `provider` or no
  `model` set, and the system-wide provider registry has both a default
  provider id and that provider has a default model configured
- **THEN** the Admin Agents list shows a neutral (non-warning) indicator
  next to that agent, labeled to convey it is using the system default

#### Scenario: Agent with no resolution path shows the warning indicator
- **WHEN** an agent's `policy.provider.default` has no `provider` or no
  `model` set, and either no system-wide default provider is configured
  or the default provider has no default model
- **THEN** the Admin Agents list shows the existing amber warning
  indicator next to that agent, unchanged from current behavior
