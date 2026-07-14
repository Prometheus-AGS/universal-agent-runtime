## Why

UAR currently loads `AppConfig` once at startup from `config.yaml`, env vars, and CLI flags. Operator changes to the config file require a process restart, which interrupts active sessions and in-flight runs. The 2026-07-13 release-readiness assessment flagged runtime config immobility as a grade-A gap for production operations.

The operator's 2026-07-13 analysis selected `notify` for file watching, `arc-swap` for lock-free config replacement, and `vaultrs` for an optional Vault adapter. Change 7 adds hot-reload without dropping active sessions or runs: the existing `AppConfig` is wrapped in an `ArcSwap`, and a background watcher atomically swaps in a new configuration on file change.

## What Changes

- Add `arc-swap` and `notify` as runtime dependencies; add `vaultrs` as an optional dependency behind a `vault` feature.
- Introduce a `ConfigManager` that owns the current `AppConfig` via `ArcSwap<AppConfig>` and watches the loaded config file path with `notify`.
- Provide a reload path that rebuilds `AppConfig` from the same sources (config file, env, CLI defaults) and atomically swaps the active instance.
- Add a `--strict-config` CLI flag and `UAR_STRICT_CONFIG` env var that turns override-conflict warnings into hard errors during reload.
- Add a `vault` feature that, when enabled, adds a Vault-backed config source via `vaultrs` (`vault://...` URLs in config values) as a lower-priority source.
- Wire `ConfigManager` into `AppState` and `start_server` so handlers can access the live config.
- Add unit tests for the reload swap and a soak-style test that verifies active sessions survive a config reload.

## Capabilities

### New Capabilities

- `config-hot-reload`: runtime, session-preserving config reload via `notify` + `arc-swap`.

## Impact

- **No restart required** for config file changes that do not affect the network listener or persistence backend connection string.
- **Active sessions and runs are preserved** because the reload only swaps the config `Arc`; runtime objects (run manager, session store) are not recreated.
- **Override conflicts are detectable:** `--strict-config` makes it an error when two sources disagree on a value.
- **Vault integration is opt-in:** the `vault` feature is off by default; operators enable it only when they want Vault-backed secrets.
- **No breaking API change** to `AppConfig` fields; the hot-reload layer wraps the existing config type.

## Out of scope

- Migrating every existing handler from `Arc<AppConfig>` to `ConfigManager` in one change. This change establishes the manager and updates the server core; incremental migration of remaining call sites is follow-up.
- Reloading secrets that are cached inside long-lived connections (e.g., LLM client internal state) without an explicit refresh. The new config is available immediately to new requests; existing clients may continue using cached values until recreated.
- A full Vault policy/role workflow. The Vault adapter supports KV-v2 reads; auth method and policy setup are operator work.
