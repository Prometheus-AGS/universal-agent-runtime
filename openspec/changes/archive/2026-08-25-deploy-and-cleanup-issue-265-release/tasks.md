## 1. Merge and Baseline

- [x] 1.1 Merge completed issue-265 closeout review #273, fast-forward the primary checkout to `origin/main`, and verify only the protected untracked files remain.
- [x] 1.2 Capture merged source revision, installed executable digest, LaunchAgent PID/status, provider IDs/count, free space, and cleanup-target sizes before building.

## 2. Build and Install

- [x] 2.1 Validate the OpenSpec change strictly, build the frontend bundle, and validate the generated static bundle.
- [x] 2.2 Build the locked server-full release binary from merged `main` and record its digest and source revision.
- [x] 2.3 Install through `packaging/native/macos/install.sh`, verify a restarted LaunchAgent PID, health/readiness, installed digest, five preserved provider IDs, and canonical provider/context-management settings routes.

## 3. Review and Asset Cleanup

- [x] 3.1 Audit related completed worktrees for tracked, untracked, ignored, and unique `.prometheus` content, then remove only those proven merged and redundant.
- [x] 3.2 Delete only reviewed regenerable Cargo/frontend build outputs after successful installation, then record remaining size and reclaimed disk space.
- [x] 3.3 Delete merged related local/remote review branches, verify the primary checkout and upstream submodule worktree inventories, and preserve unrelated active work.

## 4. Closeout

- [x] 4.1 Record deployment and cleanup evidence under `.prometheus/`, complete and archive this OpenSpec change, reflect the KBD phase, commit, and push `main`.
