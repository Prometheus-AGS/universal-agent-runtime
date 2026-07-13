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

#### Scenario: CI tests a profile without local inference
- **WHEN** Cargo discovers integration-test targets for a feature profile that omits `local-models`
- **THEN** the Burn embedding integration target is skipped instead of asserting that an intentionally unavailable local model initializes

#### Scenario: Release tests local inference
- **WHEN** the authoritative `server-full` release suite runs
- **THEN** `local-models` enables and executes the Burn embedding integration target

#### Scenario: Alternate CI executes the shared live integration binary
- **WHEN** the selected feature profile omits `local-models`
- **THEN** the RAG ingest/retrieve case is not compiled or executed while the remaining live cases still run

#### Scenario: Server-full executes the shared RAG journey
- **WHEN** the authoritative `server-full` release suite runs
- **THEN** `local-models` enables and executes the RAG ingest/retrieve case

### Requirement: Stable Archives Satisfy Native Build Prerequisites
Every Stable platform archive builder SHALL install the native protobuf compiler before building the `server-full` release binary.

#### Scenario: Linux archive build compiles A2A protobuf
- **WHEN** a Linux x64 or arm64 Stable archive job builds the candidate
- **THEN** `protobuf-compiler` is installed before Cargo invokes the build script

#### Scenario: macOS archive build compiles A2A protobuf
- **WHEN** a macOS x64 or arm64 Stable archive job builds the candidate
- **THEN** Homebrew protobuf tooling is installed before Cargo invokes the build script

### Requirement: Installed MCP Boundary Evidence Is Bounded and Diagnostic
Installed-artifact certification SHALL wait a bounded interval for the configured MCP fixture to appear and SHALL record actionable status when it does not.

#### Scenario: MCP health becomes observable after readiness
- **WHEN** the installed server is ready but its configured MCP health projection is not yet observable
- **THEN** certification retries for a bounded interval while verifying that the candidate process remains alive

#### Scenario: MCP health remains unavailable
- **WHEN** the bounded MCP health interval expires
- **THEN** certification fails with the curl status, HTTP status, response body, and server log when the process exited
