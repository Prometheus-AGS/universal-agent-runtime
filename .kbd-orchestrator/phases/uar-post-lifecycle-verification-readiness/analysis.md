# ANALYSIS: uar-post-lifecycle-verification-readiness

Project: universal-agent-runtime. Date: 2026-10-21. Stage: Analyze only.

## Owner-review result

Use the existing main-binary path first, retaining the current source candidate and store configuration as the baseline. The strongest bounded correctness work is PL-01 budget reconciliation and PL-02 refusal classification/stream delivery. Run-history source asymmetry is real, but its operational manifestation depends on durable-history eligibility; do not patch an unsupported store into apparent durability. PL-04, PL-06 and PL-07 require narrower causal evidence before choosing repairs. PL-08 needs an explicit destination counting/capacity trust policy. PL-09–12 remain supporting proposals, not new release blockers. PL-13/14 are deferred by the operator.

This is a dependency analysis and recommendation for manual review, not an executable task plan. No product edits, builds, tests, probes, certification or provider calls ran. Verification remains CLOSED. No Plan, Spec or Execute stage was entered.

## Identity, authority and preserved baseline

- Host: Travis’s Mac Pro, x86_64.
- Checkout: /Users/gqadonis/.claude/worktrees/uar-post-lifecycle-verification; verified UAR repository; detached HEAD eddd5508c902f892b20042ec545900cac16747c6, source checkpoint ancestor 5305fa4c2ca5fd91e14aa8eae75eda599e2f417f.
- Existing delegated Codex session: 01a15da2-c96d-7202-9035-3fdaa901a007. Exact model ID remains unexposed; no model/provider switch or paid call occurred.
- Installed full pack 1.13.1 at bfe572ff8a97a21c7610c44ea6b39d5f97a857b7; generation 1cd8004932bf9899614f4e1907e62a177b67844fc9fcf87edaa30c1ebc09474a. Installed Analyze skill and its protocol were read directly, rather than relying solely on cached catalog text. Prior native-session cache limitation remains recorded in installation-receipt.json.
- Operator authority: after the assessment summary, Travis said “Proceed to analyze on the UAR” in response to main-binary-first, current candidate/store baseline and deferred optional migration-status/citation-stream scope. This changes the earlier “stop before assessment” goal note for stage sequencing only. It does not open verification or authorize product implementation.
- Existing tooling inventory, generated projections, old ledger evidence and assessment are preserved. No cleanup or old phase closure performed.

### Candidate identity is two separate facts

The current source handoff is eddd5508…; no booted Mac Pro main binary was identified or tested in this stage. The existing deployment receipt docs/operations/lifecycle-image-delivery-39532243053.md binds production source 32cb3bcfc9691212ec0b0359ce22474968bc6850 to combined Linux amd64/arm64 image index sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8. It proves image delivery, not runtime acceptance. Neither image nor store was pulled, booted, replaced or reconfigured.

Read-only Git comparison shows the handoff differs from that delivered source in 12 lifecycle files: src/embedded.rs, src/mcp/registry.rs, src/sandbox/session_manager.rs, src/server.rs, src/server/lifecycle_control.rs, src/server/lifecycle_reconnect.rs, src/session/thread.rs, src/uar/memory/scoped_storage.rs, src/uar/memory/service.rs, src/uar/persistence/scoped.rs, src/uar/realtime/lifecycle.rs and src/uar/runtime/lifecycle.rs. Consequently the delivery receipt is retained as historical artifact identity; its evidence cannot certify these later changes. No substitute candidate was selected by this analysis.

### Current store baseline, without changing it

There is no checkout-local config.yaml/config.yml or .env/.env.local. The main config loader in src/config.rs:1706–1722 uses explicit --config/CONFIG_FILE first, checkout config.yaml second, then ~/.uar/config.yaml. That existing fallback file is present with SHA-256 0e8afa3b34a1ea95839ec0dec96313ffb649d517daf3cefcbe1cfacf365c9194. Its persistence provider is surreal and its endpoint class is loopback WebSocket, 127.0.0.1:28000. No credentials were copied, printed or modified. Relevant config/durability override variable names were absent from this executor environment; this says nothing about a separately launched process's environment.

Therefore an otherwise unoverridden main launch from this checkout would use the existing remote-protocol fallback, not merely the compiled surrealkv://./data/uar.db default or the checked-in embedded example. Actual running process config, server storage engine/volume persistence, durable-history capability and remote attestation remain UNKNOWN. Preserve this configuration; do not migrate or replace it as an assumed prerequisite. PL-05's memory-backed docker-compose.test.yaml describes a different fixture, not proof that the existing loopback store is volatile. A later explicitly authorized observation must discriminate those facts before any configuration decision.

## Source gap versus hypothesis: minimal boundaries

| Item | Resolved analysis | Minimal possible boundary and evidence still needed |
|---|---|---|
| PL-01 P0 | Source gap: a durable call can remain Pending after cancellation, deadline, upstream error or failed durable settlement. reserve refuses finite admission with unresolved liability. This safety refusal is intentional; automatic age-based refund is not a repair. | Reuse host-owned AccountingCall/CallUsage and accounting::observe/settle with the existing transactional scope/call CAS in providers/accounting/surreal.rs. Design exactly-once reconciliation bound to original call plus authoritative terminal/effect evidence. Preserve unknown and refuse when evidence is insufficient. Later evidence must cover confirmed terminal, response loss, uncertain upstream outcome, failed settle and restart without double charge/refund. Do not create a second budget ledger or imply absence of a task proves no effect. |
| PL-02 P1 | Source gaps: tracker.admit_tool lacks Admission context; OpenAI Error logs/breaks instead of delivering the error code. | Keep the class fix within the existing budget boundary and the transport fix within the current OpenAI mapper. Specify its compatible terminal-error representation and end-of-stream behavior before implementation. No citation extension is included. Prove refusal before downstream effect and distinguish transport disconnect. Classification can proceed independently of PL-01; final liability claims cannot. |
| PL-03 P1 | Confirmed list/get asymmetry: list_runs_for_context uses active_runs, while get_run_for_context can load durable history. Runtime restart failure remains a hypothesis conditional on store eligibility and actual writes. | Discriminator: with the unchanged main mode/config and one identified artifact, compare known-run GET and owner-scoped list before/after restart. GET succeeds/list empty supports an owner-keyed durable list/merge. Both fail sends diagnosis to writer eligibility/store/owner namespace before a list repair. If needed, extend only persistence history listing plus provider implementation and manager merge/dedup; preserve exact live owner checks and avoid phantom active runs. No whole-runtime rehydration rewrite. |
| PL-04 P1 | AG-UI transition's typed TransactionConflict→false handling exists. A2A/MCP transition failures differ. A losing transaction is not automatically a benign CAS miss. | Reuse current transition validation/readback contracts. Establish both race orderings and exact intended row/owner transition before expanding loser handling. Unrelated write conflicts must remain errors. Do not merge the state machines or map every storage conflict to false. This source correctness lane is independent of a universal harness migration. |
| PL-05 P1 | Fixture/evidence gap: checked-in remote test service uses memory; remote durable capability requires actual eligibility. Existing Mac Pro fallback uses a separate loopback service of unknown backing mode. | Preserve existing store baseline. First establish its applicable durable mode in future authorized evidence; no attestation flag is proof. If a disposable qualification fixture is later approved, isolate its persistent volume and config rather than touching the operator's current store. A representative main turn/restart needs the existing selected model/backend to be observed, not a newly mandated provider or embedding fallback. |
| PL-06 P1 | accept/publish both touch KB row; basic ingestion can create unattributed chunks. Neither fact alone identifies the cause of the recorded second-upload failure. | Retain two documents A/B and a separate KB control, including positive results before deletion. Attribute the actual typed error and competing writes before choosing a scoped retry or KB-row change. Keep transactional ingestion/publication and document-scoped deletion; do not purge document_id=None rows as if owned by A. No global retry policy, schema purge or embedding redesign. |
| PL-07 P1 | Historical evidence unavailable in the published portion, not a newly reproduced product failure. B3 is sidecar-security evidence; it does not become a universal main-binary release blocker by association. | Recover exact original candidate/log/assertion outcomes if the existing adopted main-mode requirement depends on them; classify each separately. Do not rerun the old program in Analyze, infer a fixture common cause, or roll all historical rows into this phase. See receipt search below. |
| PL-08 P1 | Production destination preparations initialize None; contract machinery is available but not fully wired into every lane. Provider-exact fit cannot be inferred from cl100k estimates or a catalog window. | Reuse DestinationRequestPreparations and RequestBudgetContract. Decide supported destination contract, capacity trust, output/reasoning reserve and treatment of approximate/unsupported counts. Preserve gateway distrust and protected context. Ensure trim and final tool-call-inclusive verification agree. This decision precedes revised reservation arithmetic; no tokenizer or SDK replacement proposed. |
| PL-09 P2 | Support proposal, not established common cause. Existing sidecar finalization and distinct main-process health adapter have different responsibilities. | Only extract a proven repeated isolation/capture/readiness/awaited-teardown primitive after a specific fixture problem is classified. Keep sidecar/main/external/in-process adapters and assertion parity. No estate-wide rewrite or forced READY/:0 rule. |
| PL-10 P2 | A generated test list cannot independently freeze requirement semantics. | Reference the frozen reviewed requirement text, stable IDs and named positive/negative/no-effect/distinct-owner assertions. A registry pilot may aid traceability; it is not a new prerequisite for existing historical acceptance. No new framework or package identified as necessary. |
| PL-11 P2 | Existing source identity, immutable receipt writes and binary checks are reusable; a common schema across all runners is not established. | Adapt the existing receipt primitives for one bounded pilot only if it serves the selected repair's observations. Explicitly bind source/build/boot, mode/config/fixtures and separate setup/body/teardown. Preserve raw logs and predecessor lineage. No repository-wide gate rewrite or new certification controller. |
| PL-12 P2 | Cargo target required-features exist; applicable deployment matrix remains a proposal. | Later derive only supported target/profile combinations from existing declarations. No Cargo execution in Analyze. A compile matrix is not proof of behavior and is not added to release scope here. |
| PL-13 | Deferred by operator; existing migration coordinator is not missing work to duplicate. | No status API, durable preflight operation table, new permission surface or retention policy included. |
| PL-14 | Deferred by operator; OpenAI's citation-event omission does not enlarge frozen R18. | No nonstandard SSE frames or citation_required product-policy change included. Preserve compatible error work under PL-02 separately. |

## Dependency order for owner discussion, not a task plan

1. Retain the source/config/artifact identities above and distinguish store capability from data-loss symptoms. That is prerequisite interpretation, not authorization to launch a preflight or certification program.
2. PL-01 is the highest correctness priority before any finite-budget safety claim. PL-02 class wrapping is independently bounded; its full liability proof depends on PL-01. Its adapter contract should be discussed alongside that repair, with PL-14 excluded.
3. PL-08's destination trust/counting decision precedes freezing new reservation arithmetic or revising R22 fixtures. Do not use a fixture limit chosen under the old catalog arithmetic to certify the new contract.
4. PL-03/05/06 apply to the chosen main mode and current store baseline. Confirm GET-vs-list and eligibility before deciding a durable-list repair. Separate fixture inadequacy from product defect; observe exact KB conflict before choosing its repair.
5. PL-04 is a focused concurrency-invariant lane. PL-07 needs historical receipt recovery/classification where applicable; neither requires PL-09 to be designed first.
6. PL-09–12 remain supporting, bounded pilots that can be deferred. PL-07 classification precedes any claim that a PL-09 harness change repairs B3. PL-13/14 remain out of scope.

This order names dependencies and minimal surfaces; it does not allocate writers, task counts, commands or execution dates. Those belong to the later manually reviewed Plan.

## B3 evidence search result

The assessment's suspected missing ledger was located at .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/receipts/r-ledger-reconciliation-2026-10-19.md. The published file has 2,144 lines, not the cited 2,782–2,795 range. Its late entries preserve candidate-specific records through c7, including sidecar binary SHA 48d30523… and prior identities, but not the later c9/B3 assertion-level result.

Its final pre-reboot preservation entry names receipts/scratch-uar-r14-2026-10-20, an untracked 5.5 GB scratch copy that intentionally excluded credential-key material and the live store. That directory is not in this handoff. The searched receipt directory contains 12 published files, without c9/B3 raw logs. Searches covered this UAR checkout's .kbd-orchestrator receipts/evidence and .prometheus records, plus named UAR project documentation; the original local UAR clone's project records also did not supply the later ledger. No unrelated project/session files, credential stores, live databases or retired sessions were opened.

.prometheus/decisions.md:2205 records individual c9 failures as causally unresolved; gotchas.md:2017 explicitly says the watcher repair is not proof that the watcher caused the c9 lock failure. These corroborate uncertainty, not a cause. Preserve the punch list's “8 failures + 1 unfinished, 26 passed” as an attributed historical count only. No evidence was located that upgrades those cases to candidate-bound PASS/FAIL here. Recovery of that specific omitted receipt segment is a narrow later handoff need, not an instruction to collect a new full security suite now.

## Build-versus-adopt and research bounds

Stack is already specified: existing Rust UAR and current SurrealDB/accounting/context/adapter contracts. No external dependency, replacement runtime or new controller is recommended. Local primary-source inspection resolves the reuse question: adapt the existing accounting CAS, history provider interfaces and destination preparations; reference existing receipt/finalization patterns. library-candidates.json records those candidates with pinned source URLs. Broad external library searches, package-registry comparisons and model calls would not resolve the present source/receipt hypotheses and were not run. The 20-minute/8-per-tier ceiling is retained; no budget exhaustion is claimed.

These decisions do not make the product READY without runtime code. PL-01/02 require bounded source repairs for their proposed outcomes; PL-03/04/06 are conditional on discriminating evidence; PL-08 requires a trust decision and wiring design. PL-09–12 may support evidence without becoming blocking scope. No adopted-contract.json was created or amended, and no product READY or acceptance disposition was written.

## Remaining decisions and readiness

Resolved by user: main binary first, keep current baseline, PL-13/14 deferred, manual review before Plan/Execute. Remaining discussion: how the next reviewed plan names the existing source handoff versus older delivered image without transferring evidence; which destination capacity/counting policy to accept for PL-08; the compatible PL-02 terminal error behavior; whether current remote store's demonstrated eligibility supports the intended durability claims; and how to retrieve only the omitted c9 receipt segment if needed. No alternative provider/store is selected, and no configuration mutation is proposed as a hidden prerequisite.

The recalled Assess lesson says “runtime claims remain UNKNOWN and verification CLOSED”; it keeps analysis evidence separate from product grading. The older UI reflection's goal-drift lesson reinforces explicit adopted scope. Its recalled UI/coverage gaps are listed below only for provenance; none becomes a blocking requirement. Optional learning can be requested with /learn-goal <topic>; no learning task is started.

- Goal Discard and completely rebuild the existing web UI: 1 — PARTIAL
- Goal Enforce app → features → shared → platform and kebab-case everywhere: 5 — PARTIAL
- Goal Mechanically enforce Flat 2.0 and the 3px ember focus exception: 6 — PARTIAL
- Goal Pass bundle/latency, WCAG, responsive, and acceptance gates: 10 — PARTIAL
- 365 Flat 2.0 style exceptions and 11 filename exceptions remain in the shrinking allowlists.
- Unit line coverage remains 33.68% against the retained 60% threshold.
- Manual screen-reader and 200% text-zoom receipts remain outstanding.
- Performance attempts are not yet append-only; C-15 preserved the failed measurement in a secondary manifest only.
- Seven changes need a canonical QA-log equivalent; C-12 may need another independent final review.

## Review limits and recommendation

Independent cross-model review is not established; no paid call or provider switch was made. The source assertions and evidence limits above are open to Travis's manual review. Prior Assessment remains intact; the B3 ledger location correction is recorded here, not silently rewritten into it.

**READY to spec as a bounded analysis artifact for owner review**, not product READY, not a Plan approval, and not execution or verification permission. The minimal next discussion is to accept/refine the dependency boundaries above and resolve the specific trust/error-contract decisions before a manually authorized Plan. Stop here.
