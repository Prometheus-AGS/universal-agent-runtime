## 1. Add dependencies
- [x] 1.1 Add `arc-swap = "1.6"` to `Cargo.toml` runtime dependencies.
- [x] 1.2 Re-use the existing `notify = "8"` dependency for the file watcher.
- [x] 1.3 Add `vaultrs = { version = "0.8", optional = true }` and a `vault` feature.
- [x] 1.4 Verify `cargo check --no-default-features --features server-full` and `--features server-full,vault` both compile.

## 2. Create ConfigManager
- [x] 2.1 Add `src/config_manager.rs` with a `ConfigManager` struct.
- [x] 2.2 `ConfigManager` holds `ArcSwap<AppConfig>` and the resolved config file path.
- [x] 2.3 Implement `ConfigManager::load(cli: Cli) -> Result<Arc<Self>, ConfigError>` that builds the first `AppConfig` using `AppConfig::load_with_cli`.
- [x] 2.4 Implement `ConfigManager::current(&self) -> Arc<AppConfig>` returning the active config via `ArcSwap::load_full`.
- [x] 2.5 Implement `ConfigManager::reload(&self) -> Result<(), ConfigError>` that rebuilds `AppConfig` and atomically swaps it via `ArcSwap::store`.

## 3. Hot-reload watcher
- [x] 3.1 Spawn a background `tokio` task in `ConfigManager` that watches the config file path with `notify`.
- [x] 3.2 On `notify` events (Modify, Create), debounce 500 ms and call `reload()`.
- [x] 3.3 Log `info!` on successful reload and `error!` on reload failure with the error.
- [x] 3.4 Stop the watcher on `ConfigManager` drop via a oneshot shutdown signal.

## 4. Strict-config mode
- [x] 4.1 Add `--strict-config` boolean CLI flag to `Cli` in `src/config.rs` (and `UAR_STRICT_CONFIG` env var).
- [x] 4.2 In `ConfigManager::reload`, when strict mode is enabled, reject the reload if the new effective config differs from the initial snapshot.
- [x] 4.3 Document the strict mode behavior in the change proposal and spec.

## 5. Vault adapter (feature-gated)
- [x] 5.1 Add `#[cfg(feature = "vault")]` module `src/config/vault.rs` that reads `vault://mount/path` and `vault://mount/path#field` URLs using `vaultrs`.
- [x] 5.2 Run `vault::resolve` after `AppConfig::load_with_cli` in `ConfigManager::load` and `reload` when the `vault` feature is enabled.
- [x] 5.3 The Vault source is configured via env vars: `VAULT_ADDR`, `VAULT_TOKEN`, and `VAULT_MOUNT` (default `secret`).
- [x] 5.4 Verify the `vault` feature compiles with `cargo check --no-default-features --features server-full,vault`.

## 6. Server integration
- [x] 6.1 Change `start_server` and `start_server_sidecar` signatures to accept `Arc<ConfigManager>`.
- [x] 6.2 Add `config_manager: Arc<ConfigManager>` to `AppState`.
- [x] 6.3 Update `src/server.rs` and `src/main.rs` to wire the manager into startup and the admin reload endpoint.
- [x] 6.4 Add `POST /.well-known/uar-config/reload` admin endpoint that calls `config_manager.reload().await` and returns the config schema when `X-UAR-Admin-Key` is present (or when admin-key auth is disabled).

## 7. Verification
- [x] 7.1 Add unit tests in `src/config_manager.rs` verifying `ConfigManager::reload` atomically swaps the config and that strict mode rejects changes.
- [x] 7.2 Add parser tests for `vault://` URL parsing in `src/config/vault.rs`.
- [x] 7.3 Run `cargo check --no-default-features --features server-full` and `cargo check --no-default-features --features server-full,vault`.
- [x] 7.4 Run `cargo test --no-default-features --features server-full --lib config_manager` and ensure green.
- [ ] 7.5 Run `openspec validate --strict --changes config-hot-reload-vault` and confirm validity.
- [ ] 7.6 Mark Change 7 implementation complete in `progress.json` and update `current-waypoint.json`.
