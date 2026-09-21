ASSESSMENT: runtime-harness-gap-closure
Project: universal-agent-runtime
Date: 2026-09-16
Codebase baseline: 226d4a0af89811975662cf3f203c699f70e8cebc; existing dirty checkout preserved.
Cross-tool progress: no changes or implementation tasks registered in this phase; 0/0.

The operator's supplied feedback is preserved in `feedback-source.txt` beside
this assessment. Phase creation verified it byte-for-byte against the attachment;
the assessment-stage evidence inventory retains its SHA-256 digest.
The preserved file is
`.kbd-orchestrator/phases/runtime-harness-gap-closure/feedback-source.txt`;
`evidence/source-inventory.json` records its SHA-256 as
`5a478d7a091cd0e337ab64b15cb41b17cf869adf09f51ea07f07b75ef65f4efe`
and the successful byte comparison against the supplied attachment.

## Assessment result

The feedback identifies useful outcomes, but several absence claims describe an older runtime. The current source already integrates capability routing, prompt dialects, paired-history reduction, explicit skill activation, graph progress events, and A2A execution through the shared thread host. None of those findings establishes full behavioral conformance.

The most concrete remaining gaps are destination-specific failover preparation, complete request budget accounting, durable A2A task correlation, tool-inclusive evaluation, and knowledge verification beyond lexical overlap. Existing activation mechanics need an outcome-quality benchmark, not replacement with another registry. These are assessment findings and inputs to analysis; this document does not authorize a particular design or mark implementation complete.

Evidence terminology: **source-confirmed** means the cited implementation was inspected; **historical** means an earlier retained report, not a rerun; **unverified** means no current behavioral evidence. No runtime defect was dynamically reproduced during this assessment. A source-confirmed mechanism is not a reproduced provider failure.

## IMPLEMENTATION STATUS — feedback-to-source-to-test matrix

| Feedback | Status / classification | Current production evidence | Remaining gap and acceptance evidence |
|---|---|---|---|
| F1 dynamic routing | PARTIAL; absence claim superseded, integration insufficiently verified | `src/llm/router.rs:39` implements capability/health selection; `src/uar/runtime/manager.rs:3161` calls it at run start. Cost/context/benchmark ordering exists at `src/llm/router.rs:171`. | Ordinary run-start builds default requirements except preferred provider (`manager.rs:3162`); no task classifier feeds that call. `src/uar/llm/router.rs:28` is a separate heuristic whose identifiers have no callers in the searched runtime/LLM source. Require task-to-constraints-to-provider tests, hard budget/latency and policy evidence, and quality/cost/latency comparison. |
| F2 per-model dialects | PARTIAL; absence claim superseded; failover preparation gap source-confirmed | `src/llm/orchestrator.rs:1112` computes dialect parameters from the primary model; `src/llm/liter_driver.rs:117` forwards them as extra request body. `manager.rs:3427` selects prompt rendering options. | Fallback dispatch clones the primary request (`orchestrator.rs:1323`) without rebuilding dialect, prompt, or context for the destination at that site. Require cross-family request-capture tests, smaller-window failover, explicit-model and descriptor-override coverage. Provider acceptance and current parameter validity remain unverified. |
| F3 history correctness and model-aware context | PARTIAL; reported pairing/empty-text defects substantially superseded by source; budget gaps remain | Typed assembly calls the shared reducer (`src/uar/runtime/turn/builtin.rs:246`); legacy setup does likewise (`manager.rs:3533`). `context/reduce.rs:178` normalizes before reduction and :195 removes severed groups. `orchestrator.rs:1090` normalizes at dispatch. `manager.rs:4711` records assistant tool calls even with empty text when results arrive. | Do not patch the old manager loop in isolation. Budget retention counts text plus overhead (`context/manager.rs:196`) while whole-history counting also includes tool arguments (`context/token_service.rs:116`). Multimodal content becomes empty text at :114. The reducer reserves a fixed 1,000 tokens (:151) and has no final whole-request budget rejection at its return. Need oversized complete-group, schemas/multimodal/output-reserve, interruption-before-result, persistence/reload, and long-conversation retention evidence. |
| F4 AG-UI/A2A integration | PARTIAL; separate-execution claim superseded; durable transport correlation missing in inspected adapter | `src/uar/api/a2a/handler.rs:25` shares `A2AThreadService` across JSON-RPC/gRPC; the service starts actor sessions (:335), submits prompts (:412), and exposes run IDs (:425). | `thread_service.rs:37` stores task/context bindings in HashMaps; :247 constructs empty bindings and :438 resolves tasks only there. Durable underlying threads do not restore this task lookup. Require restart/reconnect tests for the same task/context ID, artifact/status recovery, authorization, cancellation, and no duplicate execution. |
| F5 activation-centric skills | PARTIAL; storage-only claim superseded; comparative activation quality unverified | Runtime ranks and records candidates (`manager.rs:2833`), applies activation modes (:2863), freezes shadow top-ten candidates (`skills/activation.rs:341`), and records outcomes (:349). Explicit attachments, model activation and reattachment are covered by existing tests. | The targeted eval uses five fixture skills and returns matcher IDs (`src/uar/eval/targeted.rs:67`, :99). It does not measure end-task success or model selection under long contexts. Establish a labeled, versioned evaluation set, false-activation/recall/precision and task-success metrics. Preserve the canonical 99% Recall@10 threshold before candidate omission; freeze additional thresholds before implementation. |
| F6 orchestration and graph observability | PARTIAL; graph-event absence superseded; richer correlation/behavior unverified | `src/uar/runtime/graph/engine.rs:104` and :114 emit started/finished steps; the engine limits 1,000 iterations (:17). The direct tool loop still caps at ten (`orchestrator.rs:174`). Shared child-thread policies, budgets, and lifecycle are specified and implemented in `runtime/thread/`. | `src/uar/domain/events.rs:133` carries run ID, step and kind, not node identity; graph model wrappers deliberately defer step numbering to the engine (`graph/turn.rs:206`). Verify correlation across existing trace/lifecycle events before extending contracts. Require approval, exhaustion, error, cancellation/drain, resume and exactly-once terminal scenarios across graph/linear/child paths. |
| F7 knowledge runtime | PARTIAL; disconnected retrieval claim superseded; full evidence reasoning missing in inspected pipeline | Chat uses owner-scoped KB resolution and `RagRetrievalPipeline` (`manager.rs:2651`, :2688), renders retrieved-authority fragments (:2715), and emits citations (:2727). Pipeline decomposition, dedup, verification and audit are present (`rag/pipeline.rs:112`, :155). | Verification is content-term overlap, explicitly not fact cross-referencing (`rag/verification.rs:3`). Default filtering is annotation-only (`rag/pipeline.rs:39`); returned `KnowledgeMatch` values do not carry the internal verdict. Require unsupported/contradictory/stale-source behavior, revocation and tenant tests, evidence-to-answer traceability, and prompt-injection boundary tests. Do not equate a citation or lexical overlap with factual support. |
| F8 verification debt (including F6 tool-inclusive evals) | PARTIAL; completion pipeline exists; agentic CLI path missing; current health/coverage incomplete | `src/uar/eval/cli.rs:32` calls `chat_non_streaming`; :153 builds empty MCP/native registries. Targeted suites call decision helpers (:140), not a governed multi-step host. Existing runner/integration code exercises load/score/persist/compare. | Add a separately specified local production-host eval mode with deterministic tools, event traces, seeded regressions and baseline gating. Historical 33.68% lines and two routing-eval failures are not current measurements. Reproduce/reconcile them with exact test identities and coverage denominators before declaring debt closed. |

### Concrete findings and falsifiers

**A1 — High: primary-prepared requests cross failover boundaries unchanged (F1/F2/F3).**
The fallback loop sends `req.clone()` to a different driver; the dialect parameters were computed from the primary configuration. The existing `cache_strategy_is_preserved_on_failover_request` test at `src/llm/orchestrator.rs:2382` asserts cache preservation, not destination dialect or budget correctness. A capture test showing destination-aware adaptation before the fallback driver receives the request would disconfirm the broader integration concern. The clone itself is observed; an upstream rejection has not been reproduced.

**A2 — High: context accounting is not a full request budget (F3).**
The retention walk and whole-history counter use different fields; the latter counts tool arguments while the former does not. Both omit multimodal payload accounting through `as_text().unwrap_or("")`. The request's tool schemas are assembled later (`src/uar/runtime/turn/resolved.rs:154`), and the reducer returns messages without checking the complete serialized provider request. SlidingWindow retains oversized system content (`src/uar/runtime/context/manager.rs:120`); KeepFirstLast and progressive summarization instead condition retention on `t < budget` (:182 and :240), so required system content can be omitted. This is a source-level mismatch with the system-preservation contract, not a reproduced end-to-end failure. Reproduce a retained large complete tool group and an oversized required system/schema set across all strategies before changing behavior. A bounded complete outbound request that also retains mandatory instructions under those fixtures would narrow this finding. Model-specific chunk/placement benefits remain unmeasured.

**A3 — High: A2A correlation is transient even though execution is shared (F4).**
A fresh `A2AThreadService` initializes empty bindings; `get` reads only those maps. This is a restart-recovery gap in the inspected path, not a claim that thread persistence is absent. Existing `tests/a2a_thread_service.rs:214` and :276 cover send/get and cancel within a service lifetime. Recovery using the original task ID after replacing the service would falsify the inferred restart failure.

**A4 — High: evaluations cannot currently certify agentic runtime behavior (F6/F8).**
The CLI completion path has no registered tools and uses a non-streaming completion. `ContextEfficiencyProvider` returns a strategy description (`targeted.rs:206`), not a measurement of retained facts or context efficiency. Existing helper suites are useful and should remain. They cannot satisfy tool selection, approval, graph execution, cancellation, or task-success criteria without a new host-level evaluation contract.

**A5 — Medium: knowledge evidence verdicts stop at retrieval diagnostics (F7).**
The pipeline defaults to returning uncorroborated results and logs counts; its internal verdict is not part of the returned match contract. This behavior aligns with today's annotation-only implementation, so stronger answer-grounding behavior requires a spec delta. Whether the right mechanism is deterministic checks, model-assisted verification, or another method is an analysis decision, not settled here.

**A6 — Medium: activation quality and graph correlation need evidence, not presumed missing modules (F5/F6).**
Existing skill tests cover disabled/missing admission, threshold/mode behavior, limits, compaction and attribution. There is no representative end-task activation-quality result in the inspected evidence. Existing graph steps omit a node field; determine whether available lifecycle/trace records already meet the requested inspectability before adding another event family.

## SPEC GAP SUMMARY

The relevant canonical contracts inspected were `conversation-history-integrity`,
`model-path-resiliency`, `prompt-assembly`, `agent-thread-kernel`,
`multi-agent-orchestration`, `skill-activation-runtime`, `rag-provenance`,
`knowledge-rag-product-certification`, and `eval-harness`. This is a scoped
assessment, not conformance certification of every unrelated repository spec.

- Pair preservation, system pinning, explicit activation, shared threads and graph progress already have contracts. Preserve these and add only missing scenarios after reproductions.
- Destination-specific dialect/context failover, complete outbound budget accounting, durable A2A correlation, representative activation evaluation, and answer-support verification need explicit requirements or extensions before implementation.
- `openspec/specs/eval-harness/spec.md` still requires a “Starter suite and two-tier CI gate” and scheduled strict-baseline workflow. This conflicts with AGENTS.md. The next spec/plan must replace these with local deterministic and live-provider gates. Do not restore historical eval-nightly workflows.
- Each resulting implementation change must contain at least one spec delta. This assessment creates no implementation change and does not invent change IDs.
- Supported-profile certification is a Tier 3 milestone activity under the current Rust rules, while goals.md requires affected-profile evidence before phase completion. Analysis must reconcile this contract conflict before planning, for example by defining an explicit milestone verification boundary. This assessment neither changes the goal nor accepts a deferral. Do not silently run Tier 3 during this stage or claim server-full covers other products.

## CROSS-TOOL PROGRESS AND PRIOR EVIDENCE

Phase `progress.json` has zero changes and zero implementation tasks. No concurrent
implementation is credited to this phase. Canonical stage entry was recorded at
revision 2486. Project-wide 115/123 counters and inherited zero-length-Ping
summaries do not describe this phase's completion.

The checkout was already dirty in tool skills, KBD projections, memory and other
areas. `evidence/baseline.json` retains the complete observed status and submodule
inventory. The initial scoped `git status --short -- src tests Cargo.toml
Cargo.lock frontend/package.json frontend/vitest.config.ts` returned no entries.
Nothing from the existing work is reverted or certified by this assessment.

Reconciliation:
- The prior `handle-readiness-diagnostic-zero-length-ping` child under `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/` has no `reflection.md` at assessment time. Its readiness-diagnostic reflection remains pending. This assessment and the creation of this new phase do not complete it.
- `uar-next-harness/reflection.md` records routing/dialect/context/knowledge work and historical integration misses. It also carries old CI follow-ups, which are superseded by the deployment-only policy.
- `uar-harness-parity/reflection.md:15` records skipped RuntimeStep work in June. Current graph and tool-loop emission supersedes that absence claim.
- `eval-harness-hardening/reflection.md` records completion-provider pipeline tests; its claimed CI design is historical. Three targeted baseline files exist under `evals/results/`; this does not establish a governed agentic baseline.
- The September 4 `codex-harness-comparative-analysis/reflection.md:11` records that the first green suite missed five host-path behaviors. It reports 713 library tests, 94 broad integration tests, nine BDD scenarios and 26 doctests after corrections, with ignored tests, excluded persistence variants, and deferred real-provider 429 evidence. These are historical report counts, not tests rerun here.
- Some files named `verification.md` in the parent phase are unchecked task-list mirrors. Their existence is not proof of execution.
- `.prometheus/session-log.md:263` records historical 33.68% line coverage; :1009 and :1044 record two routing-eval failures in older runs. A later reported full pass does not substitute for current reproduction or identify which historical failures were resolved.

## BUILD HEALTH AND TEST COVERAGE

- Rust build health: **UNKNOWN** — `cargo check --locked --no-default-features --features server-full` was interrupted after 34 minutes 41 seconds without completing. Exit 143 reflects this assessment's SIGTERM, not an observed compiler defect. The log had reached the runtime crate's compiler process without reporting an error. Retained evidence: `evidence/cargo-check.log`, `evidence/cargo-check.exit`, and `evidence/cargo-check-interruption.json`. A completed check is still required before implementation claims compile health.
- Workflow-policy check: **PASS** — `pnpm github-actions-policy:validate` exited 0 and printed `GitHub Actions policy validation passed (deployment workflows only; Pages publisher: docs.yml).`
- Frontend type/build/runtime health: **UNKNOWN** in this assessment. No frontend source changed, and no frontend suite was executed.
- Test coverage: **PARTIAL** by inspected scenario presence; current numerical coverage **UNKNOWN**. `frontend/vitest.config.ts:30` includes src TypeScript/TSX and :32 sets all four coverage thresholds to 60. No fresh retained coverage measurement is claimed.
- Runtime reproductions and current eval failures: **UNKNOWN**; no Tier 1 or Tier 2 test suite ran. Test inventory below indicates scenarios available for later execution, not passes.
- Profile inventory from `Cargo.toml:137`: default/minimal, server-full, desktop-full, embedded-mobile. Embedded-mobile uses host-provided persistence/inference and excludes server transports. HTTP/A2A scenarios are not automatically applicable to it. Linux/macOS are Stable and Windows Experimental under the Rust rules. Only the server-full compile is attempted here.

Existing behavioral test inventory:
- `tests/context_history_integrity.rs`: normalization (:81/:135), direct dispatch (:198), iterative tool pairing (:253), pinned/repeated history (:382/:420), severed-group removal (:467), truncation (:508), tokenizer selection (:666), checkpoint restoration (:760/:809).
- `tests/skill_activation_runtime.rs`: catalog pressure (:282/:339), explicit/model activation (:370/:435), limits (:570), threshold/mode (:624), compaction (:676), attribution (:752).
- `tests/a2a_thread_service.rs`: persisted-thread projection (:214), cancellation (:276), unauthenticated endpoint rejection (:343).
- `src/uar/eval/integration_tests.rs`: deterministic suite/runner/persistence/baseline pipeline; `targeted.rs`: helper decision suites.
- `src/uar/rag/pipeline.rs:389`: retrieval decision audit test; the gotchas log records a previous tracing-capture race, so isolated success must not conceal a full-suite failure.

## CONSTRAINT CHECK

No application, UI, dependency, deployment, or workflow changes were made. No
guards, retries or fallbacks were introduced. Analysis must keep mutation authority
in trusted hosts and retain tenant scoping and deny-final governance. UI quality
routing applies if a later implementation actually changes UI; none is evaluated
or changed here.

Known contract conflict: the eval spec's CI instructions, described above.
The project `.kbd-orchestrator/constraints.md` is absent, as checked in the evidence
inventory. No broader architecture-compliance or security audit is claimed.
The workspace-info MCP tool is unavailable in this session; assessment is scoped
to the current repository and its recorded submodule pins.

The uncomfortable thing: current source can look substantially more complete than
the feedback and still fail a long-running cross-model tool workflow. Helper tests,
historical green summaries, and generated status counters can all miss that boundary.
The new phase needs attributable production-path evidence, not a replacement
implementation based on stale absence claims.

## GOAL PROGRESS

| Goal entry in goals.md | Status | Assessment |
|---|---|---|
| 1 purpose/scope | MET for assessment | All F1–F8 reviewed; no completion percentage asserted. |
| 2 baseline/matrix | MET for this stage | Source/test matrix and historical reconciliation recorded; runtime reproductions explicitly pending. |
| 3 F3 P0 correctness | PARTIAL | Existing pair repair and empty-text tool recording found; budget/interruption scenarios remain. |
| 4 F1 routing | PARTIAL | Router integrated; task constraints and outcome evaluations incomplete. |
| 5 F2 dialects | PARTIAL | Primary adaptation exists; destination failover evidence absent. |
| 6 F3 model context | PARTIAL | Existing model-keyed reduction; full-request and smaller-target budgets remain. |
| 7 F4 unified lifecycle | PARTIAL | Shared execution exists; durable A2A lookup/replay unproven. |
| 8 F5 activation | PARTIAL | Activation mechanics exist; representative quality metrics absent. |
| 9 F6 orchestration | PARTIAL | Shared threads and graph steps exist; requested behavioral/correlation matrix incomplete. |
| 10 tool-inclusive evals | NOT MET | Current CLI is completion/helper oriented. |
| 11 F7 knowledge runtime | PARTIAL | Governed retrieval/provenance exists; stronger evidence verification absent. |
| 12 F8 verification debt | NOT MET | Fresh failure/coverage/profile evidence still needed at appropriate tiers. |
| 13 delivery contracts | PARTIAL | Assessment findings available; analysis/spec/plan not performed. |
| 14 verification policy | MET for assessment only | Local-only checks respected; independent source review completed; cross-model review passed with two documented warning dispositions. Interrupted compile explicitly leaves build health UNKNOWN. Phase-level verification remains outstanding. |

Next stage: `/kbd-analyze runtime-harness-gap-closure` to resolve the bounded
design questions, verify provider-specific documentation, and settle the profile
verification scope before spec/plan. No implementation should start from the
historical report alone.

## Assessment verification and review receipt

Tier 0 artifact checks verified all eight feedback IDs, valid evidence JSON,
whitespace, the unchanged hashes of 28 inspected files, and byte-identical
preservation of the operator feedback. `git diff --check` returned no findings.
The local workflow-policy command and its exact successful output are recorded
above. The attempted Rust Tier 0 check exited 143 after intentional interruption;
it is not a pass. No Tier 1, Tier 2, or Tier 3 runtime checks were run.

The isolated source critic confirmed the corrected strategy-specific system
retention finding and the unresolved profile-verification contract conflict.
The first cross-model review returned PASS, zero critical findings, and two
warnings. The feedback-preservation warning is resolved by the explicit source
statement and digest. The pending-build warning is resolved as a reporting issue
by the observed interruption receipt; the underlying build-health limitation
remains open. Review artifacts are retained under `review/assess/`. Producer
identity is known only at the GPT-6 family level; the GPT-5.5 review is distinct
at that level, not independently verified against an exact deployment ID.

The second and final cross-model review also returned PASS with zero critical
findings and two warnings. The pending readiness reflection is now explicitly
reconciled above. The feedback path and digest are now in this report as well as
the evidence inventory. The review packet did not expose the preserved feedback
file to the judge, so that packet-completeness limitation is carried into the
handoff rather than misrepresented as independent verification of the bytes.
The local byte comparison passed. Both judge reports passed the anti-theater
screen with score 0.0 at strict strictness.

Changed assessment artifacts: `assessment.md` contains the findings;
`prior-context.md` records recalled context; `evidence/` contains source and check
receipts; `review/assess/` and `sycophancy/` retain independent review evidence.
The typed KBD transition and `handoffs/assess.handoff.json` record stage completion
and the next command. Existing phase goals and feedback are retained unchanged.
No unrequested product code or guards were added. Runtime reproductions,
numerical coverage, completed compile health, live-provider behavior, and broader
profile certification remain unverified for the reasons recorded above.

ASSESSMENT COMPLETE — source assessment only; implementation phase remains open.
