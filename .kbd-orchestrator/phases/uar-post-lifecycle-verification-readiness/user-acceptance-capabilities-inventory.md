# User acceptance capabilities inventory — proposed, unexecuted

2026-10-24. Execute phase `uar-post-lifecycle-verification-readiness`; source-inspected waypoint revision390, plan114, implementation144/167. Lead is revising the acceptance scope under new operator direction. This inventory does not reconcile or replace the canonical ledger. VERIFICATION CLOSED. No test, check, compiler, build, service probe, provider call, review verification, or network mutation was run by this writer. Commands used only source inspection and this document write. Ownership: this file only; preserve other writers' files.

The acceptance subject is an end user's successful work: selected skills change effective behavior; selected context policy preserves necessary information within its budget; knowledge reaches answers and survives model changes; either durable adapter produces the same result; teams keep correct conversation and task context across calls. A settings round trip, isolated CRUD, synthetic answer, or old receipt cannot establish those outcomes. Positive inference claims require the configured real provider/model through the mounted production path. Deterministic peers are fault-injection controls only. Provider/model identity, input/output receipts, authoritative record revisions, owner and thread identity, and exact source/candidate identity belong in later evidence. Secrets must not.

## Proposed acceptance cases

All rows are UNEXECUTED. “Reuse” identifies source to adapt, not an assertion that it already covers the journey. Perform the same named cases with Postgres and durable Surreal, compare outcome predicates rather than generated prose, and retain separate per-adapter receipts. Do not create an arbitrary Cartesian product of every setting. Each selectable strategy gets its own discriminating effect case; shared durability/team/ownership journeys run once per adapter with named representative strategies, with additional combinations only for a source-backed interaction.

| ID | Designed selectable feature / journey | Required user outcome and oracle | Source/spec anchors | Reusable source / gap |
|---|---|---|---|---|
| SK-01 | Classifier backend `rules` | A prompt with declared rule evidence selects the intended skill; an unrelated skill remains absent; effective instructions/tool behavior changes in a real response. Retain candidate reasons and effective activation. | `src/uar/settings/manager.rs:1920`; `openspec/specs/standard-agent-skill-discovery/spec.md` | `tests/bauar_skill_selection.rs`, `tests/skill_activation_runtime.rs`; real mounted classifier-to-response wiring coverage must be established. |
| SK-02 | `tfidf` | Discriminating lexical relevance ranks/selects intended skill versus confounder; observe effective behavior, not backend label. | same classifier settings surface | Same harness candidates; exact algorithm production wiring and thresholds remain source inventory gaps. |
| SK-03 | `wasm` | Configured classifier module produces a discriminating intended activation through the production request. Failure must be visible and must not claim module success. | same classifier settings surface | `tests/wasm_skill_execution.rs` is execution evidence source only; does not by itself prove classifier routing. Module fixture/config and real-path coverage gap. |
| SK-04 | `hybrid` | A designed conflicting-evidence fixture demonstrates the implemented combination rule in selected skills and effective response. | same classifier settings; `src/uar/runtime/matching/mod.rs` | Actual classifier implementation/combination policy must be bound before defining exact rank oracle. Do not invent weights. |
| SK-05 | `localembedding` | Semantically relevant low-overlap prompt selects intended skill; unrelated skill absent; actual local inference and effective skill behavior observable. | same settings; `src/uar/runtime/matching/vector.rs`; `openspec/specs/local-embedding-inference/spec.md` | `tests/context_history_integrity.rs` uses VectorMatcher; production classifier-to-skill routing and real model receipt gap. Disabled vector build must not be called success. |
| SK-06 | `llm` | Actual configured classification provider response drives intended selected skill and subsequent agent behavior; classifier cost/failure observable separately. | same classifier settings surface | No real-provider classifier end-to-end coverage established by this inspection. Missing implementation/capability must remain explicit. |
| SK-07 | Selection `inherit`, `auto`, `all`, `none`, `selected`; activation `legacy_overlay`, `catalog`; admission `legacy`, `shadow`, `typed` | Each supported user selection changes the effective catalog/overlay and permitted activation; `none` excludes activation, selected excludes unselected; inherit resolves actual scope. Catalog explicit activation loads the selected body; shadow cannot silently reduce catalog. | `src/uar/settings/manager.rs:3402,1615,1618`; `openspec/specs/skill-activation-runtime/spec.md` | `tests/skill_activation_runtime.rs`, `tests/skill_scoped_governance.rs`, BDD skill activation; prove precedence, required skills and mounted inference; do not equate these orthogonal settings. |
| CX-01 | Structural `none` | Full canonical history reaches the provider unchanged except documented preparation; identity and paired call/results retained across calls. Final budget still enforced. | `src/uar/context/strategy.rs:16`; `tests/context_history_integrity.rs` | Existing history integrity cases; real-provider multi-call receipt gap. |
| CX-02 | Structural `sliding_window` | Beyond max_messages, intended recent history survives and old removable history drops; system identity stays pinned; protected tool groups remain coherent. | `src/uar/context/strategy.rs:19`; `tests/test_context_strategy.rs` | Isolated message-count tests are insufficient; inspect actual provider requests and follow-up answer using retained facts. |
| CX-03 | Structural `summarize` | Threshold crossing invokes governed real summarizer on eligible host-marked prose; summary retains exact planted old fact, recent turns survive, and next real answer uses it. No marks/driver/failure preserves originals and yields honest final budget outcome. | `src/uar/context/strategy.rs:28,148`; `openspec/specs/progressive-summarization/spec.md` | `tests/context_history_integrity.rs`; source comments mention fallback while marked-prose path retains originals: bind actual mounted path, not comment-era claims. |
| CX-04 | Structural `truncate_middle` | Exact first/last retained sets observable; middle removable content absent; protected groups and system unaffected. Later answer uses retained head and tail facts. | `src/uar/context/strategy.rs:42` | History integrity source; live request/answer evidence gap. |
| CX-05 | Structural `hierarchical` | Demonstrate actual short-term turns, mid-term summary and long-term facts at configured boundaries; real answer recalls preserved facts. | `src/uar/context/strategy.rs:51` | Declared fields do not prove implementation. Need full branch/wiring inspection; missing three-tier effects are a feature gap, never satisfied by settings echo. |
| CX-06 | Structural `auto` | Resolved model capacity changes effective policy per `strategy_for_model`/`resolve_effective_strategy`, with resulting observable reduction and answer preservation. | `src/uar/context/strategy.rs:59` | Source supports Auto; settings enum at manager.rs:2263 omits auto. Resolve exact reachable surface before claiming user selectability; do not invent UI exposure. |
| CX-07 | Token `sliding_window`, `keep_first_last` | Above trigger_threshold, measured preparation retains policy-defined recent/head messages within the resolved request budget; below threshold no unwarranted reduction. | `src/uar/domain/context.rs`; `src/uar/runtime/context/manager.rs:95,99` | `tests/context_history_integrity.rs`; expose real provider request and count-quality/capacity contract. Separate token strategy from similarly named structural policy. |
| CX-08 | Token `progressive_summarization` | At threshold, governed summarizer plus budget contract creates bounded summary preserving planted fact and tail. Missing contract/driver or failure retains originals and final gate honestly admits/refuses. | `src/uar/runtime/context/manager.rs:103,306`; domain ContextConfig summary_budget/summarization_model | Real summary/model override wiring and actual threshold/budget observability need coverage. |
| CX-09 | Token `hierarchical_memory`, `none` | `none` makes no policy reduction; hierarchical_memory is selectable but currently dispatches unchanged via wildcard. Three-tier claim cannot be accepted without implemented effects or an explicit unsupported disposition. | `src/uar/domain/context.rs:11`; `src/uar/runtime/context/manager.rs:119`; settings manager.rs:2191 | Known source capability gap for hierarchical_memory; no invented behavioral oracle. |
| CX-10 | Full request budget, scope and multi-call preparation | Each request counts/reserves instructions, tools, paired history, retrieved context, team context and output under the actual destination contract; limit admits, excess refuses before effect; selected policy remains effective after restart and scoped override. | `src/uar/runtime/context/budget.rs`; `tests/context_history_integrity.rs`; phase plan COUNT-01 | Existing controlled exact-count cases are fault controls only. Real destination count quality/capacity must be explicit; no unsupported exact-fit claim. |
| KB-01 | Create/upload/index/search/ground/delete | Through product entry create KB, ingest two documents with unique facts, observe successful indexing and positive ranked retrieval, bind to agent, obtain exact grounded fact, delete A and prove A absent while B remains; UI failure/retry honest. | `openspec/specs/knowledge-rag-product-certification/spec.md`; `openspec/specs/rag-provenance/spec.md` | `tests/knowledge_base_integration.rs`, `tests/principal_knowledge_namespace.rs`, phase `tests/post_lifecycle_main.rs` KB cases. Existing delete/conflict fixture is not whole journey proof. |
| KB-02 | Change embedding model / regenerate | Index with actual model A, select actual model B, observe explicit regeneration state/revision, rebuild all retained chunks, and regain ranked retrieval plus grounded real answer from original text. Incompatible old space cannot leak into B retrieval. | `tests/kb_embedding_space.rs`; `tests/embedding_lifecycle_failover.rs`; embedding source anchors below | Deterministic embedding-space test is not positive model-regeneration proof. Exact mounted model-change action, durable job/progress/error state and resume semantics remain to be mapped. |
| KB-03 | Dimension/revision mismatch and restart during regeneration | Different dimensions and equal-dimension incompatible models are distinct controls. No mixed-generation hits; index dimension/schema migration is explicit; interrupted rebuild resumes/completes after true process restart, old/new revision selection stays coherent, no lost source documents. | `tests/embedding_lifecycle_failover.rs:88,425`; `tests/kb_embedding_space.rs`; embedding source anchors below | Lifecycle fixture explicitly requires equal actual dimension. It proves failover contract only if run; it does not establish KB regeneration/index migration. Missing source-backed resume/migration behavior is a gap, not permission to invent implementation. |
| DB-01 | Postgres / Surreal same end-user outcomes | Run SK selection, scoped policy/history, KB grounding/model-change, and team journeys on each actual durable adapter. Same owner-visible records, effective behavior, terminal classification and restart results; wrong owner cannot read/write/infer private facts. | persistence providers; `tests/principal_knowledge_namespace.rs`; `tests/bauar_native_admission_storage_fixture.rs` | Need adapter-parameterized real mounted journey harness. SurrealKV same-handle reconstruction is not process restart; Postgres isolated CRUD does not certify parity. |
| TM-01 | Supported team execution ordering | For actual supported sequential/dependency and concurrent branches, children run with intended task/identity/model; dependent child sees predecessor outcome; independent branches both contribute exactly once to final answer. Observe actual child/provider calls and graph/task identities. | `src/uar/compiler/collaboration/team_execution/{dispatch,continuations,settlement}.rs`; `src/uar/domain/collaboration.rs` | `tests/integration/live/collaboration_conformance_cases.rs`; actual supported execution enum/reachable modes captured below. No arbitrary new execution-mode names or every-mode product. |
| TM-02 | Handoff, thread and multi-call context reassembly | Successor/child receives exact permitted parent/worker thread history, required instructions, authorized skill and KB context; returns across at least two real provider/tool rounds; final answer integrates independent facts. Other team's secret sentinel remains absent. | `src/uar/compiler/collaboration/team_execution/{context,context_evidence,ownership,peer,continuations}.rs`; `src/uar/domain/team_context.rs`; `src/uar/runtime/team_execution/host.rs` | Live collaboration source; inspect raw canonical assistant-call/tool-result ordering, thread IDs, durable result references, scope and budget. A final plausible answer alone is insufficient. |
| TM-03 | Supported retry/recovery/cancel and restart | Inject one controlled failure only; retry uses defined attempt identity and avoids duplicate completed effects; cancel settles honest terminal state without late final success; cold restart restores thread/context/accepted outcomes and resumes only allowed work. Actual successful branches use real configured inference. | `src/uar/compiler/collaboration/team_execution/{recovery,settlement,waits}.rs`; `src/uar/runtime/team_execution/{recovery,controller}.rs` | Existing live conformance source; tie retry/cancel to actual supported API transitions, not assumed user mode. Run same outcome predicates on both adapters; record unsupported persistence/recovery explicitly. |

## Gaps and scope decisions for the lead

The uncomfortable result is that selectable strategy names exceed demonstrated implemented effects. Token hierarchical_memory is a source-visible no-op; structural hierarchical effects and all classifier backend wiring still require bounded source mapping. This document does not establish every listed selection works, or every team mode is reachable. Exact team enum/dispatch excerpts below are raw discovery evidence to finish that mapping, not acceptance.

Before authoring the executable gate, bind each exposed classifier and structural strategy to the mounted settings/session/agent path; record precedence and any unsupported backend/build. Complete KB model-change/revision/index migration/resume and team ordering/handoff mode source mapping. Only then freeze fixture facts, model identities, adapter setup, scenario denominator, exact commands and bounded provider allowance in the canonical plan. Unknown implementation or missing harness remains pending. The current lock requires a renewed whole-phase inventory and explicit operator instruction before any verification; this inventory opens nothing.

## Source discovery excerpts

The following bounded line extracts are source inspection only. They retain actual spelling and line anchors to prevent guessed enum/mode or migration claims. They are not exhaustive source snapshots or runtime evidence.

### src/uar/domain/collaboration.rs

- `23`: pub enum CollaborationKind {
- `84`: pub revision: Option<u64>,
- `101`: pub struct PrivateRevisionRef {
- `103`: pub revision: u64,
- `111`: pub revision: u64,
- `117`: pub enum RepresentationGrantStatus {
- `128`: pub revision: u64,
- `175`: pub revision: u64,
- `241`: pub enum ConversionDisposition {
- `257`: pub effective_binding_ref: Option<PrivateRevisionRef>,
- `305`: pub revision: u64,
- `307`: pub binding_ref: PrivateRevisionRef,
- `313`: pub policy_revision: String,
- `409`: pub revision: u64,
- `427`: pub catalog_revision: u64,
- `428`: pub binding_revision: Option<u64>,
- `465`: pub catalog_revision: u64,
- `493`: pub expected_catalog_revision: Option<u64>,

### src/uar/runtime/context/budget.rs

- `53`: pub counting_revision: String,
- `58`: pub enum BudgetOutcome {
- `75`: revision: String,
- `81`: pub enum OutputCeilingField {
- `88`: pub enum WireContract {
- `148`: (count.revision.trim().is_empty()
- `156`: revision: count.revision.clone(),
- `262`: counting_revision: input.protected_request.revision.clone(),
- `280`: if final_request.revision != plan.counting_revision {
- `282`: subject: "final_serialized_request_revision".to_string(),
- `284`: revision: final_request.revision.clone(),
- `298`: final_plan.counting_revision = final_request.revision.clone();
- `310`: revision: "fixture-v1".into(),
- `386`: revision: "fallback".into(),

### tests/kb_embedding_space.rs

- `41`: dimension: usize,
- `45`: fn shared(model: &'static str, dimension: usize) -> Arc<dyn EmbeddingBackend> {
- `46`: Arc::new(Self { model, dimension })
- `50`: let mut vector = vec![0.0_f32; self.dimension];
- `61`: let slot = usize::try_from(hash % self.dimension as u64).unwrap_or(0);
- `74`: fn vector_dimension(&self) -> usize {
- `75`: self.dimension
- `215`: async fn ingest_records_the_embedding_space_on_the_kb() {
- `232`: async fn kb_search_from_another_embedding_space_is_an_explicit_conflict() {
- `248`: json["error"]["code"], "embedding_space_mismatch",
- `254`: async fn ingest_from_another_embedding_space_is_refused() {

### tests/integration/live/collaboration_conformance_cases.rs

- `29`: const SOURCE_REVISION: &str = "7a02a249396fd77f297cdb3f9672c4ca35341a63";
- `136`: assert_eq!(installed["receipt"]["catalogRevision"], 1);
- `190`: "grant revision one",
- `192`: json!({"commandId":"c03-grant-1","expectedRevision":0,"grant":grant_v1}),
- `200`: json!({"commandId":"c03-binding-1","expectedRevision":0,"binding":binding}),
- `224`: json!({"commandId":"c03-blocked-binding","expectedRevision":0,"binding":blocked_binding}),
- `281`: assert_eq!(grant_current["revision"], 1);
- `350`: "/policyRevision",
- `361`: "policyRevision",
- `396`: ("/policyRevision", &source_binding["policyRevision"]),
- `424`: stale_policy["revision"] = json!(2);
- `425`: stale_policy["policyRevision"] = Value::String("policy:c03:2".to_owned());
- `429`: "stale policy revision refusal",
- `431`: json!({"commandId":"c03-stale-policy","expectedRevision":0,"binding":stale_policy}),
- `435`: revoked["revision"] = json!(2);
- `438`: json!({"revision":2,"revokedAt":"2026-09-27T00:00:00Z","reason":"C03 stale grant fixture"});
- `441`: "grant revocation revision",
- `443`: json!({"commandId":"c03-grant-2","expectedRevision":1,"grant":revoked}),

