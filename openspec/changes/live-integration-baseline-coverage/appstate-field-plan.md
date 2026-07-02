# AppState field construction plan (task 1.1)

Checked against the actual source (not assumed) on 2026-07-01/02. Every
constructor below exists in the codebase today; none required new
production code to be added.

| Field | Construction | Notes |
|---|---|---|
| `mcp` | `Arc::new(McpRegistry::empty())` | `src/mcp/registry.rs:52` |
| `orchestrator` | `Orchestrator::new(llm_config, mcp, native_skill_registry)` | `src/llm/orchestrator.rs:222` |
| `sessions` | `SessionStore::new()` | in-memory `HashMap`, `src/session/thread.rs` |
| `run_manager` | `RunManager::new(llm_config, mcp, sessions, skill_registry, vector_matcher, None)` | persistence arg is `None` |
| `ingest_service` | `None` | not needed by any baseline case |
| `vector_matcher` | `Arc::new(VectorMatcher::new(0.75, "src/uar/runtime/matching/models".into()))` | `initialize()` loads the **committed** `tokenizer.json` at that path — works with no external service |
| `persistence` | `None` | no baseline case needs the `PersistenceLayer` trait directly |
| `rate_limiter` | `Arc::new(AppRateLimiter::new(1000.0, 2000))` | generous test limits, no throttling in-test |
| `config` | `AppConfig::load_with_cli(Cli { config: Some(<temp yaml path>), ..all None })` | **must** pass an explicit temp file — see Risk below |
| `skill_service` | `Arc::new(SkillService::new(None, None))` | `src/uar/runtime/skills/service.rs:120` |
| `provider_registry` | `ProviderRegistry::new()` then `.seed_from_llm_config(&llm_config).await` | seeds from the resolved backend's `base_url`/model so `ModelRouter`/`is_configured` see it |
| `native_skill_registry` | `Arc::new(NativeSkillRegistry::new())` | |
| `federated_agent_registry` | `Arc::new(InMemoryAgentRegistry::new())` | `src/uar/api/a2a/registry.rs:141` |
| `actor_system` | `ActorCollaboration::new(llm_config, mcp, native_skill_registry)` | `src/uar/runtime/actor/system.rs:75` |
| `governance_engine` | `Arc::new(GovernanceEngine::with_default_permit())` | plain `::new()` is default-deny (empty Cedar policy set) — would block every tool call |
| `api_key_service` | `None` | not needed |
| `provider_service` | `None`, or `Some(ProviderService::new(InMemoryCredentialStore::new(), CredentialEncryption::from_key(TEST_KEY)))` for the credential-chain case only (pattern already used by `tests/credentials_api_integration_test.rs`) |
| `memory_service` | `Some(MemoryService::new(MemoryConfig { enabled: true, db_path: <temp dir>, embedding_provider: "local".into(), ..Default::default() }).await?)` for memory/RAG cases only | embedded SurrealKV via `db_path`, **not** a running DB service; `embedding_provider: "local"` avoids requiring a real OpenAI key (default is `"openai"`) |
| `live_bus` | `None` | not needed |
| `compiler_service` | `None` | not needed |
| `settings_manager` | `None` | not needed |
| `prompt_cache_provider` | `Arc::new(SurrealMemPromptCacheProvider::new().await?)` | despite the name, this is a plain in-memory `HashMap` cache — no DB (`src/uar/prompt_cache.rs:40-48`) |
| `user_settings_store` | `Arc::new(UserSettingsStore::new())` | already used this way in `src/server.rs` |
| `a2ui_registry` | `A2uiRegistry::with_builtins()` | already returns `Arc<Self>` |
| `model_router` | `Arc::new(ModelRouter::new(provider_registry.clone()))` | |
| `agent_sessions` | `Arc::new(RwLock::new(HashMap::new()))` | |

## Risk found and resolved: `AppConfig` file-loading is NOT hermetic by default

`AppConfig::load_with_cli` (`src/config.rs:1035-1048`), when `cli.config` is
`None`, falls back to probing `./config.yaml` (repo-root-relative to the
process's cwd) and then `~/.uar/config.yaml`. **Both files exist on the
development machine this harness was built on** (144 bytes and 2861 bytes
respectively) — a real, non-hypothetical risk. Passing `cli.config: None`
would make the harness silently load whatever the operator's real local
config contains (potentially different persistence backend, JWT secrets,
ports), producing a harness that looks like it's testing defaults but
actually isn't. **Resolution:** the harness always writes a small, explicit
temp YAML file and passes its path as `cli.config: Some(path)`, exactly like
the existing pattern in `src/config.rs`'s own `#[cfg(test)]` module
(`write_test_config_file()`), so the fallback probing path is never reached.

## Answers to design.md's open risks

- **Memory/RAG embeddability (Risk 1):** resolved — `MemoryConfig.db_path`
  drives an embedded SurrealKV file, not a network service; setting
  `embedding_provider: "local"` avoids needing a real OpenAI key. No
  `#[ignore]` needed for these two cases.
- **`stream_mode: dual` / SSE support (Risk 2):** still open — task 1.3.

## Superseding decision: use `start_server` directly, not hand-rolled `AppState`

The field-by-field plan above was research, not the final implementation.
While implementing it, `src/server.rs:75`'s public `pub async fn
start_server(config: Arc<AppConfig>)` turned out to already do exactly this
wiring — including seeding `provider_registry` from `config.llm` — so the
harness (`tests/integration/live/harness.rs`) calls it directly instead of
reconstructing `AppState`. This is simpler (no `pub(crate)` visibility wall:
`api_chat_completion` and friends are `pub(crate)`, unreachable from the
`tests/` binary, but that's fine — HTTP-level testing never calls them
directly) and stronger proof, since it's the exact same boot path production
uses.

Two things discovered only by running it, not by reading code:

1. **`start_server`'s future is `!Send`** (a `tracing::info!` call holds a
   non-`Send` value across an `.await` somewhere in its body). Fixed on the
   test side by running it on a dedicated OS thread with its own
   current-thread Tokio runtime — no production code touched.
2. **Fixture fingerprinting must match the wire request, not the UAR-internal
   model string:** the real orchestrator strips the `provider/` prefix before
   the outbound call (`gpt-5.4-mini`, not `openai/gpt-5.4-mini`), and always
   attaches a tool schema — native skills/MCP tools are registered
   unconditionally per CLAUDE.md's "tools are non-optional" design, so
   `has_tools` is effectively always `true` for real requests through this
   harness. Every baseline-case fixture must be registered accordingly.

Both harness smoke tests (`boots_and_answers_health_check`,
`chat_completion_flows_through_the_real_server_to_the_stub`) pass, proving
the real server boots and a real chat-completion request flows through the
real orchestrator/provider_registry/model_router to the stub LLM server and
back.
