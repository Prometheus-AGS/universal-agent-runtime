## ADDED Requirements

### Requirement: Root Workspace Lock Matches Every Committed Workspace Manifest

The repository SHALL commit a root dependency lock that is accepted by frozen
installation for every committed workspace manifest, including manifests
reached through workspace-owned submodules. A workspace manifest or submodule
advance MUST NOT be accepted as a certifiable source candidate while the root
lock is stale.

#### Scenario: Frozen installation accepts the committed workspace

- **WHEN** dependency installation runs in a clean checkout with frozen lock enforcement
- **THEN** installation SHALL succeed using the committed root lock
- **AND** the root lock content and digest SHALL remain unchanged

#### Scenario: Workspace submodule manifest advances without lock reconciliation

- **WHEN** a committed workspace submodule manifest no longer matches the committed root lock importer
- **THEN** frozen installation MUST exit non-zero and identify the stale lock
- **AND** source-bound build or certification evidence MUST NOT be minted from a non-frozen resolution

#### Scenario: Lock reconciliation preserves unrelated resolved versions

- **WHEN** the root lock is reconciled only to describe a committed workspace manifest advance
- **THEN** resolved versions unaffected by that manifest change SHALL remain unchanged
- **AND** any dependency upgrade MUST be authorized and verified as a separate dependency change
