# ANALYZE — the trace (read-only, file/line citations)

## 1. file/env/CLI bootstrap

- `src/main.rs:101-106` — config load; "watching" note only.
- `src/config.rs:1706` — cwd `config.yaml` discovery; `config.rs:1942,2043` —
  documented precedence: compiled defaults → `config.yaml` `llm:` → `UAR_LLM__*`
  env. CLI/env act at config-load time only (boot authority).
- Env-only credential injection seeds the provider registry;
  `src/llm/local_only.rs:149` documents the env-seed stops being honored once
  a settings row exists (rows win by design).

## 2. First-boot vs existing persisted DB

- `src/server.rs:883-925` — one `Arc<SettingsManager>` (shared, not cloned);
  `initialize_with_governance_default(&config, …)` runs ONCE at boot
  (`src/uar/settings/manager.rs:203-330`), reconciling rows:
  `manager.rs:276` keeps `SettingSource::Api`/persisted-API values over config
  on existing DBs; new keys auto-seed with config values (observed:
  `llm.embedding.*` seeded 2026-10-19T12:02:03Z on the pre-existing 1907 DB).
- Provider rows: `seed_providers_from_registry(registry, preserve_api_key =
  provider_service.is_none())` (`server.rs:923`; `manager.rs:993+`) — row keeps
  the credential only on store-less deployments; encrypted-store deployments
  strip rows (store owns the key).

## 3. Service construction / ownership ordering

- Settings cluster executes BEFORE consumer construction (moved to
  `server.rs:843-925`): provider seed → credential hydration → settings init →
  embedding overlay (`server.rs:1025-1055`) → ingest strategy from rows →
  `effective_memory` from rows (observed: `MemoryService initialized` with
  config `memory.enabled: false`) → MCP hydration deferred past
  `Arc::new(mcp_registry)` (`hydrate_mcp_from_settings` flag) →
  `SettingsApiState` (`server.rs:2082-2091`: Arc-shared manager, shared
  embedding slot, base-config snapshot).

## 4. REST update propagation

- Single update: `settings.rs:update_setting` → `set_value_if_revision`
  (`manager.rs:671-693`) → `set_value_locked` updates DB AND cache
  (`manager.rs:565-625`) → `activate_embedding_if_touched`
  (`settings.rs:856-895`) swaps the ONE shared slot. Swap writers in the whole
  tree: exactly `settings.rs:915` (API) and `server.rs:1041` (boot overlay) —
  ordered, single slot.
- Providers: `update_provider` carry-over merge (`providers.rs:276`) +
  `persist_provider_config(state, cfg, new_credential)` distinguishing
  operator-sent vs carried keys; `upsert_provider_config` preserves row keys
  on store-less deployments (`manager.rs:993+`).
- Bulk (`settings.rs:1135`) and reset (`settings.rs:980-1010`) run the same
  prevalidation + activation contract; reset preserves If-Match semantics.

## 5. Truthful activation / rollback

- Responses carry an `activation` map: `live` / `restart_required` / explicit
  422 `unsupported_setting`.
- `ensure_backend_honors_candidate` (`settings.rs:852-885`) rejects values the
  BUILT backend canonicalizes (backend/model mismatch vs candidate) BEFORE
  persist — rejection ⇒ nothing persisted (observed: row byte-identical after
  422).
- Boot overlay applies only honored rows; unhonored out-of-band rows → warn +
  keep config base (`server.rs:1031-1053`; observed 13:23:03).

## 6. Restart hydration

Rows → boot reconcile → overlay/hydration. Observed: provider credential
survives mutation+restart (P1 test 200, 4523ms real round trip); memory
service from rows (P4); embedding overlay honored-check (P2); `rag.chunk_size`
row retained across two restarts (P3).

## 7. Embedding compatibility

- Single `ArcSwapAny` slot (`swappable.rs` BackendSlot: backend+metadata in
  one Arc — no torn reads).
- Pin-per-request (`vector.rs:50`) and pin-per-job (`ingest.rs`) — a mid-job
  swap cannot interleave.
- Ingest binds the KB space (`ingest.rs:50-66`); KV readback: probe KB bound
  `{fastembed, bge-small-en-v1.5, 384}`.
- Query guard `ensure_embedding_space` (`domain/knowledge.rs:258`;
  `api/knowledge.rs:656-658`) — PRESENT, structurally unreachable here:
  fastembed honors only its packaged model (`fastembed.rs:124-127`);
  openai/voyage/cohere need credentials (`openai.rs:23-24`) → no second space
  can be produced through any permitted path. UNVERIFIED (environment gap).

## Adjudicated candidates (each: conflict or not)

a. Two swap writers — ordered boot→API, single slot: NOT a conflict.
b. `VectorMatcher::from_config` (`vector.rs:59-66`) private slot — not used by
   `server.rs` (`server.rs:596` passes the shared Arc to `::new`): NOT live.
c. Second `SettingsManager` (`runtime/manager.rs:1146`) — reads
   `run_policy.global` via `get_typed_for_admission` (`policy.rs:691-693` →
   `manager.rs:502-514`) = persistence-DIRECT by design (cache bypassed):
   NOT divergent.
d. `providers.rs:760,795` managers — `#[tokio::test]` code only.
e. Config watcher/reload (`main.rs:101`; `server.rs:1912,3057-3078`) —
   `initialize()` has exactly one call site (boot, `server.rs:904`); reload
   never re-seeds settings: rows remain sole post-boot authority.
f. Companion listener (`server.rs:525-535`) — second ingress on the same app,
   no second state.
g. `embedding_base_config` snapshot (`server.rs:2090`) — bounded: rows always
   override; fallback-only.
h. Cache vs `effective` map — documented design (`manager.rs:115-118`;
   effective frozen at boot for Restart-mode keys).

## Defect classes

- Product defects (FIXED, before/after behavioral evidence): D1 seed/upsert
  credential strip; carry-over 503 on no-key keyless PUT; vacuous live
  activation for unhonored model values; upload-path binding vacuity.
- Product LIMITATION, now honestly contracted: FastEmbed fixed packaged model
  (`fastembed.rs:124-127`) — never a bug; the defect was REST accepting and
  "live"-reporting values it cannot honor. Now 422.
- Fixture/REPORTING error: `KbConfigResponse` DTO omits `indexed_embedding`
  (`api/knowledge.rs:89-97`) — observability gap that produced the phantom
  "null" reads. Row was bound all along (KV readback). Shortest action: add
  the field in a future build (not taken — no further builds this child).
- Environment gap, not a defect: query-time guard UNVERIFIED.

## Conclusion

The common cause — bootstrap ordering + unowned authority transitions
(seed/mutation/hydration) + dishonest activation reporting — is CLOSED by the
v1–v4 boundary repairs with behavioral evidence. No remaining competing
authority or captured stale dependency was found. **No further architecture
change is justified.** Zero corrections taken in this child (≤1 permitted).
