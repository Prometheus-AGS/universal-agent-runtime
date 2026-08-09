---
type: Reference
id: uar-desktop-stable-port-blocked-by-rmcp-sse-stream-version-skew
title: UAR desktop stable-port blocked by rmcp sse-stream version skew
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- cargo
- dependency-skew
- rmcp
- sse-stream
- tauri
links:
- uar-desktop-stable-port-openspec-ready-for-apply
- uar-desktop-stable-port-apply-status-at-2026-07-27-23-37
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-27T23:55:27.899981+00:00
created_at: 2026-07-27T23:55:27.899981+00:00
updated_at: 2026-07-27T23:55:27.899981+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-27T23:54:06Z`
- Current position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This status updates the desktop stable-port work described in [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and [UAR desktop stable-port apply status at 2026-07-27 23:37](/uar-desktop-stable-port-apply-status-at-2026-07-27-23-37.md).

## Active phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via Tauri `externalBin` / `plugin-shell`, exposing its fixed port to the OS.
- Align the local-first data layer to the hybrid mobile architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized fixes using `/impeccable polish` and `/impeccable harden`.
- Absorb the six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler, verifying release status, ecosystem compatibility, and migration path from 5.9.3 before committing per dependency verification rules 22/23.

## Blocker: workspace does not compile

Verification of `desktop-stable-port` is blocked by a pre-existing workspace-wide dependency skew:

- `Cargo.lock` pins `rmcp` `2.2.0` with `sse-stream` `0.2.3`.
- `rmcp` source calls `SseStream::from_bytes_stream`.
- Locked `sse-stream` `0.2.3` only exposes `from_byte_stream` / `new`.
- `sse-stream` `0.2.5` exists and is newer than the locked `0.2.3`; it likely contains the API required by `rmcp`.
- The failure blocks `cargo check` for the entire workspace, not only the files touched by `desktop-stable-port`.

Affected verification items:

- Task 2: verify `src/bin/uar-sidecar.rs` compiles.
- Task 4: verify `src-tauri/src/lib.rs` sidecar spawn changes compile.

## Attribution

The break predates the `desktop-stable-port` edits:

- `git log -- Cargo.lock` indicated the skew came from dependabot merges pulled at session start.
- Identified commit: `563ecc2`.
- The executor did not change the lockfile because updating unrelated workspace dependencies was considered scope creep for `desktop-stable-port`.

## Proposed resolution options

1. Run `cargo update -p sse-stream` to bump `sse-stream` to `0.2.5`, then re-run verification.
   - Likely minimal fix.
   - Touches dependency graph outside the declared desktop stable-port scope.
2. Fix the dependency skew as its own separate change/commit, then resume `desktop-stable-port` verification.
3. Use a maintainer-preferred fix or report upstream first if this skew is already known.

## Current next step

A maintainer decision is needed before proceeding: approve a minimal `cargo update -p sse-stream` and any required follow-up to unblock verification, or handle the dependency skew separately before resuming desktop stable-port work.

# Citations

1. [1] stdin
2. [2] manual:universal-agent-runtime/uar-hybrid-app-architecture