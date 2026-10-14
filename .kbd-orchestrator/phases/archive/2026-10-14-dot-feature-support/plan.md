# PLAN: dot-feature-support

Project: universal-agent-runtime
Date: 2026-10-11
Stage: Plan; baseline main 2cc9a1829167411e01a8198072f60892d70a233d
OpenSpec available: YES (managed 1.14.1 refresh verified; authored paths unchanged)
Reconciled operator-selected scope: 20 changes (existing 01–17 plus 18–20), with canonical registration recorded in implementation-sequencing.json. Existing immutable task titles are historical; effective task descriptions in OpenSpec and the sequencing decision govern future execution. Changes 01–06 retain historical completed receipts; 07 implementation is recorded complete and its verification remains pending. Child architecture analysis received M3 critique and K3 PASS on the revised evidence packet; see child analysis.md and review/architecture/. This reviews the architecture analysis, not this complete parent plan or product behavior. OpenSpec and canonical task-sequencing reconciliation is recorded by the child execution/handoff artifacts.

## Outcome and boundaries

Both local operation and shared hosting are required equally for the first release. Each must provide memory, proactive follow-ups, scheduled work, connected tools, approvals, recovery, and all nine [issue #379](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379) protocol/settings/release requirements. This plan extends the existing runtime; it does not build an assistant UI or claim exact OpenAI product parity. Runtime findings remain source-supported and unreproduced until full-phase production-boundary verification. Existing image publication proves only the analyzed revision. Kubo's [Topology B conformance request](https://github.com/GQAdonis/kubo/blob/feat/ipfs-mcp-server/mcp/uar/requests/CONFORMANCE.md) requires all nine in one published installable artifact; items 1, 8 and 9 are only its reader minimum.

The uncomfortable thing: this is a multi-change reliability program, not a scheduler patch. Shared-host safety depends on real atomic storage and trusted owner exclusion. If either deployment profile cannot prove those properties, first-release certification stays blocked even when the features appear to work. Complexity estimates are relative planning bands, not delivery promises.

## First-release decisions

| Profile / surface | Accepted scope | Acceptance |
|---|---|---|
| Local | Host-managed server/sidecar with durable embedded SurrealKV; authenticated HTTP management and governed MCP/native tools | All six assistant capabilities and all nine #379 requirements, fresh-process restart, persisted approvals/work and exclusion of dead owner plus descendants |
| Shared | At least two runtime processes using the same durable networked SurrealDB, with authenticated HTTP/MCP surfaces and a trusted supervisor | The same six and all nine #379 requirements plus equal-subject tenant/issuer isolation, concurrent admission, cross-process revocation/budgets and failed-owner exclusion |
| Existing Postgres | Preserve existing behavior and implement credential/key/accounting guarantees here; observer/checkpoint-dependent assistant certification is explicitly unsupported until parity exists | Real Postgres tests for modified existing features; capability discovery must reject unsupported operations |
| Direct embedded Runtime SDK | No new blanket assistant parity promise; expose unsupported operations explicitly | Existing behavior preserved; HTTP Rust client wrappers can reach server APIs |
| In-memory / unattested remote | Development only for these durable contracts | No durability certification or silent production fallback |

Platform packaging is not platform certification. Change 11 must bind supervisor evidence to the deployment manager actually used in each tested profile and record its exclusion operation; an operator assertion without independently verifiable exclusion is insufficient. Windows native packaging history is preserved, but does not prove the new recovery contract. Unsupported supervisor environments remain explicit, never automatic lease-based recovery. Backend recommendations are settled for this plan; changing them requires a plan revision and matching evidence matrix.

Product defaults chosen for this plan: one-shot UTC and recurring IANA-zone schedules; no overlap; downtime coalesces to the latest due occurrence with visible skipped range; nonexistent DST time is skipped and repeated time uses the earlier instant. Schedule edits create a revision and cannot silently reinterpret old occurrences. Quiet hours defer delivery until the next permitted instant, coalescing follow-ups per rule/source revision; cancellation/revocation prevents new dispatch, while already dispatched uncertain effects remain held. First-release follow-ups use an owner conversation inbox, with optional already-supported connectors only after explicit connector qualification. No new notification integrations. Automatic assistant memory capture is opt-in and requires explicit retention selection; existing explicit memory writes retain their behavior. No arbitrary RRULE compatibility, unlimited scheduling throughput, universal exactly-once external delivery, or silent legacy-ownership inference.


Recurrence input contract: a tagged object accepts once(at: RFC3339 timestamp with UTC offset), daily(time: HH:mm, timezone: IANA identifier), or weekly(weekdays: nonempty unique ISO weekdays 1–7, time: HH:mm, timezone: IANA identifier). These are proposed wire fields, not existing APIs. Reject seconds, invalid zones/dates, empty weekday sets, cron/RRULE strings and past one-shot times at creation. A timezone or recurrence edit creates a new revision effective at commit; old unadmitted occurrences are cancelled, while an already admitted run retains its original instant and authority checks. Daily/weekly wall times use the specified skip/earlier DST policy. Source/docs timing analysis must preserve this schema; executable fit proof is deferred to the full-phase boundary.

Quiet hours input contract: an optional daily start/end HH:mm pair and explicit IANA timezone; interval is start-inclusive/end-exclusive, crossing midnight when start > end. Reject start = end; absence disables quiet hours. Evaluate membership by converting each candidate UTC instant to that zone, so both repeated wall-clock instants are quiet if their wall time is inside the interval. Release at the earliest future real instant outside it; a nonexistent end boundary advances to the first real instant outside the window. A policy/zone edit increments its revision and re-evaluates queued delivery before dispatch. Example: 22:00–07:00 suppresses 22:00 and 06:59 but permits 07:00; America/Chicago weekly Sunday 02:30 skips the spring gap and Sunday 01:30 selects the earlier autumn instant. Integration tests must use exact dates from the installed timezone rules and retain those fixture dates in evidence.

## Resolved operator choices

Operator decision (2026-10-11): use the existing native service managers locally—macOS LaunchAgent com.prometheus.universal-agent-runtime, Linux systemd uar.service, and Windows SCM as named in the native-service-deployment contract—and Kubernetes for shared hosting. Keep qualification evidence separate per platform; packaging/template checks do not certify recovery. Shared acceptance uses at least two UAR runtime Pods backed by the same durable networked SurrealDB.

The trusted host adapter records supervisor identity, runtime incarnation, process/container identities, containment of descendants, exclusion observation, and a single-use challenge tied to the current owner epoch. Evidence is accepted only over an authenticated host/controller boundary, never from an agent. A replacement process started automatically by a native manager must still acquire a new authorized epoch before effects. Qualification verifies actual root and descendant termination, not just a manager's stopped status.

The Kubernetes adapter binds evidence to cluster, namespace, Pod UID, node UID and container identities. On a reachable node, request controlled termination and obtain authenticated runtime-level evidence that all containers and governed descendants are stopped before reclaim. A missing Pod, force deletion, lease expiry or an unreachable node is insufficient. If node/runtime termination cannot be observed, recovery stays blocked until a trusted infrastructure controller supplies independently verifiable node exclusion evidence; support for a particular power-fencing provider is not implied. Preserve unknown external effects for reconciliation even after exclusion. Qualification must record actual adapter mechanisms and access and prove these cases before enabling reclaim.

Operator decision (2026-10-11): coalesce busy-time occurrences to the latest pending occurrence until the previous run is safely resolved. Maintain one durable overlap guard per complete owner and stable schedule identity, spanning every schedule revision. Claimed/queued execution, running work, pending approval, recoverable suspended work, cancellation in progress, and uncertain execution/effects hold the guard. Restart, wall-clock expiry and an edit do not release it. Release requires a durable terminal disposition with the previous execution excluded and its effects resolved; when required accounting remains unknown, admission must still respect its reserved liability.

While the guard is held, atomically advance the schedule cursor and replace the unadmitted pending slot only with a later due instant for the current revision, recording coalesced/skipped range and reason. On safe release, recheck revision, pause/cancel state, owner/destination authority, budget and approval, then atomically claim at most one current pending occurrence. Schedule edits cancel the old revision's unadmitted slot but retain the cross-revision guard; only due occurrences from the new revision can fill it. Cancellation clears pending intent; pausing prevents admission while retaining current valid intent for the normal coalescing policy on resume. Test competing scanners, multiple due instants during running/approval/unknown states, restart and a revision edit; no intermediate occurrence is executed and no second run starts before safe resolution.

## Ordering and ownership

All writes remain in trusted host services. No kernel gains mutation authority. Changes are production slices followed by deferred verification tasks. Finish every planned Dot production implementation item, including 18–20, migrations, operational controls, documentation and the 17 evidence runner, before any product test, lint, format, typecheck, compiler probe, build, QA detector, validator or real-path probe. Record an explicit implementation-complete inventory first; a later plan revision extends that same boundary. Existing 01–06 receipts remain historical and cannot certify the final combined revision. The 07 verification task remains pending. Bounded implementation can proceed concurrently only with disjoint files and settled contracts; assign one writer per file/build target, especially `src/server.rs`, `src/uar/runtime/manager.rs`, persistence and migrations. If 01 awaits operator pin resolution, independent production work may proceed under listed dependencies. 05 still requires resolved dependency inputs. Dependency numbers below are exact prefixes of the listed IDs. For future production task scheduling, a dependency means its required production contract is implemented, not that its deferred verification task has passed. Formal change completion and final 17 acceptance still await that verification. If KBD enforces change-complete prerequisites and will not allow this split, the lead must revise the canonical task graph before Execute; completing verification early to unlock a successor is forbidden.

Existing work reconciliation: postgres-credential-store supplies the encrypted store and startup wiring, with its real Postgres test still pending; 02/03 add identity/key guarantees and own fresh production-path proof without declaring that old task complete. ch07-durable-cost-history supplies history only; 09/10 supersede its fire-and-forget enforcement assumption while retaining historical rows. run-scoped-credentials-and-mcp-servers supplies secret reattachment and narrowing; 06 must preserve those semantics and run relevant integration regressions after implementation, not repeat its obsolete pre-implementation test ordering. afc-c09-team-admission-runtime supplies claims/epochs/uncertainty; 11 adds supervisor exclusion rather than rebuilding admission. Do not archive, complete, or reassign another phase's tasks. Preserve existing tool-approval-workflow and local-scoped-observers requirements through additive deltas. Reuse offline-reproducible-build, tenant-isolation, auth-key-management, tool-call-protocol, model-path-resiliency, multi-tenant-isolation, product-capability-support-matrix and product-validation-evidence rather than create parallel capabilities. Change 12 carries the full existing per-user isolation requirement as MODIFIED, preserving every scenario while replacing subject-only memory wording with complete verified owner identity; other additions are ADDED. Existing offline-build and genuine-inference evidence requirements remain in force; controlled protocol endpoints do not satisfy genuine provider certification.

Read plan-prior-context.md: effective-pin provenance puts 01 first; full-owner and legacy namespace lessons govern 02/12; consumer-surface reachability requires 16; prior overlapping-plan lessons define the ownership boundaries above. Automatic recall helper failed; direct scoped recall succeeded, historical completeness is unknown.

## Ordered changes

### 1. dot-01-release-provenance — Reconcile release inputs and preserve reproducible builds

- Gap / capability_gap_id: R1; library: none; domain/build work.
- Depends on: NONE.
- Scope/ownership: `versions.toml`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `.dockerignore`, `.github/workflows/deploy.yml`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity M / Medium; policy class frontier; customer value HIGH.
- Observed: The frozen-install and schema fixes already landed; policy pins disagree with effective sources and Docker omits --locked.
- Delivery: Prepare an exact intended-versus-effective pin reconciliation for the operator; do not edit versions.toml. Once policy resolution is recorded, align source/lock/build inputs, retain frozen pnpm installs and enforce locked Cargo resolution without redoing landed fixes. Preserve embedded schema inclusion and deployment-only workflow policy.
- Acceptance: From a clean recorded checkout, demonstrate frozen dependency closure and locked production packaging. Capture source/gitlinks and effective skill provenance. Publication/health of the final phase revision belongs to change 17; the existing digests prove only the analyzed revision.
- OpenSpec: offline-reproducible-build, additive requirement “Release artifacts have traceable locked inputs”.
- Backend task 1 (display 1.1): Reconcile intended release inputs and implement approved locked packaging.
- Backend task 2 (display 2.1): Verify clean source closure and packaging provenance locally.

### 2. dot-02-principal-credentials — Isolate credentials by complete verified principal

- Gap / capability_gap_id: R2; library: cand-006.
- Depends on: NONE.
- Scope/ownership: `src/uar/security/claims.rs`, `src/uar/security/middleware.rs`, `src/uar/api/credentials.rs`, `src/uar/security/credentials/resolver.rs`, `src/uar/security/credentials/store.rs`, `migrations`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Credential APIs and resolver collapse the verified identity to user_id.
- Delivery: Introduce a host-derived versioned issuer/tenant/subject key in a namespace disjoint from legacy subjects, including explicit local issuer namespace. Carry it through create/list/read/delete/resolve/use and caches on SurrealDB and Postgres. Quarantine ambiguous legacy rows; provide auditable authenticated mapping with conflict detection and resumable migration, preserving ciphertext and scope precedence.
- Acceptance: Exercise the real credential API and provider resolution with equal subjects across tenants/issuers, raw subjects matching encoded keys, direct IDs, rotation/deletion and two processes. Prove mapped migration is idempotent and unmapped rows cannot be claimed.
- OpenSpec: tenant-isolation, additive requirement “Provider credentials use verified complete owner identity”.
- Backend task 1 (display 1.1): Implement isolate credentials by complete verified principal.
- Backend task 2 (display 2.1): Verify isolate credentials by complete verified principal through production boundaries.

### 3. dot-03-durable-api-keys — Persist API keys and cross-process revocation

- Gap / capability_gap_id: R7; library: cand-006.
- Depends on: dot-02-principal-credentials.
- Scope/ownership: `src/server.rs`, `src/uar/security/api_keys.rs`, `src/uar/persistence`, `migrations`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity M / Medium; policy class frontier; customer value HIGH.
- Observed: Server startup selects in-memory API-key storage.
- Delivery: Wire durable production key storage for both existing database providers, retaining hashed secrets, authority metadata, expiry and owner identity. Read authoritative revocation/expiry before admission; storage outage cannot fall back to memory or stale allow decisions. Preserve explicit development mode.
- Acceptance: Create a key through management, restart and authenticate, revoke from runtime A and deny new admission on B, then restart both and deny again. Cover expiry, database outage and unauthorized management; do not expose key hashes or plaintext in receipts.
- OpenSpec: auth-key-management, additive requirement “API-key authority survives restart and revocation is current”.
- Backend task 1 (display 1.1): Implement persist API keys and cross-process revocation.
- Backend task 2 (display 2.1): Verify persist API keys and cross-process revocation through production boundaries.

### 4. dot-04-truthful-tool-outcomes — Preserve MCP failure and uncertain effects

- Gap / capability_gap_id: R3; library: cand-001.
- Depends on: dot-02-principal-credentials.
- Scope/ownership: `src/mcp/registry.rs`, `src/llm/orchestrator.rs`, `src/uar/runtime/tool_admission`, `src/uar/compiler/collaboration/connector_effect`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity M / Medium; policy class frontier; customer value HIGH.
- Observed: MCP transport Ok can classify isError=true as execution success.
- Delivery: Propagate application failure and original content through registry, ToolEnd, host admission and durable connector receipts. Keep transport failure distinct from post-dispatch outcome unknown; only successful receipts advance success dependencies. Reuse existing effect classification/reconciliation and never broaden retry authority.
- Acceptance: Use real scoped and direct MCP dispatch with isError=true, successful calls, pre-dispatch transport failure and disconnect after an external effect. Assert every consumer agrees on outcome, content survives and unknown work is not replayed or marked successful.
- OpenSpec: tool-call-protocol, additive requirement “Tool execution preserves application and uncertain outcomes”.
- Backend task 1 (display 1.1): Implement preserve MCP failure and uncertain effects.
- Backend task 2 (display 2.1): Verify preserve MCP failure and uncertain effects through production boundaries.

### 5. dot-05-stream-terminal-contract — Require valid provider stream completion

- Gap / capability_gap_id: R4; library: cand-007.
- Depends on: dot-01-release-provenance.
- Scope/ownership: `src/llm/liter_driver.rs`, `src/llm/anthropic_driver.rs`, `src/llm/orchestrator.rs`, `vendor/git/liter-llm`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Conditional terminal enforcement and native per-chunk decoding permit false completion.
- Delivery: Define terminal semantics for each actually selected provider protocol and enforce them on graph and legacy routes. Incrementally frame bytes/SSE/JSON including UTF-8 and CRLF in the native path; reuse inspected Liter framing without gratuitous vendored rewrites. Surface malformed/unfinished streams as errors, preserving cancellation and tool-call semantics.
- Acceptance: Through production orchestrator plus controlled protocol endpoints, split every relevant boundary and test empty-buffer EOF without terminal, trailing partial event, malformed event, cancellation and valid terminal. Inventory every supported route; unsupported routes must not advertise completion guarantees.
- OpenSpec: model-path-resiliency, additive requirement “Supported streams require protocol completion evidence”.
- Backend task 1 (display 1.1): Implement require valid provider stream completion.
- Backend task 2 (display 2.1): Verify require valid provider stream completion through production boundaries.

### 6. dot-06-fresh-process-resume — Restore authorized runs from durable checkpoints

- Gap / capability_gap_id: R5; library: cand-006.
- Depends on: dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract.
- Scope/ownership: `src/uar/api/routes.rs`, `src/uar/runtime/manager.rs`, `src/uar/persistence/mod.rs`, `src/uar/persistence/providers/surreal.rs`, `tests/test_checkpoint.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Resume requires a live run before loading its checkpoint; unsupported methods can succeed as no-ops.
- Delivery: Persist a complete owner-bound resume envelope atomically with the checkpoint and load it before live-manager lookup. Reconstruct through host admission using protected history/current bindings and explicit host secret/resource reattachment. Replace persistence no-op defaults with unsupported errors; never deserialize runnable authority from caller input. Before reconstruction can execute, acquire a durable CAS claim bound to source run, checkpoint revision and executor incarnation; duplicate resume returns the same continuation identity or an explicit conflict, never new execution. Reuse the existing epoch/ownership vocabulary. A predecessor that may still execute or has uncertain effects remains blocked until trusted exclusion/reconciliation; change 11 supplies abrupt-owner exclusion, so 06 alone does not certify abrupt takeover.
- Acceptance: After durable proof of orderly predecessor shutdown and joined descendants, launch a new manager against the same durable backend, list/resume the checkpoint and verify authorized continuation. Abrupt-loss cases must deny takeover at this change boundary; successful killed-owner takeover is verified in 11. Deny cross-owner, altered artifact/bindings and missing host resources; explicit unsupported on Postgres until implemented, with zero execution. Race authorized resume requests from two separate runtimes and prove one continuation/effect. Kill the claimant before and after effect dispatch; a second claimant cannot replay uncertain work.
- OpenSpec: dot-checkpoint-resume, additive requirement “Checkpoints restore authorized execution in a fresh process”.
- Backend task 1 (display 1.1): Implement restore authorized runs from durable checkpoints.
- Backend task 2 (display 2.1): Verify restore authorized runs from durable checkpoints through production boundaries.

### 7. dot-07-approval-recovery — Recover pending approvals without stale authority

- Gap / capability_gap_id: Approvals; library: cand-001.
- Depends on: dot-06-fresh-process-resume.
- Scope/ownership: `src/uar/runtime`, `src/uar/api/routes.rs`, `openspec/specs/tool-approval-workflow/spec.md`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity M / Medium; policy class frontier; customer value HIGH.
- Observed: Persisted approval rows do not alone reconstruct a safe executable continuation.
- Delivery: Connect existing durable approval records to restored run/checkpoint disposition. Expose current pending/expired/invalidated decisions through existing approval reads/events; claim approved continuation once under current owner, binding and policy. Preserve timeout/denial semantics and invalidate stale approval rather than replaying a captured future.
- Acceptance: Restart while pending, read it through the shipped interface and decide on a second runtime. Race duplicate decisions, expiration, revoked authority and changed bindings; assert at most one approved effect and no effect for invalidated/denied decisions.
- OpenSpec: tool-approval-workflow, additive requirement “Recovered approvals require current executable authority”.
- Backend task 1 (display 1.1): Implement recover pending approvals without stale authority.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify recover pending approvals without stale authority through production boundaries. This remains pending; interrupted historical work is not acceptance.

### 8. dot-08-durable-ingestion — Commit upload content and ingestion work together

- Gap / capability_gap_id: R6; library: cand-006.
- Depends on: dot-02-principal-credentials.
- Scope/ownership: `src/uar/api/knowledge.rs`, `src/uar/rag/ingestion_worker.rs`, `src/uar/persistence`, `migrations`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Accepted uploads retain bytes only in memory and ignore enqueue failure.
- Delivery: Persist immutable source bytes and owner-scoped job metadata in one supported-backend transaction before 202. Workers discover committed jobs independently of the in-memory queue, claim idempotent document/chunk work and expose durable failed/retryable states. Use the database initially; do not add an object store. Preserve upload limits and tenant authorization.
- Acceptance: Crash after acceptance, before dispatch and during indexing; restart workers and prove recoverable bytes plus one logical job/chunk set. Failed commit returns no acceptance; no-worker acceptance is permitted only with a durable discoverable job. Verify delete/cancel prevents later ingestion.
- OpenSpec: dot-durable-ingestion, additive requirement “Accepted uploads own durable recoverable ingestion work”.
- Backend task 1 (display 1.1): Implement commit upload content and ingestion work together.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify commit upload content and ingestion work together through production boundaries.

### 9. dot-09-idempotent-settlement — Durably settle usage without duplicate charging

- Gap / capability_gap_id: R8 settlement; library: cand-006.
- Depends on: dot-02-principal-credentials, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract.
- Scope/ownership: `src/uar/runtime/run_cost.rs`, `src/uar/persistence`, `migrations`, `tests/run_cost_settlement.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Detached ledger writes can outlive terminal completion and have no idempotency identity.
- Delivery: Replace detached acceptance with durable idempotent settlement keyed by stable run/call/attempt and scope identity. Preserve per-call/team charging without double counting. Separate execution outcome from accounting state; a persistence outage leaves durable reconciliation work or visible unresolved accounting, never an invented settlement.
- Acceptance: Replay settlements and crash between provider usage, persistence and terminal publication; assert one charge per logical scope settlement on both database providers. Verify existing legacy rows remain history and are not ambiguously deduplicated or fabricated into current enforceable spend.
- OpenSpec: dot-spend-settlement, additive requirement “Usage settlement has stable durable identity”.
- Backend task 1 (display 1.1): Implement durably settle usage without duplicate charging.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify durably settle usage without duplicate charging through production boundaries.

### 10. dot-10-durable-budget-admission — Enforce aggregate budgets across restart and workers

- Gap / capability_gap_id: R8 enforcement; library: cand-006.
- Depends on: dot-09-idempotent-settlement.
- Scope/ownership: `src/uar/runtime/cost_budget.rs`, `src/uar/runtime/manager.rs`, `src/uar/compiler/collaboration/team_execution`, `src/uar/persistence`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Enforcing trackers start empty and independent processes can each admit spend.
- Delivery: Make aggregate reservation/admission atomic in durable storage with owner scope, limit generation and stable settlement reference. Restore enforcing state before admission, reconcile team/global reservations without duplicate liability and retain unknown spend until reconciliation. Migrate legacy history only where provenance permits; block new budgeted work if the aggregate cannot be established.
- Acceptance: Two real runtime processes race for the last available budget; only affordable admission wins. Restart with outstanding/unknown reservations, interleave limit changes and settle twice; verify aggregate bounds, explicit outage denial and no release of uncertain liability.
- OpenSpec: dot-durable-budgets, additive requirement “Budget admission uses shared durable reservations”.
- Backend task 1 (display 1.1): Implement enforce aggregate budgets across restart and workers.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify enforce aggregate budgets across restart and workers through production boundaries.

### 11. dot-11-supervisor-fencing — Recover dead owners only after trusted exclusion

- Gap / capability_gap_id: R9; library: cand-001.
- Depends on: dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission.
- Scope/ownership: `src/uar/compiler/collaboration/team_execution/ownership.rs`, `src/uar/compiler/collaboration/team_execution`, `src/server.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: An abruptly dead owner cannot record the existing orderly-join evidence.
- Delivery: Add host-supervisor-issued exclusion evidence bound to owner incarnation and monotonic epoch. Local supervisor must terminate and reap the managed process group/tree; shared-host supervisor must attest infrastructure-level exclusion of the old execution domain, not a heartbeat or PID. Authenticate the evidence and CAS ownership transfer; preserve non-expiring claims and existing uncertainty reconciliation. Apply the exclusion protocol to ordinary checkpoint continuation claims as well as team owners; 06 creates fail-closed claims and this change enables safe abrupt takeover.
- Acceptance: Kill owner with active descendants; deny reclaim until exclusion evidence covers root/children, reject replayed/wrong-incarnation evidence and stale executor effects. Transfer only safe queued work. Exercise real local and shared supervisor adapters; unavailable infrastructure evidence blocks shared recovery certification. After proven supervisor exclusion, successfully resume an ordinary checkpoint and a safe queued team task in a fresh runtime; repeat the competing-resume race with old-epoch effect attempts denied.
- OpenSpec: dot-supervisor-recovery, additive requirement “Owner recovery requires trusted exclusion before epoch advancement”.
- Backend task 1 (display 1.1): Implement recover dead owners only after trusted exclusion.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify recover dead owners only after trusted exclusion through production boundaries.

### 12. dot-12-memory-isolation — Apply complete owner scope across memory surfaces

- Gap / capability_gap_id: D3 isolation; library: cand-005.
- Depends on: dot-02-principal-credentials.
- Scope/ownership: `src/uar/memory/service.rs`, `src/uar/memory/scopes.rs`, `src/uar/memory/mcp_server.rs`, `src/uar/api/memory.rs`, `src/uar/tools/memory.rs`, `src/uar/memory/context_builder.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Legacy HTTP, tools and background memory paths need one complete ownership contract.
- Delivery: Apply the verified principal namespace to write/search/direct-ID/update/delete/history and context retrieval through HTTP, MCP/native and background paths. Reuse MemoryService and existing index. Migrate only proven owner mappings, quarantine ambiguous legacy memory and ensure retrieved text never carries authority.
- Acceptance: Equal subjects across issuer/tenant plus malicious direct IDs cannot access or inject another owner's memory through any supported surface. Verify migration conflicts, history isolation, background capture scoping and prompt retrieval provenance using real store and service.
- OpenSpec: multi-tenant-isolation; MODIFIED full existing requirement “Per-user data is isolated by authenticated identity” with every scenario preserved and complete-owner memory wording; ADDED “Assistant memory is scoped by verified complete owner” covers the additional direct-ID/history/background boundary.
- Backend task 1 (display 1.1): Implement apply complete owner scope across memory surfaces.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify apply complete owner scope across memory surfaces through production boundaries.

### 13. dot-13-memory-controls — Persist capture consent retention and deletion

- Gap / capability_gap_id: D3 controls; library: cand-005.
- Depends on: dot-12-memory-isolation, dot-08-durable-ingestion.
- Scope/ownership: `src/uar/memory/auto_capture.rs`, `src/uar/memory/background.rs`, `src/uar/memory/service.rs`, `src/uar/api/memory.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: A storage engine alone does not give assistant users consent and deletion control.
- Delivery: Add owner-visible durable capture policy, provenance and retention controls to MemoryService APIs; new assistant automatic capture defaults off until opt-in. Require explicit retention choice at enablement. Use tombstones to immediately suppress deleted/expired content across records/index/history/queued capture; idempotent purge completes without retaining content in audit logs. Preserve existing explicit user memory behavior. Operator refinement: retention accepts {kind: days, days: 7|30|90|365} or {kind: until_deleted}; 24-hour UTC days run from immutable first durable capture. Shortening applies immediately to existing automatic captures; extension applies only to new captures, retains earlier shortening constraints through cleanup/restart, and cannot revive expired/deleted content. Queued capture rechecks current policy atomically at commit.
- Acceptance: Opt-out/revoke during queued capture, expire during retrieval and crash during deletion/index cleanup; no deleted content is returned and purge resumes. Verify both processes honor policy revision, export/history redact removed content and legacy records are not silently assigned new consent. Prove expiry at exact first-capture-plus-duration boundaries, reject mixed/missing/unsupported choices, and shorten then crash/extend before purge without reviving content. Capture retries retain the original anchor.
- OpenSpec: dot-memory-controls, additive requirement “Assistant capture and retention are owner controlled”.
- Backend task 1 (display 1.1): Implement persist capture consent retention and deletion.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify persist capture consent retention and deletion through production boundaries.

### 14. dot-14-durable-schedules — Persist schedules and admit unique occurrences

- Gap / capability_gap_id: D1; library: cand-001, cand-002, cand-006.
- Depends on: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission, dot-11-supervisor-fencing.
- Scope/ownership: `src/uar/runtime/observer`, `src/uar/runtime`, `src/uar/api`, `src/uar/persistence`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Observer polling does not define durable user scheduling.
- Delivery: Decide whether the pinned tokio-cron-scheduler 0.15.1 can serve only as a nonauthoritative timing adapter through source and current documentation analysis; record adoption or return-to-analysis before coding against it. Do not run a timing probe before the full-phase boundary. Implement owner-scoped one-shot UTC and recurring calendar schedules, IANA timezone, revision/cancel/pause and atomic cursor-plus-occurrence admission. Default no overlap; coalesce backlog to the latest due occurrence with recorded skipped range, skip nonexistent DST local times and select the earlier repeated instant. Current authority, budget and approval apply on every admission.
- Acceptance after the full-phase boundary: Prove adapter fit (DST, cancel, downtime, no duplicate authoritative state), then race two scanners, crash at cursor/admission, restart, pause/cancel and change schedule revision. Verify one logical occurrence and receipt, truthful skipped/coalesced state and no replay of uncertain work. A failed fit returns the adapter choice to Analyze and requires a plan revision; it does not justify an early test.
- OpenSpec: dot-durable-schedules, additive requirement “Schedules persist intent and unique governed occurrences”.
- Backend task 1 (display 1.1): Analyze exact-version timing adapter source/docs and record adoption or return-to-analysis decision without execution.
- Backend task 2 (display 2.1): Implement durable schedule management and governed occurrence admission.
- Backend task 3 (display 3.1): After the full Dot production boundary, verify schedule semantics through real restart and competing-runtime paths.

### 15. dot-15-proactive-followups — Deliver governed follow-ups from durable events

- Gap / capability_gap_id: D2; library: cand-001, cand-006.
- Depends on: dot-13-memory-controls, dot-14-durable-schedules.
- Scope/ownership: `src/uar/runtime/observer`, `src/uar/api`, `src/uar/persistence`, `src/uar/compiler/collaboration/connector_effect`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Existing observers need user-controlled follow-up decisions and durable delivery.
- Delivery: Persist opt-in rules bound to owner/source/grant and authorized conversation. Consume durable occurrences and commit admission/suppression reasons plus delivery outbox. First-release destination is a durable owner conversation inbox; existing external connectors are allowed only when individually declared supported with idempotency/reconciliation and approval policy. No new email/push connectors. Recheck quiet hours, consent, cancellation and current destination authority before dispatch.
- Acceptance: Crash at rule evaluation, decision commit and delivery acknowledgement; race scanners and revoke destination access. Prove one logical inbox item, visible suppression, quiet-hour deferral and unknown external outcome held for reconciliation. No universal exactly-once human-receipt claim.
- OpenSpec: local-scoped-observers, additive requirement “Follow-up decisions and delivery remain governed and durable”.
- Backend task 1 (display 1.1): Implement deliver governed follow-ups from durable events.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify deliver governed follow-ups from durable events through production boundaries.

### 16. dot-16-assistant-capability-surface — Expose precise supported assistant operations

- Gap / capability_gap_id: D4; library: cand-001, cand-005.
- Depends on: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups.
- Scope/ownership: `src/uar/api/capabilities.rs`, `src/uar/api`, `src/uar/memory/mcp_server.rs`, `sdks/rust/src/client.rs`, `sdks/rust/src/runtime.rs`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity M / Medium; policy class frontier; customer value HIGH.
- Observed: Current HTTP/embedded method reachability is uneven and blanket parity claims would be misleading.
- Delivery: Expose the six contracts through authenticated server/sidecar HTTP management and supported MCP/native assistant tools using the same host services. Publish per-operation capability descriptors for memory controls, schedule CRUD/pause/cancel, follow-up policy/inbox, tool outcomes, approval reads/decisions and checkpoint recovery. Postgres/in-memory/unattested remote and direct embedded Runtime must explicitly identify unsupported operations; no silent fallback. Provide API examples for consumers, not a new assistant UI.
- Acceptance: Generate an operation-by-profile inventory from actual registrations and exercise every advertised operation through real entry points. Existing HTTP Rust client can wrap APIs; do not certify absent direct embedded APIs. Assert unsupported requests fail explicitly and shared/local owners receive identical contract semantics.
- OpenSpec: product-capability-support-matrix, additive requirement “Assistant capability discovery matches reachable operations”.
- Backend task 1 (display 1.1): Implement expose precise supported assistant operations.
- Backend task 2 (display 2.1): After the full Dot production boundary, verify expose precise supported assistant operations through production boundaries.

### 17. dot-17-assistant-release-acceptance — Certify both profiles and final deployment provenance (final after 18–20)

- Gap / capability_gap_id: All; library: none; domain/build work.
- Depends on: dot-01-release-provenance, dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-09-idempotent-settlement, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-12-memory-isolation, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups, dot-16-assistant-capability-surface, dot-18-a2a-conformance, dot-19-interactive-protocols, dot-20-safe-host-settings.
- Scope/ownership: `tests`, `docs`, `.kbd-orchestrator/phases/dot-feature-support`. Proposed new modules/tests stay within this capability; names are not claims of existing APIs.
- Recommended agent: Codex; complexity L / High; policy class frontier; customer value HIGH.
- Observed: Source evidence and existing image publication do not certify an always-on assistant or Kubo Topology B.
- Delivery before the full-phase boundary: Implement the local production-boundary scenario runner, conformance fixtures, profile documentation and evidence matrix connecting every spec scenario and all nine issue #379 items to source revision, backend, entry point and expected observation. Prepare artifact provenance and installed-artifact instructions without running them. Keep implementation, product evidence, certification and publication statuses independent.
- Acceptance after the full-phase boundary: Run local SurrealKV and two-runtime durable networked SurrealDB through complete always-on journeys, negative tenant/issuer cases, fresh-process recovery, actual supervisor exclusion, two-process admission/revocation/budgets, unknown effects and all nine #379 requirements. Validate A2A/AG-UI/MCP and settings against the selected pinned schemas/policy from the actual installed published artifact under both profiles. Capture the actual tag, exact source revision, artifact checksum or OCI `sha256:` digest, amd64/arm64 provenance and narrow deployment health separately from product results. Card top-level plain `x.y.z` must equal that published tag's version. No release tag is chosen by this plan. Actions runs deployment-specific work only. Missing infrastructure, policy pins, artifact or either profile means blocked certification, never a pass.
- OpenSpec: product-validation-evidence, additive requirement “Both required deployment profiles pass before first release”.
- Backend task 3 (display 3.1, sequence 0): Implement the runner, fixtures, profile docs and evidence matrix before the full production inventory; execute before tasks 1 and 2.
- Backend task 1 (display 1.1): After the full-phase boundary, run both profile journeys and all nine local conformance checks with task 3 runner, then the customer build.
- Backend task 2 (display 2.1): After task 1, record installed published-artifact conformance, publication digests and deployment health.

### 18. dot-18-a2a-conformance — A2A 1.0 card, tasks and actual release version

- Gap / capability_gap_id: issue #379 items 1, 2 and 9; Kubo reader minimum includes 1 and 9.
- Depends on: dot-03-durable-api-keys, dot-06-fresh-process-resume. Its protocol contract must precede 19; final artifact/version acceptance belongs to 17.
- Scope/ownership: `src/uar/api/a2a/**`, card/version provenance and a narrow host route/auth handoff to the sole `src/server.rs` writer. Preserve the production `A2AThreadService` actor executor and host admission; persist or reconstruct A2A task correlation to the existing durable thread rather than create a second task authority. Define ownership and atomic transitions for competing task submissions/restarts; do not infer a race merely from the local map.
- Contract task before implementation: Pin the exact A2A 1.0 public card, methods, roles, parts, task states and auth-error schemas from an authoritative source. Define compatibility handling for existing routes/clients and the card's release-version source. Do not infer a tag from the issue's suggestion.
- Delivery: Serve anonymous `/.well-known/agent-card.json` with a usable interface and 1.0 card shape, retaining `/agent.json` as an alias where compatibility requires it and limiting the auth exemption to public card reads. Adapt task wire methods/states/errors to pinned 1.0, including input-required and auth-required, with durable task correlation across processes. Set the card's plain version from the actual package/release provenance without a hardcoded value.
- Acceptance after the full-phase boundary: Validate the public card and task exchanges against the pinned 1.0 schema, restart and resolve task correlation, preserve 401/403/auth-required semantics and assert the installed published artifact's card version equals its actual plain `x.y.z` tag version in local and shared profiles.
- OpenSpec proposal: extend the A2A protocol capability with at least one 1.0 card/task/version delta; exact capability path to reconcile before emission.
- Backend task 1 (display 1.1): Pin A2A contract and implement card, task and version production behavior.
- Backend task 2 (display 2.1): Verify A2A 1.0 and published-version behavior after the full-phase boundary.

### 19. dot-19-interactive-protocols — A2A approvals, AG-UI interruption and MCP elicitation

- Gap / capability_gap_id: issue #379 items 3, 4 and 6.
- Depends on: dot-04-truthful-tool-outcomes, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-11-supervisor-fencing, dot-18-a2a-conformance. The AG-UI interruption contract and approval policy precede MCP elicitation wiring.
- Scope/ownership: `src/uar/api/a2a/**`, `src/uar/api/adapters.rs`, MCP client handler/bindings, host approval route and the narrow manager policy fix. Give `src/uar/runtime/manager.rs`, `src/server.rs` and `src/mcp/registry.rs` one serial writer each; preserve teammates' changes.
- Observed in the child source analysis: the verified-local governance bypass in `src/uar/runtime/manager.rs` can return before `ApprovalClass::Required` is included in the required decision; existing approval CAS, protected continuation and exact-ID host route already exist. A2A/AG-UI need projections of those records, and the inspected MCP client paths use unit handlers.
- Contract task before implementation: Pin the AG-UI `RunAgentInput`, SSE interruption and `resume[]` schema plus the MCP elicitation version/capability contract. State approval precedence: a descriptor Required decision cannot be bypassed when governance is off; Ask/Deny/current authority still apply. MCP elicitation accept/decline/cancel is an interaction response and does not itself authorize a tool.
- Delivery: Reuse durable approval CAS and protected continuation. Project a pending exact approval ID as A2A input-required; approve resumes once, deny terminates without effect, with governance on or off. Add AG-UI ingress and specified `RUN_FINISHED` interrupt outcome plus `resume[]` mapping to the same host-owned decision. Implement MCP client elicitation handler/capability on every supported transport path using run-scoped bindings and durable interaction state. Do not create another approval ledger, executor or checkpoint state machine.
- Acceptance after the full-phase boundary: Validate A2A and AG-UI wire events against pinned schemas; race duplicate decisions and restarts, prove Required cannot bypass and effects occur at most once; exercise MCP elicitation accept/decline/cancel on each transport and show that acceptance alone never grants tool authorization, in both profiles.
- OpenSpec proposal: add protocol interaction deltas to existing approval, AG-UI and MCP capabilities; at least one delta is mandatory.
- Backend task 1 (display 1.1): Pin interaction contracts and implement approval/AG-UI production behavior.
- Backend task 2 (display 2.1): Implement MCP elicitation against the same host-owned interaction state.
- Backend task 3 (display 3.1): Verify interactive protocols after the full-phase boundary.

### 20. dot-20-safe-host-settings — CORS, safe shipped defaults and effective settings

- Gap / capability_gap_id: issue #379 items 5, 7 and 8; item 8 is in Kubo's reader minimum.
- Depends on: dot-03-durable-api-keys and dot-16-assistant-capability-surface for the final effective aggregate. Its CORS/defaults implementation can proceed earlier under exclusive `src/server.rs` and configuration ownership; final settings projection follows live registry/capability contracts.
- Scope/ownership: host configuration/shipped profiles, `src/server.rs`, settings API, live provider registry and settings synchronization. Do not duplicate the registry or governance authority. The server route/auth writer must be serialized with 18/19 and all other `src/server.rs` work.
- Contract task before implementation: Pin the effective-settings path and stable aggregate response with Kubo; recommended `GET /api/uar/settings/effective` and Kubo's `uar-effective-settings.schema.json` fields (`revision`, `llm.provider_host`, `knowledge_base_embedder.provider_host`, `memory_embedder.provider_host`, optional governance/native tools/MCP fields). Choose documented policy for revision/hash changes and admin Bearer JWT claims; require 401 missing/invalid credential and 403 non-admin. Resolve shipped-profile overrides and no-implicit-model behavior before editing defaults. These are recommendations pending contract resolution, not asserted existing APIs.
- Delivery: Replace permissive CORS with an explicit origin allowlist and empty default that emits no allow-origin. Ship no implicit model, local/off memory embedder, loopback bind and enabled JWT defaults, with deliberate documented deployment overrides. Expose one secret-free, read-only effective aggregate from live provider/settings/governance authorities; Bearer admin access requires no extra credential header. Preserve current settings mutation ownership.
- Acceptance after the full-phase boundary: From each installed published profile, prove empty-default CORS denial and allowlisted origins, effective socket/egress/auth defaults and overrides, 401/403/admin Bearer behavior, exact aggregate schema and revision changes without secrets. Both profiles must meet the same requirement even where explicit shared-host overrides are necessary.
- OpenSpec proposal: add host configuration and effective-settings deltas, including a narrowly scoped new capability if none exists; at least one delta is mandatory.
- Backend task 1 (display 1.1): Resolve host defaults policy and implement CORS/shipped profiles.
- Backend task 2 (display 2.1): Pin settings contract and implement the live effective aggregate plus Bearer admin route.
- Backend task 3 (display 3.1): Verify host settings after the full-phase boundary.

## Execution rounds and activation gate

These are production implementation waves, not verification windows. The existing completed 01–06 source/receipts remain historical; resume 07 production disposition and defer its pending verification. No command in a later wave can treat a prior slice as a product-test boundary.

1. Settle release pins and external A2A/AG-UI/MCP/settings/approval-policy contracts by source and documentation review. Continue independent 08 ingestion, 09 settlement and 12 memory isolation; 18 A2A wire/card work can start once its 1.0 contract is pinned. Assign disjoint file owners first.
2. After listed prerequisites, implement 10 budget admission and 13 memory controls. Implement 20 CORS/defaults early under the sole server/config writer. Implement 11 exclusion and 14 schedules after their authority dependencies; 14 timing fit is source/docs analysis only here.
3. Implement 19 approval/AG-UI after 07, 11 and 18, then its MCP elicitation after the interaction contract; implement 15 follow-ups after 13/14. Shared manager/server/MCP writers hand off serially. These independent slices need not wait for unrelated schedules where their exact dependencies are met.
4. Implement 16 capability discovery after its listed production inputs and 20 effective settings against the final live authorities. Complete 17 runner, docs, profile/configuration and artifact-provenance preparation. Include every added implementation item from any plan revision.
5. Record the explicit inventory that every Dot production behavior, persistence contract, migration, operational setting, required document, conformance fixture and 17 runner is implemented. Only then execute the deferred verification tasks as one serialized real-path gate, fix a failure and rerun its failed gate, build the customer artifact, publish and run installed-artifact acceptance for 17.

Every listed dependency wins over this wave summary. Unattended assistant activation requires the final identity, outcome, recovery, approval and budget evidence. Neither a source-level scheduler fit nor a partial card/settings reader proves certification. Pin-policy resolution and real supervisor infrastructure evidence remain explicit prerequisites; neither can be fabricated. No simultaneous writer for `src/server.rs`, `src/uar/runtime/manager.rs`, migrations, persistence or a build target.

## Verification contract

The full Dot production implementation boundary in `AGENTS.md` controls every future verification step and overrides the older per-edit/per-change Tier 0/1 schedule. Before that boundary, perform source and documentation inspection, contract decisions, implementation and test/runner authoring only; do not run tests, lint, format, typecheck, compiler probes, builds, QA detectors, validators, policy scripts, real-path probes or external product gates. Isolated architecture review is separately authorized in this Plan; it supplies design findings only and cannot certify product behavior. Existing 01–06 receipts are historical, and 07 verification is still pending. Before any future command, the lead inventories every planned production item across 01–20, including revised items, and records that each usable behavior, persistence/migration, contract, operational setting/UI, required document and runner is complete. Uncertainty keeps the boundary closed. If an indispensable early command is proposed, stop and request the operator's explicit exception naming the exact command, unfinished items and limits of the result.

After the recorded boundary, run the planned production integration gate serially through shipped orchestrator, host admission and real persistence collaborators; controlled provider/MCP endpoints may inject protocol failures. Unit/mock-only/filtered-function tests do not count as completion evidence. Existing relevant targets include tests/credentials_api_integration_test.rs, tests/test_checkpoint.rs, tests/run_cost_settlement.rs and tests/integration/live/memory_cases.rs; target names are not claims of current coverage. Include local and shared journeys, every advertised operation, issue #379 all nine and required failure cases. Record exact commands/results and read rust-workspace phase-gated verification before the first Cargo invocation. Run the required customer build for an independently usable artifact, including `pnpm build:mac:arm64` when the phase includes The Boss. Final publication, installed-artifact conformance and deployment health belong to 17 after local acceptance; deployed health is separate from product tests. A future workflow or invoked build/package/deploy script edit triggers the separate GitHub Actions rule requiring `pnpm github-actions-policy:validate` before and after editing. If such an edit remains necessary before the full-phase boundary, this is a sequencing blocker: the lead must obtain the operator's exact-command early-verification exception before that edit, or revise the production plan so the edit is unnecessary. Do not silently run, skip or defer the required pre-edit validator. No unit/integration/browser/lint/typecheck/certification suite may run in Actions, including build side effects.

Evidence must record source revision, effective pins, backend/configuration without secrets, entry point, command, exit status and observed assertions. Each scenario below is a required falsifier, not merely a test title.

- dot-01-release-provenance / Unresolved pin policy: WHEN declared policy and effective dependency sources disagree; THEN packaging certification is blocked pending recorded operator resolution, without silently editing policy or accepting drift.

- dot-01-release-provenance / Reproducible container inputs: WHEN an approved source revision is packaged for either supported Linux architecture; THEN frozen JavaScript installs, locked Rust resolution and required embedded schemas are used and immutable source provenance is recorded.

- dot-02-principal-credentials / Identical subjects: WHEN two authenticated principals share a subject but differ by tenant or issuer; THEN their credential create, read, list, delete and use operations are isolated.

- dot-02-principal-credentials / Legacy ownership: WHEN a legacy credential lacks sufficient owner provenance; THEN it is not granted to a matching subject and only an authenticated audited mapping can migrate it without key collisions.

- dot-03-durable-api-keys / Cross-process revocation: WHEN one runtime revokes a key and another receives a new request using it; THEN the second denies admission using durable current authority.

- dot-03-durable-api-keys / Durable storage outage: WHEN production key storage cannot establish current validity; THEN authentication and management fail explicitly without an in-memory or stale-authority fallback.

- dot-04-truthful-tool-outcomes / Application error: WHEN MCP returns isError=true inside a successful transport response; THEN ToolEnd, admission and durable receipts report application failure with original error content.

- dot-04-truthful-tool-outcomes / Ambiguous external effect: WHEN transport ends after an effect may have occurred without definitive result; THEN outcome remains unknown, success dependencies stay blocked and automatic replay is prohibited.

- dot-05-stream-terminal-contract / EOF without terminal: WHEN a supported provider connection closes without its required terminal event; THEN the runtime reports an incomplete/error outcome rather than successful completion.

- dot-05-stream-terminal-contract / Arbitrary byte boundaries: WHEN valid events including multibyte characters and tool arguments cross transport chunks; THEN the selected production route reconstructs the same events and succeeds only after valid terminal evidence.

- dot-06-fresh-process-resume / Fresh manager: WHEN the original process is gone and an authorized owner resumes a durable checkpoint; THEN the runtime reconstructs from protected persisted metadata after current authority and host-resource checks.

- dot-06-fresh-process-resume / Unavailable capability: WHEN the selected backend cannot persist or load resumable checkpoints; THEN the request fails explicitly and never claims a durable save or a successful empty resume.

- dot-07-approval-recovery / Pending approval after restart: WHEN an authorized owner reads a pending approval after its original process exits; THEN the current approval and continuation disposition remain visible and execution still requires valid current authority.

- dot-07-approval-recovery / Competing decisions: WHEN two processes attempt to consume the same approval or its binding has changed; THEN at most one valid continuation is admitted and stale or expired decisions cannot authorize effects.

- dot-08-durable-ingestion / Crash after acceptance: WHEN the runtime returns 202 and exits before dispatch; THEN source bytes and one owner-bound ingestion job remain discoverable and recover without duplicate indexed chunks.

- dot-08-durable-ingestion / Commit failure: WHEN source or job transaction cannot commit; THEN the upload is not accepted and no metadata falsely promises future processing.

- dot-09-idempotent-settlement / Repeated settlement: WHEN a terminal usage result is persisted twice or replayed after a crash; THEN each applicable budget scope records the logical charge once.

- dot-09-idempotent-settlement / Accounting outage: WHEN execution completes but usage settlement cannot be durably accepted; THEN execution truth is preserved and accounting remains visibly unresolved rather than reported settled.

- dot-10-durable-budget-admission / Concurrent admission: WHEN two runtimes compete for insufficient remaining aggregate budget; THEN atomic reservation admits only work within the configured bound.

- dot-10-durable-budget-admission / Restart with unknown spend: WHEN a runtime restarts while prior work has unresolved spend; THEN enforcement restores the reservation and does not reset the available budget or release unknown liability.

- dot-11-supervisor-fencing / Abrupt owner loss: WHEN a runtime dies without recording orderly shutdown; THEN a trusted supervisor must establish exclusion of its entire execution domain before a new epoch can be admitted.

- dot-11-supervisor-fencing / Stale executor: WHEN an old epoch attempts an effect or forged/replayed exclusion is submitted; THEN host admission rejects it and uncertain previous effects remain unreplayed.

- dot-12-memory-isolation / Cross-surface direct ID: WHEN a principal supplies another owner's memory ID through HTTP, MCP, native tools or history; THEN the operation is denied without revealing content or affecting the owner's record.

- dot-12-memory-isolation / Scoped context retrieval: WHEN memory is retrieved for prompt context or background processing; THEN only currently authorized owner-scoped content is returned and its text cannot grant execution authority.

- dot-13-memory-controls / Capture consent changes: WHEN automatic capture is disabled or consent is revoked while capture work is queued; THEN no new assistant capture is committed under the stale policy.

- dot-13-memory-controls / Deletion across restart: WHEN the owner deletes a memory and the process exits before index cleanup; THEN content becomes immediately unavailable to retrieval/history and durable purge resumes after restart.

- dot-14-durable-schedules / Two scanners after downtime: WHEN two runtimes find several due occurrences of the same non-overlapping schedule; THEN cursor advancement and one latest coalesced occurrence commit atomically with a stable admission identity and visible skipped range.

- dot-14-durable-schedules / DST and revocation: WHEN a recurring local time is nonexistent or repeated, or its owner revokes the schedule before admission; THEN nonexistent time is visibly skipped, repeated time selects the earlier instant, and revoked work is denied under current authority.

- dot-15-proactive-followups / Policy suppresses follow-up: WHEN an observed event matches a rule but consent, source grant or destination permission is revoked; THEN no disclosure or effect occurs and an owner-visible scoped suppression reason is retained.

- dot-15-proactive-followups / Delivery interrupted: WHEN a committed follow-up is interrupted during delivery; THEN the durable inbox item is recovered once logically, while uncertain external dispatch is held for reconciliation rather than retried blindly.

- dot-16-assistant-capability-surface / Advertised operation: WHEN a client discovers an assistant operation for its backend and entry point; THEN the operation is reachable under the same host authority and durable semantics claimed by discovery.

- dot-16-assistant-capability-surface / Unsupported profile: WHEN a client uses an operation unavailable on its backend or direct embedded entry point; THEN discovery marks it unsupported and invocation fails explicitly without an ephemeral substitute.

- dot-17-assistant-release-acceptance / Only one profile passes: WHEN local operation passes but shared-host evidence is missing or fails, or vice versa; THEN first-release assistant certification remains incomplete.

- dot-17-assistant-release-acceptance / Final release evidence: WHEN all local product acceptance gates pass and the final artifact is published; THEN the exact tag/source revision, checksum or OCI digest, backend/entry-point matrix, both architecture digests, installed-artifact #379 results and deployment health are recorded separately from product test results.

- dot-18-a2a-conformance / Public 1.0 card and tasks: WHEN an anonymous reader requests `/.well-known/agent-card.json` and an authenticated client exchanges tasks; THEN the card, interface, methods, parts, roles and states validate against the selected pinned A2A 1.0 schema, while only the public card read bypasses authentication.

- dot-18-a2a-conformance / Durable task and version: WHEN a task is queried after process restart or the published artifact is installed; THEN existing durable thread authority resolves the task, and the card's plain `x.y.z` equals the actual tagged artifact version.

- dot-19-interactive-protocols / Required approval: WHEN a Required tool is reached through A2A with governance enabled or disabled; THEN the tool remains uninvoked, the exact approval ID is projected as input-required, one valid approval resumes at most once, and denial has no effect.

- dot-19-interactive-protocols / AG-UI and elicitation: WHEN `RunAgentInput` suspends for approval or an MCP server requests elicitation; THEN pinned AG-UI events expose the interrupt and `resume[]` resolves the host-owned interaction, while MCP accept/decline/cancel is durable and acceptance never alone grants tool authorization.

- dot-20-safe-host-settings / Browser and shipped defaults: WHEN a shipped profile starts without overrides; THEN no arbitrary origin is allowed, no implicit model or remote memory embedder is selected, bind is loopback and JWT is enabled; documented shared-host overrides remain explicit and authenticated.

- dot-20-safe-host-settings / Effective admin view: WHEN a reader requests the pinned effective-settings path; THEN no credential receives 401, a non-admin receives 403, and a Bearer admin receives a secret-free live aggregate matching the selected stable schema whose revision changes with effective settings.

- dot-06-fresh-process-resume / Concurrent checkpoint resume: WHEN two authorized runtimes resume the same checkpoint revision concurrently; THEN one durable incarnation-bound claim admits at most one continuation, and uncertain predecessors remain blocked until trusted exclusion and reconciliation.
- dot-14-durable-schedules / Validated recurrence revision: WHEN an owner submits daily or weekly HH:mm plus IANA timezone, then changes that timezone; THEN invalid forms are rejected, a valid edit creates a new revision, and unadmitted old-revision occurrences are cancelled without reinterpreting admitted work.
- dot-15-proactive-followups / Overnight quiet hours: WHEN a rule uses daily 22:00–07:00 quiet hours in its explicit IANA zone; THEN 22:00 is suppressed and 07:00 is permitted, with overnight and repeated-hour membership evaluated at the candidate real instant and queued work rechecked after policy edits.

## Task model assignments

Evidence date 2026-10-11: Codex desktop exposes gpt-6.1-sol and gpt-6-astra with high/xhigh reasoning through collaboration.spawn_agent for fresh contexts; codex-cli 0.160.1 is installed. Catalog exposure is observed; no proposed execution worker was launched and configured execution access is not proven by a listing. gpt-6.1-sol is provisionally selected for source/spec integration and controlled validation because the current tool catalog identifies it for coding work; there is no task-specific comparative benchmark. gpt-6-astra/high is the documented native alternative for difficult unresolved architecture, not an automatic switch. xhigh is allocated only to authority/transaction/recovery reasoning; high to bounded integration/provenance work. No project model policy or budget override is present. These are recommendations, not provider changes or capability rankings.

Worker contract W: fresh Codex worker via collaboration.spawn_agent with fork_turns=none, selected model/effort, repository cwd /Users/gqadonis/Projects/prometheus/universal-agent-runtime, exact change/task scope and inspected paths above, shell/file tools, relevant Rust/security/protocol skills and AGENTS.md. Worker must not revert others; return patch plus source/contract handoff to the KBD lead, which alone owns task transitions. Selection must be revalidated before dispatch. Plan does not launch workers. No worker runs a product test, check, build, validator, QA detector or real-path probe before the explicit full-phase implementation inventory. An isolated Plan architecture critic receives only artifacts and criteria, not generation history; it is not product verification. Current child M3/K3 findings remain pending.

| Phase path | Change ID | Backend task ID | Requirements | Provider/model | Reasoning effort | Rationale and dated evidence | Harness and route | Worker launch and handoff | Native alternative | Availability and verification | Prerequisites |
|---|---|---|---|---|---|---|---|---|---|---|---|
| dot-feature-support | dot-01-release-provenance | 1 | Host authority, source integration; R1; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; operator pin resolution |
| dot-feature-support | dot-01-release-provenance | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; operator pin resolution |
| dot-feature-support | dot-02-principal-credentials | 1 | Host authority, source integration; R2; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-02-principal-credentials | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-03-durable-api-keys | 1 | Host authority, source integration; R7; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-03-durable-api-keys | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-04-truthful-tool-outcomes | 1 | Host authority, source integration; R3; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-04-truthful-tool-outcomes | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-05-stream-terminal-contract | 1 | Host authority, source integration; R4; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-05-stream-terminal-contract | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-06-fresh-process-resume | 1 | Host authority, source integration; R5; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-06-fresh-process-resume | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-07-approval-recovery | 1 | Host authority, source integration; Approvals; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-07-approval-recovery | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-08-durable-ingestion | 1 | Host authority, source integration; R6; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-08-durable-ingestion | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-09-idempotent-settlement | 1 | Host authority, source integration; R8 settlement; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-09-idempotent-settlement | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-10-durable-budget-admission | 1 | Host authority, source integration; R8 enforcement; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-10-durable-budget-admission | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-11-supervisor-fencing | 1 | Host authority, source integration; R9; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; real supervisor exclusion adapter |
| dot-feature-support | dot-11-supervisor-fencing | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; real supervisor exclusion adapter |
| dot-feature-support | dot-12-memory-isolation | 1 | Host authority, source integration; D3 isolation; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-12-memory-isolation | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-13-memory-controls | 1 | Host authority, source integration; D3 controls; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-13-memory-controls | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-14-durable-schedules | 1 | Source/docs timing-adapter fit; D1; text, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; exact-version source reasoning; no early probe | Codex desktop, native; CLI 0.160.1 observed | W; source/doc decision to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; resolve timing choice by inspection |
| dot-feature-support | dot-14-durable-schedules | 2 | Host authority, source integration; D1; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-14-durable-schedules | 3 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-15-proactive-followups | 1 | Host authority, source integration; D2; text, shell, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-15-proactive-followups | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-16-assistant-capability-surface | 1 | Host authority, source integration; D4; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; cross-file contract reasoning; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-16-assistant-capability-surface | 2 | Real-path failure evidence; backend/process scope; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-11 catalog; integration evidence synthesis; provisional fit | Codex desktop, native; CLI 0.160.1 observed | W; this task scope; evidence to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; listed dependencies |
| dot-feature-support | dot-17-assistant-release-acceptance | 3 | Production runner/docs/evidence matrix; all nine #379; text, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; source and artifact authoring before boundary | Codex desktop, native; CLI 0.160.1 observed | W; implementation inventory to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; all production dependencies, no execution |
| dot-feature-support | dot-17-assistant-release-acceptance | 1 | Final local production-path profile evidence and customer build; both profiles; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; only after full-phase inventory | Codex desktop, native; CLI 0.160.1 observed | W; exact receipts to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; all 01–20 production complete and completed runner task 3 |
| dot-feature-support | dot-17-assistant-release-acceptance | 2 | Final real-path/product and installed-artifact evidence; both profiles; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; only after full-phase inventory | Codex desktop, native; CLI 0.160.1 observed | W; exact receipts to KBD lead | OpenAI/gpt-6-astra, high | Catalog observed; worker untested | Recheck route; all 01–20 production complete and actual published artifact |
| dot-feature-support | dot-18-a2a-conformance | 1 | Pin A2A 1.0; implement card/tasks/version; #379 1/2/9; text, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-12 source analysis; protocol/host authority reasoning | Codex desktop, native; route recheck | W; exclusive A2A files, server handoff | OpenAI/gpt-6-astra, high | Proposed; worker untested | Pin exact schema; 03/06 production contracts |
| dot-feature-support | dot-18-a2a-conformance | 2 | Deferred A2A 1.0/published-version evidence; #379 1/2/9; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; phase-only verification | Codex desktop, native; route recheck | W; exact receipts to KBD lead | OpenAI/gpt-6-astra, high | Proposed; worker untested | Full-phase inventory and actual artifact |
| dot-feature-support | dot-19-interactive-protocols | 1 | Pin AG-UI/approval; implement A2A approval and AG-UI; #379 3/4; text, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-12 source analysis; reuse ledger/continuation | Codex desktop, native; route recheck | W; exclusive adapter/manager handoff | OpenAI/gpt-6-astra, high | Proposed; worker untested | 07/11/18 authority and A2A contracts |
| dot-feature-support | dot-19-interactive-protocols | 2 | Pin and implement MCP elicitation; #379 6; text, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-12 source analysis; separate elicitation/authorization | Codex desktop, native; route recheck | W; exclusive MCP files | OpenAI/gpt-6-astra, high | Proposed; worker untested | 19 task 1 interaction contract; 04 outcome contract |
| dot-feature-support | dot-19-interactive-protocols | 3 | Deferred A2A/AG-UI/MCP real-path evidence; #379 3/4/6; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; phase-only verification | Codex desktop, native; route recheck | W; exact receipts to KBD lead | OpenAI/gpt-6-astra, high | Proposed; worker untested | Full-phase inventory and both profiles |
| dot-feature-support | dot-20-safe-host-settings | 1 | Host CORS and shipped defaults; #379 5/7; text, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 source analysis; explicit profile defaults | Codex desktop, native; route recheck | W; exclusive server/config files | OpenAI/gpt-6-astra, high | Proposed; worker untested | Resolve default/override policy; serial server writer |
| dot-feature-support | dot-20-safe-host-settings | 2 | Pin settings shape; live effective admin API; #379 8; text, files | OpenAI/gpt-6.1-sol | xhigh | 2026-10-12 source analysis; compose registry/governance authority | Codex desktop, native; route recheck | W; exclusive server/settings files | OpenAI/gpt-6-astra, high | Proposed; worker untested | 16 capability inventory; Kubo shape/path agreement |
| dot-feature-support | dot-20-safe-host-settings | 3 | Deferred host/settings evidence; #379 5/7/8; text, shell, files | OpenAI/gpt-6.1-sol | high | 2026-10-12 scope decision; phase-only verification | Codex desktop, native; route recheck | W; exact receipts to KBD lead | OpenAI/gpt-6-astra, high | Proposed; worker untested | Full-phase inventory and both installed profiles |

## OpenSpec emission and handoff

Every new change receives proposal.md, design.md, tasks.md and at least one spec delta (ADDED, plus the explicitly required full MODIFIED block for 12). Candidate evidence for adapt recommendations is copied into each relevant design. Existing 01–17 backend task IDs/history are preserved. New 18–20 task IDs were reconciled through typed KBD registration on 2026-10-12; the child canonical-reconciliation-receipt.json records the successful registrations. Existing task IDs and historical completion remain intact. The assignment table controls model selection subject to the operator’s subsequent big-model instruction; current worker ownership is in execute-dispatch.json. These are change IDs, not shell commands to recreate existing artifacts:

`/opsx:new dot-01-release-provenance`

`/opsx:new dot-02-principal-credentials`

`/opsx:new dot-03-durable-api-keys`

`/opsx:new dot-04-truthful-tool-outcomes`

`/opsx:new dot-05-stream-terminal-contract`

`/opsx:new dot-06-fresh-process-resume`

`/opsx:new dot-07-approval-recovery`

`/opsx:new dot-08-durable-ingestion`

`/opsx:new dot-09-idempotent-settlement`

`/opsx:new dot-10-durable-budget-admission`

`/opsx:new dot-11-supervisor-fencing`

`/opsx:new dot-12-memory-isolation`

`/opsx:new dot-13-memory-controls`

`/opsx:new dot-14-durable-schedules`

`/opsx:new dot-15-proactive-followups`

`/opsx:new dot-16-assistant-capability-surface`

`/opsx:new dot-17-assistant-release-acceptance`

Proposed additions, after contract resolution: `dot-18-a2a-conformance`, `dot-19-interactive-protocols`, `dot-20-safe-host-settings`. Each needs its own spec delta before registration. The 17 proposal must include the new final acceptance dependency and all nine issue #379 scenarios.

The new spec/plan artifacts and canonical task registrations are complete. Read-only KBD metadata reconciliation established clean projections and is not product verification. Under the full-phase boundary, defer `openspec validate`, product validation scripts, QA detectors and all other product verification commands until the implementation inventory is complete; no checklist here authorizes an earlier validator. The separately authorized isolated M3 critic/K3 judge architecture review may change this draft before canonical registration. Preserve historical assessment/analysis and unrelated local work. No new runtime guards are installed in Plan; proposed guards trace to R2–R9, issue #379 observed gaps, or explicit tenant/host-effect/consent boundaries. Planning completion does not establish implementation, product evidence, release certification or publication completion.

## Review findings and operator resolution

Final isolated review: PASS with 0 critical findings and 2 warnings after two revisions. Earlier critical findings were the absent concurrent checkpoint claim and a circular abrupt-recovery acceptance condition; the revised plan requires a CAS continuation claim, orderly-restart proof in 06 and successful abrupt takeover only in 11. Exact producer identity remains unavailable, so cross-model independence is unverified.

- **Change 11 selection resolved:** existing native service managers locally and Kubernetes shared hosting. The adapter evidence/containment contract above is binding. Actual adapter qualification and infrastructure access remain execution gates; selecting a manager does not prove exclusion.
- **Change 14 policy resolved:** coalesce to the latest pending occurrence while a stable owner/schedule guard is held, including approval and uncertainty; the guard spans revisions. The explicit transaction, release and edit semantics above and corresponding spec scenarios are required acceptance.

The 2026-10-11 operator responses settled only supervisors and busy scheduling in the original 17-change plan. The 2026-10-12 operator decision adds all nine #379 requirements to this Dot release, so registered 18–20 tasks, dependencies and 17 final acceptance supersede the earlier "no task IDs change" statement. Existing registered IDs and 01–06 receipts remain historical. Real adapter qualification, deferred exact-version scheduler fit, pin-policy resolution and final production-boundary evidence remain required. The decisions are not runtime certification.

Clarification review (2026-10-11): PASS, no critical findings. A separate WARNING remains for change 13: define allowed retention durations/indefinite choice, expiration anchor and policy-revision effects on existing and queued captures before implementing retention. This operator clarification settles supervisors and busy scheduling only; it does not silently choose retention semantics. Carry that bounded refinement into Execute preparation for 13.


## Canonical scheduling reconciliation (2026-10-12)

`implementation-sequencing.json` owns the effective production/verification categories and source-task prerequisites for this plan revision. KBD supports immutable task registration and status transitions, not editing existing categories/dependency fields. Every old verification obligation remains unpassed; none is cancelled to inflate completion. Dot07 task2 is blocked by the full-phase boundary. The lead explicitly starts eligible production tasks; their in-progress state selects the real canonical cursor ahead of deferred verification. Do not use a completed-change gate as a prerequisite for source work. After each source handoff, select the next source task from this manifest and check its implementation prerequisites by source/handoff inspection.

New changes18–20 use sequence16 (ID tie ordering places them after16 and before existing17); existing change17 sequence17 cannot be rewritten. Dot17 new backend3 uses sequence0 for source-only runner implementation; original1/2 remain final evidence/publication. The immutable decision records the manifest hash and effective task-description precedence. Runtime aggregate implementation counters still include legacy verification tasks; the explicit production inventory, not those counters alone, controls the final boundary.


Sequencing transcription correction (2026-10-12): decision `dot-full-phase-sequencing-reference-correction-20261012` supersedes the original manifest digest. The three truncated `dot-18-a/1` references now name canonical `dot-18-a2a-conformance/1`. Current manifest SHA256 is `88d63bc2a6d8bd5eca63abaa89dc9b656fafc809fe5ea17e1b2682c3e1fc52c9`. Production scope and no-verification boundary are unchanged.

Effective-title correction (2026-10-12): historical 01–06 production task titles retained superseded early-check language despite the controlling scheduling note. Their effective OpenSpec/manifest wording now states full-phase timing directly; original completed receipts and canonical immutable titles remain historical. Manifest SHA-256: `62fd0bc70aca0d9ecb1adf410e7c7023083738d7405887335fa79b7616fc4655`. No status or acceptance changed.

Dot14 bounded Analyze correction: reject cron0.15.1 after exact-source fixed-offset finding; use existing UAR scanner and exact already-locked Jiff0.2.35 for timezone conversion only. Source report `implementation/dot14-timing-fit.md` supplies API/contract evidence. Cargo manifest and root dependency list authored without resolution/check; versions.toml unchanged. All timing runtime evidence remains deferred. Native Windows timezone availability is an explicit qualification obligation, not silently excluded.

Effective-settings source dependency correction: Dot20/2 requires03/1 and20/1, not16/1: its read composes pre-existing registry/config/governance authority. Dot16 inventories the finished endpoint. No interface/acceptance removed. Manifest SHA-256 `602264e5ac2e0e8f53b0ba5159f0eaa57d1e42830e9fabb757a0e17f70bfdd53`; this supersedes the effective-title correction hash.

Operator-approved recovery availability (2026-10-12): native takeover may require a confirmed reboot of the same stable host when descendant exclusion cannot be proven. Same-boot takeover is available only through an implemented, qualified containment mechanism. Service stopped, PID absence, timeout, lease expiry and Pod deletion are insufficient. Kubernetes requires actual execution-domain/container/node fencing. A reboot proves prior execution excluded, not the outcome of external actions; uncertain effects and unsettled accounting remain held. Ship the concrete controller/observation path, not merely a receipt schema. Qualification runs only at the full-phase boundary.

Session owner continuity follow-up (2026-10-13): source inspection found bare-subject session/history and conversation-policy ownership paths remaining within the existing isolation requirement. Canonical completed task12/1 cannot reopen; task12/3 records the correction without erasing its historical handoff. Preserve catalog identity, introduce disjoint full-principal session storage and quarantine ambiguous legacy rows. The production denominator is now24; 12/3 and17/3 must both complete before the source inventory or any verification.

Operator source-completion target (2026-10-12): continue until every expanded Dot production item is codecomplete, with testing pending. Final validation uses this machine and existing LKE context know-me, reusing existing Envoy TLS/HTTPRoutes. Dot17/3 includes source-bound installedPod lifecycle, controlledpeer adapters and isolatedLKE chart overrides; these are implementation, not evidence that anything is installed. The24-entry source inventory will carry verification_authorized:false. No tests/builds/checks/probes/validators/reviews/deployments start in this goal. Destructive clock/reboot/node scenarios retain their explicit isolated-environment authorization and proof requirements; the selected normalvalidation locations do not waive them.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


## Superseding delivery acceptance — operator decision 2026-10-14

The operator directed that code-complete UAR delivery stop being blocked by qualification suites and instead be accepted through actual use by agents and dependent projects. The 109-case qualification matrix, synthetic/conformance/fault suites, native reboot and clock/disk qualification, and review-suite certification are no longer prerequisites for this release. Preserve their actual historical passes/failures and unexecuted status. This decision does not mark a cancelled check passed and does not remove any implemented feature or any of the nine reliability or nine issue #379 requirements.

The release exit criteria are usable production implementation, a build of the actual current source, installation on existing authorized capacity, and a useful task executed by an existing configured agent/provider through UAR. Record the real turn, the actual tools/approvals used and their independently observed result, memory/document/accounting persistence actually exercised, exact artifact/source identity and consumer availability. Do not force unused features into artificial scenarios. Report unused protocols, adversarial edge cases and crash/reboot guarantees as unverified rather than blocking unrelated consumers. Keep actual defects, failed builds and failed consumer use actionable; code presence alone is not a passing consumer result.

Necessary artifact building and deployment health remain delivery work. All production implementation and this scope reconciliation precede any build or consumer-use execution. Gates remain serial; no new qualification scripts, matrix expansion, cache-copy workaround or paid resources. Existing node/volume/VM approvals and immutable cleanup deadlines remain. Independent teammates may inspect and fix observed code problems without making their review a new release gate. Merged-main releases for other projects may proceed using their explicitly selected source; unfinished Dot qualification is not a dependency-wide freeze.

Cargo cache IDs are reusable namespace identifiers, not source-attestation hashes. The delivery may reuse the genuinely populated existing architecture/toolchain/profile namespace. Actual source identity, dependency pins, Cargo fingerprints and produced artifact digests remain independently recorded. Do not relabel an old source archive, copy compiled outputs under a fabricated cache identity, prune caches or claim reproducibility from cache reuse.
