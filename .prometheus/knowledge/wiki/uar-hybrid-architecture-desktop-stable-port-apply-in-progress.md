---
type: Reference
id: uar-hybrid-architecture-desktop-stable-port-apply-in-progress
title: UAR hybrid architecture desktop stable-port apply in progress
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
- uar-hybrid-architecture-desktop-stable-port-implementation-status
- uar-desktop-stable-port-executor-session-completion
- uar-hybrid-architecture-pull-update-and-desktop-port-next-step
sources:
- stdin
timestamp: 2026-07-27T23:37:28.853927+00:00
created_at: 2026-07-27T23:37:28.853927+00:00
updated_at: 2026-07-27T23:37:28.853927+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-27T23:35:48Z`
- Source identifier: `manual:universal-agent-runtime/uar-hybrid-app-architecture`
- Current position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This status update continues the broader [UAR hybrid app architecture phase context](/uar-hybrid-app-architecture-phase-context.md), following the desktop stable-port preparation in [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and the implementation status captured in [UAR hybrid architecture desktop stable-port implementation status](/uar-hybrid-architecture-desktop-stable-port-implementation-status.md).

## Phase goals

### Desktop stable port and sidecar runtime

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port.
- Rationale: IndexedDB, `localStorage`, and service worker storage are origin-scoped; changing the localhost port changes the origin and makes persisted browser storage appear lost after restart.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary through Tauri `externalBin` / `plugin-shell`.
- Expose the fixed sidecar port to the operating system.

### Local-first data-layer target matrix

Align the local-first data layer to the hybrid mobile architecture per-target matrix:

- Web: keep PGlite using `idb://`.
- Desktop: move the data layer into Rust via `pglite-oxide`.
- Mobile: use SQLite plus `sqlite-vec` through `gen_ui_core` FFI.
- Constraint: `pglite-oxide` has no iOS/Android support per the corrected skill documentation.

### Mobile runtime target

- Establish the Flutter mobile target where UAR runs entirely on-device.
- Use embedded SurrealDB with `surrealkv` persistence.
- Architecture reference: `TJ-ARCH-MOB-001`.

### Frontend audit and remediation

- Run `/impeccable audit` and `/impeccable critique` across the React frontend.
- Produce a scored UI/UX defect inventory covering:
  - brittleness,
  - stalls,
  - freeze paths,
  - admin console UX.
- Execute prioritized fixes with `/impeccable polish` and `/impeccable harden`.

### Supplemental Admin/Agents UI seed work

Absorb the six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07` as seed work items:

1. `sw-scheme-safe-caching`
2. `model-warning-clarity`
3. `provider-first-model-picker`
4. `edit-panel-verification`
5. `governance-reconciliation`
6. `freeze-diagnostics`

### TypeScript 7.0 migration investigation

- Migrate the frontend toolchain to TypeScript 7.0 native compiler only after verification.
- Required checks before committing:
  - current TypeScript 7.0 release status,
  - ecosystem compatibility,
  - Vite / Rolldown compatibility,
  - ESLint compatibility,
  - `vue-tsc`-equivalent compatibility where applicable,
  - migration path from TypeScript `5.9.3`.
- Must comply with dependency-verification rules `22` and `23`.

## Current desktop-stable-port apply status

Code changes for task groups 2, 3, and 4 have been written:

- `src/bin/uar-sidecar.rs`
  - Added sidecar port override support.
- `src-tauri/Cargo.toml`
  - Updated for the Tauri shell sidecar/plugin integration.
- `src-tauri/tauri.conf.json`
  - Updated for sidecar binary configuration.
- `src-tauri/capabilities/*`
  - Updated capabilities for shell/plugin sidecar execution.
- `src-tauri/src/lib.rs`
  - Rewritten to spawn `uar-sidecar` via `tauri-plugin-shell`.
  - Shutdown uses `kill()`-based sidecar termination.

## Verification status

- A background `cargo check` was inspected with `ps aux`.
- The process was confirmed to be genuinely compiling and not stalled.
- Task checkboxes in `tasks.md` are intentionally not marked complete until `cargo check` finishes and any compile errors are resolved.

## Next actions

1. Wait for `cargo check` completion.
2. Fix any compile errors.
3. Reconcile `design.md` Decision 3.
4. Complete Task `3.3` for binary placement.
5. Run verification tasks from group 5.
6. Update `tasks.md` checkboxes after validation.
7. Continue tracking this work against the desktop stable-port remediation also referenced by [UAR desktop stable port executor session completion](/uar-desktop-stable-port-executor-session-completion.md) and [UAR hybrid architecture pull update and desktop port next step](/uar-hybrid-architecture-pull-update-and-desktop-port-next-step.md).

# Citations

1. stdin