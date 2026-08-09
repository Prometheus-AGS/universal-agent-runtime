## ADDED Requirements

### Requirement: Usable loopback parent API
The purpose-built sidecar MUST allow its supervising parent to call the loopback HTTP API after readiness without requiring an unrelated standalone-server JWT by default. This default MUST NOT change standalone UAR authentication, and an explicit sidecar JWT environment setting MUST remain authoritative.

#### Scenario: Supervised default
- **GIVEN** the purpose-built sidecar has forced loopback-only binding
- **AND** neither `UAR_SECURITY__JWT_REQUIRED` nor `JWT_REQUIRED` is explicitly configured
- **WHEN** the sidecar loads its application configuration
- **THEN** JWT enforcement is disabled for the sidecar process
- **AND** the parent can call capability, model, and completion endpoints after `READY`

#### Scenario: Explicit JWT override
- **GIVEN** an operator explicitly configures `UAR_SECURITY__JWT_REQUIRED` or `JWT_REQUIRED`
- **WHEN** the sidecar loads its application configuration
- **THEN** the sidecar preserves that explicit authentication policy

### Requirement: Pre-runtime process configuration
The purpose-built sidecar MUST finish its process-environment overrides and reserve its
ephemeral listener before creating the multithreaded async runtime. Runtime initialization
MUST NOT race process-global configuration writes.

#### Scenario: Runtime starts after bootstrap
- **GIVEN** the sidecar needs to force loopback, JSON logging, an ephemeral port, and its default authentication policy
- **WHEN** the sidecar process starts
- **THEN** it applies every process-environment override during synchronous single-threaded bootstrap
- **AND** only then creates the async runtime and initializes the HTTP application
