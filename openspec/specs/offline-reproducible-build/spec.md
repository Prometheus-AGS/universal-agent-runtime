# offline-reproducible-build Specification

## Purpose
TBD - created by archiving change make-build-offline-reproducible. Update Purpose after archive.

## Requirements

### Requirement: Release source builds offline
A release source archive plus lockfile SHALL build every supported release bundle without network access.

#### Scenario: Clean offline build
- **WHEN** caches are empty, networking is disabled, and the source archive is unpacked
- **THEN** `cargo build --locked --offline` succeeds using versioned inputs only

### Requirement: Generated inputs are traceable
Every embedded catalog/model snapshot SHALL record its upstream source, retrieval date, digest and refresh command.

#### Scenario: Catalog refresh
- **WHEN** a maintainer refreshes the catalog
- **THEN** the resulting diff and digest are reviewable before commit

### Requirement: Release artifacts have traceable locked inputs

Release packaging SHALL use approved immutable source inputs, frozen dependency installation and traceable build provenance. Unresolved policy drift SHALL prevent certification.

#### Scenario: Unresolved pin policy

- **WHEN** declared policy and effective dependency sources disagree
- **THEN** packaging certification is blocked pending recorded operator resolution, without silently editing policy or accepting drift

#### Scenario: Reproducible container inputs

- **WHEN** an approved source revision is packaged for either supported Linux architecture
- **THEN** frozen JavaScript installs, locked Rust resolution and required embedded schemas are used and immutable source provenance is recorded
