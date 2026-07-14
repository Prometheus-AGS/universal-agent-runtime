# UAR config hot-reload and Vault

## Purpose

Allow UAR to reload configuration at runtime from the watched `config.yaml` file without dropping active sessions or in-flight runs, and optionally read secrets from HashiCorp Vault.

## ADDED Requirements

### Requirement: Runtime config reload
A `ConfigManager` MUST exist that owns the active `AppConfig` behind an `ArcSwap`. The server MUST be able to atomically swap the active configuration at runtime, either automatically when the config file changes or via an explicit admin endpoint.

#### Scenario: An operator edits config.yaml
- **WHEN** the watched config file is modified on disk
- **THEN** `ConfigManager` rebuilds `AppConfig` from the same sources (defaults, file, env, CLI) and atomically swaps the active instance
- **AND** existing sessions and runs continue without interruption
- **AND** new requests observe the updated config immediately

### Requirement: Lock-free access
All runtime code that reads request-time configuration values MUST access the current config through `ConfigManager::current()`, which returns an `Arc<AppConfig>` via `ArcSwap::load`. Direct long-term ownership of `Arc<AppConfig>` by handlers is permitted, but the authoritative live config is always the one stored in `ArcSwap`.

#### Scenario: A handler reads `server.port` at request time
- **WHEN** a handler needs a runtime config value
- **THEN** it calls `state.config_manager.current()` to get the latest config
- **AND** the value is consistent for the duration of that request

### Requirement: Explicit reload endpoint
A `POST /api/uar/config/reload` endpoint MUST be available to admin callers. It MUST call `ConfigManager::reload()` and return the JSON schema of the new config (or an error if reload failed). This endpoint MUST be restricted to admin users.

#### Scenario: An admin triggers a reload
- **WHEN** an admin POSTs to `/api/uar/config/reload`
- **THEN** the server reloads the config from disk
- **AND** returns the new config schema or a 500 with the error message

### Requirement: Strict config mode
When `--strict-config` is enabled (or `UAR_STRICT_CONFIG=true`), any reload that detects an override conflict (e.g., the same key set by both the config file and an environment variable with different values) MUST return an error instead of silently picking one source.

#### Scenario: Env var and config file disagree
- **WHEN** `UAR_SERVER__PORT=8080` and `config.yaml` sets `server.port: 1906`
- **AND** strict mode is enabled
- **THEN** reload fails with a clear error naming the conflicting key

### Requirement: Vault feature (optional)
When the `vault` Cargo feature is enabled, `ConfigManager` MUST add a Vault KV-v2 source as a lower-priority layer. Values may be written as `vault://mount/path/to/key` in the config file; the Vault source resolves them to their actual secret values.

#### Scenario: Vault feature is enabled
- **WHEN** the crate is built with `--features vault`
- **AND** a config value is `vault://secret/uar/jwt_secret`
- **THEN** the Vault source reads the secret and substitutes it before deserialization
- **AND** when the feature is disabled, the literal string is kept and deserialization fails as before

#### Scenario: Vault feature is disabled
- **WHEN** the crate is built without the `vault` feature
- **THEN** no Vault code is compiled in and no runtime dependency on Vault exists

### Requirement: Session/run preservation
Reloading the configuration MUST NOT destroy the `RunManager`, session store, or any in-flight run. A test MUST verify that a run started before a reload continues to completion after the reload.

#### Scenario: A run is in progress during reload
- **WHEN** a run is active
- **AND** the config is reloaded
- **THEN** the run continues and can still be subscribed to and completed
