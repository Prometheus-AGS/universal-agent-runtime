# Lifecycle plan implementation and evidence checklist

Execute phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Source baseline: published budgeted API0f926ed6deee26c353278640e3d7c1f19982bc55;
prior embedding repair d35234ba132988e5d64b1e19aea688c7fa978690;
published image source32cb3bcfc9691212ec0b0359ce22474968bc6850 and
Boss a609869079bf6d51af196944965e6f86ee6c0758. Execution uses the saved repair
inventories at waypoint3184 / plan80 and principal repair3189 / plan82.
The latter records saved digest3902617d5158f3556f5206c69513022fc3783f95c193638764f0fcca951d878a.
This checklist is maintained from source
and existing receipts. It is not a new verification gate.

Task9.8 source completion is recorded at3196/plan86, inventory
`lifecycle-budget-source-inventory-2026-10-21.json`, digest
`251e9c10ee2a67d3f18164aeca6710a3136d46d2516e383da85ebd8a495ed0a6`.
It is published but unverified; older command receipts do not certify it.

Status meanings: **Implemented** means source and writer handoff exist;
**Verified** names actual command evidence and its narrow coverage;
**Unfinished** means a required deliverable or evidence is absent;
**Blocked** names a prerequisite or operator decision. Implementation checkboxes
do not certify runtime behavior. Canonical aggregate138/153 is not lifecycle
acceptance. General verification is CLOSED. The earlier approved embedding
command passed; the latest approval permits budgeted Pending source/API
implementation only. It authorizes no test or new gate, and the old pass does
not certify9.8. No passing command is rerun.

## Agreed implementation items

| Plan item | Implementation locations | State and available evidence |
| --- | --- | --- |
| WP1 / 1.1: resource inventory, ownership graph, admission, generations, outstanding IDs and engine closure events | `src/uar/runtime/lifecycle.rs`; `src/server/lifecycle_graph.rs`; `docs/operations/lifecycle-resource-inventory.md`; local SDK engine overlay | Implemented; runtime/resource handoffs collected. Published Linux build evidence only; shutdown transition and release evidence unfinished. |
| WP2 / 2.1: supervisor, scopes, registered tasks, retained outcomes, repeated/canceled shutdown waiters | `src/uar/runtime/lifecycle.rs`; `src/mcp/stdio_process.rs` | Implemented; runtime handoff. Owned-resource target authored but not run.9.8 adds caller-budgeted observation in source, not finite termination or execution proof. |
| WP2: weak schedule/follow-up facades and history producer/drain ordering | `src/uar/runtime/manager.rs`; runtime schedule/follow-up controllers; `src/server/lifecycle_graph.rs` | Implemented; runtime and producer-runtime handoffs. Accepted history drain and cycle-release execution evidence unfinished. |
| WP3 / 3.1: common endpoint, physical directory collision rejection, owned/borrowed persistence and memory | `src/uar/persistence/providers/{surreal,postgres}.rs`; `src/uar/persistence/scoped.rs`; `src/uar/security/credentials/maintenance.rs`; `src/uar/memory/{service,scoped_storage}.rs`; `src/server.rs`; `src/embedded.rs` | Implemented; state handoff. Physical release/reopen, borrowed usability and alias rejection evidence unfinished. |
| WP4 / 4.1: SDK sessions, workers, delivery/index/maintenance, explicit engine close, remote transport/session identity and awaited subscription close | `vendor/patches/surrealdb-lifecycle/`; `vendor/patches/surrealdb-lifecycle/README.md` | Implemented source overlay; state/trust/gRPC handoffs and provenance. Published Linux compilation is evidence for32cb3bcf only. All-engine/transport/physical-close acceptance unfinished. |
| WP4: first-party externally managed memory and pinned export | `vendor/git/surreal-memory-server/`; `vendor/git/README.md`; `src/uar/memory/service.rs` | Implemented export pinned9e36b1cd8b83a657366eb1e3ef8cd837178336de; upstream commits not pushed. Restricted export untouched by current repair. Memory lifecycle execution evidence unfinished. |
| WP4: SQLx pool/listener closure after users | `src/uar/persistence/providers/postgres.rs`; `src/uar/realtime/postgres_bus.rs`; composition graph | Implemented; state handoff. Real PostgreSQL close and transport-gap evidence unfinished. |
| WP5 / 5.1: startup dependency readiness, acquired-prefix unwind, all exit paths, independent cleanup and preserved errors | `src/server.rs`; `src/server/lifecycle_graph.rs`; `src/embedded.rs`; `src/bin/uar-sidecar.rs` | Implemented; startup and lead handoffs. Source compilation/delivery is partial evidence. Startup-failure/panic execution evidence unfinished. |
| WP5 / 5.2: standalone, service, OS signal, telemetry ownership | `src/main.rs`; `src/server/{windows_service,process_signals}.rs`; `src/bin/uar-sidecar.rs` | Implemented; entrypoint handoff. Windows/service/signal/telemetry cleanup acceptance unfinished. |
| WP5 / 5.3: watchers stop and join before dependents | `src/config_manager.rs`; `src/uar/rag/ingest.rs`; `src/uar/runtime/skills/watcher.rs`; `src/server/lifecycle_graph.rs` | Implemented; watcher-order handoff. Native watcher and accepted reload/refresh joins remain unverified. Optional skill watcher is not instantiated by current server composition. |
| WP5 / 5.4: retained blocking descendants and background producers | `src/uar/runtime/{lifecycle,manager}.rs`; `src/uar/runtime/thread/{graph_host,service}.rs`; `src/uar/runtime/team_execution/controller.rs`; `src/uar/rag/ingestion_worker.rs`; `src/mcp/{registry,runtime,interaction,binding_cache}.rs`; `src/sandbox/session_manager.rs` | Implemented; producer runtime/state/trust and sandbox handoffs. Cancellation/panic/unknown-effect joins remain unverified. |
| WP6 / 6.1: subscription EOF/lag, explicit reconnect, subscribe-before-snapshot, schedules and committed wakeups | `src/uar/realtime/`; `src/uar/api/live.rs`; `src/server/{lifecycle_reconnect,lifecycle_snapshot}.rs`; `src/uar/memory/background.rs`; runtime observer/retention controllers | Implemented; state/reconnect handoffs. EOF/gap/reconciliation and business-time execution evidence unfinished. |
| WP6: SSE consumers and typed entity graph ingestion | `frontend/src/lib/realtime/{sse-stream,uar-sse-adapter}.ts`; `frontend/src/entities/sync.ts`; `src/server/lifecycle_snapshot.rs` | Implemented; console handoff. No frontend check or browser evidence. No permission to add one. |
| WP6: streaming-only authenticated A2A peers | `src/uar/api/a2a/{client,client_stream,task_execution,thread_service,v1,grpc_v1}.rs`; `src/uar/runtime/thread/service.rs` | Implemented; A2A handoff. Unsupported-peer refusal and stream-loss behavior unverified. Operator approved compatibility change. |
| WP7 / 7.1: independent authorization, same-runtime joining, fingerprints, atomic effect/receipt, typed replay/conflict/unavailable/unknown | `src/uar/security/credentials/{migration,mod,principal_store,resolver}.rs`; `src/uar/api/credentials.rs` | Implemented; trust/producer-trust handoffs. One authoritative contention reconciliation read; no poll loop. Concurrent migration and lost-commit evidence unfinished; historical503 body still missing. |
| WP8 / 8.1: immutable fixtures, final PostgreSQL readiness, hermetic skill roots, owned process/control readiness | `scripts/release/uar_1_1_1_{support,credentials}.py`; `tests/support/sidecar_process.rs`; `tests/integration/live/harness.rs`; `src/bin/uar-env-probe.rs` | Implemented; trust handoff. No fresh fixture or sidecar execution result. Old c9 individual failures remain unresolved historically. |
| WP8: Windows cancellable inherited input and launcher | `src/bin/uar-sidecar/{inherited_input,control}.rs`; Boss `src/main/ai/runtime/uar/UarSidecarService.ts`; Windows branch in sidecar process support | Implemented; operator chose named pipe and launcher change; trust/Boss handoffs. Windows acceptance BLOCKED: no approved exact Windows packet. |
| 6.2: local-only compatible embedding live failover, retained inference/control, recovery and native engine release | `src/config.rs`; `src/uar/api/settings.rs`; `src/uar/rag/embeddings/{fastembed,failover,swappable}.rs`; `src/server.rs` | Implemented. VERIFIED direct-controller integration path in attempt3: real inference, generations1→2→3→4→5, durable ingestion/query and successful fixture closure. Whole-server wiring, HTTP controls and separate MemoryService provider remain unverified. |
| 8.2: real incompatible equal-dimension model fixture and durable generation continuity | `tests/embedding_lifecycle_failover.rs`; `tests/fixtures/local_embedding_lifecycle/` | VERIFIED scoped target: asset preparation exit0; embedding attempt3 exit0,1 passed/0 failed/0 ignored in4.35s. Actual incompatible384-dimensional inference,409 refusal, preserved binding/documents and compatible re-query passed. Broader frozen contract remains unfinished. |
| 8.3 / 8.4 / 8.5: immutable Boss candidate binding and refreshed source pin | `scripts/build-boss-candidate.mjs`; Boss `scripts/{uar-candidate-binding,local-uar-payload,prepare-local-uar-payload}.cjs`; Boss `build/local-uar-source.json` | Implemented and published for32cb3bcf. Repair d35234ba and budgeted API0f926ed6 are published; Boss pin predates both. Matching candidate/package remain BLOCKED under current scope; do not relabel an old candidate. |
| 9.3: scoped commit/push, draft PRs and usable Docker delivery | Draft UAR PR418; Boss PR74; `lifecycle-image-delivery-39532243053.md` | VERIFIED delivery at32cb3bcf: Linux AMD64/ARM64 compilation/publication succeeded; combined digest8e6fb6f4…bc1fc8. Image excludes published d35234ba repairs and current9.8 work; it certifies no runtime acceptance. No repeat dispatch. |
| 9.4 / 9.5: delivery compiler and startup Send repairs | Delivery runtime/state/MCP/startup repair handoffs | Implemented; successful later Docker delivery verifies release compilation of that source on both Linux architectures. Native/runtime contracts remain unverified. |
| 9.6: embedding digest/compiler-diagnostic source repairs | `tests/embedding_lifecycle_failover.rs`; exact paths in approved-gate state/runtime/lead handoffs | Implemented at3184 / plan80. VERIFIED compilation in attempts2/3; digest failure resolved. One nonfatal SDK import warning remains; no zero-warning or feature-matrix claim. |
| 9.7: observed fixture principal rejection | `tests/embedding_lifecycle_failover.rs`; `lifecycle-approved-gate-principal-repair-2026-10-21.md` | Implemented at3189 / plan82. VERIFIED by attempt3 passing actual principal-bound storage/search; no production guard changed. |
| 9.8: caller-budgeted Pending shutdown observation | `src/uar/runtime/lifecycle.rs`; `src/embedded.rs`; runtime resource lifecycle spec; `lifecycle-budgeted-shutdown.md` | IMPLEMENTED source; runtime and host handoffs collected. UUID identity, retained initiation before await, explicit unknown inventory/current scope snapshots and observer-only budget. No test or compilation evidence; prior embedding pass predates this change. |
| Delivery order, team ownership and provenance docs | Canonical KBD phase/child ledgers; collected source handoffs; SDK and memory provenance READMEs | Implemented source coordination; generated state updated through supported interfaces. Canonical release/preparation/reflection tasks remain unfinished. |

Relative paths above resolve from the UAR repository, except paths explicitly
marked Boss, which resolve from `/Volumes/T7/Projects/prometheus/the-boss`.
Detailed sources and receipts are linked by the implementation and repair
inventories; historical handoffs retain the verification state at authoring.

## Producer ownership and shutdown ordering

| Covered producer | Owner / stop / join implementation | Evidence and limit |
| --- | --- | --- |
| Configuration filesystem and explicit reload | ConfigManager watcher scope; close admission, cancellation, actual watcher-task/reload completion | Source only; first DAG close stage after external admission closes. Host9.8 observation may return Pending; actual watcher join remains required. |
| Ingestion filesystem | IngestService watcher task and registration readiness; stop-biased event handling; accepted ingestion awaited; native handle released | Source only; second close stage. Native/file operation can remain pending. |
| Optional skill watcher | Scope-owned event task; receiver closes before native watcher drops, releasing blocked callback | Source only; server does not instantiate compatibility API. |
| History, retention, auto-capture, response quality, skill evolution, disconnect work, model streams | RunManager scopes/retained handles; producers finish before history admission closes and receiver drains | Source only; last persistence-dependent finalizer precedes store close. |
| Observer, schedule, follow-up | Host-owned controllers, weak tool facades, observer scope; committed activity and one-shot due events; canceled worker joined | Source only; no repeated status scan as readiness proof. |
| Ingestion extraction/blocking jobs | Ingestion pool scope; task-local scope restored; blocking worker retains lease through actual return | Source only; cancellation cannot terminate uncooperative blocking code. |
| A2A/actors/graph roots/team work | Root/job registries and child scope; cancellation, retained handles and acknowledged terminal writes | Source only; streaming required before dispatch; unknown remote outcome remains unknown. |
| MCP / projected bindings / tools / subprocesses | Registry admission and retained slots; selected-service close; actual child wait/reaping; explicit force-stop receipt | Source only; signal or budget expiry alone is not exit. Uncancellable tool/handshake can remain pending. |
| Sandboxes / terminals | SessionManager scopes, busy leases, retained finalization and backend handle quarantine | Source only; unknown remote effects reserve capacity and block unsafe reuse. |
| Persistence / memory operations and subscriptions | Scoped retained dispatch, original-session stream close, worker joins, local registration removal | Source only; canceled waiter cannot drop the accepted database future. |
| SDK engine workers / maintenance / indexes / transaction leases | SDK overlay tracked drivers and workers; close admission, session drain, store close and physical destruction acknowledgements | Linux compilation only; physical release unverified. |
| HTTP / SSE / AG-UI / harness / A2A gRPC | External intake admission closes first; stream leases and serving tasks remain until actual completion | Source only; dependent services retained until closure receipt. |
| Sidecar input, OS signals, telemetry and fixture readers | Entry-point/input scope and real process handles; join input/reader before exit; signals kept until graph ends; telemetry result retained | Source only for shutdown; no Windows/native process acceptance. |

## Explicit unfinished or blocked contracts

- **Finite shutdown observation — IMPLEMENTED, UNVERIFIED.** The operator
  approved budgeted Pending reporting with retained cleanup. Task9.8 adds
  `shutdown_with_budget(Duration)` to supervisor, handle and embedded runtime.
  One UUID identifies the same retained owner; initiation registers it before
  an await. Expiry returns Pending with SnapshotPending inventory or current
  unreleased-resource snapshots; it does not cancel the owner or release a
  dependency. Completed carries the actual report and may still have
  `released=false`. The embedded wrapper first cancels its existing root token.
  Existing unbounded shutdown still awaits the same result. Runtime and lead
  handoffs are collected. No budgeted-shutdown compilation/execution result
  exists; no new test or verification gate was authorized. The host must keep
  its executor alive, and uncooperative work can remain pending indefinitely.
- **Exact native source binding — BLOCKED.** The approved candidate command
  names32cb3bcf; repair d35234ba and API0f926ed6 are published, but Boss still pins the
  old source. Publishing/rebinding repaired source does not silently authorize
  a changed build command. No old candidate may be called repaired-source proof.
- **Full release evidence — UNFINISHED.** Canonical child4.1 same-image
  supervisor preparation,4.2 independent artifact review,4.3 functional
  acceptance and5.1 reflection/handoff remain pending. The stale full-runner
  manifest cannot be reused as current evidence.
- **Frozen embedding contract — UNFINISHED.** Process restart/settings
  hydration, invalid authenticated setting mutations and every frozen condition
  retain their evidence requirements. A real-model target pass alone does not
  complete them.
- **Historical causes — UNRESOLVED.** The concurrent migration503 body and
  individual c9 sidecar traces remain missing. Source contracts do not recover
  those facts.
- **Verification intentionally not performed.** No additional check, formatter,
  lint, typecheck, feature matrix, review, Windows command, native package,
  public Boss package or full release runner is introduced. Other approved ready
  commands are held under the latest direction to rerun only embedding.

Actual command outcomes belong in
`lifecycle-verification-results-2026-10-21.md` and the per-attempt logs. This
Embedding attempt2 settled exit101: compilation completed in14m32s, then the
single test failed in3.67s at owner-scoped knowledge-base creation. The prior
digest errors are resolved; the fixture owner is rejected by the verified
principal boundary. The targeted fixture-only repair is source-complete at3189,
with no production guard change or new test. Attempt3 subsequently PASSED exit0:
1 passed/0 failed/0 ignored, compilation40.81s and test4.35s. Its cleanup
assertion requires supervisor release and persistence engine-close
acknowledgement; it does not exercise immediate reopen or full-server teardown.
Captured output and all actual commands are in the results document and logs.
No other gate ran and no passing command was repeated. Approved9.8 production
edits now postdate that pass and have no execution receipt. The finite-budget
decision is resolved; its source implementation is complete and unverified. Other acceptance and
source-binding obligations above remain open. This checklist is not a claim
that the full objective is complete.

Publication addendum: the completed repair source and receipts were committed
and pushed as d35234ba132988e5d64b1e19aea688c7fa978690 to the existing PR418
branch under the earlier source-only hook exception. No new source edit or
verification accompanied publication. The passing saved worktree includes
pre-existing seed-tool changes excluded from the scoped commit, so it is not
claimed as clean-checkout proof for that commit. See
`lifecycle-approved-embedding-repair-publication-2026-10-21.md`.

Budgeted API publication:0f926ed6 is pushed to PR418. Exact source-only commit
and push receipts are in `lifecycle-budget-publication-2026-10-21.md`. No new
verification ran; its task/source completion does not complete whole-phase
acceptance. The former shutdown-budget design blocker is resolved in source;
the current limitation is missing authorized execution evidence.
