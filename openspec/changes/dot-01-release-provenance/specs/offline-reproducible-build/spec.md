# Spec Delta

## ADDED Requirements

### Requirement: Release artifacts have traceable locked inputs

Release packaging SHALL use approved immutable source inputs, frozen dependency installation and traceable build provenance. Unresolved policy drift SHALL prevent certification.

#### Scenario: Unresolved pin policy

- **WHEN** declared policy and effective dependency sources disagree
- **THEN** packaging certification is blocked pending recorded operator resolution, without silently editing policy or accepting drift

#### Scenario: Reproducible container inputs

- **WHEN** an approved source revision is packaged for either supported Linux architecture
- **THEN** frozen JavaScript installs, locked Rust resolution and required embedded schemas are used and immutable source provenance is recorded


