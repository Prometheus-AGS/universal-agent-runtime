## ADDED Requirements

### Requirement: Lockfile-scoped advisory closure is evidenced locally
The repository SHALL pin patched transitive dependency versions in each affected authoritative lockfile, SHALL prove the vulnerable versions are absent from the resolved graphs, and SHALL include every maintained JavaScript package root in the local security receipt.

#### Scenario: Patched JavaScript dependency graphs are accepted
- **WHEN** the local dependency-security verification runs for the root, frontend, and documentation package roots
- **THEN** frozen installation succeeds, the resolved graphs contain only the approved patched versions, and each package manager audit reports no unaccepted finding

### Requirement: Unpatched documentation build dependencies have bounded acceptance
An advisory with no compatible patched release SHALL remain accepted only when its exposure is limited to trusted repository build inputs, an automated local gate rejects the affected input formats, a security owner and review date are recorded, and explicit conditions require the exception to be reopened.

#### Scenario: Affected documentation image input is rejected
- **WHEN** an affected image format is added to documentation build inputs while the exception is active
- **THEN** the local security gate fails before the documentation build can be accepted

#### Scenario: Bounded advisory exception is recorded
- **WHEN** the unpatched advisory is dismissed as tolerable risk
- **THEN** the disposition identifies the repository security maintainers as owner, records a review due date, and requires reopening for untrusted image ingestion or a compatible fixed release
