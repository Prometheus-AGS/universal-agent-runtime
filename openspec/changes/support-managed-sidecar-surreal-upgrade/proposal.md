## Why

An installed desktop profile with an embedded SurrealDB 3.x database cannot start the SurrealDB 4.0 sidecar until its store is converted. The desktop supervisor needs an offline conversion command so it can retain a rollback copy and upgrade its existing profile.

## What Changes

- Add offline sidecar commands for holding the legacy SurrealKV lock and converting a distinct staging copy.
- Verify the converted store reopens with normal database settings before returning a success receipt.
- Keep normal sidecar startup, external databases, and the operator-managed server upgrade contract unchanged.

## Capabilities

### Modified Capabilities

- `embedded-surrealdb-upgrade`: the desktop supervisor can orchestrate the existing separate-copy upgrade using an offline sidecar command.

## Impact

The UAR sidecar exposes two local process commands. The Boss coordinates the snapshot, backup, promotion and any startup recovery.
