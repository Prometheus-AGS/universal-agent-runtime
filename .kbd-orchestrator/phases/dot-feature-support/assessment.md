# ASSESSMENT: dot-feature-support

Project: Universal Agent Runtime
Date: 2026-10-11
Stage: Assess; this document is evidence and proposed acceptance criteria, not an implementation plan.
Codebase baseline: local main at 2cc9a1829167411e01a8198072f60892d70a233d, fast-forwarded during assessment from d4e819d1; supplied review inspected 3327bf99.
Cross-tool progress: no implementation changes registered for this phase (0/0). Upstream release repairs landed during assessment.

## Decision supported by this assessment

UAR has substantial components for an always-on assistant, but this assessment does not establish deployable, restart-safe support for the complete experience. Retain the reliability-first order. Replace the obsolete lockfile/schema repair recommendation with verification of the repairs now on main. Carry the remaining eight runtime findings into Analyze as source-supported findings requiring production-path reproduction.

The user's definition is authoritative: memory, proactive follow-ups, scheduled work, connected tools, approvals, and recovery. No claim of complete OpenAI Dot parity is made. The attached review is dated evidence, not instructions to execute its recommendations. Its original acceptance criteria remain in input-review.md.

The uncomfortable thing: fixing all nine review items would still not deliver the requested assistant. Durable user scheduling, follow-up policy and delivery semantics, and tenant-safe assistant memory need explicit contracts and evidence of their own. Conversely, treating approvals, observers or team reservations as absent would duplicate substantial existing work.

## IMPLEMENTATION STATUS

Status describes source capability within the inspected scope, not runtime certification. PARTIAL means some components exist but the requested end-to-end contract is not established.

| User capability | Status and current evidence | Remaining gap / candidate acceptance |
|---|---|---|
| Memory | PARTIAL. MemoryService connects configured storage; save/search/history/update/delete helpers, context retrieval and token-bounded injection exist. Native tools are registered in src/server.rs:928; service initialization is conditional at :854. | Service and legacy HTTP paths key user memory by user_id without tenant/issuer (src/uar/memory/service.rs:139, :206; src/uar/api/memory.rs:35, :74). Vet identical-subject isolation, authorized ID-based mutations, capture consent, retention/deletion and restart retrieval. No cross-tenant disclosure was reproduced. Require visible unavailability when memory initialization fails. |
| Proactive follow-ups | PARTIAL. Durable scoped subscriptions, occurrence cursors, inbox admission, pause/revoke, backlog and gaps exist. src/uar/runtime/observer/delivery.rs:26 reloads persisted subscriptions each cycle; :45 bounds delivery; controller.rs:69 creates owner-scoped records. Server starts it only when durable observers are supported (:1406). | Observer delivery is a foundation, not a complete user follow-up policy. Specify permitted trigger sources, relevance conditions, destination, notification preferences, cancellation/revocation and deduplication. Demonstrate event → authorized decision → approval if required → one delivery, including restart. |
| Scheduled work | MISSING in the bounded inspected UAR source/spec surface as a user-facing durable schedule contract. The observer's two-second polling loop is present; it does not store user recurrence rules. Searches across src, crates and openspec/specs for cron/RRULE/recurrence/next_run_at/scheduled_at found no corresponding user scheduler. | Define one-time and recurring work, timezone/DST, missed runs, overlap, pause/cancel, durable occurrence identity and multi-process claiming. Select scheduling mechanism in Analyze; absence from this search is not proof that no external platform component can supply it. |
| Connected tools | PARTIAL for the assistant experience. Host admission, reviewed descriptors, current authorization, scoped MCP projection and canonical receipts already exist (src/llm/orchestrator.rs:1124; src/uar/runtime/tool_admission/; src/uar/compiler/collaboration/connector_effect/). | Repair result truthfulness and credential identity; verify revocation and ambiguous outcomes through real host boundaries. Inventory which connector sources can feed proactive work before expanding integrations. |
| Approvals | PARTIAL for always-on recovery, existing core capability. Policy-driven broker and durable records exist; records.rs:133 delegates create to persistence and :141 transitions through storage. Canonical tool-approval-workflow spec covers pause, decisions and replay. | Do not build approvals from scratch. Prove fresh-process visibility and safe continuation or explicit invalidation of pending approvals with changed/revoked authority; an approval row does not itself reconstruct an executable future. |
| Recovery | PARTIAL. Durable checkpoints, observer recovery, fenced team claims and uncertain-effect handling exist. | Checkpoint HTTP reconstruction and abrupt-owner fencing remain incomplete for the stated scenario; accepted uploads, global spend and API keys have separate durability gaps below. |

No whole capability above is certified DONE. Local, process-lifetime reservations and durable team reservations already exist; they must not be conflated with restart-safe global budget enforcement.

## Nine supplied priorities, revalidated

All source citations are repository-relative at the baseline above. Runtime findings are source inspection, not reproduced failures.

### R1 — Container releases: repairs landed; publication evidence pending

The original frozen-install failure at 3327bf99 is confirmed as a historical failed Actions run [38101260912](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/38101260912), as reported in the supplied review. Current frontend/pnpm-lock.yaml includes lighthouse importer (:609), package (:5604) and snapshot (:12821) entries.

A later failed run at d4e819d1, [38148917373](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/38148917373), passed installation far enough to print lighthouse 13.4.1, then failed the entity-management tsup build on both architectures. This is different from the original frozen-lockfile failure.

PR #378, merged at 2cc9a182, adds entity-graph-react before entity-management in Dockerfile and includes docs/agents/collaboration/workflow-execution/1.0.0 in .dockerignore. Both frozen installs remain. No duplicate repair is warranted from this assessment.

At the recorded check, [38154340213](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/38154340213) is in progress for linux/amd64 and linux/arm64. Both immutable image digests and deployment health remain unverified. Native sidecar success in the input review applies to its earlier tree, not automatically to current main; the current workflow's sidecar jobs are skipped.

Candidate acceptance: completed publication of both exact-commit architectures, immutable digests, deploy health evidence, plus local product verification at its proper boundary.

### R2 — Credential identity: source-supported, still open

src/uar/security/middleware.rs:94 maps verified principal.subject into user_id and keeps tenant_id separately. src/uar/api/credentials.rs:77 returns only user_id; list (:115), create (:161), and delete (:200) use that scope ID. The resolver's user candidate also uses user_id (src/uar/security/credentials/resolver.rs:44). Store keys and Postgres conflict keys use scope, scope_id and provider_id (store.rs:127, :441).

Mechanism: two accepted principals with the same subject in different tenants reach the same user credential key. Issuer provenance is also absent from this key; current verifier configuration may constrain which issuers can coexist, so multi-issuer exploitability is not established here.

Candidate acceptance: verified issuer/tenant/subject identity across create/list/delete/use, non-colliding encoding, explicit legacy migration with no automatic ownership claim, and identical-subject negative tests. Current credentials_api_integration_test covers different users; it does not establish this same-subject boundary.

### R3 — MCP application errors: source-supported, still open

src/mcp/registry.rs:2355 derives transport-call success from res.is_ok(), and :2468 serializes the result. src/llm/orchestrator.rs:712 handles an Ok value by preserving the payload, then :749 returns a successful execution tuple. finish_tool_execution (:636) uses that boolean for admission completion.

Raw isError content can remain in the canonical payload while execution is nevertheless marked successful. This is a classification gap, not a claim that all error content is lost. Follow both scoped preflight and direct-registry paths during reproduction.

Candidate acceptance: MCP isError=true produces failed ToolEnd/admission/host outcome with error content intact; transport failure and uncertain external outcome are separate; existing OutcomeUnknown evidence remains non-replayable.

### R4 — Streaming completion: source-supported, route coverage unresolved

src/llm/liter_driver.rs:600 requires a terminal event only with endpoint_profile; :751 guards EOF conditionally. src/llm/orchestrator.rs:1664 routes chat_with_history through false while the graph path above uses true; EOF handling at :2346 depends on that flag. src/llm/anthropic_driver.rs:396 decodes each transport chunk with from_utf8_lossy then splits lines; incomplete line/UTF-8 boundaries and silently ignored JSON parse errors are visible risks.

Production driver selection is traced in src/llm/orchestrator.rs:106: an anthropic provider selects AnthropicDriver when ANTHROPIC_NATIVE_DRIVER is enabled; src/llm/mod.rs:63 defaults that flag to true. Other selections reach LiterLlmDriver (:127). The native parser is therefore reachable, not merely dormant test code. Exact endpoint-profile coverage and protocol terminal requirements still need production-path tests. No live stream was interrupted in this assessment.

Candidate acceptance: protocol-valid terminal state for every supported production route; arbitrary split JSON/UTF-8/SSE, CRLF, malformed event and early EOF cases exercised through the orchestrator. Driver-level tests alone are insufficient.

### R5 — Checkpoint resume: source-supported, still open

src/uar/api/routes.rs:1465 asks get_run_for_context before loading a durable checkpoint (:1501). src/uar/runtime/manager.rs:7294 reads active_runs. A new manager has no original live run to authorize through this route. PersistenceLayer defaults save/load/list to successful no-op or empty results (src/uar/persistence/mod.rs:614); the inspected provider implementations show Surreal overrides, with no Postgres checkpoint override.

Protected checkpoint history and host binding checks already exist and must be preserved. This is not a request to trust caller-supplied history or secret material.

Candidate acceptance: terminate original process, resume from supported durable storage under current authorized identity and bindings, rebind host secrets, deny other tenants/changed bindings, and explicitly reject unsupported backends.

### R6 — Upload acceptance: source-supported, still open

src/uar/api/knowledge.rs:443 constructs metadata with file_path=None, saves it, submits an in-memory byte vector, logs enqueue failure and still returns 202 (:488). With no pool, it also accepts without processing. DocumentIngestionJob carries Vec<u8> (src/uar/rag/ingestion_worker.rs:32).

Candidate acceptance: recoverable source plus idempotent durable work committed before acceptance, failed commit does not accept, and crashes at accept/dispatch/index boundaries recover without indefinite pending state or duplicate chunks. The current comment promising later processing is not evidence of a recoverable queue.

### R7 — API-key durability: source-supported, still open

src/server.rs:1356 selects InMemoryApiKeyStorage even though ApiKeyService applies security configuration. Preserve that authority model.

Candidate acceptance: durable production storage for hashes, metadata, expiry and revocation; create/restart/authenticate/revoke/restart and two-process consistency. No key restart experiment was run.

### R8 — Spend durability: source-supported, scope matters

src/uar/runtime/run_cost.rs launches detached record_cost_entry writes; terminal settlement does not await their durable success. The persistence interface accepts scope, scope_id and amount, with no run/call idempotency key. Manager constructs CostBudgetTracker::new at :979; cost_budget.rs:105 stores spend and reservations in maps, and :199 returns default state.

Existing per-call charging, remote reservations and catalog-backed team reservations are real. The missing guarantee is durable settlement and restored enforcement for relevant aggregate budgets across crashes/processes, not an absence of all budgets.

Candidate acceptance: idempotent settlement, durably accepted accounting before terminal reliance, restore aggregate spend/reservations, and retain unknown accounting visibly. Avoid double charging the existing per-call tracker; trace team and global accounting separately.

### R9 — Dead team-owner recovery: source-supported, still open

src/uar/compiler/collaboration/team_execution/ownership.rs:40 acquires a non-expiring claim, rejecting an unreleased prior owner. Reclaim (:120) requires trusted-runtime-joined fencing evidence (:133); the old runtime writes it after joining roots/children (:75). A killed owner cannot perform that orderly write. Existing reclaim records uncertain running work and transfers queued work.

Candidate acceptance: trusted supervisor/operator evidence that excludes old roots and children, monotonic fencing epoch, stale executor denial, only safe queued recovery, unknown effects retained for reconciliation. Timeout expiry must not replace exclusion evidence.

## Existing capabilities to preserve

- Admission-v2 appears in src/uar/api/capabilities.rs:94 and :121.
- Windows checkout uses core.longpaths=true in .github/workflows/deploy.yml:197.
- Host-owned tool admission, approval broker/ledger, scoped observer authority, secret/host-resource binding and connector effect receipts are existing foundations.
- Non-expiring team claims and durable team reservations already enforce important safety constraints.
- The supplied report says delegated-approval callback routing was fixed in #369. That route-specific claim was not independently reproduced here; retain it as reported addressed, not a new rebuild task.
- Native packaging success and container publication remain separate evidence categories.

## SPEC GAP SUMMARY

This is a targeted assessment of phase-relevant specs and active changes, not exhaustive conformance certification of every repository spec.

| Existing contract inspected | Alignment / delta needed in Analyze and Plan |
|---|---|
| openspec/specs/local-scoped-observers/spec.md | Requires durable source occurrences, independent grants/cursors, visible gaps and honest restart. Source implements the relevant machinery. Extend or compose with explicit follow-up triggers/delivery and a schedule contract; do not label polling as user scheduling. |
| openspec/specs/tool-approval-workflow/spec.md | Covers policy pause, decisions and replay. Establish executable fresh-process disposition, not only UI replay. No UI changes are part of this assessment. |
| openspec/specs/surreal-memory-workflow-mirror/spec.md | Defines workflow mirror storage and source-of-truth boundaries. It does not by itself specify assistant memory privacy, retention or personalization. |
| openspec/changes/afc-c09-team-admission-runtime/specs/durable-team-execution/spec.md | Already specifies reservations, ownership fences, uncertainty and safe continuations. Supervisor recovery must extend this contract rather than weaken non-expiring claims. This is an active change artifact, not assumed archived canonical spec. |
| Existing changes ch07-durable-cost-history, postgres-credential-store, run-scoped-credentials-and-mcp-servers | Reconcile ownership and status before creating overlapping changes. Their existence does not prove global budget durability or tenant-safe credential identity. |

Each implementation change will require at least one OpenSpec delta, including release-only changes. No product delta is authored prematurely in Assess. Analyze must decide which existing changes to extend and which narrow new capabilities are required.

## Prior context and knowledge gaps

Manual fallback in prior-context.md records:
- “PID explicitly supersedes both.” A prior identity dependency recommendation had been based on superseded crates. This assessment therefore checks current source and pins instead of promoting old prose into architecture.
- “Retain the full owner, including tenant, in host-only run state for later authorization.” This shapes the credential and memory review.

Every entry under prior-context.md's Knowledge gaps heading: the automatic recall helper failed to load, so historical gap inventory completeness is unknown; manual search returned no structured gap entries. Do not infer zero gaps. Optional learning suggestion: /learn-goal UAR durable scheduling and crash recovery. No learning goal was started.

Open design questions for Analyze: required latency/availability profile; local-single-owner versus shared-host deployment; schedule missed-run/DST semantics; notification policy and supported destinations; assistant memory retention/consent; supervisor exclusion evidence. These do not block the source assessment and are not decisions silently made here.

## CROSS-TOOL PROGRESS

Canonical phase progress at assessment entry revision 2590: 0 implementation tasks completed of 0 registered; assess IN_PROGRESS. No other tool's Dot implementation task is recorded. Run-wide 116/129 refers to other work and is not Dot progress.

Main moved during the session: frontend lockfile/build compatibility repairs precede d4e819d1; PR #378 advances to 2cc9a182 with two Docker-file changes. Product source findings R2–R9 are unchanged in the inspected paths from the supplied baseline.

OpenSpec 1.14.1 managed refresh succeeded. Generated integration updates from phase initialization remain in the working tree; no authored spec is intentionally replaced. Existing relevant local work is preserved, and product code has not been edited for this assessment. Local branch is main.

## BUILD HEALTH AND VERIFICATION

- Local runtime build: UNKNOWN. No Cargo, frontend build, unit, integration or browser suite was run during Assess, consistent with phase-gated repository rules.
- Current container publication: UNKNOWN/in progress at evidence capture; historical failure confirmed, current fixes present in source.
- Tier 0 policy check before and after fast-forward: pnpm github-actions-policy:validate → “GitHub Actions policy validation passed (deployment workflows only; Pages publisher: docs.yml).”
- Tier 0 document whitespace: git diff --check → exit 0, no output.
- Test inventory: PARTIAL, execution coverage percentage UNKNOWN. tests/test_checkpoint.rs exercises save/load/list and a graph node, not a fresh-process HTTP resume proof. tests/run_cost_settlement.rs covers recording/disabled/unpriced/budget alert behavior, not durable replay. tests/credentials_api_integration_test.rs covers masking, rotation, deletion and different users, not the identical-subject tenant case. Observer and live memory test sources exist; their names do not prove acceptance.
- No Tier 1/2 product completion or Tier 3 release certification is claimed. Assessment artifact checks and independent review are separate from product verification.

## CONSTRAINT CHECK

No production guards, retries, dependencies or behavior changes were added. All proposed hardening traces to reported findings or the real tenant/host-effect/restart boundaries. Product verification stays local; Actions may publish and validate deployment only.

Architecture pin reconciliation is needed: versions.toml declares liter_llm_commit c5c6caac and prometheus_skill_system_commit ad5c82c6, while checked-out gitlinks are a6047386 and f4bef455. The mismatch is observed; no dependency behavior is inferred from it and no pin is changed here.

constraints.md is absent (N/A). Canonical KBD JSON is used. The older Markdown waypoint remains stale, as recorded at initialization; it was not hand-edited. Memory recall's helper lookup failed, direct memory fallback succeeded, and model-preflight initially failed helper discovery but succeeded when given the installed shared-library root. Exact producer model identity is unavailable to this session, so review must not claim verified cross-model separation.

## GOAL PROGRESS

Goal numbering follows goals.md:
1. PARTIAL — six intended behaviors now defined and mapped; no full assistant certification.
2. PARTIAL — Assess performed; Analyze/Plan/Execute/Reflect remain.
3. PARTIAL — priority and preservation boundaries documented; reliability not repaired here.
4. PARTIAL — old lockfile/schema issues superseded by landed changes; both digests/health unverified.
5–12. NOT MET as delivery goals — R2–R9 are source-supported open work with candidate acceptance.
13. PARTIAL — existing foundations revalidated by source; callback/native outcomes retain stated evidence limits.
14. MET for provenance — preserved input and baseline compared with current main.
15. PARTIAL — verification policy followed; end-to-end acceptance remains for execution.

Assessment completion does not complete the phase. Next stage: /kbd-analyze dot-feature-support. Prioritize validating the source findings and defining the scheduling/follow-up/memory contracts before implementation planning.

## Review record

Source/deployment snapshots are in assessment-evidence.json. Sycophancy screening and isolated adversarial review receipts accompany this artifact under sycophancy/ and review/assess/. Review uses a fresh REST request; exact producer identity is unavailable, so cross-model independence is unverified. The handoff is handoffs/assess.handoff.json and retains this limitation. Artifact review does not independently reproduce the source findings.

ASSESSMENT COMPLETE

