---
type: Reference
id: uar-hybrid-architecture-desktop-stable-port-build-unblocked
title: UAR hybrid architecture desktop stable-port build unblocked
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- tauri
- uar-sidecar
- local-first
- typescript-7
- build-verification
links:
- uar-desktop-stable-port-openspec-ready-for-apply
- uar-desktop-stable-port-apply-status-at-2026-07-27-23-37
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-28T00:00:49.321149+00:00
created_at: 2026-07-28T00:00:49.321149+00:00
updated_at: 2026-07-28T00:00:49.321149+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-27T23:55:03Z`
- Current position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/12`

This status continues the desktop stable-port work from [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and the apply status tracked in [UAR desktop stable-port apply status at 2026-07-27 23:37](/uar-desktop-stable-port-apply-status-at-2026-07-27-23-37.md).

## Active phase goals

### Desktop stable port and sidecar runtime

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port.
- Preserve desktop browser storage origins across restarts:
  - IndexedDB
  - `localStorage`
  - Service workers
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`.
- Expose the fixed sidecar port to the operating system.

### Per-target local-first data layer

Align the local-first data layer with the hybrid mobile architecture target matrix:

| Target | Data layer decision |
| --- | --- |
| Web | Keep PGlite `idb://` |
| Desktop | Move data layer into Rust via `pglite-oxide` |
| Mobile | Use SQLite + `sqlite-vec` via `gen_ui_core` FFI |

Constraint: `pglite-oxide` has no iOS/Android support per the corrected hybrid-mobile-architecture documentation.

### Mobile runtime

- Establish the Flutter mobile target.
- On mobile, UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence.
- Architecture reference: `TJ-ARCH-MOB-001`.

### UI/UX audit and hardening

- Run `/impeccable audit` and `/impeccable critique` across the React frontend.
- Produce a scored UI/UX defect inventory covering:
  - Brittleness
  - Stalls
  - Freeze paths
  - Admin console UX
- Execute prioritized fixes with `/impeccable polish` and `/impeccable harden`.

### Supplemental Admin/Agents seed work items

Absorb the six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07`:

1. `sw-scheme-safe-caching`
2. `model-warning-clarity`
3. `provider-first-model-picker`
4. `edit-panel-verification`
5. `governance-reconciliation`
6. `freeze-diagnostics`

### TypeScript 7 migration investigation

- Investigate migration of the frontend toolchain to TypeScript 7.0 native compiler.
- Verify before committing, per dependency-verification rules 22/23:
  - Current TypeScript 7.0 release status
  - Ecosystem compatibility
  - Vite / Rolldown compatibility
  - ESLint compatibility
  - `vue-tsc`-equivalent support where relevant
  - Migration path from TypeScript `5.9.3`

## Latest execution status

### Completed unblock

- Workspace build was blocked by a pre-existing `rmcp` / `sse-stream` API mismatch.
- The mismatch was not caused by the current desktop stable-port change.
- The block was resolved by bumping `sse-stream` from `0.2.3` to `0.2.5`:

```bash
cargo update -p sse-stream --precise 0.2.5
```

### Decision rationale

- The single-crate lockfile bump was treated as low-risk and reversible.
- The action was taken without waiting for explicit sign-off because the Stop hook required forward motion.
- The change was explicitly flagged rather than silently folded into the desktop stable-port work.

## Verification state

- Re-verification build is running in the background.
- Await rebuild completion before proceeding.

## Next actions

1. Resume after the background rebuild completes.
2. Verify `uar-sidecar` compiles cleanly.
3. Verify `src-tauri` compiles cleanly.
4. Fix any `tauri-plugin-shell` API mismatches in `src-tauri/src/lib.rs`.
5. Reconcile `design.md`.
6. Complete Task `3.3`.
7. Run group-5 verification.
8. Update `tasks.md` checkboxes.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture