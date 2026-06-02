# make-config-authoritative-on-boot

## Why
The providers-and-models UI does not reflect what the operator actually configured (assessment D2). There are two unreconciled config-mirroring paths, and a "DB-wins-after-first-boot" rule (`manager.rs:416-419, 451`) means that after the very first boot, editing `UAR_LLM__*` or `config.yaml` has **no effect** on the registry the UI reads. Additional defects compound this:
- `OPENAI_API_KEY`-style shortcuts write to a **dead config key** `llm._provider_keys.*` (`config.rs:1060`) that `LlmConfig` does not define (`config.rs:1161-1215`) — so a provider can render as `configured` while having no usable key (`provider_catalog_status`, `server.rs:1504+` keys off registry `enabled`, not key presence).
- "Configured models" lists the **entire catalog** for a provider, not the model the user set (`registry.rs:472-497`).
- YAML `providers:` array never sets a registry default (`registry.rs:173-184`).
- Drift metadata is in-memory only (`schema.rs:58-72`) → unreliable across restarts.
- CLI `--llm-model` and legacy `LLM_MODEL` share the `set_override` tier; last-writer wins, contradicting documented precedence (`config.rs:1110-1113`).

**Product decision (R3, confirmed):** env/YAML is **authoritative on every boot**. The registry/settings are re-seeded from config at each startup, overwriting prior DB-stored provider/model config. Accepted tradeoff: admin-UI edits to provider/model config are overwritten on restart unless written back to YAML (write-back is a P3 follow-up, out of scope here).

## What changes
- Flip seeding from "insert-if-absent" to **"reconcile-from-config on every boot"**: `seed_providers_from_registry` / `seed_from_configs` (`manager.rs:398-462`) upsert `provider.{id}` rows and `llm.default_provider` from env/YAML at startup, overwriting stale DB values. Config rows win over `Api`-sourced rows for provider/model keys.
- Wire `OPENAI_API_KEY`/`ANTHROPIC_API_KEY`/etc. into `llm.api_key` (and per-provider config) instead of the dead `llm._provider_keys.*` key; update `build_client_config` (`config.rs:1129-1139`) fallback chain.
- Compute provider `configured` from **key presence**, not merely registry `enabled` (`server.rs:1504+`).
- Make YAML `providers:` set/honor a registry default; ensure `resolve_default_model` (`manager.rs:1333`) cannot resolve to an unconfigured provider.
- Make "configured models" reflect the operator's selection (the `llm.model` / per-provider `default_model`), not the full catalog. Catalog stays available as the separate `/api/models` browse list.
- Fix CLI-vs-`LLM_*` precedence so CLI wins as documented.
- (P2) Persist source/drift metadata so the UI's source/drift indicators survive restart.

## Impact
- Affected: `src/config.rs`, `src/uar/settings/manager.rs`, `src/uar/settings/provider_sync.rs`, `src/llm/registry.rs`, `src/server.rs` (startup seeding 372-528 + `provider_catalog_status`).
- Behavior change: provider/model config in the admin UI is now derived from env/YAML on every boot; UI-only edits do not persist across restart (documented; P3 write-back deferred).
- Risk: medium — changes startup data flow and an established DB-wins invariant; needs a clear migration note for existing deployments whose DB has drifted from config.
