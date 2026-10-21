# UAR configuration lifecycle map — 2026-10-19 (local qualification pass)

Source trace only; every claim cites file:line in the dirty checkout
(#419 + checkpoint-host repair). Binary under test: server-full
`a3a2ef5f…` (superseded by the lifecycle-fix build below).

## 1. Startup precedence (config load)

`src/config.rs` `load_with_cli` (~1616–1900), effective order (low → high):

1. Compiled `set_default`s (persistence, file_processing, memory, models…).
2. Config file: explicit `--config` > `./config.yaml` > `~/.uar/config.yaml`.
3. Legacy `LLM_*` env (added as a source after the file so it outranks it, #419).
4. `UAR_*` env (`Environment` source added after legacy → outranks legacy).
5. CLI `set_override`s (highest; re-applied on reload by ConfigManager).
6. Provider-key shortcuts (`UAR_OPENAI_API_KEY` etc.) as `set_default` (lowest).

`ConfigManager` (`src/config_manager.rs:19-41`): `ArcSwap<AppConfig>` current,
notify watcher on the resolved file, `strict` mode pins boot config. Reload
re-applies CLI overrides. **`AppState.config` is a boot snapshot**
(`src/server.rs:1562`); `AppState.config_manager` is the live handle. Route
consumers of `state.config` (discovery.rs:841, memory_admin.rs:440,
upload.rs:88) see boot-time values even after a file reload — file-reload
propagation to routes is NOT a supported mutation path (documented gap;
REST settings are the supported runtime path per ADR-010).

## 2. Bootstrap order (server.rs)

| Step | Line | What |
|---|---|---|
| governance preflight | 502 | |
| listeners | 517–540 | bound before optional subsystems |
| **embedding backend** | **571–588** | built from boot `config.llm.embedding`; on failure → `UnavailableEmbeddingBackend` (server still starts; vector ops error at the boundary) |
| persistence | ~600+ | memory / embedded surrealkv / remote surreal |
| IngestService | 835–840 | shares the same `Arc<dyn EmbeddingBackend>` |
| **MemoryService** | **899–911** | built from boot `config.memory`; before settings init |
| skill service | 1070 | |
| provider registry seed | 1112–1118 | `set_llm_defaults` + `seed_from_llm_config` (env/file LLM) then `seed_from_configs(config.providers)` |
| credential store / ProviderService | ~1121–1145 | `ProviderService::from_env` — `None` without `CREDENTIAL_ENCRYPTION_KEY` |
| **SettingsManager** | **1148+** | AFTER embedding/ingest/memory. `seed_providers_from_registry` (first-boot only; existing rows win), default-provider seed only if unset, `protect_provider_credentials` (only when ProviderService Some), `hydrate_provider_registry_from_settings` (DB rows hydrate registry — DB authoritative, ADR-010) |
| serving | 2678 | |

Consequence: embedding and memory services can never see persisted settings —
not at boot (built first) and not live (fixed `Arc`s). Settings rows that name
embedding/memory (`rag.embedding_provider`, `memory.*`) persist but have no
runtime consumer.

## 3. Live consumers per request

- Inference routing: `resolve_requested_model` (server.rs:5255) reads the LIVE
  registry (`default_id`, `get`, `verified_model`) per request. Provider/model
  REST mutations are live without restart (proven: R12-class, 19 Oct).
- Run policy: `manager.rs:1571-1577` reads `harness.*` settings per run via
  `SettingsManager::get_typed` (cache-backed). Settings → next-run effect works
  for keys with a live reader.
- KB search embedding: `knowledge.rs:614` → `vector_matcher.embedding_backend()`
  — boot-fixed Arc (before this fix).
- Embedding factory: `build_backend` (embeddings/mod.rs:111) knows
  fastembed/candle/openai/voyage/cohere; unknown → `BackendUnavailable`
  (explicit error). KB space guard: `KbConfig::ensure_embedding_space` refuses
  mixing embedding spaces per KB; per-KB `embedding_provider` is descriptive
  only and warns on non-default (knowledge.rs:754-768).

## 4. Single effective owner of mutable settings

`SettingsManager` (persistence-backed, cache + realtime bus) is the owner;
provider registry is an in-memory projection hydrated from it at boot and
updated DB-first by the provider routes. Mutation flows:

- `PUT /api/providers/{id}`: `prepare_provider_update` → `persist_provider_config`
  (credential store + settings row) → registry update (DB-first; on persist
  failure registry is untouched — no divergence). Create: register → persist →
  rollback register on failure.
- `PUT /api/uar/settings/{key}` and `PUT /api/uar/settings/{ns}`: revision-
  checked set through SettingsManager; live effect only for keys with a live
  reader.

## 5. Demonstrated wiring defects (PLANNED fixes — unverified until the proof section below is filled)

- **D1 — scheduled credential loss (no-encryption-key deployment).**
  `upsert_provider_config` (manager.rs:1000) stripped `api_key` from the row
  unconditionally. Boot hydration reads the row key into the registry. So on a
  deployment without `CREDENTIAL_ENCRYPTION_KEY`, ANY provider PUT (even one
  not touching credentials, 200 OK) silently scheduled credential loss on the
  next restart. Fix: preserve the existing row key when the protected store is
  unavailable and the request carries no key.
- **D2 — swallowed persist error.** Provider routes mapped any persist failure
  to a generic 500 with no log. Key-bearing PUT without the store now returns
  explicit `503 credential_storage_unavailable`; other failures are logged with
  their real reason.
- **D3 — embedding backend unreachable by settings.** Built before
  SettingsManager, fixed Arc, no settings keys for it. Fix: `llm.embedding.backend`
  / `llm.embedding.model` settings (seeded first boot from config), boot overlay
  after settings hydration, live validated swap on REST mutation (invalid backend
  → 422, nothing persisted, old backend keeps serving).

## 6. Additional settings with no consumer (PLANNED: boot-wire or explicitly reject — unverified until proven)

- MemoryService is boot-fixed from `config.memory` and built BEFORE
  SettingsManager init; `memory.*` settings persist with no consumer at boot or
  runtime. Planned: build from effective (config overlaid with persisted rows)
  after settings init; live mutation reported as restart-required.
- `rag.chunking_*` / `rag.semantic_threshold`: no consumer; the file-watcher
  IngestService strategy is hardcoded `Semantic { threshold: 0.5 }`
  (server.rs:837). Planned: construct the ingest strategy from effective rag
  settings after settings init; mutation reported as restart-required.
- `rag.embedding_provider` / `rag.embedding_model`: duplicate semantic of the
  operative `llm.embedding.*` keys. Planned: mutation rejected explicitly
  (422) with a pointer to `llm.embedding.backend` / `llm.embedding.model`.
- File-watcher reloads (`ConfigManager`) do not propagate into `AppState.config`
  consumers; REST settings are the supported runtime mutation path (recorded
  gap, not addressed in this pass).

## 7. As-built repair design (implemented 19 Oct, awaiting build+proof)

**Bootstrap reorder (server.rs).** Provider-registry seed, credential
policy/service, and SettingsManager init now run BEFORE IngestService and
MemoryService construction. MCP-from-settings hydration is deferred past
`Arc::new(mcp_registry)` (flag `hydrate_mcp_from_settings`). All fallible init
still completes before router/serve; no locks held across `.await` (settings
reads are sequential awaits, the swappable handle is a single `ArcSwapAny`).

**D1 — credential loss.** `seed_providers_from_registry(registry,
preserve_api_key)` and `upsert_provider_config(config,
preserve_existing_api_key)`: when the protected credential store is
unavailable (`CREDENTIAL_ENCRYPTION_KEY` unset), the settings row is the only
durable credential location, so a keyless mutation preserves the row key
instead of stripping it. With the store available, behavior is unchanged
(key stripped from row, held encrypted). Callers pass
`provider_service.is_none()`.

**D2 — explicit rejection.** Provider create/update map persist failures
through `provider_persist_error`: the real reason is logged
(`tracing::error!`), a missing store yields `503
credential_storage_unavailable`, other failures `500 provider_persist_failed`
with the reason (no secrets).

**D3 — live, coherent embedding swap.** New
`embeddings/swappable.rs`: backend+metadata live in ONE `BackendSlot` (owned
`String`s — no leak, no torn reads) published with a single atomic store.
Consumers PIN a slot per job (`pin()`): ingest (`ingest_file`,
`ingest_text_with_metadata`, `prepare_ingestion`) pins once and threads the
pin through chunking, embeds, and the KB space binding; KB/chat search pins
per sub-query; `VectorMatcher` keeps its derived skill index under ONE
`RwLock<SkillIndex { space, vectors }>` and rebuilds it (in-memory derived
data) when the pinned space differs — a query is never compared against
another space's vectors. Durable KB chunks are protected by the existing
`ensure_embedding_space` guard (explicit refusal, no mixing).
`VectorMatcher::index_skills` pins once so the recorded space always matches
the vectors stored.

**Settings surface.** `llm.embedding.backend` / `llm.embedding.model` added
to the llm namespace (schema + first-boot seed from config; existing DBs get
the rows on next boot reconcile). REST mutation: pre-persist validation
(builds the candidate backend; failure → 422 `unsupported_setting`, nothing
persisted, old backend keeps serving); on persist success the shared handle
swaps and the response reports `activation: live`. Boot overlay applies
persisted rows after settings hydration. `rag.embedding_provider/model` →
422 (descriptive only, pointer to llm.embedding.*). `memory.*` and
`rag.chunking_*` mutations persist + boot-wire (MemoryService built from
effective rows; IngestService strategy from effective rag rows) and report
`activation: restart_required`.

## 8. Proof plan (isolated instances, next step)

- P1 (1906, no-key deployment): keyless provider PUT → row keeps api_key →
  restart → provider test 200. Key-bearing PUT → 503 explicit.
- P2 (1907): settings GET revisions → bogus backend PUT → 422 + row unchanged
  + search still works → model swap → `activation: live` → next KB search on
  old-space KB fails explicitly (guard, no mixing) → swap back → search works
  → restart → persisted rows applied (precedence: rows > config).
- P3: rag.embedding_provider → 422; rag.chunk_size → 200 restart_required.
- P4 (1907): memory.enabled=true + isolated db_path → restart → MemoryService
  initialized from rows → disable → restart → off.

## 9. Proof results

**Binary hashes:** `f9729318…` (lifecycle v1), `852b1ffc…` (v2, + provider
carry-over fix), v3 (+ bind-on-upload) building. 1906 remains on the prior
debug binary (untouched qualification infra); 1907/1908 run the lifecycle
binaries.

**D1 extension found during P1 (v1 binary).** The update path carries the
existing credential into the prepared config (`update_provider` line ~276),
so `persist_provider_config` could not distinguish an operator-sent key from
a carried-over one: a KEYLESS PUT on a no-key deployment 503'd
(`credential_storage_unavailable`), making every provider mutation
impossible on that deployment class. Fixed by threading `new_credential`
from the request into `persist_provider_config` and preserving carried keys
in `upsert_provider_config`. Demonstrated on 1908: keyless PUT → 503 (v1,
defect) → retest on v3.

**P1 (1908, no-key class) — COMPLETE, all legs PASS:**
- First-boot seed preserved the env credential in `provider.kimi-for-coding`
  (surreal row readback: api_key present) — seed side of Fix A ✓
- `POST /api/providers/kimi-for-coding/test` → 200, `ok:true` (hydrated
  credential, real authenticated round trip) ✓
- Keyless PUT → 503 on v1 (defect: update path carries the existing key into
  persist, which then demands the protected store) → **200 on v3** with the
  `new_credential` threading fix; row retains the 72-char credential ✓
- Restart → provider test → 200, `ok:true`, 4523ms (credential durable across
  mutation + restart — the original 1906 bootstrap failure class, closed) ✓
- Key-bearing PUT → **503** `credential_storage_unavailable` with explicit
  remedy; row untouched (still the original credential, not the test key) ✓

**D4 finding during P2 (v3): vacuous live-activation for model values.**
`FastEmbedBackend` serves the packaged bge-small ONNX assets and IGNORES
`config.model` (`model_id()` is hardcoded). `build_backend` therefore
"succeeds" for any model string, so a PUT of `llm.embedding.model=bogus-*`
reported `activation: live` and hot-swapped — to another bge-small backend.
The swap+pin wiring itself is proven functional (post-swap requests
re-initialized through the new slot at 12:52:17), but the mutation accepted a
value no consumer honors — precisely what this gate forbids. Fixed (v4):
`ensure_backend_honors_candidate` compares the BUILT backend's
`backend_name()`/`model_id()` against the candidate in single update, bulk
update, and reset-target prevalidation, rejecting unhonored values with 422
before anything persists; the boot overlay applies the same check and keeps
the config-built backend (with a warning) for out-of-band rows. Honored
surface in this build: `fastembed` + `bge-small-en-v1.5` only — openai /
voyage / cohere cannot build without embedding credentials and 422 with the
build error. P2's live-swap legs are re-scoped accordingly: unhonored model →
422 (explicit), honored model/reset → 200 `live`, space-mixing prevented at
the mutation boundary in this build with the query-time guard as second layer.

**P2 remaining (needs v4):** unhonored-model 422; honored-model PUT + reset
activation `live`; boot-overlay keep-base warning on the existing bogus row;
restart precedence. **P4** boot activation already proven on v3 (see below);
re-prove on v4 restart.

**P2 partial (1907):**
- New `llm.embedding.backend/model` rows auto-seeded on the EXISTING db at
  first v1 boot (12:02:03 revisions) with config values ✓
- Baseline retrieval on new binary: probe KB "Quartzback" 0.640, r14 KB
  "Quartzback turtle basalt" 0.710 (min_score 0; route default 0.7 explains
  earlier empty reads — not a defect) ✓
- PUT `llm.embedding.backend`="bogus-backend" → **422** `unsupported_setting`
  ("embedding backend cannot be built … nothing was persisted and the active
  backend is unchanged"); row still `fastembed`; next search still 0.71 ✓

**P3 (1907):**
- PUT `rag.embedding_provider` → **422** `unsupported_setting` with pointer
  to `llm.embedding.*`; row unchanged ✓
- PUT `rag.chunk_size`=512 → **200**, persisted (rev
  2026-10-19T12:27:27Z), `activation: {"rag.chunk_size":"restart_required"}` ✓

**P4 (1907) — boot wiring + restart retention PASS (v3):** rows
`memory.enabled=true` + isolated `memory.db_path` persisted with
`restart_required` while the running process honestly reported memory
inactive; on the v3 restart the boot log shows
`MemoryService initialized (embedding_provider=local, auto_capture=true,
inject_context=true)` although the config file says `memory.enabled: false` —
the service was constructed from the persisted rows alone.

## Ledger boundary — 2026-10-19 13:13 UTC

- User-approved additive renewal: prior ceiling 432 + 200 = **632 cumulative calls**.
- Authoritative independent count: **547 tool blocks as of 13:13 UTC** → 85
  remaining. The earlier overrun above 432 is preserved (not reset, not
  retroactively reclassified); 497/632 was only the historical 12:42 snapshot.
- Scope note: the in-flight build (v4) is the mandated fix for the demonstrated
  vacuous live-activation defect (`FastEmbedBackend` ignores `config.model`, so
  v3 reported `live` for values no consumer honors). The frozen bar requires
  embedding changes to be actually live or explicitly rejected; v3 could only
  yield fake-live or NOT-READY. v4 is that minimum coherent fix, not a duplicate
  build or scope expansion. Proof plan P1–P4 and R1–R23 remain frozen.
- P1's five-leg focused proof is complete on the no-key deployment class; it is
  evidence for the configuration gate, **not** a full R-row PASS.
- Frozen query-time space-guard criterion: if no second SUPPORTED space can be
  built in this deployment to exercise it, it stays **UNVERIFIED** —
  mutation-boundary rejection is not equivalent evidence.

**indexed_embedding trace (source, v3):** the API GET KB handler projects
`KbConfigResponse` (src/uar/api/knowledge.rs:89-97), a 7-field DTO that omits
`indexed_embedding`, so the earlier "null" reads are uninformative about the
row. The domain field serializes when `Some`
(src/uar/domain/knowledge.rs:201-207). The v3 ingest path binds via
`prepare_ingestion` → `bind_embedding_space` (src/uar/rag/ingest.rs:50-66,
called before chunk/embed). Whether the v3 API uploads actually persisted the
fingerprint is decidable only by reading the `knowledge_bases` row in the
embedded KV during the v4 restart window — recorded below when executed.

## v4 results (binary sha256 96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154)

**Restart-window KV readback (1907 embedded KV, 13:22 UTC):**
- `knowledge_bases` p2-probe-kb: `indexed_embedding = {backend: fastembed,
  model: bge-small-en-v1.5, dimension: 384}` — v3 bind-on-prepare DID persist;
  the API `KbConfigResponse` DTO omission was the only reason it read null.
- default KB and the pre-v3 r14 KB: unbound (NONE), as expected.

**P2 v4 legs — PASS:**
- Boot overlay on out-of-band row (`llm.embedding.model=bogus-model-y`):
  "Persisted embedding settings are not honored by this build; keeping
  config-built backend" (13:23:03, served_model bge-small-en-v1.5) ✓
- PUT `llm.embedding.model=bogus-model-2` → **422** `unsupported_setting`
  ("not honored by the 'fastembed' backend in this build, which serves the
  packaged 'bge-small-en-v1.5' model only; … nothing was persisted"); row
  unchanged (still bogus-model-y, rev 2026-10-19T13:00:09Z) ✓
- DELETE (reset) `llm.embedding.model` with If-Match → **200**
  `activation: {"llm.embedding.model":"live"}`; hot-swap log model
  bge-small-en-v1.5 dim 384; next probe-KB search 0.6399 (bound fingerprint
  matches; next-request behavior on the honored default) ✓

**P4 — COMPLETE (all four citations):**
- Persisted rows: `memory.enabled=true` + `memory.db_path=<isolated>` (PUT
  200s, `activation: restart_required`, rev 2026-10-19T12:2xZ).
- Boot log (v4, 13:23:03): `MemoryService initialized
  (embedding_provider=local, auto_capture=true, inject_context=true)` while
  the config file says `memory.enabled: false`.
- Live behavior: memory native tools `active=true` (v4 boot) vs `active=false`
  (12:23 boot when rows were false).
- Disable/restart: PUT `memory.enabled=false` → 200 `restart_required` →
  restart → "Memory system disabled" (13:26:16); post-reset embedding row =
  default → no overlay warning; search healthy (0.6399) ✓

**Query-time space-guard criterion: UNVERIFIED (frozen).** No second
SUPPORTED embedding space can be built in this deployment: fastembed honors
only bge-small/384; openai/voyage/cohere cannot build without embedding
credentials (they 422 with the build error). A space mismatch therefore
cannot be produced through any permitted path, and mutation-boundary
rejection is NOT claimed as equivalent. The guard + ingest binding are in
place for builds with a second honored space; the criterion stays open.

## Gate verdict — CONFIGURATION LIFECYCLE: NOT READY (unwaived contract)

The frozen contract includes the query-time space-mismatch guard, and that
criterion is UNVERIFIED above: no second supported space can be built in this
deployment to exercise it, and mutation-boundary rejection is not equivalent
evidence. Under the unwaived contract the gate therefore stays **NOT READY**.
What IS proven: P1/P3/P4 in full, and P2's supported mutation/reset subchecks
(auto-seed on existing db, honored-value live activation + next-request
behavior, unhonored-value 422 with unchanged row, boot-overlay refusal of
unhonored drift, ingest-time space binding). P2 as a whole is not complete —
its query-time guard leg is open. Shortest unresolved action: run this proof
on a build/deployment with a second honored embedding space (e.g. an
embedding credential for openai/voyage/cohere, or a multi-model local build),
bind a KB to space A, swap to space B, and observe the explicit query-time
refusal.

P1–P4 are configuration-gate evidence, not R-row PASS verdicts; R-row
reconciliation resumes separately under the frozen contract.

## Parent certification checkpoint — 2026-10-19 14:12 UTC (pre-descent)

- Parent: `function-first-release-certification` (childPointer: selected, not
  entered). Descending via canonical `kbd-child-exit.sh --enter`, then nesting
  `startup-config-lifecycle-root-cause` via canonical `kbd-new-child.sh`.
- Candidate of record: v4 sha256
  `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154`
  (`target/release/universal-agent-runtime`, server-full release).
- Source binding: dirty checkout on #419 + checkpoint repair + lifecycle edits
  (providers carry-over + honored-value activation + bind-on-prepare). No
  commits, no publication. 1907/1908 configs: unchanged temp-dir fixtures.
- R1–R23 state: 15 rows PASS with receipts (R1,R2(a),R3–R6,R11,R12,R14–R20);
  R7/R9/R13 recorded contract gaps; R10 PARTIAL; R2(b–d) activation not
  firing; R21–R23 not started. Rows frozen; child does not expand them.
- Config gate: NOT READY (unwaived) — sole open leg: query-time space guard
  UNVERIFIED (no second honored space in this deployment).
- Call accounting: 579 tool blocks at 14:08:50 UTC (user's SQLite count;
  independent recount inconclusive across compaction session segments —
  recorded, not fabricated). ~585 now, ~47 under the 632 ceiling. Overrun
  history above 432 preserved. User authorized additive extensions with
  per-extension recording; forecast fits within 632, none taken yet.

## Call ledger — extension #1 + reconciliation (2026-10-19 14:30 UTC)

- User-authorized additive extension: historical 432 + 200 = 632, now
  **632 + 100 = 732 cumulative**. 432-overrun history preserved; no reset.
- User's independent exact count (session_message tool content): **610 tool
  blocks at 14:27:44 UTC**. Agent estimate ~617 was superseded.
- Agent reconciliation attempt (recorded, inconclusive): compaction splits
  opencode session segments; `session_v2.parent_id` does not chain segments
  (walk returns only the latest 27-message segment); `session_message.type`
  has no 'tool' value (assistant/user/system/compaction/...); a LIKE probe on
  tool JSON counts messages across ALL today's sessions (5644, wrong scope).
  Ledger therefore adopts the user's 610 @ 14:27:44 + 3 reconciliation calls
  = **~613 current, ~119 remaining under 732**. Method and failure recorded,
  not fabricated.
- Child `startup-config-lifecycle-root-cause` closed: verdict PASS on frozen
  exit contract (projection-derived INCOMPLETE due to 0/0 untracked ledger —
  projection untouched). Attribution corrected: P1=v3, P2/P3/P4=v4.

## Call ledger — extension #2 (2026-10-19 15:14 UTC)

- User's independent exact SQLite count (assistant tool blocks, this session):
  **723/732 at ~15:14 UTC**.
- User-authorized additive extension #2: **+200 → cumulative ceiling
  732 + 200 = 932**. Historical chain preserved: 432 (original, overrun
  preserved) + 200 = 632; +100 = 732 (extension #1); +200 = 932 (extension
  #2). No reset, no reclassification. Purpose: bounded embedding
  primary+backup child + R-row resumption. No paygo, no model switch.
- Child `embedding-primary-backup-lifecycle` created via canonical typed
  phase events (create/activate, committedLocally) at correctly nested path;
  exit contract frozen before edits (rev 2: blocker ⇒ NOT READY never PASS;
  query-guard unexercisable ⇒ UNVERIFIED; bounded same-source local build
  authorized).

## Call-ledger extension #3 — 2026-10-19 ~17:45 UTC (recorded BEFORE crossing 932)

Per operator instruction this turn ("Record the exact cumulative call
ledger and any additive extension before crossing 932"): operator
independent counts 723/732 at 15:14 UTC, 740 at ~15:24 UTC; best
reconstruction since ≈ 130 calls ⇒ estimated position **~870/932**.
Additive extension #3: **+100 calls ⇒ cumulative ceiling 1032**.
Recorded while below 932; if the operator's independent count shows the
crossing already happened, this entry still post-dates the last
confirmed sub-ceiling count (740) and the overrun segment is declared
here rather than silently absorbed. Scope unchanged: in-scope probes
only; no new build, provider, spending, security-setting change, or
publication.
