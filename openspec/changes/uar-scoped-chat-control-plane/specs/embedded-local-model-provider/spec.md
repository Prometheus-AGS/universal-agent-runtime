## ADDED Requirements

### Requirement: Embedded host model registration
UAR SHALL allow an embedded host to register local model providers with stable model identifiers, capabilities, context limits, lifecycle state, and diagnostics.

#### Scenario: Host registers a local model
- **WHEN** a desktop or mobile host registers a ready local model provider
- **THEN** the model appears in UAR routing metadata with its declared capabilities and backend diagnostics

### Requirement: Local inference remains UAR governed
UAR MUST perform context assembly, retrieval, skill selection, MCP and tool governance, persistence, and event normalization before and during a registered local-provider run.

#### Scenario: Local agent tool loop
- **WHEN** a local model emits a valid call to an allowed MCP tool
- **THEN** UAR validates and executes the tool and continues the same local-model run with the tool result

### Requirement: No implicit cloud fallback
UAR SHALL NOT route an explicitly local run to a cloud provider unless the user explicitly enables and approves that fallback.

#### Scenario: Local capability mismatch
- **WHEN** an explicitly local model cannot satisfy required capabilities
- **THEN** UAR returns a visible preflight error instead of invoking a cloud model

### Requirement: Local model lifecycle is cancellable and observable
Registered local providers SHALL expose preparation, streaming, cancellation, unload, and diagnostic operations through normalized UAR lifecycle events.

#### Scenario: Cancel during model preparation
- **WHEN** the user cancels while a local model is downloading, verifying, or loading
- **THEN** UAR requests provider cancellation and emits a terminal cancelled or actionable cancelling state
