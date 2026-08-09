---
type: Reference
id: uar-desktop-stable-port-apply-status-at-2026-07-27-23-37
title: UAR desktop stable-port apply status at 2026-07-27 23:37
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- tauri
- uar-sidecar
- local-first
- typescript-7
links:
- uar-hybrid-app-architecture-phase-context
- uar-desktop-stable-port-openspec-ready-for-apply
- uar-hybrid-architecture-desktop-stable-port-apply-in-progress
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-27T23:38:35.487619+00:00
created_at: 2026-07-27T23:38:35.487619+00:00
updated_at: 2026-07-27T23:38:35.487619+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-27T23:37:02Z`
- Source identifier: `manual:universal-agent-runtime/uar-hybrid-app-architecture`
- Current position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This update continues the broader [UAR hybrid app architecture phase context](/uar-hybrid-app-architecture-phase-context.md), after [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and the prior [UAR hybrid architecture desktop stable-port apply in progress](/uar-hybrid-architecture-desktop-stable-port-apply-in-progress.md) status.

## Active phase goals

### Desktop stable port and sidecar runtime

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port.
- Preserve IndexedDB, `localStorage`, and service worker origins across desktop restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`.
- Expose the sidecar's fixed port to the operating system.

### Local-first target matrix

Align implementation with the `hybrid-mobile-architecture` per-target data-layer matrix:

- Web: keep PGlite using `idb://`.
- Desktop: move the data layer into Rust via `pglite-oxide`.
- Mobile: use SQLite plus `sqlite-vec` through `gen_ui_core` FFI.
- Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.

### Mobile target

- Establish the Flutter mobile target.
- On mobile, UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.

### Frontend audit and polish

- Run `/impeccable audit` and `/impeccable critique` across the React frontend.
- Produce a scored UI/UX defect inventory covering:
  - brittleness
  - stalls
  - freeze paths
  - admin console UX
- Execute prioritized fixes with `/impeccable polish` and `/impeccable harden`.

### Supplemental Admin/Agents UI seed work

Absorb the six supplemental changes from `uar-grade-a-upgrade-2026-07` as seed work items:

1. `sw-scheme-safe-caching`
2. `model-warning-clarity`
3. `provider-first-model-picker`
4. `edit-panel-verification`
5. `governance-reconciliation`
6. `freeze-diagnostics`

### TypeScript 7 migration investigation

- Migrate the frontend toolchain to TypeScript 7.0 native compiler only after verifying:
  - current release status
  - ecosystem compatibility with Vite/Rolldown
  - ESLint compatibility
  - `vue-tsc`-equivalent tooling implications
  - migration path from TypeScript `5.9.3`
- Apply dependency-verification rules 22/23 before committing.

## Current implementation status

- Desktop stable-port apply is in progress.
- Code changes have been written for task groups 2, 3, and 4:
  - `uar-sidecar` port override
  - Tauri manifest/config/capabilities
  - `src-tauri/src/lib.rs` sidecar-spawn rewrite
- Background `cargo check` confirmed still alive but slow.
- Slow verification is attributed to CPU contention from an unrelated concurrent build on the same machine.
- `tauri-plugin-shell` API usage in `src-tauri/src/lib.rs` remains unverified against the real crate source.
- The shell plugin code was written from best-available knowledge and still needs compile/source confirmation.

## Next actions

1. Await `cargo check` completion and auto-resume.
2. Verify `uar-sidecar` compiles.
3. Separately verify `src-tauri` usage of the `tauri-plugin-shell` API compiles.
4. Fix any API or compile mismatches.
5. Reconcile `design.md` with the implemented architecture.
6. Complete task `3.3`.
7. Run group-5 verification.
8. Update `tasks.md` checkboxes.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture