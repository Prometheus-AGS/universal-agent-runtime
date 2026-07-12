## ADDED Requirements

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
