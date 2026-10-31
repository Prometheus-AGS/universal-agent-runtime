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
