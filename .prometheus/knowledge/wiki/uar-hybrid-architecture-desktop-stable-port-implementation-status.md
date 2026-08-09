---
type: Reference
id: uar-hybrid-architecture-desktop-stable-port-implementation-status
title: UAR hybrid architecture desktop stable-port implementation status
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
- uar-desktop-sidecar-runtime-openspec-draft-status
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-27T23:36:12.764085+00:00
created_at: 2026-07-27T23:36:12.764085+00:00
updated_at: 2026-07-27T23:36:12.764085+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-27T23:34:48Z`
- Position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This updates the broader [UAR hybrid app architecture phase context](/uar-hybrid-app-architecture-phase-context.md) and continues the desktop stable-port remediation previously prepared in [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and tracked alongside [UAR desktop sidecar runtime OpenSpec draft status](/uar-desktop-sidecar-runtime-openspec-draft-status.md).

## Phase goals

### Desktop stable port and sidecar runtime

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port.
- Rationale: IndexedDB, `localStorage`, and service worker storage are origin-scoped; changing the localhost port changes the browser origin and makes persisted data appear lost across restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`.
- Expose the sidecar's fixed port to the operating system.

### Local-first data-layer matrix

Align local-first persistence with the hybrid-mobile-architecture per-target matrix:

- Web: keep PGlite using `idb://`.
- Desktop: move the data layer into Rust via `pglite-oxide`.
- Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
- Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.

### Mobile runtime

- Establish the Flutter mobile target where UAR runs entirely on-device.
- Use embedded SurrealDB `surrealkv` persistence.
- Architectural reference: `TJ-ARCH-MOB-001`.

### Frontend audit and remediation

- Run `/impeccable audit` and `/impeccable critique` across the React frontend.
- Produce a scored UI/UX defect inventory covering:
  - brittleness,
  - stalls,
  - freeze paths,
  - admin console UX.
- Execute prioritized fixes with `/impeccable polish` and `/impeccable harden`.

### Supplemental Admin/Agents seed work

Absorb six supplemental changes from `uar-grade-a-upgrade-2026-07` as phase seed work items:

- `sw-scheme-safe-caching`
- `model-warning-clarity`
- `provider-first-model-picker`
- `edit-panel-verification`
- `governance-reconciliation`
- `freeze-diagnostics`

### TypeScript 7 migration investigation

- Migrate the frontend toolchain to TypeScript 7.0 native compiler only after verification.
- Verify current release status and ecosystem compatibility before committing.
- Compatibility areas:
  - Vite / Rolldown,
  - ESLint,
  - `vue-tsc`-equivalent tooling,
  - migration path from TypeScript `5.9.3`.
- Follow dependency-verification rules 22/23.

## Current implementation status

Desktop stable-port apply is in progress.

Completed or resolved:

- Task group 1 resolved:
  - Environment variable was already `clap`-overridable.
  - `UAR_SIDECAR_BIND_PORT` chosen as the override environment variable name.
  - `kill()`-based shutdown chosen over stdin-EOF because `CommandChild` is not auto-reaped on drop.
- Tasks 2.1–2.5 completed in `src/bin/uar-sidecar.rs`:
  - sidecar port override implemented.
- Tasks 3.1, 3.4, and 3.5 completed in:
  - `src-tauri/Cargo.toml`,
  - `tauri.conf.json`,
  - `capabilities/default.json`.
- Tasks 4.1–4.4 completed:
  - `src-tauri/src/lib.rs` rewritten to spawn `uar-sidecar` via `tauri-plugin-shell` instead of an in-process thread.
- Previously uninitialized vendor/git submodules were initialized:
  - `liter-llm`,
  - `rust-mcp-filesystem`.

Blocked / in flight:

- `cargo check` is running in the background to verify the implementation.
- Compile results are not yet available.

## Next actions

1. Await `cargo check` completion or resume after the 5-minute wakeup.
2. Fix any compile errors found by `cargo check`.
3. Reconcile `design.md` Decision 3 with the implementation's actual shutdown behavior:
   - implemented: `kill()`-only shutdown,
   - not implemented: stdin-EOF shutdown.
4. Complete Task 3.3: binary placement.
5. Work through verification tasks in group 5.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture