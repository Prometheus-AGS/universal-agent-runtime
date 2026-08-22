# provider-model-settings-certification Specification

## Purpose
TBD - created by archiving change certify-provider-model-settings-flow. Update Purpose after archive.
## Requirements
### Requirement: Configuration routes real work
Provider credentials/configuration, model defaults, and settings SHALL round-trip through their owning APIs and SHALL determine a real routed request. Values supported by resolved runtime configuration SHALL be accepted consistently by settings initialization and writes. When durable settings persistence is configured, a default-provider selection SHALL be published to live routing only after its durable selection succeeds. A deployment without a configured settings manager MAY retain registry-only default selection.

#### Scenario: Configure and route
- **WHEN** an administrator configures a provider and selects a default model
- **THEN** a subsequent routed request uses the persisted selection and the UI displays the resulting decision

#### Scenario: Secret retrieval
- **WHEN** provider or security settings are reloaded
- **THEN** stored secrets are never returned in plaintext

#### Scenario: Supported local memory provider initializes settings
- **WHEN** resolved configuration selects the supported `local` memory embedding provider and settings initialization runs
- **THEN** initialization accepts the value and seeds settings that follow the memory namespace, including the default LLM provider setting

#### Scenario: Unknown memory provider remains invalid
- **WHEN** resolved configuration selects an unsupported memory embedding provider and settings initialization runs
- **THEN** initialization fails schema validation instead of accepting an unrestricted provider string

#### Scenario: Missing default provider changes no state
- **WHEN** an administrator requests an unregistered provider as the default
- **THEN** the API returns not found and neither the durable nor live default changes

#### Scenario: Failed persistence changes no live state
- **WHEN** an administrator selects a registered provider but the durable default-provider write fails
- **THEN** the API returns an internal error and both the durable and live defaults remain at their prior values

#### Scenario: Successful default selection survives reconstruction
- **WHEN** an administrator selects a registered provider and the durable default-provider write succeeds
- **THEN** the API returns success, live routing uses that provider, and a fresh settings manager over the same persistence layer reads the same default provider
