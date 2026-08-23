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

### Requirement: YAML-defined providers are catalog-enriched before registration
Before a YAML-defined provider is first stored, UAR SHALL resolve omitted catalog-backed API-key environment names, endpoint URLs, provider metadata, and model metadata through the same embedded-catalog enrichment used by the LLM configuration path. Persisted API/UI-created provider settings remain authoritative after initial seeding.

#### Scenario: YAML provider omits catalog metadata
- **WHEN** a known provider is declared in YAML without duplicating its embedded catalog URL, key variable, or model metadata
- **THEN** UAR enriches the definition before first registration and does not overwrite a later persisted operator edit

### Requirement: Native provider seed contains only concrete supported models
The native seed SHALL include the discovered local OpenAI proxy inventory, Kimi Coding `kimi-for-coding/k3`, MiniMax `minimax/MiniMax-M3`, and credential-matched Alibaba/Qwen, Z.AI/GLM, and Moonshot catalog models. It SHALL exclude tool-only credentials and providers lacking a concrete endpoint/model.

#### Scenario: Local proxy inventory is refreshed
- **WHEN** `http://127.0.0.1:8181/v1/models` returns a model list
- **THEN** missing proxy model entries are merged into YAML without replacing existing provider, model, or default selections

### Requirement: Provider bootstrap preserves persistent settings authority
Native bootstrap SHALL merge missing YAML entries and SHALL NOT replace API/UI-created provider settings or the selected default model held in persistent storage.

#### Scenario: Persisted provider differs from catalog default
- **WHEN** the operator has edited a provider through the API or UI after initial seed
- **THEN** a service restart retains the persisted setting rather than restoring the YAML/catalog default
