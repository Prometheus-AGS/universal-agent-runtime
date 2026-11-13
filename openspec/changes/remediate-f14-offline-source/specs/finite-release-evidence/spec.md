## ADDED Requirements

### Requirement: Verify the full recursive offline source archive before F14
The release SHALL retain the full recursively tracked offline source archive, required licenses and vendored sources. The recursive OSV gate SHALL pass on the exact integrated source without path or advisory suppression before the release can be signed.

#### Scenario: An operator rebuilds the archived minimal target offline
- **WHEN** the full archive from the clean candidate is extracted with no network access
- **THEN** the documented locked minimal build succeeds using its bundled sources
- **AND** the archive manifest, digest and source pins identify the exact input

#### Scenario: Vendored desktop notices remain readable to the recursive scanner
- **WHEN** the unchanged recursive OSV scan reads the vendored GTK3 SPDX notices
- **THEN** it parses all twelve notices without a parse error
- **AND** the declared licenses, copyright, and upstream source references remain in the offline archive

#### Scenario: The recursive source scan is evaluated
- **WHEN** the existing recursive OSV command scans the exact clean candidate
- **THEN** raw results, scanner version, source SHA, tree and recursive pins are retained
- **AND** a nonzero scanner exit prevents child completion and F14 promotion

### Requirement: Use SurrealDB 4.0 for active release inputs

The active embedded SDK and standalone server SHALL use SurrealDB 4.0.
Container deployment defaults and qualification fixtures SHALL select the
same immutable 4.0 server image. Persistent companion services SHALL use
SurrealKV and retain the declared volume and datastore path.

#### Scenario: A supported deployment starts its companion database

- **WHEN** a live deployment or disposable release fixture starts the companion server
- **THEN** the server reports the pinned SurrealDB 4.0 version
- **AND** its readiness check succeeds only when the database accepts connections
- **AND** persistent lanes reopen the same SurrealKV datastore after shutdown
