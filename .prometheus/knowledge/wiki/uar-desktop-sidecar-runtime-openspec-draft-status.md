---
type: Reference
id: uar-desktop-sidecar-runtime-openspec-draft-status
title: UAR desktop sidecar runtime OpenSpec draft status
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- desktop-sidecar-runtime
- tauri
- openspec
- uar-sidecar
links:
- uar-hybrid-app-architecture-phase-context
- uar-hybrid-architecture-pull-update-and-desktop-port-next-step
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-27T23:23:07.116202+00:00
created_at: 2026-07-27T23:23:07.116202+00:00
updated_at: 2026-07-27T23:23:07.116202+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Captured: `2026-07-27T23:21:03Z`
- Position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This session continues the desktop remediation work in the broader [UAR hybrid app architecture phase context](/uar-hybrid-app-architecture-phase-context.md), specifically the second half of the `desktop-stable-port` effort described in [UAR hybrid architecture pull update and desktop port next step](/uar-hybrid-architecture-pull-update-and-desktop-port-next-step.md).

## Active phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align the local-first data layer to the hybrid-mobile-architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.
- Establish the mobile Flutter target where UAR runs entirely on-device using embedded SurrealDB / `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized UI fixes with `/impeccable polish` / `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler before committing:
  - Verify current release status.
  - Check ecosystem compatibility, including Vite / Rolldown, ESLint, and `vue-tsc`-equivalent tooling.
  - Define migration path from TypeScript `5.9.3`.
  - Follow dependency-verification rules 22/23.

## Completed artifact

Created:

```text
openspec/changes/desktop-stable-port/specs/desktop-sidecar-runtime/spec.md
```

The spec adds a new `desktop-sidecar-runtime` capability using OpenSpec `## ADDED Requirements` format.

### Requirements drafted

1. **External process isolation**
   - Tauri shell must run UAR through the existing `uar-sidecar` binary rather than embedding the server in-process.
2. **Resolved-port binding**
   - Runtime must bind to the resolved fixed localhost port used by the desktop shell.
3. **TCP-based readiness**
   - Readiness must be determined by TCP availability rather than in-process state assumptions.
4. **Lifecycle tied to app exit**
   - Sidecar process supervision must be tied to the Tauri application lifecycle and app exit.

Each requirement includes testable scenarios.

## OpenSpec state

- `proposal`, `design`, and `specs` are complete.
- `tasks` is now unlocked.
- Current artifact count: `3/4` complete.
- Next action: run `/opsx:continue desktop-stable-port` and create `tasks.md`.

## Naming collision and capability boundary

While drafting the new spec, an existing archived specification was found:

```text
openspec/specs/desktop-shell/spec.md
```

That existing spec came from an earlier archived change also named `desktop-stable-port`. It covered the stable-port-persistence half of the P0 fix, matching the already-implemented behavior found in `lib.rs`.

The current `openspec/changes/desktop-stable-port` directory is a second use of the same change name, but its scope is different: it covers the remaining sidecar-conversion work. The new spec was kept under a separate `desktop-sidecar-runtime` capability rather than folded into `desktop-shell` because:

- `desktop-shell` already captures origin-stability requirements.
- The current design preserves those origin-stability requirements unchanged.
- The new work adds process-supervision behavior for the external `uar-sidecar` runtime.

This naming collision should be handled carefully when the current change archives.

## Citations

1. [1] stdin
2. [2] manual:universal-agent-runtime/uar-hybrid-app-architecture