# uar-state handoff, batch 1, 2026-10-30

Phase: implementation under a CLOSED verification gate. Nothing below was compiled, linted or tested. Every claim is source-reasoned only.

## Files changed

New migrations (sqlx 0.9 `migrate!`; no existing file edited):
- `migrations/20261030010000_settings_source.sql` (#389): `settings.source TEXT NOT NULL DEFAULT 'config' CHECK (source IN ('config','api'))`; backfills rows with non-null `updated_at` to `'api'` (trigger paused during backfill so revision tokens are not restamped); `_uar_settings_set_updated_at()` now stamps `updated_at` only when `NEW.source = 'api'`.
- `migrations/20261030012000_sessions_ttl_scan.sql` (#390): `_uar_try_timestamptz(TEXT)` returns NULL instead of raising on unreadable text.
- `migrations/20261030020000_skill_embedding_dimensions.sql` (#395): drops `skills_embedding_idx`, `skills.embedding` becomes unconstrained `vector`, adds `skills.embedding_fingerprint JSONB`.

Rust:
- `src/uar/persistence/providers/postgres.rs`: all four Postgres changes (settings source, session TTL scan, memory refusal, skill fingerprinting) plus 4 pure unit tests (authored, not run).
- `src/uar/persistence/mod.rs`: `pub mod capabilities;`, `capability_matrix()` default method, five default-implemented skill methods (`save_skill_in_space`, `create_skill_if_absent_in_space`, `search_skills_in_space`, `skill_ids_outside_space`, `set_skill_embedding`).
- `src/uar/persistence/capabilities.rs` (new): `CAPABILITY_UNSUPPORTED`, capability-name consts, `refusal_body(capability)`.
- `src/uar/persistence/scoped.rs`: forwards the five new skill methods; adds `search_skills_in_space` and `skill_ids_outside_space` to the read-only set.
- `src/uar/runtime/skills/registry.rs`, `src/uar/runtime/skills/service.rs`: embed with the active fingerprint, save/search by space, `reembed_stale()` called at the end of `SkillService::register_builtins`.
- `src/uar/runtime/followups/{mod,controller}.rs`, `src/uar/runtime/schedules/{mod,controller}.rs`: new `CapabilityUnsupported(&'static str)` variant naming the missing capability; `code()` is `capability_unsupported`; `capability()` accessor.
- `src/uar/runtime/instance/error.rs`, `src/uar/runtime/observer/{error,controller,channel}.rs`: same code and accessor (`ObserverError::Unavailable` became `CapabilityUnsupported(&'static str)`; `ChannelObserverError` gained `CapabilityUnsupported`).
- `src/uar/runtime/team_execution/controller.rs`: not-available refusal now carries `capability_unsupported:supports_durable_agent_instances` (shutdown/cancel refusal unchanged).
- `src/uar/api/{followups,schedules,agent_instances,observers,channel_observers}.rs`, `src/uar/api/collaboration.rs`, `src/uar/api/collaboration/team_execution.rs`: error mapping emits `{"error":{"code":"capability_unsupported","capability":"<name>"}}` with HTTP 503.

OWNERSHIP NOTE: the `runtime/instance`, `runtime/observer`, `runtime/team_execution` and `api/*` files above are outside my listed ownership. I edited them with small, exact-span edits because they are the existing error-to-response mapping #394 asked me to reuse. If another writer owns them, re-check for conflicts.

## Interface effects

- `PersistenceLayer::capability_matrix(&self) -> serde_json::Value` (trait default method in `src/uar/persistence/mod.rs`; works on `ScopedPersistence` and every provider). Keys are the 13 `supports_*` method names exactly: `supports_durable_client_tools`, `supports_durable_mcp_elicitations`, `supports_durable_tool_receipts`, `supports_durable_agui_runs`, `supports_durable_a2a_bindings`, `supports_durable_run_history`, `supports_durable_accounting`, `supports_durable_ingestion`, `supports_channel_observers`, `supports_durable_observers`, `supports_durable_agent_instances`, `supports_durable_approvals`, `supports_durable_checkpoint_resume`. Values are booleans.
- New trait methods are default-implemented, so Surreal and in-memory providers are unchanged (SurrealDB behaviour is not altered; the store trait signature for `upsert_setting` is unchanged).
- Postgres `save_memory` and `search_memory` now fail with `memory_service_required: ...` exactly like SurrealDB. The only caller is the `ScopedPersistence` pass-through; a refused call is a plain error (not a transport error, so no unresolved-effect record). No other runtime code calls them.
- Refusal body shape: `{"error":{"code":"capability_unsupported","capability":"supports_durable_observers"}}`, HTTP 503. Capability names are the `supports_*` names (`durable_collaboration_catalog` for the in-memory catalog backend).
- Native skill tools (`followup_*`, `schedule_*`) surface `anyhow!(error.code())`, so they now report `capability_unsupported` without the capability name.
- Removed codes: `followup_unsupported`, `schedule_unsupported`, `instance_unavailable`, `observer_unavailable` are no longer emitted. Nothing in `src/`, `scripts/`, `sdks/` consumed them except the items listed under "Lead actions".

## What the lead must do in src/server.rs

Add the route handler (`app.persistence` here stands for the `Arc<dyn PersistenceLayer>`; the behaviour step reads `matrix.persistence.backend` and `matrix.persistence.capabilities[<flag>]`):

```rust
json!({
    "persistence": {
        "backend": config.persistence.provider,          // must read "postgres" for the Postgres scenario
        "capabilities": persistence_layer.capability_matrix(),
    },
    "memory": { /* memory store name, as the feature step "names the memory store" expects */ },
})
```
`ScopedPersistence` forwards every `supports_*`, so calling `capability_matrix()` on the scoped layer is correct. Check that `GET /api/uar/followups/` is reachable without a workspace header: `api/followups.rs::scope` returns 400 `followup_invalid` ("workspace header is required") BEFORE the controller can answer `capability_unsupported`. The behaviour step sends no explicit header in its call; either its `request()` helper adds `x-uar-workspace-id`, or move the support check ahead of scope validation (api/followups.rs, a one-line reorder I did not make because it changes validation order for every store).

## Lead actions outside my ownership

- `src/uar/api/assistant_capabilities.rs` lines ~105 and ~151 still advertise reasons `schedule_unsupported` / `followup_unsupported` (descriptive "reason" strings, not error codes). Consider `capability_unsupported` for consistency.
- `docs/proactive-followups.md:60` lists `followup_unsupported`; update to `capability_unsupported` with the `capability` field.

## Decisions

1. Settings provenance rule = SurrealDB's rule: the manager marks API mutations by supplying `updated_at`; Postgres now stores that (`source` derived from `updated_at.is_some()`), persists the caller's `updated_at` verbatim, and on read surfaces `updated_at` only for `source='api'` rows. The manager needed no change. The trigger still stamps `updated_at = now()` for API rows (preserves the existing revision token behaviour). `compare_and_swap_global_presentation_policy` now also sets `source='api'` (it already stamped `updated_at`).
2. Consequence to accept: a config-source row's revision token is `created_at`, so a config-driven data change does not change the revision (identical to SurrealDB).
3. Session keys: unambiguous decoding is done by reading the logical owner/id from the stored document and checking the row key against the three accepted forms (canonical `session_storage_key`, bare id for the anonymous owner, legacy `owner::id`). No key is split. `load_session` now adopts a legacy `owner::id` row (and the bare anonymous form, as before) under its canonical key, so `delete_session` can delete what the TTL scan returns. I found no code in the repo history that wrote `owner::id`; it is supported because the issue named it.
4. TTL scan filters in SQL with `_uar_try_timestamptz(data->>'last_activity') < $1`. Rows with unreadable timestamps or non-matching keys are counted and logged in one `warn!` per scan. `next_persisted_session_expiry` was switched to the same safe cast (a malformed row used to abort it).
5. Skills: the trait could not carry a fingerprint, so I added default-implemented `*_in_space` methods (pattern copied from `search_knowledge_scoped_in_space`). Plain `save_skill` with a non-empty vector stores a NULL fingerprint, so such a row is excluded from space-fenced search and re-embedded. Search always also fences by `vector_dims`, because the column is now unconstrained. Embedding and fingerprint are read around one `embed_batch` call; a mid-call backend swap discards the vector rather than recording it in the wrong space.
6. Re-embedding runs from `SkillService::register_builtins` (startup path that already embeds). It selects only rows with a non-NULL embedding and a different or missing fingerprint. Rows with no embedding stay unembedded (existing "not embedded yet" semantics, and it keeps startup free of a cold ONNX load).
7. `memories` table untouched.

## Unverified (verification gate closed)

- Nothing compiled. Highest-risk spots for the first `cargo check`: sqlx 0.9 requires `&'static str` SQL (I used `macro_rules! select_settings!` with `concat!` for exactly that reason); `Option<serde_json::Value>`/`Option<Vector>` binds; `row.try_get("skill_id")` inference in `skill_ids_outside_space`; `Some(*capability)` derefs in the three `capability()` accessors; `let (embedding, Some(space)) = ... else` in `reembed_stale`.
- No SQL executed: migrations, `$4::jsonb IS NULL OR ...`, `vector_dims`, `ALTER COLUMN TYPE vector` on a populated table, and the trigger pause/backfill have not run against PostgreSQL 18 or pgvector.
- Whether `ALTER TABLE ... DISABLE TRIGGER` is permitted for the migration role is assumed (owner).
- The 1024-dimension and capability scenarios in `scripts/behaviors/features/postgres-flint-forge.feature` depend on the lead's `/api/uar/capabilities` route and have not run.
- Unit tests added in `postgres.rs` (`mod tests`) are authored only.
