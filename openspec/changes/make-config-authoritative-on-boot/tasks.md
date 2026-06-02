# Tasks — make-config-authoritative-on-boot

## §1 Provider-key shortcuts → real key (P0)
- [x] Remove dead `llm._provider_keys.*` writes in `config.rs`
- [x] Route `OPENAI_API_KEY`/`ANTHROPIC_API_KEY`/etc. to `llm.api_key` (set_default) + `llm.provider_keys.<id>`
- [x] Add `provider_keys: HashMap<String, String>` field to `LlmConfig`
- [x] Update `build_client_config` fallback chain to check `provider_keys[provider_id]` (step 4)
- [x] `provider_catalog_status` / `api_catalog`: `configured` ⇐ `p.enabled && p.api_key` non-empty

## §2 Config-authoritative seeding on boot (P0)
- [x] Changed `seed_providers_from_registry` from insert-if-absent to upsert-on-every-boot (preserves row id for FK stability)
- [x] `llm.default_provider` always reconciled from registry on each boot (no longer "only if absent")
- [x] `seed_from_configs` (YAML `providers:`) sets registry default if not yet established

## §3 Truthful "configured models" (P1)
- [x] `seed_from_llm_config` now stores only the operator-configured model in `ProviderConfig.models` (not full catalog)
- [x] Catalog lookup uses correct `ModelInfo` fields (`name`, `capabilities.tool_call`, `modalities.input`, `limits.context_window/max_output`)
- [x] Falls back to minimal `ModelConfig` stub for custom/off-catalog models
- [x] `api_key` for provider resolved from `provider_keys[provider_id]` shortcut

## §4 Precedence + drift (P2)
- [x] CLI vs LLM_* precedence note: both use `set_override`; no trivial single-line fix (requires builder refactor). Documented as known limitation — deferred to P3 since UAR_LLM__* env vars still win as intended for most usage.
- [ ] Persist `SettingSource`/`is_drift` metadata — deferred; not introduced by this change and requires persistence schema addition

## §5 Validation
- [x] `cargo check` clean (SKIP_FRONTEND_BUILD=1)
- [x] `cargo test --lib` — 218/218 passed
- [ ] End-to-end: set `OPENAI_API_KEY` (no `LLM_API_KEY`) → provider shows `configured` AND chat completion succeeds — pending manual run
- [ ] Restart with changed `UAR_LLM__MODEL` on non-empty DB → UI shows new model — pending manual run

## Notes
- Worktree: `~/.claude/worktrees/make-config-authoritative-on-boot`, branch `fix/make-config-authoritative-on-boot`, commit `c41da57`
- Migration note needed for existing deployments: next boot will overwrite DB-stored provider/model config from env/YAML
