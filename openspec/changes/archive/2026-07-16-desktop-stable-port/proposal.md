## Why

The Tauri desktop shell binds the embedded server to `127.0.0.1:0` (a random
free port) on every launch and navigates the webview to that origin.
IndexedDB, localStorage, and Service Worker caches are scoped per-origin, and
origin includes the port — so every desktop launch silently orphans the
PGlite thread/message cache (`idb://uar-threads`), wipes localStorage (theme,
onboarding state), and strands Service Worker caches under dead ports. This
is a P0 data-integrity defect: users lose their local-first state on every
restart of the desktop app, and the loss is invisible because server-side
SurrealDB data survives, masking the bug in casual testing.

## What Changes

- `resolve_localhost_port` in `src-tauri/src/lib.rs` stops defaulting to a
  random ephemeral port. It resolves a stable port (from config, with the
  existing `TAURI_LOCALHOST_PORT` override honored first) and persists the
  resolved value across launches so the webview origin is identical every
  time the app starts.
- Port-in-use conflicts are resolved deterministically once and the outcome
  is persisted (app config directory), not re-randomized on the next launch.
- No change to the embedded-server-vs-sidecar question — that is
  `desktop-sidecar-conversion`, a separate change that depends on this one
  landing first (both changes touch the same origin-stability contract, but
  this change must ship independently since it fixes an active data-loss bug
  today, before the sidecar work is ready).

## Capabilities

### New Capabilities
- `desktop-shell`: Governs the Tauri desktop application's runtime shell —
  server bootstrap, webview origin stability, and (in later changes) sidecar
  process lifecycle. This change establishes its first requirement: stable,
  persisted origin across launches.

### Modified Capabilities
(none — `desktop-shell` does not exist yet in `openspec/specs/`)

## Impact

- **Runtime UX**: desktop users stop losing conversation-thread cache, theme
  preference, and onboarding state on every app restart. No visible UX
  change when the fix works correctly — that is the point; the current
  visible symptom (state resetting) disappears.
- **Provider compatibility**: none. This change does not touch LLM provider
  routing, request construction, or any server-side API surface.
- **Realtime state**: none. Server-side SurrealDB persistence (threads,
  memory, settings) already survives port changes and is unaffected; only
  the client-side, per-origin cache stability is in scope.
- **Affected code**: `src-tauri/src/lib.rs` (`resolve_localhost_port`,
  `wait_for_ready` call site), `src-tauri/tauri.conf.json` if a config-driven
  default port is introduced.
- **KBD workflow state**: this is change 1 of 12 in
  `.kbd-orchestrator/phases/uar-hybrid-app-architecture/plan.md` (Round 1,
  no dependencies). On completion, `progress.json` for this phase must be
  updated via `scripts/kbd-validate-progress.sh --mark-implementation-complete`
  before `desktop-sidecar-conversion` (change 10, which depends on this one)
  can be dispatched.
