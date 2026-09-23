## Context

One-page design required by the-boss decision D3 before any agentic-chunking code. See `proposal.md` for why. *Operator decision 2026-09-23: approved* (D1-D7 as written; D8-D9 added the same day for the request-scoped credential and ownership). Constraints read from code:

- `Chunker::chunk` returns `Vec<String>` (`src/uar/rag/chunking.rs:44-84`). Other strategies trim chunks (`with_trim(true)`, `:53-66`), so only agentic chunking promises exact reconstruction.
- `ChunkingStrategy::Agentic` is a unit variant and is serialized inside stored `KbConfig` (`src/uar/domain/knowledge.rs:181-184`). Changing its shape would touch stored data; this design keeps it a unit variant.
- The ingestion worker already loads the knowledge base for each job (`src/uar/rag/ingestion_worker.rs:206-215`) and then calls `ingest_text_with_metadata` (`:191-200`), which uses the single service-level chunker (`src/uar/rag/ingest.rs:124-132`).
- `MockLlmDriver` (`src/llm/mock_driver.rs:12-31`) replays scripted `NormalizedEvent` responses and records requests; it suits a deterministic falsifier.
- Upload (`upload_document`, `src/uar/api/knowledge.rs:383-477`) reads only the multipart `file` field (`:405-420`), saves the document `Pending`, submits `(document, bytes)` to `IngestionWorkerPool::submit` (`src/uar/rag/ingestion_worker.rs:441-470`) and answers 202. The job type `DocumentIngestionJob` derives `Debug` and `Clone` and not `Serialize` (`ingestion_worker.rs:31-39`). A failed ingestion stores `e.to_string()` in the document status (`:143-147`).

## Goals / Non-Goals

**Goals:** model-chosen boundaries, exact reconstruction, bounded model input per call, deterministic fallback, an explicitly chosen knowledge-base strategy honoured at ingest, and no change for knowledge bases that never chose one; a request-scoped credential so agentic ingestion works where UAR holds no key, and a fallback the host can see.

**Non-Goals:** re-chunking stored documents; overlap between chunks; a per-knowledge-base model choice; an ingestion cost budget; cancellation inside the window loop (see Risks).

## Decisions

### D1 — Boundaries are unit offsets, not character offsets
The document is first cut into units: `split_inclusive('\n')` lines, and any line longer than 400 characters is cut again with `split_inclusive(['.', '!', '?'])`. Units are contiguous byte ranges covering the whole input. The prompt lists each unit of the window with its index (`[12] ...`) and asks for `{"boundaries": [i, j, ...]}` — the indices of units that start a new chunk because the topic changes.
*Why:* models miscount characters, and a character offset can land inside a word or a UTF-8 sequence. Unit offsets cannot. *Alternative:* raw character offsets snapped to the nearest whitespace. Rejected: snapping makes "boundaries match the model's offsets" untestable and hides model error. This is the concrete meaning of "LLM-proposed boundary offsets" in D3; falsifier 3's "boundaries match the mock's offsets" is asserted as chunk start byte offsets equal to the proposed units' start offsets.

### D2 — Windowing carries the unfinished tail forward
A window holds consecutive units up to 8,000 characters, starting at the last accepted boundary. Boundaries the model returns inside the window are accepted; the text after the last accepted boundary becomes the start of the next window. If a full window yields no boundary, a boundary is forced at the window's end, so no chunk exceeds one window and each iteration advances at least one unit. The last window's tail is the final chunk. A single unit longer than a window is emitted as its own chunk.
*Alternative:* fixed, non-overlapping windows with a forced cut at every edge. Rejected: it cuts mid-topic every 8,000 characters.

### D3 — Validation, reconstruction, fallback
Per window, indices must be integers, strictly increasing, and inside `(window_start, window_end)`. Whitespace-only chunks merge into the previous chunk. After the loop the chunker checks that the concatenation equals the input byte for byte. Any failure — no driver, call error, unparseable reply, invalid indices, failed reconstruction — returns `Recursive { size: 512 }` for the whole document (the default size of `default_chunk_strategy`, `knowledge.rs:64-66`), logs a reason code (`no_llm`, `no_credential`, `call_failed`, `parse_error`, `invalid_offsets`, `reconstruction_mismatch`) and reports `recursive_fallback` as the chunk strategy. A partial agentic result is never mixed with recursive chunks. The chunker returns the reason code, never the driver's error text, so no provider error body (which may echo a key or URL) reaches a log line or the stored document status.

### D4 — The model handle comes from the host
`Chunker` gains `with_llm(Arc<dyn LlmDriver>)`; `IngestService::new` takes an optional driver. `src/server.rs` builds it once with `crate::llm::orchestrator::build_driver(&llm_config)` (`src/llm/orchestrator.rs:52`), the constructor runs use when no primary driver is supplied. A build failure leaves it `None` and agentic chunking falls back with `no_llm`. This host driver is used only when the upload carries no `model_credential` and the request is not launch-token-authenticated (D8).

### D5 — Strategy per job, explicit choices only
`ingest_text_with_metadata` takes an optional strategy. The worker passes `Some(kb.config.chunk_strategy)` when `kb.config.chunk_strategy_explicit` is true, and `None` otherwise. `None` means the service-level default strategy — `Semantic { threshold: 0.5 }`, built at `src/server.rs:738-742`, the only `IngestService::new` call in the tree. A chunker is built per job — it holds only `Arc`s. The strategy actually used is written to chunk metadata key `chunk_strategy`. The directory watcher's `ingest_file` path (`ingest.rs:46`) keeps the service default. The embedded host builds no `IngestService`, so this change does not touch embedded ingestion.
*Operator decision 2026-09-23:* knowledge bases with no explicit strategy keep semantic chunking at 0.5 for new uploads. This replaces the earlier proposal that switched them to recursive chunking.

### D6 — An explicit-choice marker, because a stored default is indistinguishable
A stored default cannot be told apart from an explicit choice. `KbConfig::chunk_strategy` is a plain `ChunkingStrategy` (`src/uar/domain/knowledge.rs:179-184`); a missing field deserializes to `Recursive { size: 512 }` (`:64-76`), and `KbConfig::default()` stores the same (`:204-213`). The API stores `Recursive { size }` for `"recursive"`, for any unknown name, and for a request with no `chunk_strategy` (`src/uar/api/knowledge.rs:734-745`, `:693-711`). The config-file path is no better: `ChunkingConfig::strategy` defaults to `"recursive"` with size 512 (`src/config.rs:1006-1026`), so a written `recursive` looks like an omitted table.

The minimal marker is one field: `KbConfig::chunk_strategy_explicit: bool`, `#[serde(default)]`. Existing records read `false`. It is set to `true`:
- by `build_kb_config` when the create request carries `chunk_strategy` or `chunk_size`;
- by `merge_kb_config` under the condition that already rewrites the strategy (`req.chunk_strategy.is_some() || req.chunk_size.is_some()`, `src/uar/api/knowledge.rs:727-730`).

It is never set by `KbConfig::default()` or by `ensure_default_knowledge_base` (`src/uar/defaults.rs:220-288`). Both production hosts call that function with `None` (`src/server.rs:760`, `src/embedded.rs:360`); only `src/bin/test_db_setup.rs:48` passes a config, and that path cannot tell a written table from a defaulted one, so it stays unmarked. Once set, the marker is not cleared; there is no "back to default" request.

`KbConfigResponse` gains `chunk_strategy_explicit` (`src/uar/api/knowledge.rs:81-88`). The `chunk_strategy` echo (`:648`) is unchanged, so an unmarked knowledge base still echoes `Recursive { size: 512 }` while its uploads are chunked semantically. A client reads the marker to know which applies.
*Alternative:* make `chunk_strategy` an `Option`. Rejected: it changes a stored field's type and the response echo, and old records carry a concrete value anyway, so `Some(Recursive{512})` would still be ambiguous.

Not confirmed from code: that every persistence provider round-trips a new key inside `config`. `migrations/surrealdb/schema.surql:53-57` declares `knowledge_bases` `SCHEMAFULL` with `config TYPE object`; Postgres stores `config` as one column (`src/uar/persistence/providers/postgres.rs:701`). Contract test 1.9 proves the round-trip on SurrealKV; task 4.3 checks the other providers.

### D7 — Document text is untrusted
Document content goes into the prompt (prompt-injection surface at a real boundary). The reply can only move boundaries: it contributes indices, never text, and reconstruction guarantees stored chunks are the document's own bytes. The reply is not logged verbatim.

### D8 — A request-scoped model credential for one ingestion
*Operator decision 2026-09-23:* the-boss uses UAR knowledge bases, and sidecar mode persists no provider keys, so an upload may carry its own credential. The shape, validation and confinement are those of the run credential (`run-scoped-credentials-and-mcp-servers` design D10, with its D1 type rules); this change reuses its input type and validator and does not define a second one.
- **Carrier.** An optional multipart text field `model_credential` on `POST /api/uar/knowledge-bases/{id}/documents`, holding the JSON object `{provider_id, provider_kind, base_url, api_key, default_model?}`. The loop at `knowledge.rs:405-420` reads it next to `file`. It is parsed into the run credential's input type (deserialize-only, `SecretString` key and base URL, redacting `Debug`) and checked by the same validator: `provider_id` pattern, base-URL rules, `openai_compatible` or `anthropic` kind, and the local-only refusal. Only the error codes differ, because this is not a run: 422 `ingest_credential_invalid` and `ingest_credential_provider_kind_unsupported`. The body is fixed text; it never includes the URL, the key or a serde message (serde errors can quote the offending value).
- **Model.** Ingestion has no routing, so the model is `default_model`. When the knowledge base's effective strategy is agentic, a credential without `default_model` is 422 `ingest_credential_invalid`. For any other strategy the credential is validated and dropped at the route.
- **Driver choice.** Credential present → a driver built from it with the run credential's mapping (run-provider kind, unprefixed model, `api_key_env: None`, no `ProviderRegistry::register`, no `ProviderHealthMonitor` record). No credential and the request carries `HostAuthenticated` (the launch-token marker, `sidecar-session-principal` design) → no driver; fallback `no_credential`, no model call, so the document never goes to UAR's own configured endpoint in the-boss. No credential in standalone mode → the host driver (D4).
- **Lifetime.** `DocumentIngestionJob` gains the credential as a field whose type has no `Serialize` and a redacting `Debug` (the job derives `Debug`, `ingestion_worker.rs:31`). The executor builds the driver, attaches it to the per-job chunker (D5), and drops job, driver and chunker when `execute` returns. `IngestionResult` does not carry it. The credential is never written to the document, chunk metadata, settings or logs.
- *Not confirmed:* whether `prometheus_parking_lot`'s `WorkerPool` clones or keeps a job after `execute` (`submit_async`, `ingestion_worker.rs:468`). Task 5.3 checks it by inspection.
- *Alternative rejected:* a stored per-knowledge-base credential. It persists a key, which sidecar mode forbids, and outlives the ingestion.

### D9 — Fallback reporting, ownership and the capability flag
- **Reporting.** `KnowledgeDocument` gains `chunk_strategy` (the strategy used, e.g. `agentic`, `recursive_fallback`, `semantic`) and `chunking_fallback` (a D3 reason code or null), both `Option<String>` with `#[serde(default)]`. The executor writes them with the `Indexed` status through a new persistence method beside `update_document_status` (`src/uar/persistence/mod.rs:205-211`, which writes status only). `doc_to_response` (`knowledge.rs:656-676`) returns both, so `GET .../documents/{doc_id}` and the document list show them. A fallback is not a failure: the status stays `indexed`. When the fallback is certain at upload — sidecar request, agentic effective strategy, no credential — the 202 response already carries `chunking_fallback: "no_credential"`. `knowledge_documents` is `SCHEMAFULL` (`migrations/surrealdb/schema.surql:67-77`), so the schema gains two `option<string>` fields; whether that file is applied to existing embedded databases was not checked (task 5.3).
- **Ownership.** Every knowledge-base handler takes the owner from `Extension<UserContext>`: list `:181`, create `:209`/`:226`, get `:252`, update `:277`, delete `:332`/`:342`, upload `:392`/`:432`, search `:594`; the worker stores chunks under `job.document.owner_id` (`ingestion_worker.rs:196`). With `sidecar-session-principal`, `user_id` is the asserted principal, so no scoping code changes; test 1.13 proves isolation.
- **Flag.** `ingest_scoped_credentials` joins the `GET /api/uar/capabilities` vocabulary (`sidecar-launch-security` Decision 12). It is listed only once D8 and D9 work; the-boss offers `agentic` only when it is listed.

## Risks / Trade-offs

- [No usable credential, e.g. inside the-boss] → the host sends `model_credential` with each upload (D8). Without it the ingest falls back with `no_credential` and the document reports it (D9); the-boss hides `agentic` until `ingest_scoped_credentials` is listed.
- [A host-supplied base URL can point at an internal service] → accepted on the same grounds as the run credential: the caller is the launch-token-authenticated host; local-only mode still confines it to loopback.
- [Knowledge bases are per principal] → a knowledge base is visible only to the principal that created it. Which the-boss scope a principal names (one conversation or wider) is the host's choice, not this change's.
- [Cost and latency] → one sequential model call per window; a 1 MB document is about 125 calls. Not charged to any run budget.
- [Shutdown during a long agentic ingest] → the window loop does not observe the worker's cancellation token (`ingestion_worker.rs:182`). Should the loop check it between windows? Not added without an observed hang.
- [Behaviour change for existing knowledge bases] → none. Operator decision 2026-09-23 (task 0.3): unmarked knowledge bases keep semantic chunking at 0.5; existing chunks are untouched. Contract test 1.9 holds it.
- [The echoed strategy disagrees with the one used] → an unmarked knowledge base echoes `Recursive { size: 512 }` and is chunked semantically. The additive `chunk_strategy_explicit` field is the only signal; the-boss must read it before showing a strategy.
- [A request that sends only `chunk_size` marks the choice explicit] → it already rewrites the stored strategy to `Recursive { size }` today (`knowledge.rs:727-730`); after this change that rewrite also takes effect at ingest.

## Migration Plan

No data migration. `Agentic` stays a unit variant. `KbConfig` gains one `#[serde(default)]` boolean; stored records read `false`. `KnowledgeDocument` gains two `#[serde(default)]` optional strings; stored records read `None`. Deploy with or after `run-scoped-credentials-and-mcp-servers` (credential type and validator) and `sidecar-session-principal`. Rollback is a code revert: an older UAR ignores the unknown field on read (no `deny_unknown_fields` on `KbConfig`, `knowledge.rs:155-157`), all uploads go back to semantic chunking, and knowledge bases storing `Agentic` keep their existing chunks.
