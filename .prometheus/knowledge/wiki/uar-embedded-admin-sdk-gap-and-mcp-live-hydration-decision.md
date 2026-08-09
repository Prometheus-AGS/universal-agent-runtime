---
type: Reference
id: uar-embedded-admin-sdk-gap-and-mcp-live-hydration-decision
title: UAR embedded admin SDK gap and MCP live hydration decision
tags:
- universal-agent-runtime
- embedded-uar
- sdk-runtime
- mcp-registry
- surrealdb
- hybrid-app-architecture
- mobile-offline
links:
- uar-desktop-stable-port-openspec-ready-for-apply
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-28T00:26:42.926753+00:00
created_at: 2026-07-28T00:26:42.926753+00:00
updated_at: 2026-07-28T00:26:42.926753+00:00
revision: 0
---

## Context

- Phase: `uar-hybrid-app-architecture`
- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system/rust/vendor/universal-agent-runtime`
- Captured: `2026-07-28T00:16:26Z`
- Current position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 0/1`

This session extends the hybrid app architecture work previously tracked in [UAR desktop stable port OpenSpec ready for apply](/uar-desktop-stable-port-openspec-ready-for-apply.md) and related desktop stable-port execution notes.

## Phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the OS.
- Align local-first storage with the hybrid mobile architecture target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` through `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support.
- Establish the Flutter mobile target where UAR runs entirely on-device with embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory, then execute prioritized fixes with `/impeccable polish` and `/impeccable harden`.
- Absorb 6 supplemental Admin/Agents UI items from `uar-grade-a-upgrade-2026-07` as seed work:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler before committing:
  - verify release status;
  - verify Vite/Rolldown compatibility;
  - verify ESLint and `vue-tsc`-equivalent ecosystem support;
  - define migration path from TypeScript `5.9.3`;
  - follow dependency-verification rules 22/23.

## Storage and embedded-admin findings

The audit gap is not missing persistence; it is missing SDK exposure. Current storage state:

| Subsystem | Storage today | Gap |
|---|---|---|
| Skills | DB-backed via `save_skill` / `list_skills` on SurrealDB | SDK does not expose operations |
| Knowledge bases | DB-backed | SDK does not expose operations |
| Memory | DB-backed | SDK does not expose operations |
| MCP servers | `SettingsManager` under key `mcp.servers`, which is DB-backed | Reachable only via HTTP |

`not_on_embedded` is accurate for current KnowMe behavior because the SDK `Runtime` lacks these operations. The storage layer already exists and should not be rebuilt.

## MCP persistence pattern already exists

`mcp_admin.rs` already implements the intended control flow:

1. `stored_servers`
2. `persist`
3. `hydrate_registry`

Config files seed the DB, the DB drives the live registry, and mutations re-hydrate without a reboot. The problem is architectural placement: this implementation is trapped in the HTTP layer, so embedded clients cannot call it directly.

## Correct architectural fix

Move admin handler bodies into transport-free service modules and expose them through the SDK `Runtime`.

Planned shape:

- Extract 4 UAR admin service modules:
  - skills admin service;
  - knowledge admin service;
  - memory admin service;
  - MCP admin service.
- Expose approximately 20 methods on the SDK `Runtime`.
- Convert HTTP routes into thin adapters over the same services.
- Replace KnowMe `not_on_embedded` branches with direct SDK calls.
- Remove `unavailable` markers added for embedded admin gaps once SDK support exists.

Rationale: embedded and remote modes should share one implementation path. Parallel HTTP-only and SDK-only implementations would drift.

## Open decision: MCP live re-hydration semantics

`McpRegistry` stores `server_config` in an in-memory `RwLock<HashMap>` populated through `load_from_file`. Making saved edits take effect live requires re-hydrating the registry on write. That touches lifecycle for already-connected MCP servers and may interrupt:

- in-flight tool calls;
- authentication renegotiation;
- active server sessions;
- reconnect error handling.

Two implementation options are under decision:

### Option A — hydrate on write

- Apply full live reload immediately after writes.
- Explicitly handle disconnect/reconnect behavior for already-connected MCP servers.
- Matches the requested live-reload behavior.

### Option B — defer edits to connected servers

- Hydrate on write for new and removed servers.
- Defer edits to already-connected servers until the next session.
- Surface pending state with an `unavailable`-style reason.
- Lower reconnect churn risk; suggested as safer initial shipping behavior.

Skills, knowledge, memory, and SDK surface work proceed identically under either option.

## Immediate next step

Await A/B decision on MCP live re-hydration for already-connected servers, then:

1. extract the 4 admin services in UAR;
2. expose admin methods on SDK `Runtime`;
3. replace KnowMe `not_on_embedded` branches;
4. remove now-false embedded-unavailable markers.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture