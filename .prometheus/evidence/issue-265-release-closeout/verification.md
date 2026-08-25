# Issue 265 release deployment and cleanup evidence

Date: 2026-08-25

## Merge baseline

- UAR PR #271, UAR PR #272, UAR PR #273, upstream PR #67, and upstream PR #68 are merged.
- Primary checkout was fast-forwarded to merged `main` commit `e38a5ba8e0059b408b30fb27ce7c6a12ced8473d` before building.
- `crates/prometheus-skill-system` resolves to merged upstream commit `602750ec61bc4674b51231fb36f3bfee3af42b7e`.
- Protected untracked `versions.toml` and the former phase `prior-context.md` were not staged, rewritten, or removed.

## Build verification

- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm frontend:boundaries`: reported three existing per-row graph-write findings in `frontend/src/features/providers/model/providers-store.ts`; no frontend source was changed during this closeout.
- `pnpm build`: passed with the existing PGlite direct-`eval` dependency warnings.
- `node scripts/validate-static-bundle.mjs static`: passed with 11 referenced assets.
- `cargo build --locked --release --no-default-features --features server-full`: passed in 53.99 seconds.
- Global strict OpenSpec spec sweep: 103 passed and 1 failed on the pre-existing `entity-surreal-live-adapter` RFC 2119 warning for “Checkpoint Replay on Reconnect”; the targeted change and archived change both passed strict validation.
- The Rust build reported three existing library warnings (`MAX_BODY_BYTES`, `MAX_REDIRECTS`, and missing `Debug` for `WasmHostState`) plus future-incompatibility notices for `nix 0.31.3` and `redis 1.2.1`. This phase did not change those sources.

## Installation and live proof

- Pre-install UAR LaunchAgent PID: `30131`; post-install PID: `31143`.
- Built and installed binary SHA-256: `d4c91708afa4b173d7b9b5ff3ffaa917f02e0129804e1e206e622c7bcb0d7dcf`.
- Installed static `index.html` SHA-256: `775ecf5f57b0e0a8cb03b853532480755892f919ca4cef8eb8c18bd720f3a3c1`.
- `codesign --verify --verbose=2` passed on the built and installed executable.
- `packaging/native/macos/install.sh --binary target/release/universal-agent-runtime --static-dir static` installed and restarted `com.prometheus.universal-agent-runtime`.
- Startup waited for SurrealDB on port 28000, then completed normally; `/healthz` and `/readyz` returned HTTP 200.
- `/api/uar/settings/providers` and `/api/uar/settings/context-management` returned HTTP 200.
- Singular `/api/uar/settings/provider` and underscored `/api/uar/settings/context_management` returned HTTP 404 as expected.
- Durable provider IDs remained `alibaba`, `kimi-for-coding`, `local-openai-proxy`, `minimax`, and `zai`. Outer settings-row UUIDs are intentionally read-time proxy IDs and are not provider identity.
- Installed-service Playwright route proof passed 1/1 in 2.5 seconds.
- Current and backed-up configuration SHA-256 both equal `9fe2e6ac9f75bdff2c8b05d6bfa502420b990b5f1da3f94f78f97c2b58063e46`.
- Rollback artifacts remain at `~/.prometheus/backups/uar/universal-agent-runtime.pre-e38a5ba8.20260825T113000Z`, `config.yaml.20260825T112535Z`, and `static.20260825T112535Z`.
- The installer did not touch `ai.prometheus.sovereign-sync`. It independently restarted during the session and was observed healthy afterward at PID `94089`, run count `7`.

## Worktree and branch cleanup

- Removed clean merged upstream worktrees `fix-kbd-uninitialized-runtime` and `prometheus-skill-system-kbd-run-rollover` after confirming their commits are ancestors of upstream `main` and PRs #68/#67 are merged.
- Removed clean UAR worktree `pr-268-resolution` after confirming no tracked, untracked, ignored, or unique `.prometheus` content. PR #268 was closed unmerged, so its conflict-resolution branch was obsolete rather than merged.
- Deleted local branches `codex/pr-268-resolution`, `codex/fix-settings-namespace-routes`, `codex/fix-kbd-uninitialized-runtime`, and `codex/kbd-run-rollover`.
- Deleted the remaining remote `codex/fix-settings-namespace-routes` branch and pruned stale issue-265/upstream refs. GitHub had already deleted the other merged remote branches.
- Final inventories contain only the primary UAR worktree and its embedded detached upstream submodule checkout.

## Disk cleanup

- Explicit pre-clean logical sizes: root `target` 1,448,924 KiB; root `node_modules` 1,716,208 KiB; `frontend/node_modules` 1,902,008 KiB; `frontend/test-results` 109,128 KiB.
- Deleted those four regenerable directories with explicit depth-first paths; `cargo clean`, package-manager cache deletion, and broad globs were not used.
- All four directories were confirmed absent afterward.
- Immediate filesystem free space increased from 183,849,100 KiB to 185,240,228 KiB: 1,391,128 KiB (about 1.33 GiB) physically reclaimed. Logical deleted size is larger because APFS and pnpm share file storage.
- Installed artifacts, source `static/`, configuration, databases, logs, package-manager caches, and rollback backups remain intact.
