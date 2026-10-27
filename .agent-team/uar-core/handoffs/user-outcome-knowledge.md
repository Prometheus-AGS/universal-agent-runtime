# Task 2.6 — concrete regeneration design before implementation

Phase `uar-post-lifecycle-verification-readiness`; **verification CLOSED**. Source inspection/authorized authoring only. No runtime/SQL/import/test/build/check/probe. This design is recorded before any new public action or production edit.

## Source-observed baseline

The existing mounted router is `/api/uar/knowledge-bases`. Durable uploads already accept immutable bytes into namespace-specific `knowledge_ingestion` records, with fenced `IngestionJob` CAS and restart discovery; `DocumentIngestionExecutor` extracts retained bytes, prepares pinned-backend chunks, then `publish_ingestion` atomically replaces one document's visible chunks. The indexed fingerprint guard rejects a new space, so changing a model is currently refusal rather than regeneration. PostgreSQL `knowledge_chunks` and `principal_knowledge_chunks` use `VECTOR(384)` plus dimension-specific HNSW indexes; another dimension cannot be stored without a schema/index change. Both provider publish paths already lock/transactionally update the KB and exact job record. Source bytes remain present after indexing.

## Minimal bounded behavior proposed for approval within the existing plan

Add an authenticated owner-scoped `POST /api/uar/knowledge-bases/{id}/regeneration` and `GET` at the same path. POST uses the ACTUAL currently pinned global embedding fingerprint (not arbitrary KB metadata or a per-KB pretend model), requires an ingestion pool, and atomically starts/retries one KB-wide durable cohort using the existing retained source jobs. GET reports durable cohort identity, previous/target fingerprints, per-document attempt state/count and bounded last error. Existing per-document `/retry` remains for normal failed uploads; a regeneration retry uses the KB POST and its retained cohort. While running or failed, KB retrieval returns an explicit regeneration-unavailable response; no partial/mixed generation may reach search or grounded chat. Model-selection settings remain the existing real global settings routes, changed before the POST.

### Durable state machine and transaction fence

`KbConfig.regeneration: Option<KnowledgeRegeneration>` contains a generation UUID, previous indexed fingerprint, target actual fingerprint, frozen document IDs, state (`running`, `failed`, `complete`), completed count and last error. One atomic `begin_knowledge_regeneration(owner,kb,target)` provider transaction locks the KB, freezes every current document, requires retained bytes for every member and a durable ingestion-capable store, revokes each prior attempt by replacing each job generation/state with fresh pending work, stores the new cohort marker and sets documents pending. The old indexed fingerprint stays unchanged until the cohort completes. Reject concurrent running POST; a failed cohort can be retried atomically from retained sources with a fresh cohort/attempt identity. No old worker may publish after the requeue CAS.

Workers use the existing lifecycle-owned bounded pool and restart discovery. Ingest preparation detects a running cohort member and pins the real backend; it requires its fingerprint exactly equal target, but skips the old-space binding refusal only for that authorized cohort member. Chunk metadata includes the cohort generation. Provider `publish_ingestion` permits the target fingerprint only for the current cohort member, replaces that document's chunks transactionally, marks its job indexed, recomputes cohort completion from all exact member job identities, and changes indexed_embedding to target + state complete ONLY when every member is indexed in this cohort. Thus no in-memory-only coordinator or fire-and-forget task is introduced. A worker failure updates durable job retry/error plus cohort error/progress truthfully. Exhausted job marks cohort failed; explicit KB retry requeues it. An empty KB transitions complete immediately with target identity and zero documents.

All retrieval paths must consult the current marker BEFORE scoring and reject running/failed cohorts. Provider `search_knowledge[_scoped]` must additionally fence retrieval transaction/read identity so a concurrent begin cannot expose mixed vector generations after an earlier API/runtime guard read. Upload/new document, per-document retry/delete and KB config mutation during regeneration must be fenced in the same provider KB transaction to keep cohort fixed; whole KB delete may revoke everything normally. Generic `save_knowledge_base` must preserve/validate the current regeneration marker to prevent a stale API/settings write overwriting it. The publication transaction is the completion owner; restart discovers pending/processing/retryable retained jobs. The target model must still be active after restart or jobs explicitly fail/retry with backend mismatch rather than generating different vectors.

### Different dimensions

Do not ALTER a global column to each model's dimension: other owners/KBs may use another dimension. PostgreSQL needs a migration to unconstrained `vector` storage (retaining per-row dimensions) and removal/replacement of fixed-dimension HNSW indexes with dimension-aware indexes or the existing exact-scan distance path with explicit dimension predicates. Required correctness is no mixed scoring, actual new-dimension storage and retrieval; an index strategy must not pretend one global dimension. Surreal's stored vector arrays/distance path must enforce selected KB/current dimension and continue using only the completed cohort. The source-backed fixture observes actual vector lengths, cohort markers, retained-source identity and post-restart state on both stores.

## Root-owned integration requested

- Approve this concrete public action/state behavior, or send the required narrower design correction before production authoring.
- Add the exact `PersistenceLayer::begin_knowledge_regeneration(owner_id: &str,kb_id: &str,target: &EmbeddingFingerprint)->Result<KnowledgeRegeneration>` transaction method and provider implementation; add/get progress can use existing owner-scoped KB get. The type will live in `domain/knowledge.rs` under this teammate's ownership.
- Add a defaulted optional cohort field to `IngestionJob` OR implement the equivalent exact member-generation mapping in the KB cohort. Recommendation: defaulted `regeneration: Option<String>` on the job, preserved by processing/retry and checked at publish; no need to alter the normal ingestion transition contract. I will use that exact field after root agrees.
- Root owns provider accept/transition/publish/delete/save/search fences, persistence/scoped dispatch, dimension migration and `rag/mod.rs` registration. Root also updates runtime retrieval to reject `KnowledgeBase::regeneration_unavailable` before query; this teammate handles mounted knowledge REST retrieval.
- Root owns server state composition if a new handle becomes necessary; current design deliberately reuses ingestion_pool and adds no worker owner.

## Ownership and remaining implementation

This teammate owns knowledge API/domain, ingest/executor, NEW `rag/regeneration.rs`, cases_knowledge.py and observers_knowledge.py. Root owns all shared persistence/provider/host/module files. No files outside this design handoff have yet been modified for task2.6. U14/U15 fixtures must demonstrate private document fact/citation grounding, actual global embedding A→B same-dimension and A/B→C changed-dimension, useful answers afterwards, no mixed reads, retained bytes and an interrupted process restart with durable resumption. Static source and authored cases cannot certify those outcomes.

## Resumed task2.6 implementation handoff — 2026-10-25

Phase `uar-post-lifecycle-verification-readiness`, stage `user-outcome-implementation`. **Verification CLOSED.** The approved design above was implemented after approval; its original ownership list is superseded by the lead packet assigning KB-specific persistence/provider spans to this teammate. Other writers share this checkout and their changes were preserved. This is a source handoff, not acceptance, independent review or permission to execute.

### Production implementation present

- `src/uar/domain/knowledge.rs`: defaulted optional `KbConfig.regeneration`; durable previous/target fingerprints, frozen document membership, generation UUID, running/failed/complete status, progress/error/time metadata; `regeneration_unavailable` fences both running and failed cohorts.
- `src/uar/api/knowledge.rs`: mounted authenticated owner-scoped POST/GET `/{id}/regeneration`, actual live global fingerprint target, durable-store/pool admission, retained source errors (422), conflict mapping (409), durable count/status projection; ordinary member retry refuses cohort retries; upload/config/delete conflicts now map to explicit status. Public config responses include cohort and actual indexed embedding. Query pins one backend and routes through identity-aware storage search.
- `src/uar/rag/regeneration.rs`: exact current cohort/member/target admission; normal preparation cannot write across an incomplete cohort or incompatible indexed space. PostgreSQL publication/transition recompute durable progress against exact member records; only complete membership changes the indexed identity. Failed cohorts stay unavailable pending explicit whole-KB retry.
- `src/uar/rag/ingest.rs`: prepare once with pinned real backend; deterministic attempt chunks include embedding-space, attempt and cohort provenance. Authorized cohort preparation skips only the old indexed-space refusal. Legacy direct writes carry actual embedding-space metadata. `ingestion_worker.rs` uses the existing retained-source, supervised, bounded lifecycle-owned pool and exact processing-job publication. Parking Lot 0.4 rejected/expired drain changes were preserved.
- `src/uar/persistence/ingestion_records/mod.rs`: optional durable job cohort marker, preserved across processing/retry and transition identity checks. `persistence/mod.rs` declares begin and identity-aware search; scoped dispatch and runtime manager query pinning are lead-owned integration.
- `src/uar/persistence/providers/ingestion_records/{postgres,surreal}.rs`: cohort begin freezes current owner documents, requires retained bytes and no orphan chunks, replaces attempts atomically, conflicts against KB marker, and permits explicit failed-cohort restart. Accept/transition/publish/member delete/member save/KB save participate in the KB fence; whole-KB delete revokes normally. Publication refuses old attempts and incompatible targets, preserves source bytes and only switches indexed identity after all members finish. Empty KBs immediately publish coherent target metadata. Search excludes incomplete cohorts and mismatched fingerprint/dimension before scoring. Failed cohorts are excluded from dispatcher discovery/due scheduling: otherwise pending/retryable members would repeatedly redispatch although their cohort is terminal and claims/publications are fenced. PostgreSQL uses owner+KB joins; Surreal uses owner/KB tuples, so another principal's identically named KB cannot reopen a failed cohort.
- `src/uar/persistence/providers/{postgres,surreal}.rs`: KB-specific delegation/fences are wired into provider implementations, including transactional generic member writes and deletes. No unrelated shared provider spans were overwritten.
- `migrations/20261025010000_knowledge_regeneration_dimensions.sql`: drops the two fixed-dimension KB HNSW indexes and removes both vector typemods; existing owner/KB indexes plus exact identity/dimension-fenced search provide correctness. No replacement ANN index is claimed.
- `src/uar/api/settings.rs`: candidate/prevalidation/activation/reset/bulk handling includes `llm.embedding.vector_dimension` with supported integer/dimension agreement. Lead owns setting definition and boot hydration. Separately approved narrow provenance addition pins one backend coherently and emits its actual sanitized endpoint: username/password/query/fragment removed, invalid/non-HTTP endpoints emit null, and no parse error/raw endpoint is returned. This is a real operator-facing credential boundary.

### Concrete authored acceptance sources

`scripts/acceptance/cases_knowledge.py` exports three 3-repetition durable cases, parameterized by the shared runner across both mandatory adapters:

- `U14.knowledge-lifecycle`: actual global model A selection, two freshly random private facts uploaded by mounted multipart API, durable source-hash/nonzero-vector/actual-dimension observations, ranked exact-document search, genuine grounded answers with exact numbered citation and persisted public `agui.rag_citations` document/KB/snippet correlation. Replacement uses the API's actual delete+upload contract; old content must disappear. Deleting one document must yield UNKNOWN for its fresh-session question while the retained fact remains useful. True process restart preserves retained rows and usefulness; KB deletion removes documents/jobs/chunks and stays removed after another restart.
- `U15.embedding-regeneration`: real A/B equal-dimension and C changed-dimension bindings; all actual settings saved with revisions and actual serving fingerprint/endpoint or local provenance compared. Multiple retained real documents generate enough chunk work for honest interruption. Old-space query must explicitly refuse. A→B must actually change stored vector digests, preserve source bytes, publish only the current generation and recover exact grounded answer/citation. C rebuild must expose an unavailable query, then the actual invocation-owned server process group is killed with SIGKILL and reaped. While down, source observer must establish the same running cohort with unfinished durable members; a too-fast completed fixture BLOCKS instead of claiming recovery. A true restart must hydrate the exact C fingerprint/dimension, existing dispatcher resumes, every retained document/vector is in the same completed cohort, ranked facts/citations recover and survive another cold restart. In-progress successful query results are accepted only if every returned row is tagged with target fingerprint/current generation; mixed vectors cannot pass.
- `U16.knowledge-adapter-contract`: one common logical record/revision contract independently observed on each actual admitted durable adapter. A fresh retained private fact, actual vectors and indexed identity, saved agent artifact, ranked document and exact grounded citation must remain identical/useful after a true PID change. Receipts name backend, actual physical store identity, common contract ID, logical IDs, retained hash, actual dimensions, expected/actual fact and process boundary. One backend invocation never claims the other passed. This case supplements the runner's requirement that every other durable family also run on both adapters.

`scripts/acceptance/observers_knowledge.py` implements the `knowledge` hook with no argv action: it uses `UAR_ACCEPTANCE_STARTUP_CONFIG` and the existing real `observers_startup.Store` adapters. Exact authenticated principal tables are read in the owned run-derived Postgres schema or loopback Surreal DB. It requires the random KB's invocation-owned name, one verified owner partition and exact member/chunk ownership; returns actual config/cohort, documents, retained job metadata/source byte counts+SHA-256, nonzero finite vector lengths/digests, embedding/cohort provenance and physical vector storage type. PostgreSQL observes actual `format_type` and requires `vector`, not an asserted migration label. Deleted KB observation refuses orphan records. Raw retained bytes, vector values and arbitrary upstream diagnostics are not emitted. Store connection details are inherited from the frozen startup observer contract rather than invented connection commands.

### Exact integration contract for task2.9

Freeze and load `cases_knowledge` alongside other case modules; include all three case IDs above in required manifest. Bind observer `knowledge` to exact frozen Python executable + `observers_knowledge.py` file (no argument). Freeze `observers_knowledge.py`, `observers_startup.py`, config JSON and interpreter as existing runner contract requires. Both adapters must bind `UAR_ACCEPTANCE_STARTUP_CONFIG`/credential references through explicit environment mappings. Postgres needs the newly migrated schema; Surreal observer transport uses the same pinned owned disk service and run-derived DB as the runtime.

Fixture `knowledge` object requires finite positive `index_seconds` and `regeneration_seconds`, bounded `regeneration_documents` (8..128; default8), and exactly three `embedding_spaces` objects. Each binding needs `kind="genuine"`, `synthetic=false`, `fingerprint={backend,model,dimension}`, nonempty `model_source_identity`, and either `source_kind="approved_hosted_endpoint"` plus exact sanitized `endpoint`, or `source_kind="local_asset_manifest"` plus actual runtime manifest identity. A/B fingerprints differ while dimensions agree; C changes dimension. Hosted source identity is the frozen operator-approved endpoint/model provenance and actual runtime selection. It is not cryptographic proof an operator's remote peer cannot be fake. The installed single-model packaged local path cannot manufacture real A/B/C by relabeling the same assets; use actual supported genuine bindings or the case blocks.

Lead must reconcile setting definition + restart hydration for vector dimension, matching engine source ownership, all-source inventory, fixture resource/usage accounting and exact future commands. This handoff does not open the lock.

### Verification and scope self-check

Commands executed for this resume were `cat`, `sed`, `rg`, `ls` source inspection and authorized heredoc/`apply_patch` source/document writes, plus team messages. **No test, import, py_compile, format, lint, typecheck, Cargo command, build, policy validator, database query, model call, service or acceptance invocation ran.** No independent review verification ran. Every Rust/Python/SQL compatibility claim and product outcome remains unverified until the operator explicitly opens the exact full-phase gate. There is no passing result or acceptance credit.

No unrelated feature was added. Guards trace to the approved fixed-cohort/no-mixed-generation requirement, actual missing-retained-source boundary, real owner partition, prior incompatible embedding comparison, terminal-cohort redispatch failure, or operator-facing endpoint credential boundary. No speculative timeout/retry layer was added. The source reader surfaced no additional required production behavior outside this task. Potential SQL/Python/Rust compatibility issues cannot be resolved by an early probe under this lock and are intentionally unverified.

## Cohere OpenAI compatibility dimensions option — source-only follow-up

User selected OpenAI compatibility interface. Root changed only existing profile
COHERE_EMBEDDING_URL to https://api.cohere.ai/compatibility/v1; this writer neither
mutated nor sourced the profile. Credential variable name COHERE_API_KEY was
inspected as NAME ONLY (not its value). Config option llm.embedding.send_dimensions
now defaults true in serde/default/config loader and propagates through
EmbeddingConfig into OpenAiBackend. False suppresses unsupported dimensions field;
true preserves the old non1536 conditional. Configured model/vector_dimension and
embedding fingerprint do not change implicitly. Actual returned vectors now fail
DimensionMismatch if their widths differ, rather than padding/truncating.

Changed config.rs; rag/embeddings/{mod,openai}.rs; necessary full literal in
tests/embedding_lifecycle_failover.rs; typed settings schema/seed in
settings/manager.rs; API settings candidate/prevalidation/live mutation/reset/
namespace paths; server boot boolean hydration; narrow docs/config/
cohere-openai-embeddings.md and .example.yaml. The setting supports an explicit
false without inferring compatibility from endpoint host. Local failover ownership
still rejects live mutation. Existing settings writers' other spans preserved.

Example uses actual existing COHERE_API_KEY name, backend openai, Cohere
compatibility endpoint, embed-english-v3.0/1024/send_dimensionsfalse. Other documented
separate choices embed-multilingual-v3.0/1024 and embed-english-light-v3.0/384 are
listed without availability claims. Composer notified. This switch governs RAG
embedding backend only; separate memory SDK request contract is not silently
changed or certified by it.

No tests, imports, format/check/build, provider/service/embedding calls, or review
verification executed. Gate CLOSED; all behavior and compilation unverified.

## Confirmed owned-memory OpenAI/Cohere compatibility source trace

No additional production edits. Actual UAR src/uar/memory/service.rs openai branch
passes memory.embedding_model, embedding_base_url, embedding_dim and resolved
embedding_api_key into EmbeddingProvider::OpenAI. Generic embedding_api_key is
first priority; openai_api_key/OPENAI_API_KEY/LLM_API_KEY are fallback. To reuse the
operator's COHERE_API_KEY without ambient fallback, fixture renderer must assign
that protected named value to memory.embedding_api_key; memory.embedding_api and
legacy embedding_provider both openai, model embed-english-v3.0, embedding_dim1024,
base_url exact approved budget embedding listener base. Native cohere is not this
interface: SDK hardcodes https://api.cohere.ai/v1/embed and does not honor the
configured compatibility base.

Vendored surreal-memory embeddings/openai.rs EmbeddingRequest is input Vec<String>,
model String, optional dimensions with skip_serializing_if=None. Actual embed_batch
always sets dimensions=None. Thus actual body contains ONLY input and model; no
dimensions or encoding_format. URL is base_url trimmed trailing slash + /embeddings.
Authorization is literal Bearer plus api_key. dimensions_override is used only by
dimensions() (otherwise unknown names default1536). Set embedding_dim1024 explicitly;
this does NOT add an unsupported request field. No new SDK/production switch is
needed for Cohere compatibility memory.

The SDK embed_batch maps returned vectors directly and does not enforce response
length against dimensions_override, despite its constructor doc's assertion wording.
Storage startup configures dimension-specific HNSW indexes and validates existing
record widths; see storage/surreal.rs ensure_embedding_indexes and
validate_existing_embedding_dimensions. Actual retained-vector lengths and useful
memory retrieval must therefore be proved by future concrete case receipts, not
by claiming an adapter check that isn't present. Source compatibility is not model
availability or grounded retrieval acceptance.

Root/composer notified of composition blockers: current composer enforces local
memory and clears memory base/key; change those explicit fixture paths for approved
OpenAI/Cohere memory. Also both RAG OpenAI and memory SDK prepend Bearer to api_key,
while earlier budget ingress_authorization_env contract expects the FULL header.
Using that same full-header env as raw api_key produces double Bearer; using a raw
env against full-header comparison refuses. Root must resolve with an explicit
frozen ingress prefix contract or separate raw/full references. No secret values
were read/copied/printed by this trace, and no production auth change was made.

Gate CLOSED. Source reads and this handoff append only; no import/tests/checks/
builds/provider/service calls or probes. Live service availability, model access,
actual vector widths, memory persistence/retrieval and approved charge exposure
remain unverified.
