---
type: Reference
id: uar-desktop-stable-port-openspec-ready-for-apply
title: UAR desktop stable port OpenSpec ready for apply
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- tauri
- openspec
- uar-sidecar
- local-first
links:
- uar-hybrid-app-architecture-phase-context
- uar-desktop-sidecar-runtime-openspec-draft-status
- uar-desktop-stable-port-executor-session-completion
- uar-hybrid-architecture-pull-update-and-desktop-port-next-step
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-27T23:35:09.731668+00:00
created_at: 2026-07-27T23:35:09.731668+00:00
updated_at: 2026-07-27T23:35:09.731668+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: [`uar-hybrid-app-architecture`](/uar-hybrid-app-architecture-phase-context.md)
- Captured: `2026-07-27T23:22:42Z`
- Change: `desktop-stable-port`
- Phase status: `executing`
- Progress: `changes 4/12`

## Phase goals in scope

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via Tauri `externalBin` / `plugin-shell`, exposing its fixed port to the OS.
- Align the local-first data layer with the hybrid-mobile-architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory, including brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized React frontend fixes with `/impeccable polish` and `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler before committing, including release status, Vite/Rolldown compatibility, ESLint compatibility, vue-tsc-equivalent tooling, and migration path from TypeScript 5.9.3 under dependency-verification rules 22/23.

## Current desktop stable-port status

The `desktop-stable-port` OpenSpec work validates cleanly and all four expected artifacts are complete:

- `proposal`
- `design`
- `specs`
- `tasks`

The latest recorded position is:

```text
uar-hybrid-app-architecture › desktop-stable-port | status: executing
Progress: changes 4/12
Last: desktop-stable-port: tasks.md drafted, all 4/4 artifacts complete (proposal, design, specs, tasks), openspec validate passed
Next: /opsx:apply desktop-stable-port (implement tasks), or continue to the next queued change
```

This continues the same remediation thread captured in [UAR desktop sidecar runtime OpenSpec draft status](/uar-desktop-sidecar-runtime-openspec-draft-status.md), [UAR desktop stable port executor session completion](/uar-desktop-stable-port-executor-session-completion.md), and [UAR hybrid architecture pull update and desktop port next step](/uar-hybrid-architecture-pull-update-and-desktop-port-next-step.md).

## Next actions

- Run `/opsx:apply desktop-stable-port` to implement the validated tasks task-by-task.
- Alternatively, run `/opsx:archive` after implementation lands.
- If continuing the broader phase instead of applying this change immediately, proceed to the next queued change under `uar-hybrid-app-architecture`.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture