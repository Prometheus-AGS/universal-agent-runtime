## ADDED Requirements

### Requirement: Offline desktop migration commands

The desktop sidecar SHALL expose explicit process modes for a supervisor to hold the native SurrealKV database lock and convert a staged copy. Both modes SHALL run before normal server bootstrap and SHALL open no network listener. Normal sidecar startup SHALL continue to reject an unconverted 3.x datastore.

#### Scenario: The host holds exclusive access during preparation

- **WHEN** the supervisor launches `--lock-embedded-database <absolute-database-path>` for `runtime.db` or, for recovery, `.runtime-db-upgrade/original`
- **THEN** the helper SHALL exclusively acquire the existing SurrealKV `LOCK` file without creating, truncating or writing it
- **AND** the helper SHALL flush `MIGRATION_LOCKED` and retain the lock until its inherited stdin reaches EOF or the process exits
- **AND** the helper SHALL fail without a success marker if the lock cannot be acquired
- **AND** on Windows, the helper SHALL open the file for shared read, write and delete access so the host can rename its containing directory on a filesystem supporting open-handle renames

#### Scenario: The host prepares a separate converted copy

- **WHEN** the supervisor launches `--migrate-embedded-database <absolute-runtime.db>` on a verified independent staged copy at `.runtime-db-upgrade/stage-<UUID>/runtime.db`
- **THEN** the helper SHALL require a legacy SurrealKV manifest and refuse a linked, already converted, or partially converted stage
- **AND** the helper SHALL open the stage with the embedded SDK's explicit upgrade option, close it, reopen it with ordinary settings and close it again
- **AND** the helper SHALL flush `MIGRATION_COMPLETE` only after the current manifest format and clean reopen are confirmed
- **AND** the host SHALL retain the untouched original as an independently recoverable copy

### Requirement: Desktop sidecar source is checkout-compatible on Windows

The source used to build the Windows sidecar SHALL allow Git for Windows to check out the release commit before building.

#### Scenario: Historical orchestration alias is carried in the source tree

- **WHEN** the Windows sidecar runner checks out the release commit
- **THEN** the historical alias has a Windows-compatible filename and still refers to its original history directory

### Requirement: Desktop sidecar releases require a clean production build

The release job SHALL reject compiler and linker warnings before packaging the sidecar. For macOS, it SHALL use the Mach-O linker supplied by the pinned toolchain and the size-optimized release profile.

#### Scenario: The default Apple linker reports oversized unwind data

- **WHEN** the macOS sidecar release is built
- **THEN** the job uses the pinned Mach-O linker and checks the unfiltered Cargo diagnostics before publishing the archive
