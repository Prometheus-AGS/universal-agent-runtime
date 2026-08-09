---
type: Reference
id: uar-desktop-stable-port-paused-for-mcp-persistence-merge
title: UAR desktop stable-port paused for MCP persistence merge
tags:
- universal-agent-runtime
- hybrid-app-architecture
- desktop-stable-port
- tauri
- uar-sidecar
- mcp-server
- persistence-migration
links:
- uar-hybrid-architecture-pull-update-and-desktop-port-next-step
- uar-desktop-stable-port-openspec-ready-for-apply
- uar-desktop-stable-port-apply-status-at-2026-07-27-23-37
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-28T00:27:25.264938+00:00
created_at: 2026-07-28T00:27:25.264938+00:00
updated_at: 2026-07-28T00:27:25.264938+00:00
revision: 0
---

## Context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-28T00:26:13Z`
- Current position: `uar-hybrid-app-architecture › desktop-stable-port`
- Status: `paused`, blocked on external merge
- Progress: `changes 4/12`

This status follows the broader hybrid architecture work described in [UAR hybrid architecture pull update and desktop port next step](/uar-hybrid-architecture-pull-update-and-desktop-port-next-step.md) and the desktop stable-port preparation in [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md). It supersedes the active apply status in [UAR desktop stable-port apply status at 2026-07-27 23:37](/uar-desktop-stable-port-apply-status-at-2026-07-27-23-37.md).

## Pause reason

Work was stopped because another agent is actively migrating the MCP server, knowledge base, and skills persistence layer from config-file storage to a database-backed implementation. That migration overlaps the same areas currently needed to verify this change:

- `rmcp` dependency/API usage
- `src/uar/mcp_server.rs`
- `src/uar/memory/mcp_server.rs`
- likely `fastembed.rs` embedding backend behavior

Continuing to patch those files would likely conflict with the in-flight persistence migration. No further changes should be made until that merge lands.

## Desktop stable-port implementation state

The `desktop-stable-port` implementation is written but **not yet compile-verified** because verification is blocked by unrelated MCP/persistence-layer breakage.

Files changed for the intended desktop stable-port work:

- `src/bin/uar-sidecar.rs` — fixed-port override support.
- `src-tauri/src/lib.rs` — Tauri shell spawns the sidecar instead of embedding the server in-process.
- `src-tauri/Cargo.toml` — Tauri-side dependency/config changes.
- `src-tauri/tauri.conf.json` — sidecar/external binary configuration.
- `src-tauri/capabilities/default.json` — capability updates for sidecar/plugin-shell behavior.

Goal preserved from the phase: replace random-port-per-launch desktop behavior with a stable persisted localhost port so IndexedDB, `localStorage`, and service-worker origins survive restarts; run UAR through the existing `uar-sidecar` binary via Tauri `externalBin` / `plugin-shell`.

## Other changes left in working tree

- Submodules initialized:
  - `vendor/git/liter-llm`
  - `vendor/git/rust-mcp-filesystem`
  - Rationale: required to run `cargo check`; considered harmless and safe to leave.
- `Cargo.lock` updated:
  - `sse-stream` bumped from `0.2.3` to `0.2.5`.
  - Rationale: fixes a real `rmcp` / `sse-stream` API mismatch.
  - Risk: touches shared lockfile, but likely independent of the persistence migration.
- MCP API rename applied:
  - `Content` → `ContentBlock` in `src/uar/mcp_server.rs`.
  - `Content` → `ContentBlock` in `src/uar/memory/mcp_server.rs`.
  - Risk: highest conflict risk because the concurrent persistence migration is likely restructuring this code.
  - State: left as-is per operator instruction; not reverted and not extended.

## Deferred verification and next steps

After the persistence-migration merge lands:

1. Rebase or re-check the working tree against the merged state.
2. Re-evaluate whether the `Content` → `ContentBlock` rename is still required.
3. Re-evaluate whether the `sse-stream` `0.2.3` → `0.2.5` lockfile bump is still required.
4. Resume `desktop-stable-port` verification.
5. Complete remaining tasks from task groups 3–6:
   - binary placement
   - remaining API verification
   - `design.md` reconciliation
   - verification tasks

## Phase goals still pending beyond this pause

- Align local-first storage by target:
  - Web: keep PGlite `idb://`.
  - Desktop: move data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support.
- Establish Flutter mobile target running UAR fully on-device with embedded SurrealDB / `surrealkv` persistence per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend.
- Produce and remediate a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin-console UX.
- Incorporate six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate TypeScript 7.0 native compiler migration before committing:
  - verify current release status
  - verify Vite/Rolldown compatibility
  - verify ESLint compatibility
  - identify `vue-tsc`-equivalent path if relevant
  - define migration path from TypeScript `5.9.3`
  - follow dependency-verification rules 22/23

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture