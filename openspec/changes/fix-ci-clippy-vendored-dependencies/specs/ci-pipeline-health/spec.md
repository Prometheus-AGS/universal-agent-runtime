## ADDED Requirements

### Requirement: Candidate CI Lints the Supported Product Without Dependencies
The exact-source CI gate consumed by release certification SHALL lint the locked `server-full` UAR library targets and SHALL exclude dependency-owned Clippy lints.

#### Scenario: Vendored dependency has its own denied warnings
- **WHEN** a vendored path dependency emits Clippy warnings under its crate-local policy
- **THEN** the UAR CI lint step excludes dependency linting while continuing to lint the supported `server-full` UAR library

#### Scenario: Supply-chain certification selects CI evidence
- **WHEN** supply-chain certification validates a successful exact-SHA CI run
- **THEN** a static workflow contract proves that the CI run used the authoritative locked `server-full` Clippy and Cargo check commands

### Requirement: Candidate Workflows Preserve Deterministic Prerequisites
Release and resilience workflows SHALL preserve deterministic test configuration, install required native build tools, and allow the runtime's documented graceful-shutdown budget to elapse before escalation.

#### Scenario: Release tests use recorded fixtures
- **WHEN** the release workflow runs deterministic BDD scenarios
- **THEN** its configured model matches the recorded fixture model

#### Scenario: Resilience builds the server-full binary
- **WHEN** the resilience workflow compiles the installed archive
- **THEN** `protoc` and the required Linux native build dependencies are installed

#### Scenario: Container receives SIGTERM
- **WHEN** the non-root resilience job stops the healthy container
- **THEN** Docker allows more than the runtime's 30-second graceful-shutdown budget before escalating

### Requirement: Optional-Transport Tests Respect Feature Profiles
Integration tests that directly import optional transport dependencies SHALL declare their required Cargo features.

#### Scenario: CI tests a profile without A2A transport
- **WHEN** Cargo discovers integration-test targets for a feature profile that omits `a2a-transport`
- **THEN** the gRPC integration target is skipped instead of failing compilation on intentionally unavailable tonic APIs

#### Scenario: Release tests the server-full product
- **WHEN** the authoritative `server-full` release suite runs
- **THEN** `a2a-transport` enables and executes the gRPC integration target

#### Scenario: CI compiles the Postgres credential store
- **WHEN** the alternate CI feature profile enables `postgres-backend`
- **THEN** the credential-store implementation compiles without unused-import warnings
