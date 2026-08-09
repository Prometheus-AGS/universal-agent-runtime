---
type: Reference
id: mistral-rs-fork-upstream-sync-pr-and-repository-cleanup
title: Mistral.rs fork upstream sync PR and repository cleanup
tags:
- mistral-rs
- upstream-sync
- pull-request
- cargo-clean
- disk-cleanup
- rust
sources:
- stdin
timestamp: 2026-07-20T02:23:27.342009+00:00
created_at: 2026-07-20T02:23:27.342009+00:00
updated_at: 2026-07-20T02:23:27.342009+00:00
revision: 0
---

## Summary

Two maintenance tasks were completed for the Prometheus fork of `mistral.rs`:

- Opened PR **#6 — Merge upstream mistral.rs v0.9.0 into Prometheus fork**.
- Removed the repository-local Rust build output via `cargo clean`.

## PR opened

- PR: [#6 — Merge upstream mistral.rs v0.9.0 into Prometheus fork](https://github.com/GQAdonis/mistral.rs/pull/6)
- Base branch: `master`
- Head branch: `features/upstream-sync-v0.9.0`
- PR body documents:
  - What upstream `mistral.rs` v0.9.0 brought in.
  - Which Prometheus fork features were preserved.
  - Which fork changes were dropped.
- No `Test plan` section was included, matching project convention.

## Disk cleanup

- Ran `cargo clean` for the repo.
- Removed the only repository-local `target/` directory.
- Reclaimed approximately **3.5 GiB** across **8400 files**.
- Reported free space changed from **136 GiB** to **139 GiB**; the `df` delta rounds to 3 GiB at that granularity.
- No additional build artifact directories were found:
  - No other `target/` directories.
  - No docs `node_modules/` directories.
  - No docs `dist/` directories.
- Remaining directories are source assets; notably `res/` is about **25 MiB** of committed banner images.

## Preserved shared Cargo caches

The cleanup intentionally did **not** remove shared Cargo cache locations:

- `~/.cargo/git/checkouts` — about **1.2 GiB**, shared across Rust projects and includes the `candle` checkout.
- Global `~/.cargo` registry cache.

Rationale: clearing these shared caches would force re-downloads and rebuild-related fetches for other Rust projects.

If additional disk reclamation is needed, use `cargo cache` or manually prune `~/.cargo/git/checkouts`, understanding this affects shared Cargo state.

## Current status

- PR #6 is open.
- Repo-local disk cleanup is complete.
- Nothing is pending.
- Reviewing or merging PR #6 will require rebuilding `target/` from scratch.
- A metal check rebuild is expected to take approximately **6 minutes**.

# Citations

1. stdin