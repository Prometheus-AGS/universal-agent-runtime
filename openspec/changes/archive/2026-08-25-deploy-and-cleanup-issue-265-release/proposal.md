## Why

The merged settings-route and KBD runtime fixes are not fully delivered until the native macOS service is rebuilt from the merged default branch, installed with operator state preserved, and verified before regenerable build assets and completed review worktrees are removed. This change makes that release closeout explicit and auditable.

## What Changes

- Build the frontend static bundle and server-full release binary from synchronized `main`.
- Install and restart the macOS LaunchAgent through the supported installer while preserving configuration, provider data, and installer backups.
- Verify health, readiness, canonical settings routes, provider continuity, and the installed binary identity.
- Remove only merged, clean review branches and worktrees after checking for unique `.prometheus` history.
- Remove regenerable build outputs after installation evidence records their sizes and the reclaimed disk space.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: Require a post-merge macOS release closeout to prove installed source identity and operator-state continuity before related worktree and build-asset cleanup.

## Impact

- Affects the local macOS LaunchAgent installation, its backed-up binary/static/config assets, KBD release evidence, local Git worktrees and merged review branches, and regenerable Cargo/frontend build output.
- Does not change runtime APIs, provider payloads, persistence formats, frontend behavior, or realtime entity state.
- KBD workflow state advances through a dedicated deployment-and-cleanup phase.
