## ADDED Requirements

### Requirement: Every Active pnpm Workspace Root Has a Frozen-Compatible Lock

The repository SHALL commit a lock for every independently active pnpm
workspace root, including a nested workspace used by product build, test, or
certification commands. Each lock MUST match all committed manifests reachable
from its workspace declaration, including manifests supplied by pinned
submodules, and reconciliation MUST preserve resolved versions unrelated to the
manifest changes that made the lock stale.

#### Scenario: Nested workspace accepts frozen installation

- **WHEN** dependency installation runs from an independently active nested pnpm
  workspace in a clean checkout with frozen lock enforcement
- **THEN** installation SHALL succeed using that workspace's committed lock
- **AND** both the nested lock and repository-root lock content and digest SHALL
  remain unchanged

#### Scenario: Nested workspace submodule advances without lock reconciliation

- **WHEN** a manifest reached through a nested workspace's pinned submodule no
  longer matches the nested workspace's committed lock importer
- **THEN** frozen installation MUST exit non-zero and identify the stale lock
- **AND** source-bound build, test, or certification evidence MUST NOT be minted
  from a non-frozen resolution

#### Scenario: Nested lock reconciliation preserves unrelated resolutions

- **WHEN** a nested lock is reconciled only to describe committed manifest or
  submodule-manifest changes
- **THEN** pre-existing package and snapshot resolutions unaffected by those
  changes SHALL remain unchanged
- **AND** any independent dependency upgrade MUST be authorized and verified as
  a separate dependency change

#### Scenario: Every command uses the lock for its actual execution root

- **WHEN** a build, test, or certification command selects a nested pnpm
  workspace as its execution root
- **THEN** lock validation evidence SHALL name and hash that nested lock
- **AND** a successful repository-root frozen install MUST NOT substitute for
  validation of the nested workspace lock
