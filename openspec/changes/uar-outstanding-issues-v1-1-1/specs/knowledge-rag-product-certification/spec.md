# Spec Delta

## ADDED Requirements

### Requirement: Release evidence distinguishes local coverage from published-child coverage

**Stable tasks:** function-first child FC1–FC9; parent V2/P1–P3. Release
certification SHALL complete all revised implementation and documentation before
any verification, record a new whole-phase source/fixture inventory, and obtain
the operator's exact instruction for preparation, review and acceptance actions.
An implementation-complete inventory SHALL NOT grant execution authority.

#### Scenario: Local candidate retains consequential functional coverage

- **WHEN** the operator authorizes local function-first acceptance after preparation and independent review
- **THEN** the runner executes all six reduced groups: binding, streams, mcp-outcomes, credentials, recovery and combined
- **AND** the four existing native binding-history, incidental-error, SDK-credential and in-memory migration supplements retain separate source-evidence labels
- **AND** folded scenario labels retain their consequential assertions, all supported credential backend transaction/journal decisions, compiler profile controls and genuine supervisor recovery
- **AND** missing or unrun required observations are blocked, not skipped passes

#### Scenario: Each published child proves the explicitly bounded release program

- **WHEN** an immutable arm64 or amd64 published child is selected after authorized P1 publication and that child's fresh preparation
- **THEN** the runner executes exactly streams and recovery locally against that unchanged child
- **AND** the receipt binds actual source, fixture, child/index digest, preparation, publication provenance and native/emulated identity
- **AND** release documentation explicitly states that budget refusal, compiler restriction, credential migration and MCP failure handling were not directly repeated on that published architecture and remain local-candidate evidence only
- **AND** no native-source result, local tag or other architecture substitutes for that child's required observations

#### Scenario: Prepared runtime identity cannot replace qualified exclusion evidence

- **WHEN** a recovery invocation uses a prebuilt image
- **THEN** its image ID, platform, source and fixture identity match the independently retained preparation receipt and same-image supervisor identity receipt
- **AND** a genuinely operator-qualified controller, matching controller source/mechanism/IDs, actual predecessor journal registration, queued and active work, distinct replacement fence and real physical boot transition exist for that invocation
- **AND** the recovery path records and consumes one fresh proof while retaining uncertainty, denying replay and preserving accounting
- **AND** image identity or a rewritten old receipt never establishes controller qualification or fresh physical exclusion

#### Scenario: Changed implementation invalidates preparation and permission

- **WHEN** source, fixtures or implementation configuration change after the inventory or image preparation
- **THEN** the gate closes and renewed whole-phase inventory, exact operator preparation instruction, fresh same-image staging and exact acceptance instruction are required
- **AND** a consumed proof or previous architecture's supervisor setup cannot be reused for a later recovery run

#### Scenario: Missing published preparation remains an explicit pending artifact

- **WHEN** P1 has not supplied actual immutable child digests or a child's fresh qualified preparation is incomplete
- **THEN** its manifest remains a non-executable template and P2 remains pending
- **AND** no invented digest, standalone unapproved preflight or rebuilt replacement image is used to satisfy the requirement

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

