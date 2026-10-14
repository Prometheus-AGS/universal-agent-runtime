# Dot architecture and scope analysis

2026-10-12. Supplemental source analysis during child Plan reconciliation. Source: local `main`, HEAD `b4244aeaa7e931240ae0f09ae31bae3f45bda244`, with existing uncommitted work. No production tests or builds ran. This is not completion or runtime-conformance evidence.

## Decision and limits

Preserve the runtime architecture and repair specific durability, policy, and protocol seams. The inspected source does not justify a whole-runtime rewrite. Complete all nine issue #379 requirements in the same Dot release, as the operator decided. The previous child recommendation for a separate release is superseded.

The uncomfortable thing: earlier assessment cited test-only A2A policy construction as production evidence. Execution also accumulated substantial test/support infrastructure before the operator's required full-phase boundary. Neither activity establishes that the corresponding production rewrite is necessary. Correct those claims before revising the plan.

The reviewed scope is runtime lifecycle, budget authority, checkpoint recovery, A2A/AG-UI/MCP adapters, effective settings and host defaults. This is not an exhaustive architecture audit of UAR. Exact remaining effort is unmeasured. Existing dirty checkpoint/approval improvements are useful implementation, not certified behavior.

## Evidence and necessary repairs

1. **Reuse execution; finish protocol correlation and interruption.** Production constructs `A2AThreadService` in `src/server.rs:1458` and dispatches via actors in `src/uar/api/a2a/thread_service.rs:471`. Bindings and task entries remain process-local (`35–52`); task lookup consults them (`497–509`). The adapter projects Working until mailbox completion and InputRequired only for yielded completion (`78–85,132`). Persist or reconstruct the mapping to the existing durable thread, and project pending human interaction. Do not add another executor or independent task authority. `task_execution.rs:568` setting Auto is inside the test module declared at 422–423 and is withdrawn as evidence of production policy.

2. **Repair approval precedence; reuse approval authority.** In `src/uar/runtime/manager.rs:6147`, verified-local governance bypass can return before descriptor `ApprovalClass::Required` is included in `approval_required` at `6180–6182`. The branch already guards host-required, recovered, child and explicit Ask/Deny cases. This ordering also exists in HEAD. Source reachability is traced for a standalone root using Auto policy, disabled governance, and no collaboration/instance revalidator demanding approval: MCP descriptors are Required (`mcp/registry.rs:167`); default standalone admission returns Auto (`runtime/tool_admission/standalone.rs:63–74`, installed at `manager.rs:1000`); orchestrator derives host_requires_approval only from Ask (`llm/orchestrator.rs:525–528`). GovernanceBypassed then resolves as approved (`580–581`), while standalone resolve does not check the descriptor class (`standalone.rs:77–106`). This is not claimed for paired hosts or independently restrictive revalidators. Runtime reproduction remains deferred. Required approval must remain authoritative with governance enabled or disabled. Reuse durable approval CAS (`thread/approvals/records.rs:45`), protected continuation (`recovery.rs:44`), and the existing exact-approval-ID host route (`server.rs:2616`). A2A/AG-UI project and resolve that authority. MCP elicitation is a distinct interactive request: accepting elicitation must not implicitly grant tool authorization.

3. **Durable budget admission is substantive work.** `CostBudgetTracker` uses an `Arc<Mutex<Inner>>` (`src/uar/runtime/cost_budget.rs:189`); terminal settlement in `run_cost.rs:68` spawns detached database writes without stable settlement identity. These files are unchanged relative to HEAD. History alone does not serialize admission between processes or restore reserved liabilities after restart. Preserve calculation and limits; add durable reservations, idempotent settlement and restored aggregates through the existing persistence backends. This justifies Dot09/10, not replacing the runtime/accounting stack.

4. **Crashed-owner recovery is distinct from orderly restart.** New checkpoint storage already uses transactional immutable-envelope/single-source-run claims and requires predecessor epoch `orderly_excluded` (`src/uar/persistence/providers/checkpoint_records/surreal.rs:199`). Manager records orderly exclusion only after producer/resource joins and effect reconciliation (`manager.rs:1733`). A crashed process cannot do that. Dot11 must let trusted native supervisors/Kubernetes exclusion evidence advance the existing ownership authority. Do not treat lease expiry or missing Pod as termination evidence, release uncertain effects, or create a second recovery state machine. Backend scope follows the parent plan: local/shared SurrealDB checkpoint recovery is required; existing Postgres credential/key/accounting guarantees remain required, while checkpoint/observer-dependent assistant parity is explicitly unsupported until implemented. Do not silently expand this into Postgres checkpoint parity.

5. **Protocol and settings integration belong at existing boundaries.** A2A router/handler/types already isolate transport (`api/a2a/mod.rs:63`); old wire fields/states and hardcoded card version need adaptation (`types.rs:116,262`, `agent_card.rs:22`). AG-UI already has event projection, but approvals are CUSTOM-only (`api/adapters.rs:365`) and ingress is OpenAI-shaped (`server.rs:4698`). MCP's inspected client paths use unit handlers (`mcp/registry.rs:698,779,834,1400`) with existing run-scoped bindings (`mcp/runtime.rs:831`). Add interactive handlers/ingress and project the host-owned records rather than replace transports. Effective settings must compose the live provider registry (`llm/registry.rs:502,650`), persisted settings synchronization (`settings/provider_sync.rs:7`) and governance snapshot (`api/settings.rs:302`), with Bearer admin authorization and a secret-free aggregate revision. The existing settings-list response is not an authoritative aggregate.

## All nine issue #379 requirements remain owned

| Requirement | Existing seam | Necessary addition |
|---|---|---|
| 1 public A2A 1.0 card | A2A router/card and auth middleware | Exact public card path, pinned 1.0 schema, usable interface and narrow anonymous exemption |
| 2 A2A 1.0 tasks | Handler/types/A2AThreadService | Pinned methods, roles, parts and states; durable task correlation; preserve errors |
| 3 A2A human approval | Host approval ledger and continuation | Input-required projection, exact decision identity, once-only approve/deny, Required precedence |
| 4 AG-UI interruption/resume | Event projector and existing host decisions | RunAgentInput ingress, specified terminal interrupt outcome and resume[] mapping |
| 5 CORS | Server CORS composition | Empty default deny; explicit origin allowlist |
| 6 MCP elicitation | Client construction and run bindings | Capability/handler on every transport path; distinct durable accept/decline/cancel interaction |
| 7 shipped safe defaults | Configuration/profile composition | No implicit model, local/off embedder, loopback default, JWT enabled; deliberate deployment overrides |
| 8 effective settings | Live registry/settings/governance authorities | Stable documented aggregate shape/path, Bearer admin 401/403 contract, revision/hash, no secrets |
| 9 release version | Card constructor and packaging | Actual published plain x.y.z version, tied to the tagged artifact |

Pin external contract versions and effective-settings acceptance shape before implementing their wire mapping: a mapper otherwise has no stable definition of accepted fields, states and errors. Do not guess a release tag. Items 1/8/9 enable Kubo's reader; all nine require final published-artifact evidence to close Topology B.

## Why the effort looks large

The Dot plan combines nine reliability concerns with memory controls, schedules, follow-ups, capability discovery and local/shared release operation. Issue #379 adds protocol conformance. Some breadth is required by that approved scope; it is not evidence of bad architecture by itself.

The working tree is mixed and not cleanly attributable to Dot. Source inventory observed 47 tracked source files with +2,058/-626 lines, plus seven untracked source files totaling 1,609 lines. Six tracked test files have +744/-380 lines; 15 untracked test/support files total 7,260 lines. These figures exclude other categories and do not distinguish all inline tests. They are only a mixed-checkout inventory: they establish neither Dot-attributable expansion nor the necessity or wastefulness of any scenario. No provenance breakdown has been established, and no scope or deletion decision may rely on these counts. Before final gating, each scenario must trace to an explicit requirement/failure boundary; consolidate duplicated process/peer plumbing without removing distinct production guarantees. No arbitrary line-count reduction target is justified.

## Plan consequences

- Retain existing host/kernel capability separation, principal identity, actor execution, approval records, checkpoint host, MCP bindings, memory service and provider adapters.
- Preserve existing completed-source work and historical receipts. Earlier receipts do not certify the final combined revision.
- Correct stale production citations and remove the separate-release recommendation. Do not infer a need for new approval, execution, recovery or settings authorities.
- Keep real dependencies: enforceable budgets need durable reservation/settlement state so restart cannot reset liabilities; safe schedule recovery needs owner exclusion so a replacement cannot overlap a still-running executor; complete proactive behavior needs memory controls to honor capture/deletion policy and schedules to generate durable due occurrences. These prerequisites govern completed behavior, not every independent adapter edit.
- Remove artificial implementation serialization: CORS/defaults, card/config adapters and contract definition do not require finished schedules/follow-ups. Final capability advertisement still must truthfully reflect finished operations. Bounded owners may work concurrently only with disjoint files and settled shared contracts.
- Complete every production item in the combined phase, including migrations, operational controls, documentation and acceptance-runner implementation, before any product verification. Record the full implementation inventory first. The operator separately authorizes this source/artifact adversarial review; that does not authorize product tests or builds.
- Then run the planned serialized real-path integration/customer-artifact gates, with observed local/shared restart, identity, unknown-effect and protocol scenarios. Publication and installed-artifact conformance remain separately recorded obligations.

## What would change this decision

A traced production path that cannot use the existing host authority without violating required ownership/atomicity would justify a narrowly named redesign. Evidence that the mapped durable records cannot represent required interruption or ownership transitions would change the reuse proposal. File size, test-only construction and uncalled legacy modules do not establish those facts. The final integration gate may reveal errors this source analysis cannot establish.

## Review scope clarification

The reuse recommendations are prospective design constraints, not a claim that every dirty file has been audited for duplicate authority. Seven untracked source files cover MCP outcome normalization, checkpoint resume envelope, Surreal checkpoint persistence, checkpoint host capability, approval continuation recovery, API-key storage and principal credential storage. This source pass examined the checkpoint host/persistence and approval continuation seams; whole-tree duplicate-authority classification remains unperformed. No deletion or broad refactor is authorized on that basis. A2A binding concurrency needs explicit ownership/atomic transition design in the adapter task; the local map observation does not by itself establish a race or deadlock.

## Independent review disposition

M3 critic reviewed the first packet and returned BLOCK (2 critical, 4 warning, 1 suggestion). Its missing-source and test-boundary evidence findings were valid for that first packet. Expanded excerpts, baseline comparison, corrected citation, dependency rationale and explicit scope limits were provided in a second packet. K3 judged that revised packet PASS (0 critical, 0 warning, 9 suggestions, including seven dispositions of M3 points). K3 incorrectly describes several resolved original-packet findings as M3 overreach; the record treats them as addressed findings, not critic errors. Two remaining citation suggestions were resolved by correcting the card version to line22 and removing the unsupported legacy-store sentence. A backend-scope clarification preserves the existing parent plan's explicit Postgres assistant limitation; it is not a new parity commitment. Those final editorial clarifications are not a new runtime certification.

The artifact-only review supports proceeding with targeted plan reconciliation. It does not establish that all dirty source has been architecturally audited or that the combined feature implementation is complete. Review packets, source hashes, normalized findings and execution details are under review/architecture/.
