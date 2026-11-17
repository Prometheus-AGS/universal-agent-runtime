# Spec Delta

## MODIFIED Requirements

### Requirement: Shipped inference and memory defaults require deliberate selection

An installable UAR profile SHALL NOT select an implicit remote model or remote memory embedder. With no explicit persisted/operator model choice, inference SHALL fail with an actionable missing-model outcome, except for the eligible Apple first-run bootstrap specified below. The shipped memory embedder SHALL resolve to a supported local provider or Off. Existing persisted operator choices SHALL remain authoritative after upgrade.

#### Scenario: No model selected

- **WHEN** a fresh shipped profile receives inference without an explicit configured model and is ineligible for Apple first-run bootstrap
- **THEN** it returns a clear missing-model outcome without remote model egress

#### Scenario: Local or disabled memory embedding

- **WHEN** a fresh shipped profile initializes memory without an operator override
- **THEN** its effective memory embedder is local or Off and no remote embedding request is made

#### Scenario: Existing operator choice survives upgrade

- **WHEN** an installation has a persisted operator-selected model or provider setting
- **THEN** upgrading to the safe-default release does not silently replace that selection

## ADDED Requirements

### Requirement: Eligible Apple first run uses the available local system model

A supported native Apple profile with no persisted model choice, explicit configuration or bootstrap opt-out SHALL default to the already available Apple on-device system model for setup assistance. Eligibility SHALL require the supported architecture, chip, OS, compiled adapter, Apple Intelligence state and runtime model availability. The resolved destination and bootstrap origin SHALL be visible and persist before routing.

#### Scenario: New eligible Mac
- **WHEN** a new Apple Silicon Mac has the supported OS and adapter, enabled Apple Intelligence, ready system assets and no prior choice or opt-out
- **THEN** setup assistance uses the available on-device system model without credentials, a UAR-initiated download or remote inference

#### Scenario: Unsupported or unavailable Apple model
- **WHEN** the device, chip, OS or adapter is unsupported, Apple Intelligence is disabled, or system assets are unavailable or still preparing
- **THEN** UAR exposes the specific reason, retains manual setup and an actionable missing-model state, and neither enables Apple Intelligence nor downloads assets nor selects a cloud fallback

#### Scenario: Existing choice or explicit opt-out
- **WHEN** a persisted model choice, explicit operator configuration or bootstrap opt-out exists
- **THEN** bootstrap retains it across restart and upgrade, including when the selected provider is unavailable

#### Scenario: Eligible mobile profile
- **WHEN** an iOS or iPadOS native UAR profile satisfies its own device, chip, OS, adapter and system-model availability checks
- **THEN** the same local bootstrap contract applies without assuming that every ARM device or Mac-qualified adapter is supported

### Requirement: Setup transitions remain authorized and recoverable

UAR SHALL persist observable setup state, current and proposed provider/model, operation identity, availability reason and consent scope. Downloads, provider switches, cloud transitions and credential acquisition or use SHALL each require explicit consent for their target and effects. Credentials SHALL remain outside prompts, logs and ordinary settings. Cancellation, retry, resume and fallback SHALL preserve prior routing and recheck authority.

#### Scenario: Operator declines stronger provider setup
- **WHEN** the bootstrap model proposes weights, an export, a service/provider switch, cloud inference including Private Cloud Compute, or credentials and the operator declines
- **THEN** no proposed effect occurs and the previous usable local or manual state remains available

#### Scenario: Interrupted setup
- **WHEN** setup is canceled, interrupted, resumed or retried
- **THEN** the UI shows idle, checking, awaiting-consent, acquiring, validating, ready, canceled, failed or unavailable state as applicable, reconciles prior effects before retry, and does not activate a partial provider or duplicate an uncertain operation

#### Scenario: System model becomes unavailable
- **WHEN** the active bootstrap model becomes unavailable
- **THEN** UAR shows the reason and offers manual setup or an explicitly authorized fallback without silently using a remote provider

### Requirement: Apple adapters preserve execution and lifecycle authority

Apple generation adapters SHALL participate in UAR provider routing with explicit supported capabilities and execution placement. UAR SHALL remain the sole agent and tool execution authority. Custom-model acquisition, integrity, installation, load leases, unload and cancellation SHALL remain owned by flint-loader; Apple system assets remain OS-owned. Direct native inference SHALL use the same lifecycle authority and SHALL NOT introduce another agent/tool loop.

#### Scenario: Tool-capable model
- **WHEN** an Apple-backed model proposes a tool call
- **THEN** it returns the proposal to UAR for policy, approval, execution, audit and continuation; an SDK session never dispatches an independent tool loop

#### Scenario: Unsupported model or capability
- **WHEN** a requested model format, quantization, hardware resource or capability lacks declared support
- **THEN** admission fails with a typed actionable outcome rather than claiming that custom-provider support makes every model compatible

### Requirement: Generation selection never changes embedding identity

Generation-provider selection SHALL NOT change knowledge or memory embedding registry identity, revision, dimensions, preprocessing or index binding. An embedding change SHALL require its own explicit selection and approved reindex plan, including affected indexes, resume behavior and cutover. Incompatible vectors SHALL NOT be mixed or queried together.

#### Scenario: Change generation provider
- **WHEN** setup activates Apple, MLX, Core AI or a flint-loader-backed generation model
- **THEN** existing embedding selections and indexes remain bound to their original identities and no automatic re-embedding begins

#### Scenario: Explicit embedder migration
- **WHEN** the operator separately approves a different embedding identity or dimensions
- **THEN** UAR exposes the reindex progress and compatibility boundary and cuts over only after the new index is ready, retaining recoverable prior state
