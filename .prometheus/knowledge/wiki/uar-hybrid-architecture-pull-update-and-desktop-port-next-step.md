---
type: Reference
id: uar-hybrid-architecture-pull-update-and-desktop-port-next-step
title: UAR hybrid architecture pull update and desktop port next step
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- tauri
- local-first
- mobile-architecture
- typescript-7
links:
- uar-hybrid-app-architecture-phase-context
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-20T02:25:25.809444+00:00
created_at: 2026-07-20T02:25:25.809444+00:00
updated_at: 2026-07-20T02:25:25.809444+00:00
revision: 0
---

## Context

This note updates the [`uar-hybrid-app-architecture`](/uar-hybrid-app-architecture-phase-context.md) phase for `universal-agent-runtime`.

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-20T02:23:39Z`
- Phase status: `executing`
- Progress: `changes 4/12`

## Active phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align the local-first data layer to the hybrid mobile architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` through `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per the corrected skill documentation.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized frontend fixes using `/impeccable polish` and `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07` as seed work items:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler before committing:
  - Verify current release status.
  - Verify ecosystem compatibility with Vite/Rolldown, ESLint, and `vue-tsc` equivalents.
  - Define migration path from TypeScript 5.9.3.
  - Follow dependency-verification rules 22/23.

## Pull result

Repository pull completed successfully.

- Git fast-forwarded from `06d7172` to `563ecc2`.
- Included a submodule update for `prometheus-entity-management`.
- Changed areas included:
  - CI workflows
  - `Cargo.lock`
  - `Cargo.toml`
  - Frontend lockfiles
  - Rust source files

Notable Rust changes mentioned:

- Added `src/llm/external_driver.rs`.
- Updated `src/uar/runtime/manager.rs`.
- Updated `src/normalized.rs`.
- Updated SDK types.

## Current milestone state

- Last completed work item: `admin-agent-model-warning-clarity`
- Completion: `13/13` tasks complete
- Archived as: `2026-07-16-admin-agent-model-warning-clarity`
- Overall phase progress remains: `changes 4/12`

## Recommended validation

Before proceeding with the next implementation task, run the locked Rust build check for the server-full feature set:

```bash
cargo check --locked --no-default-features --features server-full
```

## Next planned task

Next operation:

```text
/opsx:new desktop-stable-port
```

This should start the stable persisted localhost port work for the desktop/Tauri data-loss fix.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture