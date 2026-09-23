## Why

D4 puts the-boss's UAR data in the shared Docker SurrealDB 3.2.4, namespace `uar`, signed in as a namespace-scoped user with role `EDITOR` — never root. UAR cannot do that today. Remote sign-in is root-only: `SurrealDbProvider::new` always calls `signin(Root { .. })` and falls back to `root`/`root` when no credentials are configured (`src/uar/persistence/providers/surreal.rs:45-56`). The shared database also holds other services' namespaces (`memory` for surreal-memory, `compass`), so a root credential in UAR's hands can read and destroy all of them.

Two more gaps block the desktop case. An unreachable database at startup is a hard failure (`src/server.rs:606-626`), so a stopped Docker daemon means no agent runtime at all, while the project rule is that everything must still work with the Docker services down. And the documented remote example uses `http://127.0.0.1:8000` (`config.remote.surreal.yaml`), while UAR's manifest enables only `kv-surrealkv` on top of `surrealdb` 3.2.4's default features `protocol-ws` and `rustls` (`Cargo.toml:427`; `surrealdb-3.2.4/Cargo.toml` `[features] default`). `protocol-http` is not requested by UAR, so `http://` URLs most likely fail; whether another crate enables it through feature unification was not checked (no `cargo tree` this session).

## What Changes

- New `persistence.surreal_auth_level`: `root` (default, today's behavior), `namespace` or `database`. At `namespace`/`database` UAR signs in with the SDK's `Namespace`/`Database` credentials (`surrealdb-3.2.4/src/opt/auth.rs:37-69`) using `surreal_user`, `surreal_pass`, `surreal_ns` (and `surreal_db`), and never falls back to `root`/`root`. Missing credentials at those levels are a configuration error.
- In sidecar mode a remote URL with `surreal_auth_level: root` is refused at startup (D4: never root).
- `ws://` / `wss://` are the supported remote schemes. When the SDK rejects an `http://`/`https://` URL, the startup error names the `ws://` equivalent. The remote example config moves to `ws://`.
- New `persistence.remote_fallback`: `none` (default, today's hard failure) or `embedded`. With `embedded`, a connection-level failure to the remote at startup (refused, DNS failure, or no connection within `persistence.remote_connect_timeout_ms`, default 3000) starts UAR on an embedded SurrealKV store at `persistence.fallback_database_url` instead of exiting. Authentication, permission and migration errors still fail startup — they are misconfiguration, not an outage.
- The fallback store is local-only as a whole. It is never merged into the remote automatically, and UAR does not switch stores while running. `GET /api/config/persistence` reports the effective store, whether fallback is active and why, with credentials redacted from the URL. `GET /api/uar/persistence/local-only` lists the sessions held in the local-only store, whether or not it is the active store, so a host can mark them and later offer an explicit move. The move itself is not part of this change.
- No SurrealDB feature that needs ≥ 3.3.0 is used (no access `CONTEXT`, no SCRAM).

## Capabilities

### New Capabilities
- `surreal-persistence-connection`: how UAR authenticates to a remote SurrealDB (scoped, never root in sidecar mode), which URL schemes it supports, when it falls back to an embedded store, and how the local-only fallback store is reported and kept separate.

### Modified Capabilities
- None. No existing spec states SurrealDB connection or sign-in behavior.

## Impact

- **Code:** `src/config.rs:644-681` (new fields; `Debug` redaction stays), `src/uar/persistence/providers/surreal.rs:35-92` (auth level; connect timeout), `src/server.rs:606-673` (fallback decision around provider construction), `src/server.rs:3309-3325` (`persistence_info_handler`), a new local-only inventory route, `config.remote.surreal.yaml` (ws URL, namespace user), settings schema for the new persistence keys (`src/uar/settings/manager.rs`, persistence type near `:1351`).
- **APIs:** `GET /api/config/persistence` gains `configured_mode`, `fallback` and a redacted URL; `mode` becomes the effective mode (it differs from today only while fallback is active). New read-only `GET /api/uar/persistence/local-only`.
- **Runtime UX:** with Docker stopped, the-boss still gets a working runtime and can show "local only" for sessions created then. Settings, providers and MCP servers stored in the remote are not available during fallback; the-boss's run-scoped credentials and tools (D1) are unaffected.
- **Provider compatibility:** unaffected.
- **Realtime state:** the live-query bus runs on whichever store is active (`server.rs:651-655`); events during fallback come from the local store only.
- **Operations:** someone with root on the shared SurrealDB must create the `uar` namespace, database and `EDITOR` user once. The statement is in `design.md`; the provisioning script lives with the Docker services in prometheus-skills-mini, not in UAR.
- **Dependencies:** none new. Uses `surrealdb` 3.2.4 as pinned in `versions.toml`.
- **KBD workflow state:** yes — list this change in the-boss child phase `the-boss-universal-agent-runtime` progress.
