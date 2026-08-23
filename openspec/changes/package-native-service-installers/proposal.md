## Why

UAR has no tracked native supervisor templates or repeatable install/upgrade/uninstall entrypoints. Operators currently have no safe way to preserve state, establish platform paths, or route service logs.

## What Changes

- Add macOS LaunchAgent, Linux systemd, and Windows SCM packaging below `packaging/native/`.
- Add install, upgrade/start-stop, credential-refresh, and uninstall entrypoints.
- Establish correct platform program/state/config/log locations and non-destructive behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: concrete native packaging and lifecycle files.

## Impact

- Adds packaging/scripts only; it does not deploy Linux or Windows from the macOS host.
