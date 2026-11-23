## 1. Preconditions and ownership

- [ ] 1.1 Obtain a single-writer assignment from the lead for `src/main.rs`, `src/config.rs`, `src/config_manager.rs`, `src/server.rs`, `src/uar/settings/manager.rs`, `src/uar/settings/provider_sync.rs`, `src/uar/settings/schema.rs`, `src/uar/api/settings.rs`, `src/uar/api/administration_capabilities.rs` and `src/uar/api/openapi.rs`. None of these is in the `uar-state` ownership list. Agree with `uiux-system-model-live-default` which writer owns `manager.rs` provider and default-provider code.
- [ ] 1.2 Agree the response shapes of `boot-status`, `reload` and `drift` (design D6-D8) with the owner of `uiux-settings-truthful-status-and-removals` before either change implements them.
- [ ] 1.3 Confirm `uiux-realtime-topics-and-replica-refresh` exports the `ProviderRefresh` trait this change calls (design D7). If it does not exist yet, define the trait in `src/uar/settings/` and let that change implement it.

- [ ] 1.4 Treat `fixtures/cfg-oracle.json` as a frozen oracle input: the boundary runner reads it, the implementer does not edit it, and a needed change is a revision of this change.
- [ ] 1.5 Depend on `uiux-settings-truthful-status-and-removals` for the apply-class table (`live`, `next_turn`, `restart`, `inert`; its design D1/D3). Reload reconcile (4.1) reports `restart_required` and `not_applied` and updates the effective map from that table, not from the current three-way `setting_apply_mode`. Until the table exists, treat every key outside `live`/`next_turn` as `restart`.

## 2. Last-known-good copy and fail-closed boot fallback (build item 7)

- [ ] 2.1 Add the LKG module: directory resolution (`UAR_LAST_KNOWN_GOOD_DIR`, else `~/.uar/last-known-good/`), the digest record `{version, sha256, saved_at, source_path, format, binary_version}`, and the atomic owner-only write (temporary file, `fsync`, rename, directory `fsync`). Reuse the `sha2` dependency already declared at `Cargo.toml:631`.
- [ ] 2.2 Split `AppConfig::load_with_cli` into typed stages: source read, layer build with deserialize, validation (every check at `src/config.rs:2012-2030`, vault resolution, `security.validate`). Return `LoadFailure::{SourceRead, SourceParse, Refusal}`. Keep the public `load_with_cli` signature working for callers outside `ConfigManager`.
- [ ] 2.3 Make `ConfigManager::load` and `load_without_watcher` read the resolved file once and keep `{bytes, sha256, path}` as the validated source of the config that booted. Save it through 2.1 after the readiness checkpoint in `start_server`, only when the boot used the primary file. A save failure logs `config.lkg.save_failed` and records its class; it never stops the start.
- [ ] 2.4 Add a load variant that substitutes bytes for the file (`config::File::from_str`), leaving CLI, environment, legacy and provider-key layers untouched.
- [ ] 2.5 Implement the fallback decision in `ConfigManager::load` exactly as design D2: eligible failure stage only, existing file only, server run only, not strict, not disabled by `UAR_CONFIG_FALLBACK=off`, digest-valid copy, second load passes all stages, second load not remote. One-shot commands (`main.rs:92`) and `--strict-config` / `UAR_STRICT_CONFIG=true` never fall back. Ineligible cases return the original error unchanged.
- [ ] 2.6 Emit `config.boot.fallback_active` with class, YAML key path when the parser reports one, failed source path, failed digest, copy digest and copy saved time (`failed_source_path` is the configured file's path). Never log a parsed value or raw parser text on this path. Record the same in the boot status store.
- [ ] 2.7 Keep the watcher on the primary path during a fallback start; on the first successful reload set status to `fallback_cleared_by_reload` and leave the saved copy untouched.
- [ ] 2.8 Keep `main.rs:99` reporting the original error when fallback is ineligible; make the exit code unchanged. Add a comment listing the refusal stage so a future validation added to the Validate stage is excluded by construction.

## 3. Provider seed and tolerant persisted values (build items 7 and 9)

- [ ] 3.1 Extract the per-key branch of `initialize_with_governance_default` (`src/uar/settings/manager.rs:241-327`) into one pure `decide` function returning `Insert | OverwriteConfigOwned | KeepWithDrift | Unchanged`, and call it from the boot pass. Behavior of the boot pass must not change.
- [ ] 3.2 Replace `seed_providers_from_registry` with `reconcile_providers` per design D5. Derive `desired` from a scratch `ProviderRegistry` seeded through `seed_from_llm_config` / `seed_from_configs`; read those functions first and confirm they have no effect outside the scratch registry.
- [ ] 3.3 Decide and document where the offered-model set is stored (field inside the row's `data`, or an `internal.*` settings row). Read `ProviderConfig` decoding for unknown-field tolerance and check whether an `internal` settings type exists for the Postgres foreign key before choosing. The behavior is fixed: an operator-removed model is not re-added.
- [ ] 3.4 Stamp seeded provider rows config-owned (`updated_at: None`, replacing `src/uar/settings/manager.rs:997`); keep the stamp on operator writes (`:1082`). Do not migrate existing rows.
- [ ] 3.5 Make `load_provider_configs_from_db` and `protect_provider_credentials` skip an undecodable `provider.*` row, return `{configs, skipped: [{key, reason}]}`, and never rewrite or delete the row. Update `hydrate_provider_registry_from_settings` and its call site (`src/server.rs:1173-1180`) to pass skipped rows to the boot status store.
- [ ] 3.6 Make the persisted embedding overlay (`src/server.rs:1244-1267`) skip each of `backend`, `model`, `vector_dimension`, `send_dimensions` independently on a decode error or an out-of-range dimension, record `{key, reason}` in the boot status store, and keep the existing keep-config-built-backend branches. Leave governance bootstrap fail-closed handling and the local-only audit as they are.

## 4. Reload reconcile, drift and routes (build item 8)

- [ ] 4.1 Implement `SettingsManager::reconcile_from_config(&AppConfig) -> ReconcileReport` using `decide`: merge per key into the cache, never delete a row, skip `governance.*`, write config-owned rows with `updated_at: None`, update the effective map only for keys the class table marks `live` or `next_turn`, report `restart` keys, and `inert` keys with a YAML boot reader that were edited in the YAML, under `restart_required`, and keys the table lists with no reader under `not_applied`. Report keys only.
- [ ] 4.2 Add one reconciler with a single-flight lock shared by the file watcher path and the explicit route. Have `ConfigManager::reload` expose a reload notification (watch channel) that the reconciler subscribes to.
- [ ] 4.3 Wire the reconciler in `start_server` under the existing supervisor, after settings initialization. After provider rows are written, call the `ProviderRefresh` trait object (task 1.3) so the live registry matches the rows before the route returns; with no settings manager or no bus it is a no-op.
- [ ] 4.4 Run `reconcile_providers` (3.2) from the reconciler on every successful reload.
- [ ] 4.5 Add `POST /api/uar/settings/reload` (admin-key guarded) returning `{status, reloaded_at, applied, retained_with_drift, restart_required, not_applied, skipped}`; a rejected reload returns `422 {status: "rejected", class}` with no write.
- [ ] 4.6 Make `POST /.well-known/uar-config/reload` (`src/server.rs:3726`) run the same pipeline and keep returning the schema document.
- [ ] 4.7 Keep strict-mode behavior: a reload rejected by `StrictConflict` performs no reconcile and reports `rejected`.
- [ ] 4.8 Extend `GET /api/uar/settings/drift` additively with `config_value` (through existing masking), `detected_at`, `detected_by`; provider drift lists `fields` only. Hold the desired value in memory from the last reconcile. `config_value` goes through the same `x-sensitive` masking as the stored value (`src/uar/api/settings.rs:557-597`); the resolve and reload responses carry keys only.
- [ ] 4.9 Add `POST /api/uar/settings/drift/{key}/resolve` with `{"resolution":"use_config"}`: store the desired value config-owned, clear drift, return `{key, revision}`; `404` without drift; `409` for `llm.default_provider` and `governance.*`.
- [ ] 4.10 Add `GET /api/uar/settings/boot-status` (admin-key guarded) with the shape in design D8, backed by one boot status store populated by tasks 2.3, 2.6, 3.5, 3.6.

## 5. Registry, documentation and consumers

- [ ] 5.1 Register the three new routes in `src/uar/api/administration_capabilities.rs` (alongside `settings.reload`) and `src/uar/api/openapi.rs`.
- [ ] 5.2 Document `UAR_LAST_KNOWN_GOOD_DIR`, `UAR_CONFIG_FALLBACK`, the fallback eligibility rules and the residual risk in the configuration reference and `example.config.yaml` comments. Add the container note that the directory must be on a writable volume (`k8s/base/uar-deployment.yaml:63-65`).
- [ ] 5.3 The console screens that render boot status, drift with the `use_config` action and the reload report are owned by `uiux-settings-truthful-status-and-removals`: #4.5 (boot banner), #4.6 (drift list with one-click resolve), #4.7 (reload report), #4.8 (realtime wiring), #1.9 (response-shape agreement), and its acceptance tasks #5.7 and #5.10. This change delivers the routes and response shapes (design D6-D8), including `failed_source_path` in boot status, and adds no frontend code.

## 6. Boundary acceptance (runs only at the phase boundary)

These tasks describe what the final integration gate exercises. The store fixture tool is built once beforehand: `cargo build --locked --example surrealql_store_runner --no-default-features --features surreal-backend`. They must not be executed
before the whole phase's implementation is complete. Each scenario uses real files, a real
embedded store and the production binary; none sends a credential, JWT or login; each runs in
its own scratch root with no live service port. Build once, serialized:
`cargo build --locked --no-default-features --features server-full`.

- [ ] 6.1 SCN-CFG-001: start from a YAML with environment-supplied sentinel secrets; inspect `lkg/` for byte equality, digest, modes and absence of sentinels; run the no-file and read-only-directory controls.
- [ ] 6.2 SCN-CFG-002: start, stop, replace `config.yaml` with invalid YAML, start; verify readiness on the copy, status body, no value text; run the remote-profile control.
- [ ] 6.3 SCN-CFG-003: start with `security.jwt_required: false`, stop, apply each frozen refusal edit in turn (`jwt-fallback-secret`, `admin-key-missing`), start; verify non-zero exit, no listener, the frozen log text, no fallback line, copy unchanged; run the deliberate-secret positive guard.
- [ ] 6.4 SCN-CFG-004: write the frozen malformed embedding rows (bad type, then out of range) into the stopped scratch store with `surrealql_store_runner`; start; verify readiness, status entry, row unchanged; run the malformed `governance.enabled` control.
- [ ] 6.5 SCN-CFG-005: create a provider through the real API, stop, write the frozen undecodable `provider.broken` row (credential-like sentinel), start; verify the surviving provider, status entry, row unchanged and the sentinel absent from status, responses and logs.
- [ ] 6.6 SCN-CFG-006: operator edit versus YAML change across reload and restart, additive model merge, operator-removed model stays removed, untouched provider follows YAML, invalid reload returns `422` with no write.
- [ ] 6.7 SCN-CFG-007: reload report lists `resilience.requests_per_second` as applied, `memory.max_context_tokens` and `server.port` (YAML edit) as restart-required and `vision.auto_detect` as not-applied, then a DB edit of `server.port` reads back `not_applied`; drift resolve with `use_config`; `409`, `404` and `400` controls.
- [ ] 6.8 SCN-CFG-008: drift on `memory.openai_api_key` with three sentinels; verify masked drift, reload and resolve responses, no sentinel in any body or process output, and the stored operator value confirmed by a post-check on the stopped store.
- [ ] 6.9 Record, for every scenario, the exact argv, scratch root, build identity, exit codes and observed bodies in the phase evidence store; confirm each scratch root is removed. A scenario whose prerequisite is unavailable is `NOT_EXERCISED`, not passed.
