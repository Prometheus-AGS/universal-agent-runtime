---
type: Reference
id: liter-llm-mcp-provider-metadata-resources-in-uar-phase
title: liter-llm MCP provider metadata resources in UAR phase
tags:
- universal-agent-runtime
- liter-llm
- mcp-resources
- hybrid-app-architecture
- desktop-stable-port
- tauri
- local-first
links:
- uar-hybrid-app-architecture-phase-context
- uar-hybrid-architecture-pull-update-and-desktop-port-next-step
sources:
- stdin
timestamp: 2026-07-20T02:27:52.491935+00:00
created_at: 2026-07-20T02:27:52.491935+00:00
updated_at: 2026-07-20T02:27:52.491935+00:00
revision: 0
---

## Context

- Phase: [`uar-hybrid-app-architecture`](/uar-hybrid-app-architecture-phase-context.md)
- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Captured: `2026-07-20T02:25:05Z`
- Phase status: `executing`
- Progress: `changes 4/12`
- Related phase update: [UAR hybrid architecture pull update and desktop port next step](/uar-hybrid-architecture-pull-update-and-desktop-port-next-step.md)

## Active phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align the local-first data layer to the hybrid mobile architecture target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per corrected skill documentation.
- Establish the mobile Flutter target where UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized UI/UX fixes with `/impeccable polish` and `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07` as seed work items:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Migrate the frontend toolchain to TypeScript 7.0 native compiler only after verifying release status, ecosystem compatibility, and migration path from TypeScript 5.9.3 under dependency-verification rules 22/23.

## Session finding: `liter-llm` MCP interface

The question concerned the `liter-llm` MCP server's tool/resource interface, not a Rust compile error. The meta-cognition routing block for compile errors did not apply because there were no error codes or matching domain keywords.

`liter-llm` exposes provider and model metadata through MCP **resources**, not a provider-query tool.

### Tools

The MCP server exposes:

- `chat`
- `embed`
- File, batch, and response management tools, including:
  - `create_file`
  - `list_files`
  - `create_batch`
  - `list_batches`
  - `create_response`

### Resources

Provider/model information is exposed via these resources:

- `liter-llm://models` — model catalog
- `liter-llm://providers` — provider list
- `liter-llm://provider/{name}` — provider detail for a named provider
- `liter-llm://pricing/{model}` — per-model pricing

## Phase tracker

- Last completed work item: `admin-agent-model-warning-clarity`
  - Status: `13/13 tasks complete`
  - Archived as: `2026-07-16-admin-agent-model-warning-clarity`
  - Phase progress after completion: `changes 4/12`
- Next planned operation: `/opsx:new desktop-stable-port`

# Citations

1. stdin