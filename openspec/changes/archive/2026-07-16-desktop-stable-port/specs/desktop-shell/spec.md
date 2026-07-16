## ADDED Requirements

### Requirement: Stable webview origin across desktop launches
The Tauri desktop shell SHALL resolve the local server port such that the
webview's origin (`http://127.0.0.1:<port>`) is identical across successive
launches of the application on the same machine, so that per-origin browser
storage (IndexedDB, localStorage, Service Worker caches) persists between
sessions.

#### Scenario: Default launch reuses the configured server port
- **WHEN** the desktop app starts and `TAURI_LOCALHOST_PORT` is not set
- **THEN** the embedded server binds to `app_config.server.port` (not an
  OS-assigned ephemeral port), and the webview is navigated to that same
  `127.0.0.1:<port>` origin on every launch

#### Scenario: Explicit port override is honored
- **WHEN** the desktop app starts and `TAURI_LOCALHOST_PORT` is set to a
  valid port number
- **THEN** the embedded server binds to that port, taking priority over the
  configured default

#### Scenario: Configured port is unavailable
- **WHEN** the desktop app starts, `TAURI_LOCALHOST_PORT` is not set, and
  `app_config.server.port` is already bound by another process
- **THEN** the server resolves an available port once, persists that
  resolved port to the Tauri app-config directory, and reuses the persisted
  port (instead of re-resolving a new random port) on subsequent launches
  until the originally configured port becomes available again

#### Scenario: Per-origin browser state survives a restart
- **WHEN** a user has an active desktop session with local-first data cached
  under the app's webview origin (e.g. thread history in IndexedDB) and
  restarts the application
- **THEN** the restarted session's webview origin matches the prior
  session's origin, and the cached data remains accessible
