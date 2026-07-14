## 1. Secrecy for jwt_secret
- [x] 1.1 `SecurityConfig::jwt_secret` changed from `String` to `secrecy::SecretString`.
- [x] 1.2 `src/server.rs` — `ApiKeyService::new(..., config.security.jwt_secret.expose_secret())`.
- [x] 1.3 `src/uar/security/middleware.rs` — `resolve_user_context(..., state.config.security.jwt_secret.expose_secret(), ...)`.
- [x] 1.4 `src/uar/settings/manager.rs` — `json!(config.security.jwt_secret.expose_secret())` in the settings bootstrap.
- [x] 1.5 `tests/settings_persistence.rs` — `jwt_secret: "test-secret".to_string().into()`.

## 2. schemars derive across the config tree
- [x] 2.1 Added `schemars = "1"` to `Cargo.toml` (checked crates.io for the current release rather than assuming a version).
- [x] 2.2 `#[derive(schemars::JsonSchema)]` added to all 30 `Deserialize`-deriving structs/enums in `src/config.rs`.
- [x] 2.3 `#[schemars(with = "String")]` on `jwt_secret` (SecretString has no `JsonSchema`/`Serialize` impl by design — the schema represents it as an opaque string).
- [x] 2.4 Cascaded the derive to 3 external types `AppConfig` references: `uar::runtime::matching::intent::{ClassifierConfig, ClassifierBackend}`, `llm::registry::{ProviderConfig, ProtocolSetting, ModelConfig}`, `uar::context::strategy::ContextStrategy`.

## 3. Schema endpoint
- [x] 3.1 `AppConfig::json_schema() -> serde_json::Value` in `src/config.rs`, using `schemars::schema_for!(AppConfig)`.
- [x] 3.2 `GET /.well-known/uar-config` route in `src/server.rs` (`uar_config_schema_handler`), registered alongside `/health`/`/healthz`/`/readyz`.
- [x] 3.3 Unit tests: schema has the expected top-level properties; `jwt_secret` resolves (directly or via `$ref`) to an opaque `{"type": "string"}`, not the internal `SecretBox` shape.

## 4. Verification
- [x] 4.1 `cargo check --no-default-features --features server-full` clean.
- [ ] 4.2 `cargo check --no-default-features --features server-full --tests` — **verify this pass includes the jwt_secret fix** (was run once before the schemars additions and passed; re-run pending as part of the phase's consolidated validation to also cover the schemars-derive changes across all test binaries, not just the lib).
- [x] 4.3 `cargo test --locked --no-default-features --features server-full --lib config::` green (new schema tests pass alongside the pre-existing `config` module tests).
- [ ] 4.4 **Deferred to consolidated validation pass**: full-workspace `cargo fmt --all -- --check` and `cargo clippy`.

## 5. Deferred / out of scope (see proposal.md)
- [ ] 5.1 Split `src/config.rs` into submodules to get under 800 lines. Not attempted this pass — large mechanical refactor, separate risk profile from the schema/secrecy work actually delivered. Follow-up candidate.
- [ ] 5.2 `secrecy::Secret` for `llm.api_key` / `PROVIDER_*_API_KEY`. Deferred — wider, less-audited call-site surface than `jwt_secret` (test fixtures + `seed_from_llm_config` + provider-credential subsystem). Follow-up candidate.
- [ ] 5.3 `pnpm generate-config-types` TS codegen script. Deferred — needs a schema-source path that doesn't require booting the full server; a `--print-config-schema` CLI flag/example binary is the natural next step. Follow-up candidate.
- [ ] 5.4 `#[derive(ConfigLayer)]`. **Does not exist** — the plan's premise was incorrect (see proposal.md). No action possible or needed; the existing builder-pattern env-var wiring is unaffected by this change.
