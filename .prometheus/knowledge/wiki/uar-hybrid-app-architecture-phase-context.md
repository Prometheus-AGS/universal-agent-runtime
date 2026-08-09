---
type: Reference
id: uar-hybrid-app-architecture-phase-context
title: UAR hybrid app architecture phase context
tags:
- universal-agent-runtime
- tauri
- desktop-stable-port
- local-first
- mobile-architecture
- typescript-7
- ui-ux-audit
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-20T02:24:00.224471+00:00
created_at: 2026-07-20T02:24:00.224471+00:00
updated_at: 2026-07-20T02:24:00.224471+00:00
revision: 0
---

## Phase context

- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
- Phase: `uar-hybrid-app-architecture`
- Captured: `2026-07-20T02:23:14Z`

## Phase goals

### Desktop data-loss fix

- Fix the P0 desktop data-loss bug caused by using a random localhost port per launch in `src-tauri/src/lib.rs`.
- Replace random-port-per-launch behavior with a stable, persisted localhost port.
- Rationale: IndexedDB, `localStorage`, and service worker storage are origin-scoped; changing the port changes the origin and makes persisted browser storage appear lost after restart.

### Tauri sidecar architecture

- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary.
- Use Tauri `externalBin` / `plugin-shell` for sidecar execution.
- Expose the fixed port to the operating system.

### Local-first data-layer target matrix

Align the data layer with the `hybrid-mobile-architecture` skill's per-target matrix:

- Web: keep PGlite using `idb://`.
- Desktop: move the data layer into Rust via `pglite-oxide`.
- Mobile: use SQLite plus `sqlite-vec` via `gen_ui_core` FFI.
- Constraint: `pglite-oxide` has no iOS/Android support per the corrected skill documentation.

### Mobile target

- Establish the Flutter mobile target.
- UAR must run entirely on-device on mobile.
- Persistence uses embedded SurrealDB with `surrealkv`, per `TJ-ARCH-MOB-001`.

### UI/UX audit and remediation

Run the following against the React frontend:

- `/impeccable audit`
- `/impeccable critique`

Expected output:

- Scored UI/UX defect inventory covering:
  - Brittleness
  - Stalls
  - Freeze paths
  - Admin console UX

Then execute prioritized fixes with:

- `/impeccable polish`
- `/impeccable harden`

### Supplemental Admin/Agents UI seed work

Absorb six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07` as seed work items:

- `sw-scheme-safe-caching`
- `model-warning-clarity`
- `provider-first-model-picker`
- `edit-panel-verification`
- `governance-reconciliation`
- `freeze-diagnostics`

### TypeScript 7.0 toolchain migration

- Migrate the frontend toolchain to TypeScript 7.0 using the native compiler.
- Before committing, verify:
  - Current TypeScript 7.0 release status.
  - Ecosystem compatibility.
  - Vite / Rolldown compatibility.
  - ESLint compatibility.
  - `vue-tsc`-equivalent tooling status.
  - Migration path from TypeScript `5.9.3`.
- Follow dependency-verification rules 22/23 before committing dependency changes.

## Pull/session summary

A pull completed successfully:

- Fast-forwarded from `06d7172` to `563ecc2`.
- Included a submodule update for `prometheus-entity-management`.
- Changed areas included:
  - CI workflows
  - `Cargo.lock`
  - `Cargo.toml`
  - Frontend lockfiles
  - Rust source files

Notable Rust changes:

- Added `src/llm/external_driver.rs`.
- Updated `src/uar/runtime/manager.rs`.
- Updated `src/normalized.rs`.
- Updated SDK types.

## Next step

Per the position tracker, the next planned operation is:

```text
/opsx:new desktop-stable-port
```

Optional pre-step due to the pull size:

```bash
cargo check --locked --no-default-features --features server-full
```

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture