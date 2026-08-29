## ADDED Requirements

### Requirement: Vendored runtime inputs have immutable provenance
Each refreshed vendored runtime input SHALL identify an exact upstream commit reachable from its authoritative remote, and generated provider-catalog artifacts SHALL be reproducible from that pinned input without network-dependent content changes.

#### Scenario: Vendor provenance is verified
- **WHEN** a release source archive is prepared
- **THEN** every refreshed vendor pointer or curated snapshot names a remotely reachable immutable commit and the regenerated provider catalog matches a second deterministic generation

#### Scenario: Offline archive rebuilds the refreshed inputs
- **WHEN** the prepared source archive is extracted into a network-isolated build environment
- **THEN** the locked release build completes using the archived vendor sources and generated catalog without fetching their upstream repositories

#### Scenario: Operator-local files stay outside the archive
- **WHEN** the release checkout contains ignored credentials, private deployment variables, build outputs, or other untracked operator files
- **THEN** source packaging selects tracked repository and recursive-submodule inputs only and excludes those local files from the archive
