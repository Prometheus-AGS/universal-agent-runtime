## Context

See `proposal.md` for motivation. The merged fixes span a frontend static route correction and an exact upstream KBD submodule pin, while the supported macOS installer atomically replaces the executable and static bundle, backs up configuration/static content, and restarts `com.prometheus.universal-agent-runtime`. Local Cargo output, frontend dependencies, and completed Git worktrees consume material disk space but are recoverable only after installation evidence and memory audits are complete.

## Goals / Non-Goals

**Goals:**

- Install artifacts built from synchronized `main` through the supported native installer.
- Prove source/binary identity and preserve the five configured provider identities across restart.
- Remove only completed review worktrees and branches after checking tracked, untracked, ignored, and `.prometheus` content.
- Reclaim space from regenerable build outputs without using `cargo clean` or removing rollback backups.

**Non-Goals:**

- Change application code, provider configuration, payloads, persistence, or service definitions.
- Delete operator configuration, databases, logs, installer backups, package-manager caches, or unrelated worktrees.
- Restart `sovereign-sync`, whose binary is unaffected by this release.

## Decisions

### Build and install from the merged primary checkout

The release binary and frontend bundle will be built after `main` fast-forwards to the merged review commit. Their source revision and digests will be recorded before installation. Building from a review worktree was rejected because it weakens the claim that the installed artifact matches the merged default branch.

### Use the shipped atomic macOS installer

`packaging/native/macos/install.sh` will receive the release binary and validated static directory. Its configuration and static backups provide the rollback boundary. Manual executable copying was rejected because it would bypass configuration merge, permissions, plist refresh, and restart behavior.

### Gate deletion on live service and memory evidence

Provider IDs/count, health, readiness, canonical provider and context-management routes, LaunchAgent PID, and executable digest will be checked after restart. Each related worktree will be inspected for status and unique `.prometheus` files before removal. A clean Git status alone is insufficient because ignored project memory can exist.

### Delete only regenerable workspace outputs

Cleanup is limited to Cargo build output and frontend dependency/build directories that package manifests and lockfiles can regenerate. Installed artifacts and rollback backups remain. `cargo clean` is rejected by repository policy; explicit reviewed directories are removed only after path and size checks.

## Risks / Trade-offs

- [Release build consumes additional temporary disk space before cleanup] → Record free space first, build serially in the existing target directory, then clean only after installation succeeds.
- [Installer restart briefly interrupts port 1906] → Capture provider identity and current service state before installation, then poll health/readiness immediately afterward.
- [A completed worktree contains unique project memory] → Compare `.prometheus` content against merged `main`; retain the worktree if uniqueness cannot be disproved.
- [Deleting dependency directories makes the next build slower] → Restrict deletion to regenerable assets and report exactly what was removed.

## Migration Plan

1. Merge completed reviews and synchronize `main` and submodules.
2. Capture source revision, provider identity, LaunchAgent state, installed digest, free space, and cleanup-target sizes.
3. Build and validate the frontend static bundle and server-full release binary.
4. Install through the macOS installer, then verify service health, readiness, settings routes, provider continuity, executable digest, and restart PID.
5. Audit and remove only merged related worktrees/branches with no unique project memory.
6. Remove explicit regenerable build outputs, measure reclaimed space, and retain installer backups for rollback.

Rollback uses the newest executable/static/config artifacts beneath `~/.prometheus/backups/uar/` if post-install verification fails; cleanup does not begin on a failed installation.
