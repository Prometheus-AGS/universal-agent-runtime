# Spec Delta

## ADDED Requirements

### Requirement: Shipped inference and memory defaults require deliberate selection

An installable UAR profile SHALL NOT select an implicit remote model or remote memory embedder. With no explicit persisted/operator model choice, inference SHALL fail with an actionable missing-model outcome instead of silently routing to a bundled model. The shipped memory embedder SHALL resolve to a supported local provider or Off. Existing persisted operator choices SHALL remain authoritative after upgrade.

#### Scenario: No model selected

- **WHEN** a fresh shipped profile receives inference without an explicit configured model
- **THEN** it returns a clear missing-model outcome without remote model egress

#### Scenario: Local or disabled memory embedding

- **WHEN** a fresh shipped profile initializes memory without an operator override
- **THEN** its effective memory embedder is local or Off and no remote embedding request is made

#### Scenario: Existing operator choice survives upgrade

- **WHEN** an installation has a persisted operator-selected model or provider setting
- **THEN** upgrading to the safe-default release does not silently replace that selection

### Requirement: Admin reads one current effective settings aggregate

UAR SHALL expose a stable documented read-only effective-settings path whose response matches the selected Kubo acceptance schema or a documented stable shape with a recorded Kubo translation. It SHALL contain a nonsecret aggregate `revision` and current `llm.provider_host`, `knowledge_base_embedder.provider_host`, and `memory_embedder.provider_host`; supported governance, native-tool and MCP fields SHALL reflect their owning authorities. A change in any exposed effective value SHALL change the documented aggregate revision/hash. No secret, credential or raw private configuration value SHALL appear.

#### Scenario: Current provider and governance values

- **WHEN** a Bearer admin reads effective settings after a provider or governance change
- **THEN** the aggregate reflects current live routing/governance rather than stale per-key stored rows and its revision/hash changes according to the pinned policy

#### Scenario: Local embedder normalization

- **WHEN** a supported local or disabled embedder is effective
- **THEN** its provider-host value follows the selected documented schema mapping and does not invent a remote endpoint

#### Scenario: Secret-free response

- **WHEN** configured providers or MCP servers contain credentials
- **THEN** the effective response exposes only allowed nonsecret host/capability information
