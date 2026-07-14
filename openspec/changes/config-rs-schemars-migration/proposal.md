## Why

`AppConfig` (`src/config.rs`) has no machine-readable schema: SDK
codegen, the admin UI, and external tooling cannot introspect the
shape of `config.yaml` / `UAR_*__*` env vars without hand-maintained
duplicate type definitions. Two of the most sensitive fields
(`security.jwt_secret`, and by extension provider API keys threaded
through `LlmConfig`) were plain `String`s with only ad hoc `Debug`
redaction — a `Serialize`/logging call site could leak them.

## What Changes

- `secrecy::SecretString` for `SecurityConfig::jwt_secret` (the
  Cargo dependency was already present but unused for this field);
  4 real call sites updated to `.expose_secret()` explicitly.
- `schemars::JsonSchema` derived across the full `AppConfig` tree
  (30 structs/enums in `src/config.rs` plus 3 external types it
  references: `ClassifierConfig`/`ClassifierBackend`,
  `llm::registry::ProviderConfig`/`ProtocolSetting`/`ModelConfig`,
  `uar::context::ContextStrategy`).
- `AppConfig::json_schema() -> serde_json::Value`, generated live
  from the struct definitions via `schemars::schema_for!`.
- New `GET /.well-known/uar-config` endpoint serving that schema.

## Capabilities

### New Capabilities

- `uar-config-model`: `AppConfig`'s live JSON Schema plus the
  `SecretString`-wrapped `jwt_secret` field.

## Impact

- **New dependency:** `schemars = "1"` (verified against the latest
  crates.io release, not an assumed minimum).
- **API surface:** new `GET /.well-known/uar-config` route,
  additive — no existing route changed.
- **Call sites touched:** `src/server.rs` (API key service init),
  `src/uar/security/middleware.rs` (JWT decode), `src/uar/settings/manager.rs`
  (settings-bootstrap JSON), `tests/settings_persistence.rs` — all
  now call `.expose_secret()` explicitly instead of relying on
  implicit `String` access.
- **No behavior change** for any of those 4 call sites — the
  plaintext value is still available where it legitimately was
  before; the type system now makes every access site explicit and
  greppable.

## Out of scope (scope corrections vs. the original plan)

- **Reducing `src/config.rs` below 800 lines** (currently ~2,090
  after this change's additions). The original plan called for
  splitting the file into submodules. That is a large, separate
  mechanical refactor with its own regression risk (30+ structs,
  a 300-line `impl AppConfig` env-loading block) and is not required
  for the schema/secrecy goals this change actually delivers;
  tracked as follow-up, not blocking.
- **`#[derive(ConfigLayer)]` for declaring `UAR_*__*` env vars.**
  No such derive macro exists in the `config` crate (verified against
  its current docs/source) or elsewhere in this dependency tree —
  the plan's mention of it was aspirational/incorrect. The existing
  builder-pattern env-var wiring in `AppConfig::load_with_cli`
  already works and is unchanged by this proposal.
- **`secrecy::Secret<String>` for every `PROVIDER_*_API_KEY` and
  `LLM__API_KEY`.** Audited the actual call-site surface for
  `llm.api_key`: unlike `jwt_secret`'s 4 well-defined sites, it's
  read across many test fixtures (via struct-update syntax) and at
  least one non-obvious consumer (`seed_from_llm_config`), plus the
  wider provider-credential system in `src/uar/security/credentials/`
  already uses `secrecy::SecretString` at its own boundary. Wrapping
  `llm.api_key` too is real, valuable follow-up work, but has a much
  larger and less-audited blast radius than `jwt_secret` alone —
  deferred rather than rushed.
- **`pnpm generate-config-types` TS codegen script.** Deferred:
  needs a schema-source path that doesn't require booting the full
  server (DB, providers, etc.) just to print a schema — e.g. a
  small `--print-config-schema` CLI flag or example binary. Tracked
  as follow-up alongside the file-split above.
- **Backward-compat verification for legacy `LLM_*` env vars.**
  Unaffected by this change (no env-var-handling code was touched);
  no new verification needed.
