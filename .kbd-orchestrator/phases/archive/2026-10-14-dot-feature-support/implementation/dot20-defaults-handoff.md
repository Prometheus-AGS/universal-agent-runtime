# Dot20 safe host defaults implementation handoff

Phase: `dot-feature-support`. Task: Dot20 task 1, implementation authoring only. Canonical task status is owned by the lead and is not marked complete by this handoff. Dot20 effective settings task 2 remains separate and pending.

The uncomfortable thing: authored defaults and fixture assertions do not establish installed behavior. Existing service environments, persisted choices, native packaging, dependency compilation and real socket/egress/auth behavior remain unverified until the lead's full-phase implementation-complete inventory exists.

## Production changes by file

- `src/config.rs`: native/default host becomes `127.0.0.1`; `ServerConfig.cors_allowed_origins: Vec<String>` is an empty-default serde field; `UAR_SERVER__CORS_ALLOWED_ORIGINS` parses only that key as a comma-separated list. Compiled and serde LLM model defaults are empty. Memory stays disabled and defaults to local `BAAI/bge-small-en-v1.5` when explicitly enabled. Auth defaults remain true.
- `src/llm/registry.rs`: empty global models do not seed a synthetic OpenAI provider. Provider catalog discovery alone does not select a default. The first enabled provider with an explicit nonempty `default_model` may seed a default when no global selection exists. Empty resolution returns `None`; disabled or empty provider defaults do not report a selected model. Existing registration preservation and persisted hydration paths remain intact.
- `src/llm/orchestrator.rs`: `build_driver_inner` rejects an empty model with an explicit selection diagnostic before constructing any provider client.
- `tests/settings_persistence.rs`: adapts its literal `ServerConfig` to the new empty origin field.
- `packaging/native/default-config.yaml`: retains loopback, adds an empty origin list, requires JWT and authenticated settings mutations, removes the published loopback signing secret and documents required service-environment secrets.
- `config.embedded.yaml`: requires JWT, removes the development signing secret and hardcoded OpenAI provider/model, adds empty origins.
- `config.remote.surreal.yaml` and `config.remote.postgres.yaml`: default to loopback and empty origins/model; remove published signing-secret placeholders. Remote database access alone does not imply a public host listener.
- `example.config.yaml`: documents safe loopback/origin/model defaults and environment-supplied auth secrets.
- `docs/config/home.uar.embedded-openai.example.yaml`: safe loopback/JWT/origin/model defaults; memory disabled with local embedder. Its explicitly documented OpenAI KB example remains an operator opt-in example, separate from memory.
- `docker-compose.prod.yaml` and `docker-compose.prod.postgres.yaml`: keep deliberate shared-container `0.0.0.0` listeners, require JWT by default, disable memory by default with local embedding settings; remove the implicit model in the Surreal profile. Explicit environment overrides remain operator-controlled.
- `k8s/base/configmap.yaml` and `k8s/helm/uar/values.yaml`: retain deliberate shared-pod bind; explicitly require JWT and default memory off.
- `.env.example`: no implicit model/API key selection; explicitly enables JWT and names the required settings-admin secret; documents local bind and exact-origin environment option.
- `packaging/native/common/merge_provider_config.py`: discovered keys/proxy models add inventory without `default_model`; removes Alibaba auto-default selection and saved-model rewrite routines; missing settings-mutation auth defaults to true without altering an explicit saved value.
- `packaging/native/common/Merge-ProviderConfig.ps1`: equivalent inventory-only behavior and removal of automatic Alibaba model/default rewrites. Obsolete rewrite-only helper functions are removed with their callers.
- `docs/configuration.md`: corrects LLM/listener defaults and links the operational safe-defaults guide.
- `docs/config/safe-host-defaults.md`: documents native credential provisioning, preserved saved choices, exact-origin syntax, explicit shared-host identity policy, test-fixture distinction and deferred actual-artifact acceptance scenarios.

## Lead-owned integration already reported authored

The lead owns these source changes and reported authoring them during this packet:

- `src/server.rs`: replaces permissive CORS with fallible `build_cors_layer(&config.server.cors_allowed_origins)`; exact list matching, invalid/empty/wildcard/null entries rejected, no default allow-origin header. Replaces Anthropic first-provider/first-model/`default` literal fallbacks with explicit requested or saved model resolution and missing-selection errors. Explicit known `provider/model` works without a global default.
- `src/uar/runtime/manager.rs`: ignores empty skill preferred strings, resolves an explicit saved registry default when the global default is empty, and bypasses availability routing when no model has been selected so catalog candidates cannot silently replace missing configuration. The driver emits the missing-model error.
- `src/uar/settings/manager.rs`: an empty global model seeds an empty new `llm.default_provider`; nonempty legacy semantics and existing persisted rows remain unchanged. This prevents a synthetic `default` row from blocking initial persistence of an explicit YAML selection.

## Contracts and decisions

1. Configuration key is `server.cors_allowed_origins`; environment form is `UAR_SERVER__CORS_ALLOWED_ORIGINS`, comma-separated without spaces. Empty default is represented by omission or YAML `[]`; an explicitly empty environment value is an invalid empty entry, not a wildcard.
2. Native HTTP and gRPC share the configured host; compiled native default is loopback. Shared Compose/Kubernetes listeners remain deliberately wider and authenticated. Remote identity admission still needs the existing exact issuer/audience/algorithm/workspace policy, documented in the guide; a bind address grants no tenant authority.
3. An API key or discovered model list is inventory, not an explicit model selection. Saved operator model/provider choices remain authoritative. New default models are empty; default memory is off/local. Local model weight download is possible only when that local embedder is explicitly enabled/used.
4. New native installations must provision JWT signing and settings-admin secrets in the existing service environment file before startup. Existing native env generation preserves those entries but does not copy arbitrary launching-shell variables. Missing required secrets intentionally fail startup.
5. `config.test.yaml`, `test-config.yaml`, development/test Compose fixtures are not installable defaults and retain their deliberate fixture behavior. No running service or operator `.env` was modified.
6. Guard provenance: the empty-model guard closes observed implicit model selection; registry guards close discovered-catalog/default routing; CORS validation hardens the actual browser-origin boundary and prevents wildcard-list panic; missing auth secret validation already existed and is activated by safe profiles. No speculative unrelated guards were added.

## Authored fixtures, not executed

- `src/config.rs`: `safe_component_defaults_require_explicit_model_and_origins` asserts no default model, disabled/local memory and empty omitted CORS list.
- `src/llm/registry.rs`: `credentials_and_model_discovery_do_not_select_a_default` exercises global credential-only seed, catalog-enriched provider seed and empty resolution.
- `src/llm/registry.rs`: `explicit_provider_model_survives_an_empty_global_default` exercises preserved explicit provider model selection across an empty global seed.
- Lead-owned `src/server.rs` CORS fixture now asserts configured-origin behavior; lead owns further empty/unmatched/preflight fixtures.
- `docs/config/safe-host-defaults.md` supplies the actual installed local/shared acceptance inventory covering sockets, egress, auth, model selection, upgrades and CORS.

## Verification and unresolved work

No tests, Cargo commands, compiler probes, formatters, linters, builds, OpenSpec validation, installer executions, policy validators or independent review gates were run. Commands used only read source/docs or wrote the assigned artifacts. No fixture result is claimed. Full Dot implementation is still incomplete, so no verification tier was entered.

The lead explicitly resolved the earlier script pre-edit policy-validator timing conflict against the latest direct operator instruction forbidding all verification before full-phase implementation. Therefore the workflow-invoked native merger edits were authored without running `pnpm github-actions-policy:validate`. That exact command remains a mandatory deferred final gate; it is neither skipped permanently nor reported passed. Workflow definitions and test-disabled build behavior were not changed.

Remaining acceptance: compile/type/format and fixture behavior; Python/PowerShell actual native install/refresh preservation; provisioned-secret native startup; actual local/shared listener, auth, CORS, model and embedder egress; installed artifact identity. No production task1 design blocker remains in this packet, contingent on the lead-owned integration changes above. Task2 effective-settings/schema/auth/revision work is not covered.

## Documentation provenance

Applied `prometheus-rust-workspace`, `rust-router`, repository Rust best practices and relevant borrowing/error-handling references. Repository pins remain unchanged.

Context7 resolved `/tower-rs/tower-http` and documented `AllowOrigin::list` / `From<Vec<HeaderValue>>`, empty/no-match behavior and wildcard panic: <https://github.com/tower-rs/tower-http/blob/main/_autodocs/api-reference/cors.md>.

Context7 resolved `/rust-cli/config-rs` and documented `list_separator`, `try_parsing`, and `with_list_parse_key` using the collected key after separator translation: <https://github.com/rust-cli/config-rs/blob/main/_autodocs/api-reference/environment.md>.

No dependency was introduced and no upstream behavior claim is used beyond those documentation contracts. Installed-version compilation remains deferred.
