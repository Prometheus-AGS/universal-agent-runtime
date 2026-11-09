## Purpose
Define local exact-source Rust library compilation and preserved regression evidence. Compilation alone does not establish installed-runtime, recovery, provider or release qualification.

## ADDED Requirements

### Requirement: Exact-source library test baseline
Local acceptance SHALL compile the library test target with locked dependency resolution and no default features for minimal, server-full and server-full,postgres-backend on one unchanged committed candidate. Temporary source patches SHALL NOT count as candidate acceptance.

#### Scenario: Compilable candidate
- **WHEN** the local acceptance safeguard runs on a clean committed candidate with matching dependency checkouts
- **THEN** it records source/tree/toolchain and each feature profile and compiles each library test harness serially
- **AND** it reports failure if any command fails or source changes

#### Scenario: Dirty evidence candidate
- **WHEN** acceptance mode is invoked with uncommitted or mismatched dependency sources
- **THEN** it refuses before compiling and does not claim committed-candidate acceptance

### Requirement: Preserve scoped regression evidence
The baseline SHALL retain and execute existing presentation-history replay/provenance and embeddings/OpenAPI assertions at the completed phase boundary. Compilation and regression results SHALL remain distinct from installed-runtime certification.

#### Scenario: Local pre-commit use
- **WHEN** relevant Rust source or manifest changes are committed using the existing hook
- **THEN** the hook invokes the library compilation safeguard
- **AND** working-tree compilation alone does not produce a release qualification claim

### Requirement: Current fixture construction without weakened assertions
Integration fixtures SHALL construct the current production types and preserve their existing behavior assertions. Shared fixture code that is intentionally unused in one including target SHALL use a reasoned lint expectation scoped to that target; warnings SHALL NOT be hidden with blanket allowances or compiler flags.

#### Scenario: Runtime embedding backend becomes swappable
- **WHEN** the matcher requires a swappable backend but a test owns a concrete or trait-object backend
- **THEN** the fixture adapts that same backend to the current wrapper contract
- **AND** its embedding behavior and assertions remain unchanged

#### Scenario: Test producer send fails
- **WHEN** the fixture producer cannot send its release notification
- **THEN** the failure reaches the existing anyhow result with its declared message
- **AND** the fixture does not discard the failed send to obtain a passing compile

### Requirement: Baseline does not revive historical opt-in qualification
The baseline SHALL compile test targets without executing historical installed/recovery qualifiers. Required regression assertions and the library inventory SHALL retain their real execution evidence. A development-only control SHALL retain its original backend and assertions behind its explicit existing feature; its qualifier SHALL reject missing required features before parent-side effects, never fabricate a passing receipt.

#### Scenario: Shipping feature profile compiles legacy fixture target
- **WHEN** the shipping feature profile does not export the development memory backend
- **THEN** the legacy target still compiles with an explicit required-feature precondition
- **AND** the baseline does not execute or count that qualifier as a passing recovery case

#### Scenario: Retained development control compilation
- **WHEN** approval_recovery is compiled with server-full, postgres-backend and in-memory-backend
- **THEN** its original memory deadline assertion block is compiled
- **AND** no historical qualifier or credential setup is executed by the compile-only check

### Requirement: Warning-free inherited-input compilation preserves ownership
The affected sidecar compilation SHALL use the pinned supported Unix asynchronous descriptor registration contract without deprecation suppression. The inherited input SHALL keep its sole owned file, existing nonblocking/read semantics and error propagation. Windows behavior SHALL remain unchanged by this Unix repair.

#### Scenario: Register owned inherited Unix input
- **WHEN** the sidecar transfers its owned nonblocking inherited input into asynchronous registration
- **THEN** registration uses the supported API with its lifetime/descriptor invariants justified at the unsafe boundary
- **AND** registration failure propagates its I/O cause and releases the owned file without a fabricated success
- **AND** the affected compile-only approval/sidecar gate emits no Rust warnings
