# Spec Delta

## ADDED Requirements

### Requirement: Authenticated knowledge reupload recovers current-principal content

**Stable task:** S2. An operator with retained original source bytes SHALL be able to authenticate as the verified current principal, create or select a principal knowledge base, and reupload those bytes through the supported knowledge API. The resulting durable job and indexed content SHALL remain in that principal's namespace after a joined restart. This capability SHALL NOT adopt, rename, or mutate legacy knowledge rows.

#### Scenario: Reupload indexes and retrieves after restart

- **WHEN** an authenticated principal uploads retained source bytes through `POST /api/knowledge/{id}/documents`, the durable job reaches indexed state, and the runtime performs a joined restart
- **THEN** that principal can retrieve the indexed content through `POST /api/knowledge/{id}/search`
- **AND** the retrieved document and job retain their principal-scoped identities

#### Scenario: Legacy rows remain preserved and isolated

- **WHEN** legacy knowledge rows have the same subject-like value as a verified principal
- **THEN** authenticated listing, document lookup, retry, and search do not expose or adopt the legacy rows
- **AND** reupload creates new principal-scoped records without changing legacy knowledge-base, document, chunk, or job identifiers

#### Scenario: Original source is unavailable

- **WHEN** a legacy row is present but its original source bytes are unavailable
- **THEN** the operational recovery documentation identifies that reupload cannot recover that document
- **AND** the product does not claim recovered content, synthesize source bytes, or perform a privileged legacy import

