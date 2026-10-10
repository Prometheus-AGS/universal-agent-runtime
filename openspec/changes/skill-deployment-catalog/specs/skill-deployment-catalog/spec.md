## ADDED Requirements
### Requirement: Trusted stored skill deployment catalog
UAR SHALL expose GET /api/uar/skills/deployment-catalog and its existing /api/skills alias as an authenticated Admin Read endpoint with schemaVersion 1. It SHALL return current registry entries including enabled and tombstoned state, availability and explicit missing-metadata reasons. It SHALL expose the exact portable domain SkillRef separately from private installedLocation binding metadata. It SHALL return stored identity only without filesystem scans or guessed digests. required=true and config={} SHALL be documented authoring defaults rather than discovered execution settings. Existing SkillResponse/list and Owner-match responses SHALL remain unchanged.

#### Scenario: Complete stored artifact
- **WHEN** an authenticated caller supplies the configured admin key and reads a complete enabled non-tombstoned skill
- **THEN** the entry contains its stored id/version/digest/entrypoint/requiredTools, an available status and a separate private installedLocation

#### Scenario: Missing metadata or unavailable artifact
- **WHEN** a current entry lacks required artifact metadata, is disabled or is tombstoned
- **THEN** it reports unavailable with explicit reasons and does not invent missing fields or an identity

#### Scenario: Caller lacks admin authority
- **WHEN** the principal is anonymous or the supplied admin key is absent, wrong or not configured
- **THEN** the endpoint refuses the request without returning installed locations, even when ordinary settings mutation authentication is disabled

### Requirement: Discovery grants no authority
Catalog selection SHALL NOT authorize skill execution, required tools or private binding activation. Canonical binding resolution and execution admission SHALL continue validating exact SkillRef, installedLocation, enabled and tombstone state at use. Capability inventory and OpenAPI SHALL describe the endpoint as Admin Read.

#### Scenario: Artifact changes after discovery
- **WHEN** a selected skill changes identity or enabled/tombstone state before binding or execution
- **THEN** canonical resolution refuses required stale identity through existing diagnostics rather than accepting catalog discovery as authority
