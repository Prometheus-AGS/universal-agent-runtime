# Analysis — agent-team-concurrency-repair

2026-10-26. Stack specified: Rust/Tokio, pinned patched SurrealDB3.3.0, existing UAR team execution and portable Node/Cucumber BDD. Canonical child ID: uar-verification-simplification::agent-team-concurrency-repair.

## Decision

**READY to Spec for a bounded repair contract; not ready to execute or verify.** Adapt current skill reconciliation and terminal settlement; retain the executor. No new dependency, database, task framework or model is justified. The precise served-build error categories remain open and must not be written as established root causes.

The uncomfortable finding is a startup/admission invariant mismatch in the inspected source: filesystem skills can be published in memory without the durable rows later demanded by admission. A second gap allows a worker to finish locally after terminal persistence fails, leaving the durable attempt apparently running. These are more specific than “concurrency is broken.”

## Evidence and causal alternatives

### R1: categorized, private failure evidence

Both saved member runs emitted skill_selection_unavailable. Current manager.rs:5149–5177 masks the capture cause. registry.rs:130–153 distinguishes StoreUnavailable (including query/decoding failure) from StorageMissing (registered ID absent in storage). The captured error does not tell us which occurred.

Safe owner/operator evidence should distinguish these categories without publishing skill bodies, secrets or another owner's metadata. Existing run/attempt/root IDs, effective source identity, provider IDs and counts are sufficient diagnostic boundaries. This is a contract requirement for the next stage, not permission for a live probe now.

### R2: filesystem startup versus durable selection — leading explanation

The retained PID64463 log records a successful database connection at08:49:12.719770Z, 210 filesystem skills, zero Database Skills and a210-entry registry at08:49:15.442677–.444626Z. No persistence-write failure is recorded. Successful empty startup reads weaken a startup decoding-failure hypothesis, but cannot rule out a later transient failure.

Source chain:
- src/server.rs:1749–1771 registers filesystem-skills-{index} plus db-skills.
- SkillService::initialize, service.rs:302–332, calls register_all_loaded, registry.rs:274–278; this publishes entries without writing them to persistence.
- reconcile_config_skills searches fs-skills at service.rs:339–345; reconcile_standard_agent_skills searches agent-skills at432–440. The inspected server path registers neither name.
- matching_snapshot always calls reconcile_selection before matching. Any registered ID absent from durable rows causes StorageMissing, even if this team selects no skills.

This is a concrete source mismatch and a strong explanation of the boot counts plus later error. It remains conditional for the retained artifact because the inspected checkout has not been proved identical to built source28137dc…. Falsifiers include a served-source path that persists those providers elsewhere, all registered IDs present at failure time, or a retained typed StoreUnavailable cause.

Adapt the existing reconciliation mechanisms (cand-001), preserving API-created override precedence, saved scoped enablement, explicit roots and startup readiness. Do not delete skills, replace missing records with fabricated defaults, bypass all reconciliation, or treat every discovered filesystem body as permission to overwrite durable configuration.

### R3: catalog result handling and terminal settlement

The actual recorded message is “failed to read collaboration catalog CAS result.” In storage.rs:135–148, UPDATE RETURN AFTER is followed by response.take(0); an error at that boundary is not proof of a generation mismatch, rollback or successful commit.

service.rs:419–429 converts storage error to Storage(error.to_string()). Runtime execution.rs:243–249 logs failed terminal settlement and returns. controller.rs:313–329 then marks the local worker finished; normal drain dispatches queued work, not that durable running attempt. The saved calculation remains running/reserved-unknown; the writer separately failed/settled. Their current state is unqueried.

Reuse cand-002:
- Preserve typed statement errors separately from result-decoding/transport errors. Pinned SDK query.rs:704–753 supports check/take_errors; these APIs do not themselves determine a committed outcome.
- UAR ingestion_records/surreal.rs:50–77 demonstrates bounded retry only for typed rolled-back conflicts. AG-UI CAS handling in providers/agui_runs/surreal.rs:38–66 rereads the exact predicate and keeps inconclusive errors. These are references, not authorization to copy retry policy wholesale.
- Reconcile the original terminal outcome against the same owner/workspace/attempt/run/fence and durable fields. An exact already-committed outcome should converge without a second model/tool dispatch or another usage-revision increment.
- If still unconfirmed, retain visible uncertainty and all unknown reservations. Any bounded settlement-only retry must require a known eligible outcome and stable identity. Changed fences, conflicting output, unknown commits and exhausted bounds must not become success.

Upstream issues [7472](https://github.com/surrealdb/surrealdb/issues/7472) and [7428](https://github.com/surrealdb/surrealdb/issues/7428) explain why structured query/commit errors matter. They do not prove this captured error is a conflict, nor establish behavior of our patched build. Context7 returned main-branch examples; local3.3.0 vendored source is the API authority. No upgrade is proposed.

### R4: preserve executor and prove concurrent merge

controller.rs:207–229 keys serial lanes by owner/workspace/team/**member**; different members do not share that lane. The code also bounds active capacity and spawns member workers. This permits overlap in source; it neither proves real overlap nor rules out other scheduling faults.

Adopt cand-003 unchanged as the execution mechanism. Retain the existing project graph: independent calculation and outline, then writer article consuming both attributed output artifacts. Do not substitute two queued tasks or client futures for simultaneous live runs. Keep real model, exact member/session correlation, successful terminal results and selected artifact IDs. Sequential snapshots can miss a short overlap; address only if that failure is observed after admission works. Full ordinary-agent KB parity, autonomous project decomposition, backend parity, cancellation and other platforms remain outside this specific acceptance claim.

## Candidate and readiness comparison

| Candidate | Verdict | Why not a new library? | Readiness effect |
|---|---|---|---|
| cand-001 existing skill registration/reconciliation | Adapt | The problem is UAR's provider identity and persistence invariant | R1/R2 require application changes; runtime READY cannot be claimed now |
| cand-002 pinned Surreal response APIs plus UAR CAS/receipts | Adapt | Existing APIs expose useful errors; terminal identity and accounting policy belong to UAR | R3 needs application code and bounded reconciliation; no SDK upgrade assumed |
| cand-003 Tokio member lanes and team artifacts | Adopt existing | No evidence supports a replacement executor or distributed task framework | R4 needs repaired-path evidence, not a new executor |

The parent fourteen obligations, consumption history and signed-contract records are not replaced by these R1–R4 analysis sections. Spec/Plan must preserve or explicitly reconcile them; this document is not a replacement exit contract.

## Research scope and limitations

Research began09:20:10Z; hard limit20minutes/eight queries per tier. External queries: tier1 two GitHub code searches, both authentication failures; tier2 one Context7 resolution and one documentation query; tier3 zero; tier4 one Firecrawl developer-index query required for dependency archaeology. No further broad search was justified after pinned-source/API and reuse evidence answered the library question. Tier3 maintenance metrics and alternatives are unassessed; no fresh-package adoption or latest-version claim is made. The developer-index query was version-behavior research, not a stack-comparison campaign.

The local source supplies first-party reuse candidates; evidence entries labelled local are source inspection, not a successful remote GitHub search. Provider/call/store probes, registry commands, tests, builds and code changes were not run.

## Prior context and knowledge gaps

The recall's relevant parent lesson is to separate installed-artifact identity from authored-source claims and keep conditional persistence findings distinct. Its old parent-stage approval statements are historical information, not authority for this child.

The recalled knowledge-gap list concerns another UI phase: rebuilding UI; app/features/shared/platform layering; Flat2.0 spacing/focus; bundle/latency/WCAG gates;365 style and11 filename exceptions;33.68% old coverage; assistive-technology receipts; performance receipt immutability; seven changes needing QA-log equivalents. These are historical recall, not measured here and not imported into this repair. Optional learning only: /learn-goal UI scope and architecture; /learn-goal Flat2.0 conventions; /learn-goal accessible UI verification; /learn-goal coverage interpretation; /learn-goal append-only performance evidence. None is run.

Local open questions are exact served-build selection category, whether filesystem rows were absent at that instant, original CAS error classification/commit outcome, and stable reconciliation for a no-usage terminal outcome. Current source repeats a matching unknown-usage settlement with an incremented usage_revision; this is a deduplication concern for the repair contract, not permission to replay the saved attempt.

## Review boundary and handoff

Adversarial artifact review uses a fresh harness-native critic with artifact/goals/constraints only. REST model preflight/dispatch was not attempted under the closed runtime/provider-probe scope; this is a declared review-method deviation, not a fabricated gateway failure. Model separation is unverified; carry that warning forward. No final local certification is implied.

Next: Spec the provider reconciliation invariant, private typed diagnostics and same-attempt terminal reconciliation with explicit uncertainty. Plan must bind a safe way to resolve the remaining evidence gaps—one separately named read-only diagnostic or categorized evidence from the authorized repaired-path gate—before claiming the root cause or accounting certainty. No standalone probe or automatic verification retry is authorized here.


## Adversarial review outcome

Fresh isolated artifact critic returned PASS,0 critical/0 warning/0 suggestion findings, with six explicit checked classes. Findings and anti-theater screen are under review/analyze/; screen score0.0. The packet supplied complete documents, goals and prior handoff, with no producer history. Producer/judge model identity was not independently resolved; cross_model_check remains unverified-producer-unknown and isolation_mode harness-native. REST preflight/dispatch was intentionally not attempted under closed provider-probe scope, so this is an explicit skill-method deviation and a handoff WARNING, not a failed-gateway claim or full cross-model review. Spec/Plan must not treat it as product certification.

Research cutoff09:39:20Z, within20 minutes from09:20:10Z. No further research dispatch will run for this analysis.
