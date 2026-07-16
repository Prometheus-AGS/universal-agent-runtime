## Context

`src-tauri/src/lib.rs::resolve_localhost_port` currently:
1. Honors `TAURI_LOCALHOST_PORT` if set (already stable — no change needed here).
2. Otherwise binds `TcpListener::bind("127.0.0.1:0")` and takes whatever
   ephemeral port the OS hands back.
3. Falls back to `app_config.server.port` only if the bind itself fails.

Step 2 is the bug: it is the *default* path (no env var set), and it
deliberately asks the OS for a random port every time. The webview is then
navigated to `http://127.0.0.1:<that random port>`, so the browser origin —
and therefore every per-origin store (IndexedDB `idb://uar-threads`,
localStorage, Service Worker cache) — changes on every launch.

## Goals / Non-Goals

**Goals:**
- The desktop webview's origin (host:port) is identical across app restarts
  on the same machine, by default, with no environment variable required.
- Preserve `TAURI_LOCALHOST_PORT` as an explicit override for users/CI who
  need a specific port.
- No data migration needed — this only changes which port the *next* launch
  picks; browser storage tied to old random-port origins is simply orphaned
  going forward (already the current behavior; not being fixed retroactively).

**Non-Goals:**
- Not converting the embedded server to a sidecar (`desktop-sidecar-conversion`,
  change 10, depends on this one landing first).
- Not adding any UI for the user to choose or view the port.
- Not handling the case where the stable port is occupied by a *different*
  unrelated process on the machine (see Risks).

## Decisions

**D1 — Default to `app_config.server.port` instead of an OS-assigned ephemeral port.**
The config struct already has a `server.port` field (used as the fallback
today only if the ephemeral bind fails). Flip the priority: try the
configured port first; only fall back to ephemeral-and-persist if that
specific port is genuinely unavailable (e.g., another instance of the app
already running, or a real conflict).
*Alternative considered*: always bind `:0` and persist whatever the OS
assigns via a config file. Rejected as unnecessary complexity — the app
already ships a configured port; reusing it is simpler and matches user
expectation (same port every time, matching most desktop-app conventions).

**D2 — On conflict, resolve once and persist the fallback port.**
If the configured port is taken (e.g., the user already has another instance
running, or `uar-jwt-proxy`/a dev server is on it), fall back to an ephemeral
bind *once*, then write the resolved port to the Tauri app-config directory
(`app.path().app_config_dir()`) so subsequent launches reuse it instead of
re-randomizing. This keeps origin stability even in the conflict case,
without requiring the user to manually set `TAURI_LOCALHOST_PORT`.
*Alternative considered*: fail hard on conflict and ask the user to close
the other instance. Rejected — worse UX for a problem the app can resolve
transparently, and conflicts are expected to be rare in practice (single
desktop app instance is the common case).

**D3 — `TAURI_LOCALHOST_PORT` keeps top priority, unchanged.**
No behavior change for anyone already setting it (e.g., automated testing).

## Risks / Trade-offs

- **[Risk]** A persisted fallback port could itself become stale (e.g., a
  different app claims it between launches). → **Mitigation**: the existing
  bind-and-retry logic already handles a single conflict; if the *persisted
  fallback* port is also taken on a later launch, repeat D2's resolve-and-
  persist step rather than reverting to pure-random. Origin only changes
  when truly unavoidable, and only once per actual conflict.
- **[Risk]** Users who already hit this bug have orphaned per-origin browser
  data sitting under old random-port origins with no cleanup path. →
  **Mitigation**: out of scope for this change (no retroactive data
  migration per Non-Goals); acceptable since that data was already
  functionally lost the moment the origin first changed.
- **[Trade-off]** Reusing `server.port` as the default means the desktop
  app's local port is predictable/discoverable on the machine. This is
  acceptable and consistent with the operator's stated goal (`desktop-sidecar-conversion`,
  change 10) of exposing the port to the OS for other local consumers.

## Migration Plan

No data migration. Deploy as a normal code change:
1. Ship the updated `resolve_localhost_port`.
2. On first launch after upgrade, the app binds `server.port` instead of a
   random port — a one-time origin change from whatever random port the
   previous launch used (expected and harmless, matches Risk 2 above).
3. All subsequent launches reuse that same origin.

No rollback complexity — reverting the binary reverts to the old (buggy)
random-port behavior, no persisted state to clean up.

## Open Questions

- Should the resolved/persisted fallback port (D2) be surfaced anywhere in
  the UI (e.g., an "about" or diagnostics panel) for support purposes? Not
  required for this change; flag for `desktop-sidecar-conversion` since that
  change already deals with exposing the port to the OS.
