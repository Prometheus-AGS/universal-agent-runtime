## MODIFIED Requirements

### Requirement: Configuration routes real work
Provider credentials/configuration, model defaults, model enablement, and scoped run policy SHALL round-trip through their owning UAR APIs and SHALL determine a real routed request, including a registered embedded local model when Local mode is selected.

#### Scenario: Configure and route
- **WHEN** an administrator configures a provider and selects a default model
- **THEN** a subsequent routed request uses the persisted selection and the UI displays the resulting decision

#### Scenario: Secret retrieval
- **WHEN** provider or security settings are reloaded
- **THEN** stored secrets are never returned in plaintext

#### Scenario: Disabled model is unavailable
- **WHEN** a provider or model is disabled in UAR configuration
- **THEN** it is excluded from enabled-only catalogs and cannot be selected by a new run

#### Scenario: Local model route
- **WHEN** a run explicitly selects an eligible registered local model
- **THEN** UAR invokes that provider and records the model, backend, and capability decision in the effective policy
