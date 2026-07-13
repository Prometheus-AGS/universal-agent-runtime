## ADDED Requirements

### Requirement: Candidate CI Lints the Supported Product Without Dependencies
The exact-source CI gate consumed by release certification SHALL lint the locked `server-full` UAR library targets and SHALL exclude dependency-owned Clippy lints.

#### Scenario: Vendored dependency has its own denied warnings
- **WHEN** a vendored path dependency emits Clippy warnings under its crate-local policy
- **THEN** the UAR CI lint step excludes dependency linting while continuing to lint the supported `server-full` UAR library

#### Scenario: Supply-chain certification selects CI evidence
- **WHEN** supply-chain certification validates a successful exact-SHA CI run
- **THEN** a static workflow contract proves that the CI run used the authoritative locked `server-full` Clippy and Cargo check commands
