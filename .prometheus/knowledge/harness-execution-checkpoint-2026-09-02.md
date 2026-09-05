# Harness execution checkpoint — 2026-09-02

## Position and delivery delta

Execute: `skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture › codex-harness-comparative-analysis`. KBD revision 1783.

The single active phase contains five implementation rounds and ten changes. Implementation has reached the end of round 3, through change 6. There are 73/182 real tasks checked off and 109 remaining. Two alias ledger entries are excluded: one complete and one in progress. Only changes 1 and 2 are fully complete; changes 3–6 have pending test/evidence/acceptance tasks.

The operator changed the execution order to implementation first, tests only at phase end. No compilation, test run, formatting run, OpenSpec validation, parity report, live smoke, or completed-change critic acceptance was performed for changes 3–6 during this pass. The runtime allowlist file exists because production shadow comparison reads it; the parity corpus and its report remain unfinished.

## Files changed in the implementation pass

Paths below are relative to `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`.

| File | Change |
| --- | --- |
| `Cargo.toml` | Direct pinned backon dependency. |
| `Cargo.lock` | Root dependency entry for the already-resolved backon version. |
| `src/config.rs` | Stream idle timeout, skill activation/reattachment settings, and legacy/shadow/typed harness mode. |
| `src/llm/provider_error.rs` | Typed provider errors and retry classification. |
| `src/llm/anthropic_driver.rs` | Typed HTTP/transport/stream failures. |
| `src/llm/liter_driver.rs` | Typed request and stream-item errors. |
| `src/llm/external_driver.rs` | Preserve supplied typed errors; classify unknown external failures. |
| `src/llm/mod.rs` | Provider-error exports. |
| `src/llm/orchestrator.rs` | Backon retry, ordered failover, idle timeout, per-step skill projection, immutable request/tool snapshots, attribution, and shadow manifests. |
| `src/llm/router.rs` | Qualified preferred-model routing after candidate health/capability filtering. |
| `src/server.rs` | Idle-timeout hydration, harness configuration wiring, cursor handling, and removal of the HTTP-only memory prepend. |
| `src/skills_api.rs` | Preserve public skill-list responses through scored-result projection. |
| `src/uar/api/routes.rs` | Skill attachments, typed failures, and typed execution-request adapter. |
| `src/uar/api/openapi.rs` | Run attachment/failure schema. |
| `src/uar/api/skills.rs` | Adapt scored matching to existing skill responses. |
| `src/uar/compiler/ir.rs` | Preserve explicit artifact instructions. |
| `src/uar/compiler/to_artifact.rs` | Forward instructions instead of hardcoding an empty vector. |
| `src/uar/compiler/completeness.rs` | Required fixture-field update for instructions. |
| `src/uar/domain/skills.rs` | Scored candidates, accepted IDs, threshold/margin enforcement, and preference tie-break. |
| `src/uar/eval/targeted.rs` | Adapt the existing evaluator to accepted skill results. |
| `src/uar/runtime/manager.rs` | Compose typed requests/turns, route and prepare resources, host-gated activation, memory fragments, retention, terminal outcomes, and legacy/typed selection. |
| `src/uar/runtime/mod.rs` | Export prompt and turn modules. |
| `src/uar/runtime/prompt/mod.rs` | Prompt module exports. |
| `src/uar/runtime/prompt/fragment.rs` | Authority, retention, stable fragment hashes, and reclaimable bodies. |
| `src/uar/runtime/prompt/assemble.rs` | Deterministic section order and model-dialect rendering. |
| `src/uar/runtime/prompt/manifest.rs` | Body-free metadata and shadow evidence. |
| `src/uar/runtime/prompt/interrupted.rs` | Explicit partial-turn marker. |
| `src/uar/runtime/native_skills/mod.rs` | Activation-tool module export. |
| `src/uar/runtime/native_skills/activate_skill.rs` | Run-local, model-only activation capability. |
| `src/uar/runtime/skills/mod.rs` | New skill-runtime module exports. |
| `src/uar/runtime/skills/catalog.rs` | Budgeted complete-eligible-set catalog, fair description trimming, and omission count. |
| `src/uar/runtime/skills/activation.rs` | Eligibility/scope/dependency/limit checks and run-local activation state. |
| `src/uar/runtime/skills/retention.rs` | Bounded request-only reattachment after summary/history reduction. |
| `src/uar/runtime/skills/usage.rs` | Request-snapshot attribution and graph-driver body/usage adapter. |
| `src/uar/runtime/skills/registry.rs` | Deterministic listing and preserved vector scores. |
| `src/uar/runtime/skills/service.rs` | Scored matching contract; no below-threshold activation fallback. |
| `src/uar/runtime/turn/mod.rs` | Typed assembly exports. |
| `src/uar/runtime/turn/request.rs` | Shared owned execution request. |
| `src/uar/runtime/turn/plan.rs` | Pre-I/O decisions and attachment deduplication. |
| `src/uar/runtime/turn/resolved.rs` | Immutable turn/step records with non-serializable credentials. |
| `src/uar/runtime/turn/contributors.rs` | Seven fixed contributor stages and narrow-only validation. |
| `src/uar/runtime/turn/builtin.rs` | Built-in artifact, policy, memory/RAG, skill, tool, context, and lifecycle contributors. |
| `src/uar/runtime/turn/shadow.rs` | Redacted comparison with exact-fingerprint exemptions; comparison never chooses dispatch. |
| `src/uar/settings/manager.rs` | Idle-timeout and harness schemas/defaults. |
| `src/uar/settings/resilience_policy.rs` | Retry backoff policy and idle-timeout overrides. |
| `src/uar/telemetry/metrics.rs` | Shadow recall, exact invocations, and per-skill attribution separate from ordinary totals. |
| `src/uar/telemetry_disabled.rs` | Matching no-op functions for telemetry-disabled profiles. |
| `tests/settings_persistence.rs` | Required fields added to an existing configuration fixture; no test execution. |
| `tests/fixtures/turn_parity/intentional_deltas.json` | Empty production-consumed allowlist; no unmeasured exemptions. |
| OpenSpec task files and KBD projections | Implementation-task receipts only; evidence tasks remain open. |
| `.prometheus/session-log.md`, `.prometheus/decisions.md`, this checkpoint | Append-only decisions, risks, and handoff. |

## Verification actually performed

`git diff --check` returned no output and exit code 0 after each edit. `rg` was used to inspect runtime call sites, removed fallback text, configuration defaults, and adapters. These are static checks, not substitutes for the prescribed Rust verification tiers. KBD task receipts advanced to revision 1783. A memory-mirror write failed during skill-retention task start; local append-only records preserve the checkpoint.

## Scope and guards

No unrelated product features were added. Existing unrelated changes to `tests/uar_integration.rs` and KBD projection noise were preserved. No files were deleted, no new commits were created, and `versions.toml` was not changed during this pass.

New guards protect named boundaries: skill eligibility/enablement, maximum active skills, missing/conflicting MCP dependencies, contributor attempts to widen access, invalid matcher confidence values, oversized catalogs/bodies, and provider establishment/idle failures. Tiny catalog windows produce an explicit budget failure rather than an oversized catalog. No speculative retry categories or default-mode flip were added.

## Unverified and uncomfortable

- Compilation and formatting are unverified. Existing test callers of `SkillMatchResult` still require phase-end migration; the old liter stream-error test also expects the previous successful error-event shape.
- Full behavior remains unverified for graph-native model activation/tool execution, step snapshots on graph calls, all-unhealthy routing, cross-provider fallback credentials, and reconnecting the chat POST rather than the run-specific stream endpoint.
- The 2,000-entry catalog scenario must distinguish a minimum catalog that fits from extreme pressure where the specification explicitly permits omissions.
- Shadow independently renders prompt sections but reuses prepared legacy reduction to avoid a second paid summary call. It does not independently verify the reducer algorithm. No parity success is claimed.
- Changes 7–10 are not implemented. The required sandbox decision before change 7 is not present in the decision log. Do not infer it from a generic instruction to continue.

## Next action

Ask the operator whether change 7 should implement OS-native sandboxing for stdio children or reject `sandboxed: true` at config load. Record that choice before starting `projected-mcp-runtime` task 0.1. Continue the remaining production tasks, then perform the consolidated phase-end test/evidence work. Do not flip the default without both recorded parity and live-smoke evidence.

## Subsequent checkpoint — round 4, change 9 production tasks

KBD revision 1803: **78/182 real tasks checked off**, 104 remaining. Change 9 is 5/14: production tasks 2.1, 2.2, 3.1, 3.2, and 3.3 are checked; all nine test/evidence tasks remain open. Still only two changes are fully accepted. No build, tests, formatter, or OpenSpec validation ran in this continuation.

The preceding next-action paragraph was too broad if read as a stop on all of round 4. The authoritative plan gates **change 7**, and schedules changes 8 and 9 independently in the same round. The sandbox choice remains unresolved, but it does not prevent work on those independent changes. Change 9 was implemented sequentially in the current checkout because its uncommitted change-6 dependency is here and no other implementation writer is active.

### Files changed in this continuation

| File | Change |
| --- | --- |
| `src/uar/runtime/project_instructions.rs` | Trusted root-to-cwd walk, ordered configurable names, overrides, canonical containment, lazy subtree-on-read hook, and retention of accessed subtrees only while root/config/trust remain unchanged. |
| `src/uar/runtime/world_state/sections.rs` | Stable environment/time/permissions/project-instruction IDs and texts; substitutable clock; positive configurable 60-second buckets. |
| `src/uar/runtime/world_state/merge_patch.rs` | Pure RFC 7386 generator/applier, atomic arrays, null deletion, and explicit errors for unrepresentable null-valued member changes. |
| `src/uar/runtime/world_state/contributor.rs` | Host-only baseline with exact history witnesses, full/diff/removal contributions, and reservation for either a delta or full replay. |
| `src/uar/runtime/world_state/runtime.rs` | Session/clock/discovery host bridge; speculative shadow contributions do not commit. |
| `src/uar/runtime/world_state/mod.rs`, `src/uar/runtime/mod.rs` | Module exports. |
| `src/config.rs` | Discovery and world-state configuration; reject path-bearing file names and non-absolute trusted roots. |
| `example.config.yaml` | Document empty trust list, optional names/overrides, root markers, and clock granularity. |
| `src/session/thread.rs` | Tenant-session-owned baseline and accessed instructions, omitted from serialized input; clear/reset invalidation. |
| `src/uar/runtime/context/reduce.rs` | Explicit `history_rewritten` report for structural, budget, summary, and normalization changes. |
| `src/uar/runtime/prompt/fragment.rs` | Host markers and escaping, without changing policy fragment content or hashes. |
| `src/uar/runtime/turn/request.rs` | Host-selected optional working directory; does not confer trust. |
| `src/uar/runtime/turn/contributors.rs` | Carry the selected world-state contribution as owned assembly output. |
| `src/uar/runtime/turn/builtin.rs` | Reserve context before reduction and separate active-body reattachment so it follows world-state insertion. |
| `src/uar/runtime/turn/shadow.rs` | Do not duplicate world-state history in the pinned base prompt; use current rather than stale world-state metadata on later steps. |
| `src/uar/runtime/manager.rs` | Host discovery, shared legacy/shadow capture, typed context registration, selected-path session commit, and context reservation. |
| `src/server.rs` | Pass operator discovery and world-state settings to the manager. |
| `src/uar/runtime/native_skill.rs` | Backward-compatible contextual execution hook supplied by the governed host. |
| `src/uar/tools/file_tools.rs` | Successful allowed native reads load newly entered subtree instructions. |
| `src/llm/orchestrator.rs` | Supply native execution context and update world state after reduction/before body reattachment on subsequent tool-loop calls. |
| OpenSpec task file, KBD projections, append-only memory | Five production task receipts, decisions, and this checkpoint. |

### Evidence and limits

- `git diff --check` returned no output and exit code 0 after each edit. Source call-site inspection found one runtime `RunExecutionRequest` literal and one `ContextStage` construction; both were updated. The KBD progress read reported revision 1803, change 9 = 5/14, and normalized total = 78/182 (same two historical aliases excluded).
- Compilation, formatting, all behavior tests, and critic acceptance remain unverified. Phase-end fixture migration must include the two new `AppConfig` fields and the extended request/context/report types.
- Graph-native tool dispatch and per-call typed snapshots remain change-8 work. Initial manager assembly includes world state on the graph seed, but the old graph driver path does not become a native-tool loop through change 9.
- Shadow compares a shared host snapshot and reuses legacy reduction; later-step world-state metadata is shared host input, not independent proof of the capture algorithm. Real phase-end integration tests must cover clock rollover, checkpoint restore, compaction, nested file access, permissions, and cross-session isolation.
- No unrelated features, dependency changes, workflow edits, commits, deletions, or speculative default flips were added. Trust-boundary guards cover untrusted roots, traversal/symlink escape, host-marker closure, invalid granularity, and world-state context overflow. The agent-runtime-security skill kept subtree discovery downstream of the existing native read authorization and path allowlist.

Next safe implementation: `thread-native-subagents` (change 8, 0/25). Keep change 7 untouched pending the sandbox decision. Do not start the phase-end test sequence until remaining production code is ready, except that change 10 must wait for its explicit parity/live-smoke evidence gate.

## Subsequent checkpoint — change 8 thread contracts

KBD revision 1807: **79/182 real tasks checked off**, 103 remaining. Change 8 task 2.1 is implemented (1/25). The complete phase remains in Execute; only changes 1 and 2 are accepted. Following the KBD single-task turn contract, the next production task is 2.2, fail-closed policy intersection.

### Files changed

| File | Change |
| --- | --- |
| `src/uar/runtime/thread/mod.rs` | Thread/status/result/edge records; inherited owner/root identity; canonical paths; record validation; pending-failure and active-turn transitions; stable list keys; read-only watch handle with explicit premature-publisher-close errors. Result debug output is redacted. |
| `src/uar/runtime/thread/spawn.rs` | Explicit artifact/prompt/task name and default-none history mode, with no model-supplied owner or authorization fields. Forks count user-delimited turns and retain only user messages and final assistant replies, excluding system and tool traffic. |
| `src/uar/runtime/thread/messages.rs` | Typed root/owner/sender/recipient envelope, host-assigned sequence, trigger flag, redacted debug output, and body-only user-message conversion. |
| `src/uar/runtime/thread/limits.rs` | Atomic tree admission with hard ceilings of 4 concurrent children, depth 3, and 16 total. Guards distinguish pre-persistence rollback, uncertain persistence, committed lifetime count, and active-turn release. |
| `src/uar/runtime/mod.rs` | Export the thread module. |
| OpenSpec tasks, KBD projections, append-only memory | Task 2.1 receipt and implementation checkpoint. |

### Evidence, scope, and risks

- `git diff --check` returned no output and exit code 0 after each edit. Static placeholder/unchecked-panic search in `src/uar/runtime/thread` returned no matches. KBD reported change 8 = 1/25 and normalized total = 79/182 at revision 1807, with the same two historical aliases excluded.
- No builds, tests, formatter, or validation suite ran. Compilation, concurrency behavior, history-fork examples, and provider persistence are unverified. These modules are not yet connected to model tools, actors, graph nodes, or A2A.
- No unrelated features, dependencies, workflow edits, commits, or deletions were added. The new guards address the specified identity boundary, explicit fork exclusions, fifth/fourth/seventeenth-child refusal scenarios, and ambiguous database-write cancellation. The security skill kept authorization outside model-controlled spawn data.
- Integration must call `ChildReservation::begin_persistence()` before awaiting the atomic thread+edge write, then `commit()` only on confirmed persistence. `abort_unpersisted()` is valid only on proof of no write. Dropping an in-flight write retains lifetime capacity and requires host reconciliation of that unknown outcome; it must not be treated as a safe retry or a persisted live child.
- `AgentHandle` observes the latest state, not a historical completion log. Callers that need a prior run's outcome after a thread has resumed must use durable run/history records. A closed publisher while nonterminal is an error, never fabricated completion.
- Tokio watch APIs were checked against the official documentation through Context7; no Tokio pin changed (`Cargo.lock` resolves 1.53.1). The relevant source is https://docs.rs/tokio/latest/tokio/sync/watch/index.html.

Next: implement change 8 task 2.2, then task 2.3 persistence and the governed service/adapters. Keep the phase-end test sequence deferred and the default-flip evidence gate intact.

## Subsequent checkpoint — change 8 policy intersection

KBD revision 1811: **80/182 real tasks checked off**, 102 remaining. Raw projection is 81/184; the same two historical aliases (one complete) remain excluded. Change 8 is 2/25. Execute remains in round 4 of 5; changes 1 and 2 alone are accepted. The preceding continuation made production-code progress, not a wait or no-progress turn.

### Files changed

| File | Change |
| --- | --- |
| `src/uar/runtime/thread/policy_intersection.rs` | Added host-owned `ThreadPolicy`, concrete resource intersection, credential and tool identity bindings, sandbox grant intersection, strict budget/rate ceilings, root approval identity, strict nested policy decoding, and a captured narrowed artifact. |
| `src/uar/runtime/thread/mod.rs` | Export the policy intersection module. |
| `openspec/changes/thread-native-subagents/tasks.md` and KBD projections | Original task 2.2 receipt through the qualified-title driver; no alias task added. |
| `.prometheus/decisions.md`, `.prometheus/session-log.md`, this checkpoint | Append-only decision, evidence, and next-task records. |

### Contract and integration notes

- The new restriction-only `uar.thread_policy` artifact extension requires version 1. It accepts optional credential-grant and sandbox restrictions; omission/null inherits, empty collections deny, and unknown fields/versions fail closed. It has no owner, root, approval, or user-authorization fields.
- Credential grants are exact `(target, binding_id)` pairs, where target is a provider or MCP server. A binding is an immutable host-resolved credential/service binding, including explicitly credential-free services. No key values appear in this policy. A child must not re-resolve credentials through child/session/user/system/env fallbacks. Provider routes without an inherited binding fail; unavailable fallback providers are excluded.
- `ThreadPermissions` also requires a complete host tool-ID map, distinguishing Native from MCP `{server_id, tool_name}`. Missing tool bindings fail closed; server identity is never guessed from tool-name syntax. Server-specific tool declarations and credential exclusions remove the corresponding tool IDs.
- Sandbox filesystem and environment IDs name immutable host bindings, not paths or environment lookup instructions. Read/write/network grants intersect; execution mode takes the stronger isolation requirement. The host executor must enforce the resulting bindings or reject the run before execution. This does not implement or decide change 7's OS-native MCP sandbox.
- Existing `budgets` fields are decoded strictly, including nested rate limits, and take minimum ceilings. Zero stays zero; negative/non-finite cost limits fail. Budget counters are not reset or implemented by this pure module; task 5.1 must charge/enforce root-shared usage.
- Base artifact policy and `uar.run_policy` restrictions intersect independently, so an extension cannot erase the base deny list. Parent `All`/`Auto` selections become concrete `Selected`/`None` results: the current manager only enables filters for those latter modes. MCP compiler declarations restrict existing identities; their URL/auth/env definitions are removed from the execution copy, never from the original artifact, and cannot replace inherited connections.
- The root approval run ID and verified owner are private immutable fields inherited by `intersect()`. The type has no deserializer, permissive root default, or approval-grant method. Actual approval transport remains service/adapter work.

### Evidence and uncomfortable limit

`git diff --check` returned no output and exit code 0 after every code edit. Static search for `todo!`, `unimplemented!`, `panic!`, `.unwrap(`, and `.expect(` in the new module returned no matches (rg exit 1). KBD end-task returned revision 1811; the task surface shows 2.1 and 2.2 checked, 23/25 still open.

No builds, tests, formatter, or validation suite ran. The module is exported but has no execution caller yet. Compilation, policy behavior, actual credential/sandbox enforcement, approvals, persistence, and end-to-end child runs remain unverified. No Rust verification tier or whole-change acceptance is claimed. The security skill shaped the host-only binding and authority boundary; KBD's single-task turn contract leaves task 2.3 for the next continuation.

No unrelated features, dependencies, workflow changes, commits, or on-disk deletions. All guards address the specified non-widening/unsupported-policy boundary, missing host identities, or the observed mode-gated filtering behavior. Next: task 2.3 persistence, then the governed service and adapters; keep tests deferred until the phase-end gate.

## Subsequent checkpoint — change 8 thread persistence

KBD revision 1815: **81/182 real tasks checked off**, 101 remaining. Raw projection is 82/184 with the same two historical aliases (one complete). Change 8 is 3/25; only changes 1 and 2 are accepted. Execute remains in round 4 of 5. This continuation implemented task 2.3; no subsequent task was started.

### Files changed

| File | Change |
| --- | --- |
| `src/uar/persistence/agent_threads.rs` | `PersistedAgentThread` with independent storage revision; typed store failures; shared root/child insertion checks, immutable-lineage and turn-transition validation, lookup scope checks, and deterministic thread/edge ordering. |
| `src/uar/persistence/mod.rs` | Export helper module and require six owner-scoped methods: create root, atomically create child+edge, load thread, revision-checked update, list tree threads, list tree edges. No default success/no-op implementation. |
| `src/uar/persistence/providers/memory.rs` | Thread and edge maps under one lock, atomic creation and updates, and shared validation/ordering. Still intentionally process-local, not durable. |
| `src/uar/persistence/providers/postgres.rs` | Transactional insertion, root-first parent/root shared locks, guarded updates, owner-scoped reads, and common decoded ordering. |
| `migrations/20260902000000_agent_threads.sql` | New thread/edge tables, owner-qualified primary/foreign keys, unique canonical paths, and storage revisions. Existing migration runner picks up this file. No deletion cascades. |
| `src/uar/persistence/providers/surreal.rs` | Apply thread schema at connection; atomic creation transactions; parent/root write fences and snapshot checks; revision-checked updates; all-statement error inspection; typed JSON envelopes and common decoded ordering. |
| `migrations/surrealdb/agent_threads.surql` | Idempotent thread/edge table and owner-qualified unique-index definitions. Explicit provider inclusion also covers embedded deployments. |
| OpenSpec task, KBD projections, append-only memory | Original task 2.3 receipt, decision, session summary, and this checkpoint. |

### Contract and integration notes

- Every persistence call takes a verified host owner separately from the record. Loads never fall back to another owner or legacy unscoped key. New-child writes require a matching persisted live parent and root and an exact `AgentEdge::between` result.
- `revision` is a storage compare-and-swap token, independent of `history_revision`. It starts at zero and advances on every successful update, bounded consistently to signed 64-bit storage capacity. History cannot move backwards; identity, root, parent, path, artifact, and creation time cannot change. Terminal results can be replaced only by a new run with a greater history revision.
- Memory uses one lock for records and edges. PostgreSQL creates both in one transaction and checks affected row counts. SurrealDB creates both in one transaction and inspects every statement error; `spawn_fence` is a write-conflict fence on parent/root decision records, not a lifetime count or public revision.
- SurrealDB stores the complete Rust envelope as a JSON string and selects typed strings, avoiding timestamp/unsigned-value coercion and the existing generic value-unwrapping path. PostgreSQL stores the same envelope as JSONB. Both return the caller's successfully persisted snapshot only after the write completes; transport errors do not prove rollback.
- Edge reads capture edges before loading thread records. Lineage is immutable and there is no deletion API, so later concurrent inserts do not invalidate already-read edges. All returned records are validated and sorted by Rust canonical-path/ID keys; database collation is not the ordering contract.
- The service must call `ChildReservation::begin_persistence()` before awaiting creation, commit the permit only on confirmed persistence, and reconcile the exact child ID plus edge after an uncertain write. Duplicate creation is a refusal, not an upsert. Do not release lifetime capacity or retry with a new ID merely because a database call returned an error.
- Source references checked through Context7: [SurrealDB transactions](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/rust/concepts/transaction.mdx), [index syntax](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/query-language/statements/define/indexes.mdx), and [SQLx transactions](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html). The installed SurrealDB 3.2.4 source confirmed `IndexedResults`, `take_errors`, and `check`; no dependency pin changed.

### Evidence and uncomfortable limit

`git diff --check` returned no output and exit code 0 after every edit. Static method inventory found all six required methods in the trait and each of the three providers. Placeholder/unchecked-panic search in the new shared helper returned no matches (rg exit 1). KBD end-task returned revision 1815 and change 8 = 3/25.

No builds, formatter, database connections, migrations, validation suites, or tests ran. Compilation, SQL/SurrealQL execution, transactional races, restart recovery, and provider parity remain unverified. In particular, the new schema has not been applied to a live database. The store methods are implemented but no thread execution service calls them yet. No Rust verification tier or change acceptance is claimed.

No unrelated additions, dependency changes, workflow changes, commits, or on-disk deletions. Guards address the real owner/lineage boundary, half-spawn failure, stale status writes, root/parent cancellation races, and the existing history-revision semantics. The runtime-security skill guided owner scoping; KBD's single-task boundary leaves task 3.1 for the next continuation. Tests remain deferred until phase end. Change 7's sandbox choice and change 10's evidence gate remain open.

## Subsequent checkpoint — change 8 native agent descriptors

KBD revision 1819: **82/182 real tasks checked off**, 100 remaining. Raw projection is 83/184, retaining the same two historical aliases (one complete). Change 8 is 4/25. Only changes 1 and 2 are accepted; Execute is at round 4 of 5, not four accepted rounds. Task 3.1 is complete as descriptor work; task 3.2 has not started.

### Files changed

| File | Change |
| --- | --- |
| `src/uar/runtime/thread/control.rs` | Host-only turn context and root-user grant; required `AgentThreadHost` dispatch contract; strict request decoding, live caller/root/native-binding checks, result scoping, metadata-only receipts, descendant interrupt requests, and cancellable current-state waits. |
| `src/uar/runtime/native_skills/agents/mod.rs` | Five native tool implementations with strict JSON schemas, explicit spawn-authorization descriptions, model-only exposure, effect/approval classifications, and a fresh eligible per-turn registry. |
| `src/uar/runtime/thread/mod.rs` | Export control module. |
| `src/uar/runtime/native_skills/mod.rs` | Export agents module; preserve earlier activate-skill export. |
| OpenSpec task, KBD projections, append-only memory | Original task 3.1 receipt, decision, session summary, and this checkpoint. |

### Contract and integration notes

- All tool descriptions state that spawning requires explicit user or artifact authorization. Spawn eligibility requires an exact `spawn_agent` declaration on the original registered artifact or a verified root-user grant, as well as intersected tool selection and an exact Native binding. Expanded wildcard lists and child message text are not authorization. The context must never receive a narrowed artifact in place of the original authorization source.
- `RootDelegationGrant` is host-created and non-deserializable, bound to owner/root run, with revocation. The host API assumes its caller already verified an explicit root-user decision; no text parser infers one. Concrete adapters must independently recheck live authority at mutation, including revocation and root lifetime.
- `AgentThreadHost` has mandatory operations and no default/no-op implementation. Its scope carries immutable caller and policy, not credentials in tool arguments. Real execution, durable mailbox ordering, persistence-before-publication, exact bindings, root approvals, budget charging, and tracked mutation lifetimes remain service/adapter work.
- Registry instances must stay turn-local. Reusing equivalent descriptors in a global registry can retain another turn's handler through descriptor deduplication. Merge the fresh registry through existing collision checks when wiring the manager.
- Message requests contain body, recipient, and trigger flag only. Receipts omit body; sender/root/owner/sequence come from the host. Lists expose child identity/path/artifact/run/status without prompts, history, credentials, or result bodies. Wait responses include explicitly requested terminal outcomes with run IDs.
- Interrupt accepts descendants only and reports `cancellation_requested` separately from status. Wait defaults to 30 seconds, allows a zero snapshot and at most 60 seconds of observation, uses no spawned polling jobs, and does not cancel children on timeout. Subscription setup precedes that observation timer. Caller checks after setup and completion prevent snapshot paths from bypassing cancellation/stale-turn validation. A prematurely closed nonterminal publisher is an error, not success.
- FuturesUnordered/StreamExt usage was checked through Context7 against futures 0.3.32 documentation: https://docs.rs/futures/0.3.32/futures/stream/futures_unordered/struct.FuturesUnordered.html. No dependency pin changed.

### Evidence and uncomfortable limit

`git diff --check` returned no output and exit 0 after each edit. Static placeholder/unchecked-panic search in the two new files returned no matches (rg exit 1). Cross-module source search confirms native handlers call `AgentToolContext`; it also confirms `registry_for_turn` has no execution caller and `AgentThreadHost` has no concrete implementation yet. KBD end-task exited 0 at revision 1819; its change projection reports 4/25.

No builds, formatter, tests, database operations, OpenSpec validation, or acceptance critic ran. Compilation, schema acceptance, wait/cancellation races, authorization enforcement, and end-to-end agent tools remain unverified. No Rust tier or whole-change acceptance is claimed. The runtime-security skill shaped the explicit host-identity/authority boundary; KBD's single-task turn contract leaves the next task to the next continuation.

No unrelated features, dependencies, workflow edits, commits, or deletions. Guards address the specified authorization/owner/root boundary, hostile tool arguments, stale caller turns, and cancellation/completion ambiguity. Next: task 3.2 lifecycle events, AG-UI mapping, and live graph RuntimeStep; then concrete host/adapters. Keep tests deferred until phase end, change 7 gated on the sandbox choice, and change 10 gated on parity/live-smoke evidence.

## Subsequent checkpoint — change 8 lifecycle and live graph steps

KBD revision 1823: **83/182 real tasks checked off**, 99 remaining. Raw projection is 84/184 with the same two historical aliases (one complete). Change 8 is 5/25; only changes 1 and 2 are accepted. Execute remains round 4 of 5. Task 3.2 is recorded complete; task 4.1 has not started.

### Files changed

| File | Change |
| --- | --- |
| `src/uar/domain/events.rs` | Added AgentThreadStarted/Updated/Finished/Error variants; content-free AgentLifecycle, typed status/outcome, and RuntimeEventSink host publication trait. |
| `src/uar/runtime/thread/events.rs` | Pure PersistedAgentThread::lifecycle_event projection for confirmed insertion/update, using shared storage transition validation and parent/child lineage checks. |
| `src/uar/runtime/thread/mod.rs` | Export events module. |
| `src/uar/api/adapters.rs` | Transport-free official SUBAGENT_* mappings; named CUSTOM events for updates or failures before a child run exists; persisted timestamps and nested lifecycle metadata. |
| `src/uar/api/sse.rs` | Legacy agui.subagent.started/updated/finished/error mapping from the same normalized variants. |
| `src/uar/runtime/graph/engine.rs` | execute_with_events and shared execution core; live awaited start/finish events around each node; engine-owned checkpoint-continuing step numbers. |
| `src/uar/runtime/manager.rs` | Implement RuntimeEventSink on existing RunEventEmitter, pass it to graph execution, and remove post-completion step replay. Earlier edits preserved. |
| OpenSpec task, KBD projections, append-only memory | Original task 3.2 receipt, decision, session summary, and this checkpoint. |

### Contracts and follow-through

- Lifecycle payload includes root/parent/child IDs, captured parent and current child run IDs, path, artifact, typed status and terminal classification, storage revision, stable `agent-thread:<thread-id>:<revision>` identity, and the stored transition timestamp. It contains no owner credentials, prompt/history, output body, or raw backend error.
- The host must pass the captured parent turn associated with this child turn, not a later resumed parent. The projector checks the verified owner and exact edge, requires insertion revision zero/pending or the exact next persisted revision, and suppresses same-status/same-run writes. A restored latest snapshot is not proof of an unseen transition; recovery cannot invent start/finish events from it.
- Official SUBAGENT_STARTED carries actual subagentRunId and path as name; nested children additionally carry parentSubagentRunId when the captured parent is non-root. SUBAGENT_FINISHED omits content-bearing result and optional upstream outcome; terminal classification is in lifecycle metadata. SUBAGENT_ERROR uses static AGENT_FAILED/CANCELLED codes and generic messages. No raw backend code/message is copied.
- A pending child has no run ID. Pending/update events map to `CUSTOM` name `uar.agent_thread.updated`; a pre-start failure/cancellation maps to `uar.agent_thread.error`. This preserves failures without fabricating a required subagentRunId. The official mapper remains transport-free for embedded clients; SSE framing retains existing ordered event IDs and sequence enrichment.
- Context7 resolved `/ag-ui-protocol/ag-ui`; current source schema excerpts confirmed SubagentStarted/Finished/Error required fields and CUSTOM name/value. Source: https://github.com/ag-ui-protocol/ag-ui/blob/main/sdks/typescript/packages/core/src/events.ts. No dependency or protocol pin changed.
- Graph events use the same awaited RunEventEmitter lock/history/broadcast path as ordinary run events. A finished boundary means the node returned, not that it succeeded; an error still becomes a run error. Cancellation drops the pending execution and emits the existing run-cancelled event, not a fabricated finished step.
- Step identity comes from an engine-owned counter initialized by checkpoint iteration. Node-returned state cannot reset it. The existing 1000-node ceiling is checked before increment, so invalid high checkpoint counters do not overflow or fabricate another node. Trace data remains available; it is no longer used to replay progress after completion.

### Evidence and uncomfortable limit

`git diff --check` exited 0 with no output after each edit. Static placeholder/unchecked-panic search in the new projector and modified graph engine returned no matches (rg exit 1). Source search found `execute_with_events` called from RunManager and RuntimeEventSink implemented by the real event emitter. Both wire mappings are connected to existing event switches. KBD end-task exited 0 at revision 1823; the change projection reports 5/25.

No build, formatter, test, database operation, OpenSpec validation, or acceptance critic ran. Compilation, upstream schema compatibility, live step timing, replay ordering, checkpoint counter behavior, and cancellation races remain unverified. No Rust tier or whole-change acceptance is claimed.

`lifecycle_event` has no host execution caller yet: the concrete child service/adapters must call it after confirmed commits, publish on the root stream, and provide ordered/deduplicated recovery. The graph's old two-message child execution and output prefix remain until task 4.2. The new metadata is not proof that persisted children currently execute end-to-end.

No unrelated features, dependencies, workflow edits, commits, or deletions. Guards address the specified owner/lineage and content-disclosure boundary, revision reordering/duplicate transition signals, and checkpoint-owned step identity. AG-UI and runtime-security skills shaped correlation, explicit cancellation, and content-free payloads. Next: task 4.1, the concrete host service and AgentActor mailbox/user-context adapter, then graph and A2A adapters. Tests stay deferred until phase end; change 7's sandbox and change 10's evidence gates remain open.

## Subsequent checkpoint — shared-kernel actor adapter, task 4.1 still in progress

KBD revision **1825**. Task 4.1 was begun as task 17/25 and has **not** been ended or checked off. Change 8 remains **5/25**; normalized total is **83/182**, 99 remaining. Raw projection is 84/184 with the same two historical aliases (one complete). The plan defines one active phase with five implementation rounds and ten changes, not five accepted phases. Execute is at round 4; only changes 1 and 2 are accepted. The goal remains active.

### File-by-file production delta since the preceding checkpoint

| File | Change |
| --- | --- |
| `src/uar/runtime/actor/messages.rs` | Host-created, non-deserializable ActorOwner; reject missing/anonymous/inconsistent identity; owner/tenant actor namespace; listed session/thread/run IDs. |
| `src/uar/api/actors.rs` | Actor identity middleware returns 401 before body extraction; all actor handlers pass the authenticated owner. |
| `src/uar/runtime/actor/system.rs` | Shared RunManager/persistence; strict artifact resolution; owner-qualified name reservation and lookup; private kernel sessions; state observation; immediate cancellation outside the busy mailbox. |
| `src/uar/runtime/actor/agent_actor.rs` | Replace independent Orchestrator and history vector with ActorThreadSession; reply from committed terminal result; sender identity stays metadata; reject standalone tool-result injection; reconcile uncertain writes on shutdown. |
| `src/uar/runtime/thread/actor_host.rs` | Persist root/start before kernel entry and terminal result before reply. Retain the exact expected envelope during writes, reconcile by exact owner-qualified read, close recovered unstarted turns without replay/substitution, and refuse unresolved transitions. |
| `src/uar/runtime/thread/execution.rs` | Lossless ordered completion capture, first failure retained through terminal boundary, explicit cancellation, channel closure on producer disappearance, and live mailbox-waiter detection. |
| `src/uar/runtime/thread/mod.rs` | Declare the actor-host and completion modules. |
| `src/uar/runtime/manager.rs` | Direct actor entry into the same request kernel with host run ID/cancellation/completion; strict registered-artifact lookup; terminal errors for orchestrator/graph-driver startup failure; pre-dispatch cancellation checks; weak completion observation in retained run state so SSE disconnect respects a waiting actor. |
| `src/server.rs` | Construct ActorCollaboration with the shared RunManager. |
| `.prometheus/decisions.md`, `.prometheus/session-log.md`, this checkpoint | Append decisions, evidence, current counts, and unfinished integration. No task completion receipt added. |

### Execution and recovery contracts

- Direct-user call chain: actor API → owner-qualified ActorCollaboration → AgentActor mailbox → ActorThreadSession → RunManager.start_actor_turn → existing execute_request_inner. Exact artifact resolution propagates storage errors and rejects unknown IDs instead of substituting the default. The explicit root-user system prompt override changes only the execution copy.
- A database error is not evidence of rollback. The pending envelope includes storage revision, identity, status, timestamp, and result. Only an exact read-back clears uncertainty. A failed/missing/older/different read leaves the actor unresolved; no new ID, repeated write, or model turn is authorized. Same-request confirmation continues the original request. Later confirmation of a start closes that unstarted request as failed/cancelled before accepting new content. Terminal recovery preserves the completed result. There is no background blind retry.
- Completion capture is owned by emitter instances, not retained event history. A weak observation link in RunStreamState distinguishes a live mailbox owner from SSE-only viewers without keeping a disappeared producer alive. Last-viewer disconnect does not cancel a waiting actor; explicit run/actor/server cancellation is unchanged. Producer loss becomes a failed completion, not empty success.
- Actor namespace includes verified tenant as well as user. Underlying RunManager/thread ownership still uses user ID; this is not a claim that all kernel APIs provide tenant isolation.
- Tokio channel semantics were resolved through Context7: [Sender::is_closed](https://docs.rs/tokio/latest/tokio/sync/oneshot/struct.Sender.html). Earlier API checks used Axum middleware/request extensions and Tokio watch/JoinHandle documentation. No dependency pin changed.

### Evidence and uncomfortable remaining work

`git diff --check` returned exit 0 with no output after edits. Static call-site search found identity middleware, actor kernel entry, recovery on execute and shutdown, and the manager's mailbox-waiter check. Searches for the old independent actor orchestrator/history/prompt-prefix path and placeholder/unchecked-panic patterns in the actor adapter returned no matches. No build, formatter, tests, database operations, migrations, OpenSpec validation, or acceptance critic ran. **No Rust verification tier is claimed.** Compilation and all behavioral/race assertions remain unverified until phase end.

Task 4.1 is partial. The collaboration message still executes the target actor as an independent root; it must become a persisted child under the source root with intersected policy. AgentThreadHost still has no concrete implementation, and the five native agent tools' per-turn registry is not wired into RunManager. Exact frozen runtime bindings, root approvals/budget enforcement, durable child lifecycle publication, and restart recovery remain unfinished. Do not end task 4.1 or begin task 4.2 yet. Graph/A2A adapters and tree-wide budget/cancellation tasks also remain pending.

Resume task 4.1 directly: do not repeat its begin-task event. The patched KBD driver uses semantic ID `4.1`, positional index `17`, total `25`, and qualified title `thread-native-subagents::4.1` for its eventual completion receipt. Do not pass a positional ID as the semantic task ID or create another alias. Tests remain deferred to phase end under the operator's explicit order. Change 7's sandbox decision and change 10's parity/live-smoke gates remain open.

No unrelated features, dependencies, workflows, commits, or on-disk deletions were added. Guards trace to the authenticated actor boundary, exact artifact identity, uncertain commits, producer loss, or observer-versus-owner cancellation. OpenSpec/KBD and actor/security skills shaped these changes; they did not authorize accepting the unfinished child-host behavior.

## Subsequent checkpoint — concrete thread host, task 4.1 still in progress

KBD revision remains **1825**. Continue the existing task 4.1; do not repeat begin-task, end the task, or enter 4.2. Change 8 remains **5/25**, canonical phase implementation **2/10**, normalized task inventory **83/182** (99 remaining). Execute remains round 4 of 5 within the same active phase. The goal is not complete.

### Production delta

- `src/uar/runtime/thread/service.rs`: concrete root-scoped `ThreadService` implementing all six `AgentThreadHost` operations. Owns root-wide admission, exact policy context checks, typed note/trigger mailboxes, child workers, subscriptions, descendant interruption, persisted child/edge transitions, and confirmed lifecycle publication. Model execution is outside the mutation lock.
- `src/uar/runtime/thread/mod.rs`: exports the service module.
- `.prometheus/decisions.md` and `.prometheus/session-log.md`: append the service decision, remaining integration boundary, verification limits, and operator completion-status instruction. This checkpoint is append-only too.

This supersedes the preceding checkpoint's statement that `AgentThreadHost` has no implementation. It does **not** supersede its warning that child execution is not connected end-to-end. `ThreadExecutionHost` is a mandatory bridge with artifact, canonical-history, admission, and execution methods; it has no implementation yet. `ThreadService` has no production attachment or RunManager constructor caller. Actor collaboration still enters the target's independent root. Do not report task 4.1 complete.

### Failure handling and authority boundary

Mutation receipts are separate from host-owned jobs, so dropping a caller does not abandon a partly committed write. Unknown writes preserve the exact expected record and creation reservation. Recovery requires an exact owner-qualified record and creation edge; it does not replay a prompt. Accepted queued triggers retain the active-child slot between turns. The child executor is supervised so a lost producer becomes failure, not empty success. A resumed turn is committed before admission/history preflight; failure then closes that turn instead of exposing its predecessor's successful result. History retrieval uses the captured previous thread snapshot.

Guards trace to specified owner/root/turn and immutable-policy authority, uncertain storage commits, admission limits, cancellation, and producer loss. No unrelated features or speculative fallback paths were added. No dependencies, workflows, commits, migrations, database operations, or deletions in this continuation.

### Required continuation

Implement the concrete RunManager execution bridge using the same kernel, exact frozen provider/MCP/native/skill bindings, enforceable sandbox authority, root approvals, and root-shared per-call budget checks. Register one service per root so concurrent attachments cannot reset counters. Reconcile the actor's later root turns with the service's single-root-run lifetime. Wire root message consumption and committed root closure, then replace the independent-root collaboration path. Do not resolve child credentials through global/agent/session fallbacks or attach a second LLM loop. A message receipt acknowledges queueing; a latest-state wait before worker start can still observe the preceding terminal state and needs integration-level treatment. Durable restart/delivery behavior remains unverified.

The operator requires executing the named `kbd-status` skill after **every completed task, change, or phase**. Read `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md` and produce its report after the genuine completion receipt. No completion boundary has been reached in this continuation. Keep tests at phase end.

### Observed evidence

`git diff --check` exited 0 with no output after each edit. Static inventory found six host operations and calls to policy intersection, persistence, admission, history, and lifecycle projection; no placeholder/unchecked-panic matches were found in the service. Source search found no production service constructor or concrete execution bridge. No compilation, formatter, tests, OpenSpec validation, or acceptance critic ran. **No Rust tier passed.** Behavior, compilation, concurrency, lifecycle ordering, and storage recovery remain unverified until phase-end checks.

## Subsequent checkpoint — root approval broker wired, task 4.1 still open

Position unchanged: revision **1825**, Execute round **4/5**, phase implementation **2/10**, actual task inventory **83/182** (99 remaining), thread-native-subagents **5/25**. This continuation did not finish a numbered task, change, or phase. Do not end 4.1 or begin 4.2. The previous continuation was progress, not a blocked/no-progress turn; this one also makes real production edits.

### File-by-file delta

| File | Change |
| --- | --- |
| `src/uar/runtime/thread/approvals.rs` | New host-only ApprovalBroker with weak root-channel index; request-only RootApprovalChannel; serialized pending requests; host-minted approval IDs; exact-ID resolver; drop cleanup; root/caller cancellation and five-minute total wait bound. |
| `src/uar/runtime/thread/mod.rs` | Export approvals internally. |
| `src/uar/runtime/manager.rs` | Replace pending_approvals map with broker. Register root channel before assembly; gate calls channel.request. Add resolve_approval_request while retaining legacy resolve_approval. Run cancellation uses the shared token. Internal execute_request_inner has optional inherited channel; both existing callers pass None. Inherited-child gates cannot bypass narrowed approval policy through local governance-off. Old standalone helper tests/helpers stay cfg(test), not production evidence. |
| `src/uar/domain/events.rs` | Optional, serde-compatible approval_id on ToolCallApprovalRequired. |
| `src/uar/api/adapters.rs` | Forward approvalId on official AG-UI custom approval event. |
| `src/uar/api/sse.rs` | Forward approval_id on legacy and runtime approval events. Runtime approval entity identity derives from host request ID when present, with the old fallback for historical events. |
| `src/uar/api/routes.rs` | Typed approval request accepts optional approval_id; existing root-owner check precedes broker resolution. |
| `src/server.rs` | Raw JSON approval route rejects non-string/non-null IDs and forwards valid IDs after its existing owner lookup. |
| `.prometheus/decisions.md`, `.prometheus/session-log.md`, this checkpoint | Append observed races, decision, verification limits, and continuation requirements. |

### Authority and failure behavior

The old production map held one sender per run; concurrent children would overwrite siblings. It also inserted after publishing, racing immediate responses. The new queue inserts before publication, holds a per-root serialization guard until resolution/drop, and removes only the matching request. Root and requesting-child cancellation are separate inputs. A dropped/expired request cannot clear another request's slot. Resolver access is not present on the channel supplied to descendants. Child requests reject omitted, wrong, or stale approval IDs. Legacy run-only root decisions remain compatible and do not gain a replay-correlation guarantee.

No caller supplies an inherited channel yet. The private parameter alone is **not** a safe general child execution entry: all resource resolution elsewhere in execute_request_inner still follows ordinary root behavior. The concrete child entry must bind frozen resources and inherited authority together before calling it. Do not expose child execution with only an approval channel added.

### Next code work and remaining risks

- Implement the concrete ThreadExecutionHost and bind it to the same RunManager kernel using captured provider clients/configuration, MCP connections, native handlers, skill snapshots, sandbox enforcement, root approvals, and root budget state. Do not re-resolve global/agent/session credentials for a child.
- `ActivationContext::activate` currently calls McpRegistry::from_config for skill dependencies. A frozen child activation path must reuse inherited live bindings and refuse unavailable dependencies; it cannot run source artifact connection/auth/env declarations.
- Register one ThreadService per root, settle the actor's later-root-turn lifetime, wire root messages/closure, and replace independent-root actor collaboration. Tree limits and root cancellation cannot be reset by reattachment.
- Current browser approval transports (`frontend/src/services/run-tools-api.ts` and `frontend/src/features/runtime/api/runtime-console-api.ts`) send only `{ approved }`. Root compatibility remains; child support needs request-ID forwarding before UI exposure. No frontend work was done, and future changes must follow the project UI/entity routing instructions. HTTP clients can supply the new field already.
- At phase end, test real broker/manager/API behavior: concurrent children, approval before/after publication, exact-ID replay refusal, root/child cancellation while queued or pending, timeout cleanup, legacy-root compatibility, and event ID propagation. Old await_approval/resolve_pending_approval unit helpers retained under cfg(test) do not verify the broker.

Observed checks: `git diff --check` exited 0 with no output after each edit; static source inventory found production registration/request calls, both HTTP resolver calls, and approval-ID propagation. No production pending_approvals field references remain. No builds, formatter, tests, database operations, migrations, OpenSpec validation, acceptance critic, commits, workflow changes, dependency changes, or deletions. No Rust verification tier is claimed; compilation and runtime behavior remain unverified. The new guards address the specified root authority and observed slot/publication/cancellation races. Nothing outside the requested implementation was intentionally added.

Continue task 4.1 directly. The eventual end-task receipt must use semantic ID `4.1`, index `17`, total `25`, title `thread-native-subagents::4.1`, then execute the named `kbd-status` skill. Keep the whole goal active and keep tests at phase end. Change 7's sandbox decision and change 10's parity/live-smoke gates remain open.

## Subsequent checkpoint — immutable executable binding primitives

Revision **1825**, task 4.1 still open, change 8 **5/25**, implementation **2/10**, actual tasks **83/182**, 99 remaining. Execute remains round 4/5. No new task begin/end, completion acceptance, or post-completion status signal. The last two goal turns both made production-code progress; neither was a verified wait or an impasse.

### File-by-file production delta

| File | Change |
| --- | --- |
| `src/mcp/registry.rs` | BoundServices captures exact transport Arcs plus a view-close token. freeze_bindings snapshots selection, checks live connections, re-lists selected descriptors on the same connections, and rejects changed descriptors. require_bound_servers checks identity/revocation/closure. Frozen calls never reconnect, use fake test/mirror shortcuts, expose connection recipes, replace config, merge resources, or add tools. Filtered frozen views preserve bindings and derive child close tokens. Borrower shutdown closes only the view; owned remove_server revokes/cancels the retained slot. McpMergeError distinguishes collisions from frozen-binding rejection. |
| `src/uar/tools/descriptor.rs` | ToolAssemblyError::FrozenBinding for attempted tool addition to an immutable execution view. |
| `src/uar/runtime/skills/activation.rs` | A frozen registry requires a live inherited view and existing dependency identities; it never starts skill commands/URLs/auth/env. Ordinary root startup path remains. |
| `src/llm/mod.rs` | Object-safe with_bound_model with explicit refusal for unsupported driver implementations; the host still owns policy checks. |
| `src/llm/liter_driver.rs` | Same-qualified-provider rebinding shares the existing DefaultClient, not a newly resolved config/client. |
| `src/llm/anthropic_driver.rs` | Anthropic-only rebinding shares HTTP client and retains key, endpoint, output/cache/thinking defaults. |
| `.prometheus/gotchas.md`, `.prometheus/decisions.md`, `.prometheus/session-log.md`, this checkpoint | Append the mutable-slot and constructor-credential traps, implementation limits, and continuation state. |

These APIs have **no production root-capture caller yet**. They are prerequisites for the concrete ThreadExecutionHost, not proof that children execute. Do not mark task 4.1 complete. The previous checkpoint's claim that all activation unconditionally starts dependencies is superseded only for frozen views; ordinary roots still use the previous path.

### Source evidence and verification limits

`git diff --check` exited 0 with no output after edits. Static constructor inventory found the new field initialized in every McpRegistry constructor; filtered views retain exact bindings, and with_native_tool/merge/upsert refuse mutation. Source calls show bound dependency checks from activate and same-client rebinding on both built-in drivers. No compilation, formatter, tests, database operations, migrations, OpenSpec validation, or acceptance critic ran. No Rust tier passed; all behavioral, race, API-compatibility, and compilation claims remain unverified.

The vendored liter-llm checkout is at the exact versions.toml commit `c5c6caac617eb931cd5009146a70831422ec236c`. Read `vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs` around DefaultClient (671 onward): constructor environment-key resolution, retained config/client/provider, per-request model routing and credential-provider binding. No vendor code changed. UAR build_client_config also reads environment indirection; child setup must not invoke either constructor again. Same-provider driver rebinding is now available. External drivers without a rebinding implementation can be reused for their original model; they cannot silently claim support for another model.

Uncomfortable API delta: McpRegistry::merge now returns McpMergeError, not ToolCollision directly. Repository callers use inferred/map_err handling, but explicitly typed downstream Rust callers require migration. Server removal now propagates revocation to previously retained views. Added guards trace to the required immutable credential/connection boundary and the observed shared-slot/reconnect behavior, not speculative hardening. No unrelated features, dependency changes, workflow edits, commits, or file deletions.

### Resume directly at the concrete integration

Replace the private approval-only optional kernel argument with a complete inherited-resource bundle; every current caller still passes None. Carry the exact intersected policy, private child session/history, native handlers without parent-bound activate_skill/agent controls, frozen MCP view, frozen SkillRegistry, bound primary/fallback drivers, captured model config, root approval channel, and root budget/cancellation state together. No child path may call fresh global credential/provider or dependency resolution. Construct fresh activate_skill and agent-tool handlers for the child's context and preserve normal descriptor collision checks.

Manager points that need explicit bound branches: effective-policy resolution/backfill; skill matching/catalog source; native registry/ActivationContext creation; preferred/routed/credential model selection; summarization driver construction; primary/fallback driver creation; approval channel; post-run skill evolution; history/session ownership. Passing only resolved_policy is insufficient. The current LLM intent classifier (`src/uar/runtime/matching/intent/llm.rs`) constructs a fresh minimal Orchestrator from LlmConfig inside classify; give it a bound-driver path before using it for a child, rather than silently switching the child's matching algorithm. Auxiliary classifier/summarizer calls must participate in root accounting too.

Use the existing kernel request semantics carefully: initial/queued delegated messages end in a user message. A host adapter can split the last user body into request.input and keep earlier canonical messages in checkpoint_history, so matching sees the actual task and the input is appended once. Preserve the child's own tool-paired history on resumed turns; fork filtering applies only to initial parent history. Root attachment uniqueness, later actor-root-turn lifetime, root message consumption/closure, actor collaboration, sandbox enforcement, and root per-call budgets remain open.

An asynchronous question is outstanding for change 7: port native sandbox support or reject sandboxed stdio until a supported runner exists. No answer has been inferred, and independent thread work remains available. Keep tests at phase end; at that boundary verify hot replacement/reconnect/removal, borrower closure isolation, descriptor races, no dependency startup, same-client credentials under changed environment, and unsupported/cross-provider rebinding refusal. Eventual task completion must still be followed by the named kbd-status skill.

## Subsequent checkpoint — connected root snapshots; clean compile-only T0

Position is unchanged: revision **1825**, Execute round **4/5**, task **4.1**
open, thread-native-subagents **5/25**, implementation **2/10**, actual tasks
**83/182** with **99 remaining**. No task begin/end, archive, or acceptance
transition occurred. The previous and current turns both made production-code
progress. This is not a blocked-goal condition.

### Delivered source changes in this continuation

| File | Delta in this continuation |
| --- | --- |
| `src/uar/runtime/turn/bindings.rs` | New RunModelBindings captures root primary/fallback clients, config and health once; constructs orchestrators from the same Arcs. New RunSkillBindings captures the run's registry and optional matching view. Both have production manager callers. |
| `src/uar/runtime/turn/mod.rs` | Registers the private bindings module. |
| `src/uar/runtime/manager.rs` | Captures skill definitions before matching; matches, renders catalog and activates from the same registry. Candidate definitions are resolved from the snapshot, not accepted from a mutable vector result. Captures model clients after root routing/credentials and before reduction. Initial summarization, tool loop and graph use those clients; removes duplicate client/fallback creation paths. |
| `src/uar/runtime/skills/service.rs` | Read-only SkillMatchingSnapshot captures registry, config and legacy agent selections. Shared matching helper preserves algorithms and scores; vector scores are joined to captured eligible bodies. No CRUD API is exposed by the view. |
| `Cargo.toml` | Existing BackON 1.6.0 addition now selects std/tokio-sleep explicitly and explains omitted default sleepers. No version change. |
| `Cargo.lock` | Adds Tokio to BackON's dependencies, preserving prior root BackON entry and every package version/checksum. |
| `src/uar/runtime/actor/system.rs` | Memory-provider fallback is feature-gated. Without it or configured persistence, actor creation refuses before launching an actor. |
| `src/server.rs` | Clones effective_input for kernel ownership because the SSE capture still needs the original. |
| `src/uar/runtime/turn/shadow.rs` | Formats SHA bytes individually using the existing prompt hash convention; fixes unsupported LowerHex on the digest array. |
| `src/uar/runtime/thread/service.rs` | Removes redundant Future import; redacted HostedThreadTurn/ThreadService Debug. |
| `src/uar/runtime/skills/registry.rs` | Gates existing visible_vector_matches helper and its SkillMatch import to tests, its only callers. |
| `src/uar/runtime/native_skill.rs` | Debug for NativeExecutionContext shows only presence of project instructions. |
| `src/uar/runtime/native_skills/activate_skill.rs` | Redacted ActivateSkillTool Debug, no context/body dump. |
| `src/uar/runtime/skills/activation.rs` | ActivationContext Debug shows eligible count, not bindings or skill content. |
| `src/uar/runtime/skills/usage.rs` | Debug for numeric/model/skill usage metadata; driver Debug omits inner client and activation state. |
| `src/uar/runtime/thread/mod.rs` | AgentHandle Debug shows identity only, not watched contents. |
| `src/uar/runtime/thread/control.rs` | Redacted or metadata-only Debug for grant, scope, send request, interrupt receipt, tool context; never includes message body. |
| `src/uar/runtime/thread/policy_intersection.rs` | ThreadPolicy Debug includes root/artifact IDs and tool count, not artifact/policy bodies. |
| `src/uar/runtime/turn/contributors.rs` | Redacted/count-only Debug for assembly inputs/state/contribution and contributor registry. |
| `src/uar/runtime/turn/builtin.rs` | ContextStage Debug omits driver, exposes model and context limit. |
| `src/uar/runtime/turn/resolved.rs` | ResolvedStep Debug omits request, registries and content. |
| `src/uar/runtime/world_state/contributor.rs` | Debug reports history-rewrite flag only. |
| `src/uar/runtime/world_state/runtime.rs` | Debug reports workspace count only. |
| `.prometheus/gotchas.md`, `.prometheus/decisions.md`, `.prometheus/session-log.md`, this checkpoint | Append observed failures, fixes, verification, scope and continuation state; no historical entries rewritten. |

### Verification — actual observed results

`git diff --check` returned exit 0 with no output after edits.

The compile-only command was:

```text
cargo check --locked --no-default-features --features server-full
```

1. Exit 101 before compilation: lockfile required update. The cached BackON
   1.6.0 manifest enables optional timer dependencies by default while its lock
   entry had only fastrand. Selected the Tokio/std features needed by the actual
   `.retry(...)` path and added the already-resolved Tokio edge. Consulted
   Context7 and Firecrawl developer index; documentation:
   https://docs.rs/backon/1.6.0/backon/ . No package upgrades or unlocked command.
2. Exit 101: three application errors (feature-gated memory provider, moved
   effective_input, digest LowerHex) and one redundant import. Fixed the cited
   code, retaining public constructor compatibility and persistence guarantees.
3. Exit 0 in 31.32s with 23 warnings. Not accepted under the zero-warning rule.
4. After redacted Debug implementations and test-only helper gating, exit 0,
   zero warnings: `Finished dev profile [unoptimized + debuginfo] target(s) in
   30.92s`.

**Tier 0 now passes for server-full.** This supersedes earlier no-compilation
checkpoints only. No tests, test authoring, formatting, OpenSpec validation,
database/migration runs, acceptance critic, commits, workflows or deletions.
Other profiles, test targets, live behavior, races, and child integration remain
unverified. The tests-only-at-phase-end instruction does not require skipping
the separate compile-only Tier 0 check.

### Scope and security accounting

No unrelated features were added. Compilation fixes above are defects observed
in the accumulated ten-change implementation. Snapshot candidate filtering
protects the explicit frozen-resource boundary. Actor admission's missing-store
guard follows the real feature-disabled build and persisted-thread requirement.
Debug redaction protects actual prompt/credential/message fields; no blanket
lint suppression was introduced. No new speculative guards.

### Still required — do not mark 4.1 complete

The complete inherited-resource bundle and concrete ThreadExecutionHost remain
unimplemented. The optional inner-kernel argument is still approval-only, and
all existing callers pass None. Root model/skill captures are now connected;
root MCP freeze and policy-qualified same-provider rebinding still have no
production caller. Keep the captured clients and skill registry when adding the
child branch, rather than recapturing against global settings.

The skill snapshot keeps existing host persistence/embedding references for
matching; it freezes definitions, not vector-ranking state or all read I/O.
The separate LlmClassifier still builds its own client. Preserve its configured
semantics through a bound-driver path before using it for child execution.
Auxiliary calls still need tree-wide budget/cancellation enforcement.

Remaining integration: exact intersected policy and model credential grants,
fresh child-bound activate_skill/agent handlers, frozen MCP dependency views,
root budget/cancellation/sandbox enforcement, unique root attachment and actor
later-turn lifetime, root message consumption/closure, actor collaboration,
then graph and A2A adapters. The planned change-7 sandbox choice is still
outstanding; no decision was inferred. Typed-default flip remains evidence-gated.

Skills used: kbd-apply, openspec-apply-change, actor-model,
agent-runtime-security, dependency-pin-discipline, debugging-and-error-recovery,
and the user-named kbd-status. Debugging paused feature additions while errors
were repaired; regression/E2E tests remain deferred. kbd-status was executed at
the checkpoint from waypoint/progress/position plus actual OpenSpec inventories.
Always execute it again after every genuine completed task/change/phase.

## Subsequent checkpoint — cost admission on captured drivers

Revision **1825**, task **4.1** remains partial, change **5/25**, total **2/10**
and **83/182** (99 remaining), Execute round **4/5**. Previous and current goal
turns both made code/evidence progress; no blocked-goal condition or numbered
completion. No new begin/end, archive, or status-after-completion trigger.

### Files changed in this continuation

- `src/uar/runtime/cost_budget.rs`: short synchronous ledger lock, multi-scope
  atomic cost replacement, status-only reads, root ModelCallBudget and wrapped
  model driver. Before each request, deny Exceeded/poisoned ledger/cancelled
  root. During the stream, account priced cumulative Usage updates before
  exposing the event; repeated updates do not double-charge and cache corrections
  can lower the request estimate. Preserve gauges/threshold traces. Rebinding
  keeps the same payer scopes and cancellation token.
- `src/uar/runtime/turn/bindings.rs`: model capture requires ModelCallBudget,
  checks admission, wraps both primary and each fallback client. Existing
  summarization, ordinary and graph paths consume these captured wrappers.
- `src/uar/runtime/manager.rs`: installs existing agent cost ceilings before
  capture, supplies root run/session/agent/global payer scopes, and changes
  end-of-run tracker operations to status reads to avoid double charging.
- `.prometheus/gotchas.md`, `.prometheus/decisions.md`, `.prometheus/session-log.md`,
  this checkpoint: append implementation, limitations, call-site correction,
  and actual verification evidence.

### Evidence and remaining risk

`git diff --check`: exit 0, no output after edits. Compile-only T0:
`cargo check --locked --no-default-features --features server-full` passed with
zero warnings in 28.74s, 9.27s and finally 7.85s. No tests, formatters, databases,
migrations, OpenSpec validation, acceptance critic, dependency changes, commits,
workflow edits or deletions. No unrelated feature. The poisoned-ledger guard
protects the real financial authorization boundary; cancellation and Exceeded
guards implement explicit requirements. No speculative guards were added.

Do not mark task 4.1 or 5.1 done. This covers **priced, reported cost usage** of
captured drivers, including initial summarization and graph driver calls. It
does not implement missing/unpriced-usage handling, reservations for concurrent
in-flight calls, token/rate/tool/time budgets, narrowed child limits, or concrete
child admission. Existing durable cost roll-ups remain at ordinary-run completion
and omit graph/summarization paths; their completeness is not claimed. Existing
agent/session scope naming is retained. No remote billing cancellation guarantee.

Important correction to earlier checkpoints: RunManager's private classifier is
constructed only by `create_classifier` at manager.rs initialization; that
factory falls back to Hybrid for Llm. `rg` found no production caller of
`create_classifier_with_resources` under src/. The standalone LlmClassifier's
fresh-client construction is not currently in the shared manager path. Preserve
the existing captured classifier semantics; do not build a speculative adapter
for that unused path as a prerequisite to child integration.

The major next implementation remains the complete inherited-resource branch
and concrete ThreadExecutionHost. Current inner-kernel optional input is still
approval-only and every caller passes None. No production root MCP freeze or
child policy-qualified model selection exists yet. Preserve the new budget
wrappers when passing/rebinding clients to children; do not create a child payer
or run the fresh root capture path for a child. Actor root records currently
reuse a thread while assigning later run IDs, whereas ThreadService::attach
requires a fresh root tree on its original run: resolve that lifecycle seam,
do not silently accept stale root authority. Keep actual actor collaboration,
root messages/closure, sandbox enforcement, graph/A2A and phase-end acceptance
in scope. The planned change-7 sandbox decision remains outstanding.

## Subsequent checkpoint — inherited assembly branch

Task 4.1 is still open. No KBD end-task or new begin-task fired. Canonical
position remains revision 1825, Execute round 4/5, 2/10 changes and OpenSpec
83/182 tasks. The exact requested kbd-status skill was executed in this turn.

### Files changed

- `src/uar/runtime/turn/bindings.rs`: InheritedRunBindings contains policy,
  thread, controls, models, skills, frozen MCP, native registry, harness, cwd and
  approval channel. BoundModel adds opaque provider credential grant IDs.
  RunModelBindings has selected primary/fallbacks plus a retained still-granted
  catalog. credential_grants() supplies host-only policy inputs; for_policy()
  rejects foreign provider grants, selects/rebinds only captured clients,
  preserves root cost wrappers and narrows the retained catalog. Child config
  removes API key/env/base URL/provider-key recipes; no constructor is invoked.
- `src/uar/runtime/manager.rs`: optional inherited argument replaces the old
  approval-only argument. Validate child owner/run/artifact/private session,
  checkpoint-history presence, control scope/policy pointer, approval root and
  frozen MCP before session mutation. Use policy's narrowed artifact, skip
  global policy/model backfill, clone captured harness/cwd/skills, filter bound
  native/MCP resources, and register fresh child activation/agent handlers.
  Child model selection bypasses global routing/credentials, root capture and
  agent-limit/new-payer setup. Existing ordinary root behavior is retained.
- `src/uar/runtime/thread/control.rs`: crate-only scope identity accessor.
- `src/uar/runtime/thread/approvals.rs`: crate-only root identity accessor.
- decisions.md, session-log.md and this file: append-only rationale/evidence.

### Verification and constraints

git diff --check passed. Compile-only T0 command:
`cargo check --locked --no-default-features --features server-full`.
Initial pass: 23.62s, one unused RootApprovalChannel import; fixed immediately.
Next pass: zero warnings, 10.95s. Final catalog pass: zero warnings, 14.50s.
No tests, test authoring, formatter, OpenSpec validation, DB/migration execution,
acceptance critic, dependency changes, workflow changes, commits or deletions.
No unrequested feature. Added checks trace to the real delegation authority
boundary, not speculative defensive behavior.

Every execute_request_inner caller STILL passes None. The new branch is not a
complete production child feature. Root MCP freezing/permission capture and
attachment plus a concrete ThreadExecutionHost remain next. Do not forget the
actor root lifecycle seam: subsequent actor turns reuse a thread but advance
run_id; service attachment requires the original fresh root run. Keep owner
qualified canonical history, actual collaboration, root mailbox/closure,
sandbox enforcement, root-shared narrowed budgets and joined cancellation in
scope. Do not expose the new branch without those host responsibilities.
Tests remain deferred to phase end; runtime behavior and task acceptance are
unverified. Do not mark 4.1 or 5.1 complete, and rerun kbd-status after each
genuine completed task/change/phase.

## Subsequent checkpoint — actor root lifetime and producer unwind

### Implemented in the live actor path

- actor_host.rs: each execute creates a fresh AgentThread::root (new thread and
  root_run_id), keeping the same owner/session history. Do NOT restore the old
  begin_turn(new_run_id) on an old root. persist() chooses create_agent_root
  when IDs differ, update_agent_thread for the current root's transitions.
  Live/unresolved predecessors cannot be replaced. Existing exact-write
  recovery handles uncertain new-root creates without replaying model work.
- actor/system.rs: stop_actor awaits its join handle; shutdown_all drains
  handles, releases the registry lock, cancels all, then joins all. No shutdown
  timeout or detached replacement was added.
- thread/execution.rs: RunCompletionCapture stores terminal results instead of
  immediately sending them. New RunCompletionGuard holds only the capture Arc,
  not an emitter. On drop it releases a frozen result, reports kernel_panicked
  during panic unwind, or closes a missing-terminal sender without fake success.
- manager.rs: capture is Arc<std::sync::Mutex<RunCompletionCapture>>; weak
  ownership checks and emitter record use the short synchronous lock. Create
  completion_guard after emitter, move it into the spawned run as the first
  body local. Assembly early returns drop the original guard; execution locals
  drop before the moved guard. SSE terminal events keep their existing timing.

### Verification and scope

`cargo check --locked --no-default-features --features server-full` passed with
zero warnings for all three cohesive edits (11.15s, 10.03s, 18.13s). Repeated
git diff --check passed, including append-only memory updates. No tests, test
authoring, formatter, OpenSpec validation, acceptance critic, DB/migration work,
dependency changes, workflow changes, commits or deletions. No unrequested
feature. Live-root refusal addresses the observed root/run identity mismatch;
panic/missing-terminal handling is the actual producer completion boundary.

Unverified: integration behavior, abnormal shutdown/persistence recovery, and
the complete phase acceptance. The mailbox/main-future join claim excludes
existing detached cost/evolution maintenance and remote billing. Root attachment
and a concrete ThreadExecutionHost remain absent; every inherited caller still
passes None. Actor collaborate still enters another independent root and must
be replaced with an actual child. Do not mark task 4.1 or 5.2 done.

Position: waypoint revision 1825, Execute round 4/5, canonical 2/10 changes,
OpenSpec 83/182 tasks, thread-native-subagents 5/25. No begin/end boundary this
turn. Goal remains active; this turn is progress. Tests remain at phase end.
The next useful dependency is projected-mcp-runtime task 0.1: evaluate the
native port and either implement it or the explicitly planned config-load
rejection. Earlier notes treating that gate as necessarily an operator-only
choice are superseded: unlike versions.toml, the task delegates this decision
to execution. Do not silently assume a sandbox exists, manufacture host grants
without bindings, or expose children with unenforced narrower restrictions.

## Subsequent checkpoint — MCP task 0.1 implemented

Execute round 4/5. The projected-mcp-runtime sandbox gate selects rejection,
an explicit permitted outcome of task 0.1. It is not awaiting an operator-only
choice. Source-backed rationale is appended to the phase decision log and
.prometheus/decisions.md. No native sandbox port or new dependency.

config.rs validates McpConfig via an UncheckedMcpConfig/TryFrom deserialization
boundary, retaining the serialized shape. Shared entry validation rejects only
Stdio { sandboxed: true, .. }, naming the server and missing OS-backed stdio
backend. registry.rs checks the whole map before connecting and both launch
helpers before provisioning/spawn. HTTP into_stored and embedded save validate
before any writes/removals/deferral. Embedded hydrate validates the full effective
map before seeding or dropping connections, including disabled entries.

All four per-edit T0 cargo checks passed, zero warnings, final 10.18s. No tests
or formatting or acceptance review; 1.8 and all other tasks remain open. Server
boot still logs config rejection and can continue empty; no whole-process abort
claim. Thread-native-subagents::4.1 remains incomplete and unenforced child
sandbox restrictions still cannot be admitted. Next production task is MCP 2.1
(catalog), not the deferred test section. End-task 0.1 and full kbd-status follow;
read the post-hook waypoint/progress rather than assuming projected counts.

Post-boundary receipt: KBD end-task succeeded at revision 1829. Full kbd-status
readback confirms canonical implementation 2/10, actual OpenSpec 84/182 tasks,
98 remaining, projected-mcp-runtime 1/22. Project-wide implementation 103/120
is not a phase count. The waypoint's exactNextCommand still names the already
complete fail-closed-tool-arguments; do not replay it. Next planned production
task is projected-mcp-runtime::2.1. The ledger's COMPLETE evidence/certification/
publication summaries describe prior rollback/PR274/42-of-42 work, not acceptance
of this ten-change phase. No final-phase completion or goal completion claimed.

## Subsequent checkpoint — MCP task 2.1 catalog

src/mcp/catalog.rs now exists and src/mcp/mod.rs exports it. Immutable
ServerDefinition::new(name, source, configuration, required, authentication)
validates empty server/skill/credential identities and unsupported sandbox flags.
Private fields expose read-only accessors. ServerSource::authority derives
Global > Skill > Child; sandbox policy derives from configuration. Authentication
is Unknown/NotRequired/Required/Authenticated with an opaque host binding_id.
These are host observations, not server-provided claims or executable grants.

ServerConfigHash is opaque Eq/Hash with redacted Debug, no Serialize/Display.
It uses existing sha2 on declared inputs (not resolved ambient environment),
length prefixes, transport discriminator, ordered args, sorted env. Bindings in
2.3 must still include owner/auth identity/resolved environment. Definition Debug
omits launch config/hash and redacts authentication binding IDs.

McpCatalog::from_definitions stores BTreeMap<name, BTreeMap<ServerSource, Arc<
ServerDefinition>>>. Same-source identical duplicates coalesce, conflicts fail.
server_names/candidates/definitions provide stable read-only iterators. Distinct
source declarations remain separate; task 2.2 must resolve authority before
binding any server. No projection or manager consumer exists yet. Do not claim
actual connection reuse or immutable execution bindings from this data structure.

Compile-only T0 passed zero warnings in 28.92s. No tests/formatting/review.
Current KBD begin-task revision was 1831. End 2.1 then kbd-status. Read new
waypoint/progress for final counts; next production task 2.2. Thread-native
subagent 4.1 and complete physical sandbox/budget integration remain open.

Post-boundary receipt: end-task 2.1 succeeded, revision 1833. kbd-status
readback: canonical 2/10 implementation changes; actual OpenSpec 85/182 tasks,
97 remaining; projected-mcp-runtime 2/22. Global project counter stays 103/120.
Waypoint still names completed fail-closed-tool-arguments, so next production
work remains 2.2 under the written plan. New untracked catalog also passed
git diff --no-index --check against /dev/null. Goal remains active.

## Subsequent checkpoint — 2026-09-03, MCP task 2.2

New src/mcp/projection.rs is exported. McpServerProjection::resolve(catalog,
policy, scope) chooses source-qualified definitions using ALL resolved eligible
IDs (including All/Auto/Inherit modes), with None denying. Scope carries active
skills plus optional current child UUID; inactive/foreign sources do not enter.
Highest authority wins (global/skill/child); conflicting settings at that rank
fail. Identical settings choose the first stable origin. Lower-ranked conflicts
do not poison a higher winner.

ServerToolCatalog::new(definition Arc, compiled descriptors, complete) checks
MCP/server identity and provider collisions. McpServerProjection::with_tools
uses only the selected origins, compares config hash/required/auth state, and
requires one complete consistent snapshot for each selected server. Different
complete snapshots fail rather than unioning potentially removed tools.
McpStepProjection maps exact server definitions and ProjectedMcpTool (server Arc
plus unchanged descriptor Arc). Hidden/policy-excluded tools are omitted;
Deferred is retained in tools(), omitted by model_tools(). Debug surfaces omit
config, auth identities, schema, descriptions and hashes.

No caller in manager.rs or turn/ yet; task 4.1 remains explicit wiring. No live
registry changes. Binding-cache owner/auth/environment validation and real
sandbox/Cedar enforcement remain host responsibilities, NOT proven by a catalog
association supplied to ServerToolCatalog. Root-delegated bindings must stay
frozen; do not resolve fresh global resources for children. Disabled global
definitions must stay excluded by the host's resolved eligibility rather than
allowing a skill to revive their names during later universe construction.

T0 passed without warnings in 45.12s; tracked/untracked whitespace checks passed.
No tests, test authoring, formatting or critic. Begin task revision 1835; end
task 2.2 and kbd-status follow. Next production task 2.3. Goal remains active.

Post-boundary: task 2.2 completion succeeded at revision 1837. KBD's memory
mirror write failed; local append-only records above are the fallback and work
continues. Full kbd-status readback confirms canonical 2/10 implementation
changes, OpenSpec 86/182 tasks (96 remaining), projected-mcp-runtime 3/22.
Project-wide implementation remains 103/120. The waypoint still names already
completed fail-closed-tool-arguments; next actual production work is 2.3.

## Subsequent checkpoint — 2026-09-03, MCP task 2.3

binding_cache.rs is exported. McpBindingRequest::new takes ActorOwner,
Arc<ServerDefinition>, Arc<McpBindingEnvironment>; its private key includes
user/tenant, server/source/config/required/auth and full environment/cwd.
Environment::new requires absolute cwd and BTreeMap<OsString,OsString>.
All secret-bearing Debug is redacted. Required auth rejects before startup.

McpBindingCache::get_or_connect takes Arc<Request> and an FnOnce returning a
future Result<McpRegistry,McpBindingError>. Exactly one caller owns that future;
waiters share a watch completion. The registry must be an owned single-server
registry matching declared config and descriptors, not a frozen borrower.
The resulting Arc<McpBinding> must be retained throughout calls; registry()
rejects revocation. A ready binding does NOT prove complete discovery yet.

RefreshGuard clears cancellation/failure, generation UUIDs prevent stale
publication, invalidate(key)/invalidate_owner(owner) revoke and cancel. The
cache retains retired registries for reap_retired()/shutdown(); shutdown stops
admission, cancels/awaits outstanding refreshes, then joins transport closures.
No spawned tasks. registry.rs adds begin_shutdown for synchronous cancellation
and calls it from existing async shutdown. Source/auth enums derive Hash.

CRITICAL NEXT: the connector is still a host callback contract, not wired to
RunManager. Do not pass current McpRegistry::from_config unchanged: it reads
ambient env/cwd and reconnect does too, and optional discovery can be incomplete.
Implement snapshot-aware launch/reconnect with partial-resource drop cleanup,
complete catalog handling and lifecycle in 3.1/4.1/4.2. Runtime consumers still
do not use catalog/projection/cache. Thread-native-subagents::4.1 remains open.

T0 first passed with two warnings in 41.87s; both fixed. Final T0 passed with
zero warnings in 31.44s. Whitespace checks no diagnostics (new-file no-index
exit 1 reflects added content). No tests, test authoring, formatter or critic.
Begin task revision 1839; local memory fallback after mirror failure. End task
2.3 and kbd-status next. Next production task is 3.1. Goal remains active.

Post-boundary receipt: task 2.3 completed at revision 1841; matching position
projection read. MCP 4/22; actual OpenSpec 87/182, 95 remaining; canonical phase
implementation 2/10 and project-wide 103/120 unchanged. Memory mirror failed;
local append-only fallback is recorded. kbd-status completed; next is 3.1.

## Subsequent checkpoint — 2026-09-03, MCP task 3.1 PARTIAL

Begin-task projected-mcp-runtime 3.1 ordinal13/22 succeeded, revision1843,
memory mirror failed. NO end-task: task3.1 stays unchecked. No running compiler
or tool session remains at this checkpoint. Goal stays active, this is progress.

config.rs adds pub(crate) expand_from_environment(input, BTreeMap<OsString,
OsString>) with existing placeholder parser, :- defaults, strict missing/nonUTF8
interpolation and redacted errors. BindingEnvironment::resolve(cwd,inherited,
config) resolves overrides simultaneously against inherited then overlays them.

binding_cache.rs API CHANGED: connector futures now return ConnectedMcpServer,
not bare McpRegistry. ConnectedMcpServer::new(registry,catalog) owns transport;
Drop cancels unconsumed outputs. McpBinding owns a complete ServerToolCatalog
and exposes catalog(). registry_matches checks catalog identity/completeness and
every descriptor against the registry. Entry stores catalog on successful
generation-checked publication. invalidate/owner invalidation/shutdown clear it.
retire_connection(ticket) rotates generation/cancels/retires transport but keeps
complete catalog for later preparation. pin(request)->McpBindingTicket;
catalog(ticket), get_or_connect_pinned(ticket,request,connector) reject stale
generations, preventing old prepared steps from reviving revoked identities.

New runtime.rs exported. McpConnector async trait returns ConnectedMcpServer.
NO implementation yet. McpRuntimeManager::new(cache, Arc<dyn McpConnector>,
readiness_timeout,call_timeout) rejects zero budgets. prepare(request) pins a
ticket, always gets readiness for Global or missing cached catalogs, otherwise
defers skill/child startup. PreparedMcpServer provides catalog() for projection,
wait_until_ready() (compares complete discovered snapshot with prepared one),
call_tool(ProjectedMcpTool,args) with exact identity checks + one outer call
deadline including readiness; upstream host still owns schema/policy/approval.
retire_connection awaits cache cleanup. No manager consumer yet, grep confirmed.

NEXT: finish concrete connector under task3.1, not task3.2 or tests. It must
consume captured cwd/full env with env_clear, resolve commands against snapshot
inputs, preserve those inputs in reconnect slots, discover all tools pages with
bounded readiness, and unwind partial launch/discovery resources. Existing
from_config reads ambient env and only lists first page; not a compliant adapter.
Current rmcp::TokioChildProcess/RunningService cleanup behavior and list_all_tools
were inspected in ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-3.1.2.
Context7 official docs: docs.rs/rmcp/latest/rmcp/service/struct.Peer.html,
docs.rs/rmcp/latest/rmcp/service/struct.RunningService.html,
docs.rs/tokio/latest/tokio/process/struct.Command.html. Library drop cleanup is
not proof that UAR shutdown joins partial launches; phase-end tests must prove it.

HTTP detail: pinned rmcp Cargo.toml uses reqwest 0.13.2 range, Cargo.lock selects
0.13.4. Direct UAR reqwest is0.12.28. SDK with_client requires its own client
implementation, no reqwest client re-export was found. The default SDK factory
reads ambient/system proxy settings; an explicit snapshot-aware client must not
silently ignore captured HTTP_PROXY/HTTPS_PROXY/ALL_PROXY/NO_PROXY inputs. Current
Context7 reqwest docs cover ClientBuilder::no_proxy and explicit Proxy settings.
No dependency edits made. Original pasted handoff lists four specific operator
gates (jsonschema pin, liter error decision, sandbox decision, final flip evidence),
not a blanket ban on implementation dependencies. Do not invent extra gates;
inspect pinned versions and choose the smallest compatible connector integration.

T0 passed zero warnings twice:46.55s and34.57s. No tests, test authoring, fmt,
critic, dependency edits or workflow changes. Whitespace checks had no output;
no-index returns1 for new files. Actual totals87/182 (95remaining), MCP4/22,
canonical2/10 andproject103/120. Waypoint/position1843match, change/taskfields
stillnull but progress MCP IN_PROGRESS. The stale exactNextCommand names done
fail-closed-tool-arguments; continue the already-begun3.1 implementation.

## Subsequent checkpoint — 2026-09-03, task 3.1 stdio connector

Still task3.1, no end-task. Revision1843; totals87/182, MCP4/22, canonical2/10.
No active compiler/tool session at handoff. T0 passed zero warnings in32.21s.
No tests/fmt/critic/dependency edits. Previous turn and this turn are progress.

registry.rs now has public async connect_stdio_binding(Arc<McpBindingRequest>)
->Result<ConnectedMcpServer,McpBindingError>. runtime.rs adds unit
StdioMcpConnector implementing McpConnector via this constructor. This adapter
is explicitly stdio-only; do not use it as the complete production HTTP/stdIO
adapter or declare task3.1 done. No RunManager consumer yet (task4.1).

Private connect_stdio_snapshot validates sandbox/stdio/auth requirement,
resolves absolute program via snapshot_command (captured MCP_SERVER_DIR/PATH/
cwd, Unix execute permission, WindowsPATHEXT), spawns with env_clear/fullenv/
cwd/kill_on_drop, bounds handshake20sec. connect_stdio_binding lists ALL pages
within10sec, compiles descriptors/collisions via compile_discovered_tools, builds
complete ServerToolCatalog, and publishes an owned single-server registry.
Discovery failure awaits service.cancel normally; cancellation falls through
SDK-owned cleanup. Do NOT equate that with host-joined partial-child cleanup.

ClientServiceState now has snapshot:Option<Arc<SnapshotBinding>>, where snapshot
captures request and complete catalog. Initial legacy slots haveNone. New stdio
constructor setsSome; replace_configured_service clears it when changing config.
begin_reconnect returns ReconnectAttempt{slot,entry,snapshot,generation}, whose
Drop calls finish_reconnect. install_reconnected_service no longer decrements;
the guard handles successful/failed/cancelled paths. Reconnect uses snapshot
whenSome, original legacy connect whenNone with20sec timeout. Snapshot reconnect
re-lists allpages under10sec and compares ALL descriptors before install. A
changed catalog closes replacement and fails. Existing generation check kept.

HTTP PIN REQUEST SENT: add reqwest_mcp="0.13.4" under[pins] in versions.toml.
The dependency-pin-discipline SKILL.md was fully read this turn; it requires
every new direct pin there, and AGENTS forbids agent edits to versions.toml.
This is a real skill prerequisite, superseding the prior assumption that an
alias could be added solely from Cargo.lock. Check for user's edit/reply before
manifest changes. No dependency mutation made. First pending occurrence, with
substantial other progress; strict blocked threshold NOT met. Original goal
remains active; do not treat lack of a pin as permission to replace the HTTP
adapter with a weaker one or handwrite a whole protocol just to evade the pin.

Existing official cached index:
/Users/gqadonis/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/re/qw/reqwest
contains0.13.4, yanked=false, checksum219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3,
matchingCargo.lock. Direct crates.io API curl returned403; exact version docs
URLs failed web open. Context7 official rmcpdocs loaded list_all_tools/cancel;
prior reqwestdocs cover explicit Proxy/NoProxy and no_proxy. A direct alias to
the already-locked SDKclient avoids upgrading UAR's0.12 HTTP consumers, but
requires the operator pin. Select features against actual pinned manifests.

Remaining3.1: HTTPconnector exact URL/proxy/credential inputs, production
combined adapter, and partial-launch shutdown supervision. Safe code work on
the latter remains if pin is absent; do not run tests prematurely. Later4.1
must wire RunManager/ResolvedStep and preserve governed tool execution.

## Subsequent checkpoint — 2026-09-03, supervised stdio partial attempts

Supersedes the prior connector signatures and missing partial-launch ownership
note, not the pending behavior tests. New private src/mcp/stdio_process.rs has
StdioProcessSupervisor(Arc<SupervisorState>) with RwLock closed-admission flag,
root CancellationToken, TaskTracker, and sticky cleanup-failure AtomicBool.
spawn registers a tokio Child reaper while holding the admission read lock.
SupervisedStdioTransport delegates JSON-RPC to rmcp AsyncRwTransport, cancels
the reaper on drop, and awaits its watch completion on close. Reaper waits for
natural exit or cancellation; after cancellation it waits three seconds then
kill/reaps. Shutdown closes admission under write lock, cancels, closes tracker,
awaits its cancel-safe barrier and returns cleanup failure if any occurred.
Supervisor Drop only initiates cleanup; explicit shutdown remains required.
This handles direct children only, not a new sandbox/process-tree guarantee.

registry.rs connect_stdio_binding is now pub(crate), with second argument
StdioProcessSupervisor. connect_stdio_snapshot takes a supervisor reference;
SnapshotBinding stores its clone so reconnect uses the same tracked owner.
runtime.rs StdioMcpConnector is no longer a unit struct; construct with Default.
McpConnector now requires async shutdown()->anyhow::Result<()>. The runtime
manager's shutdown returns Result, draining cache then connector. No other
connector impl or RunManager consumer exists under src/tests as of this turn.

T0 command cargo check --locked --no-default-features --features server-full
passed zero warnings in26.57s, session21058 finished exit0. No tools/builds live.
No tests/test authoring/fmt/critic/dependency/workflow/pin edits. Whitespace
checks had no diagnostics. Task3.1 unchecked, MCP4/22, actual87/182, canonical2/10,
round4/5, waypoint revision1843; no end-task hook. This turn made source progress,
so do not count it toward a consecutive blocked-goal audit.

Remaining3.1: operator adds reqwest_mcp="0.13.4" under[pins] to versions.toml;
then implement snapshot-aware HTTP URL/credential/proxy client and combined
connector without upgrading UAR's existing reqwest0.12 users. Pin still absent
at latest read; dependency-pin-discipline requires it and agent may not edit
versions.toml. Real lazy/reconnect/cancel/shutdown proof stays at phase end.

## Subsequent checkpoint — 2026-09-03, task 3.2 complete

Task3.1 is still open for the HTTP alias pin; no pin edits. Independent task3.2
was begun and completed through the qualified KBD driver this turn. begin-task
revision1845, end-task1847, both exit0. Actual OpenSpec3.2 checked; MCP5/22,
actual88/182 tasks (94 remaining), canonical2/10 changes, round4/5. kbd-status
read/render completed after the task hook. Waypoint still null task/change and
stale exactNextCommand fail-closed-tool-arguments; do not follow that stale hint.

New public preflight.rs exports McpServerFailure (environment/auth/connection/
discovery/timeout, actionable secret-free text), McpPreflightWarning (private
fields, Serialize/getters), McpPreflightError (RequiredServer, Interrupted,
Projection), McpPreflight (private exact projection/prepared-server map/warnings
with getters). Private prepare_servers is called by new runtime manager method
preflight(&McpServerProjection,&ActorOwner,&McpBindingEnvironment).

Every selected definition resolves its env overrides against one captured
parent map. Required availability failure aborts; optional availability warns
and omits only that exact server. InvalidBinding/Invalidated/Cancelled/
ShuttingDown and other projection/call invariant failures abort regardless of
optional status. It never falls back to a lower origin or forces startup of a
complete cached skill/child catalog. Ready shared cache entries survive another
server's preflight failure for future reuse. projection.rs gains private
without_unavailable_optional_servers plus InvalidOptionalOmission error; this
rejects required/unselected omissions and leaves eligible_tools unchanged.
After filtering, preflight builds an exact step via existing with_tools.

T0 cargo check --locked --no-default-features --features server-full passed
zero warnings in31.79s. Compile session13504 and hook session43664 are finished;
no live process handle to wait on. No tests/test authoring/fmt/critic/dependency
edits. Calls exist across runtime/preflight/projection; RunManager still must
consume the API in task4.1. Behavioral evidence remains phase-end work.

Next safe independent code task:3.3 state machine/normalized events/status
metrics. Read current source before choosing wiring; preserve task3.1's missing
HTTP implementation and remaining full-phase scope. Both this and previous
turn made progress, so no consecutive blocked audit applies. One semantic
KBD task per turn remains in force, with kbd-status after every completion.

## Subsequent checkpoint — 2026-09-03, task 3.3 complete

Resumed3.3 (already begun1849), completed through qualified end-task driver at
revision1851, exit0. MCP6/22, actual89/182 tasks (93 remaining), canonical2/10
changes, round4/5. Status readback uses matching position.json revision1851:
chain0/0 >2/3 >2/10. Waypoint task/change null and exactNextCommand points at
already-done fail-closed-tool-arguments; ignore that stale hint. Older completion
evidence/certification/publication labels still refer to PR274/archive42/42.

Added McpServerStateChanged normalized event and typed lifecycle payload with
random event/binding/generation IDs, sequence, UTC timestamp, server, six states
and secret-free reason codes. mcp/lifecycle.rs publishes under a short lock and
feeds existing bool status recorder; registry and cache use it. Cache observes
exact owner/auth/config/env keys, runtime exposes observe. Initial snapshot plus
subscription admission is atomic. Bounded lag is returned explicitly; resync
returns Option<NormalizedEvent> (None after publisher drop). Subscriptions hold
Weak<RwLock<LifecycleState>> and do not keep publisher/sender alive.

Validated cache publication calls registry.attach_lifecycle. Shared service
slots retain publisher plus cache UUID distinct from existing config u64.
Reconnect is single-flight per slot; begin emits Connecting, checked install
Ready, typed failure AuthRequired/Failed, Drop Cancelled only while same-generation
Connecting. Shutdown emits terminal ShuttingDown; admin replacement detaches
old lifecycle identity. A late install or cancellation cannot resurrect it.
Projected reconnect avoids independent bool metric writes that could race state
events. Snapshot handshake preserves typed rmcp authentication challenge outcome;
discovery mismatch remains IncompleteCatalog. No failed call replay or changed
authority/env inputs. Global legacy status labels are deliberately unchanged.

Production AG-UI adapters accept uar.mcp.server_state CUSTOM and agui.mcp.state.
RunManager does not yet subscribe or forward binding events; task4.1 owns it.
Source searches confirm cross-module helper/adapter callers, not full live flow.
Task1.7 and live lazy/reconnect/cancel/shutdown tests remain phase-end work.

T0 cargo check --locked --no-default-features --features server-full passed
zero warnings in32.23s (before interruption),23.66s and11.95s (resumed edits).
All build/hook sessions finished; no live handles. No tests/test authoring/fmt/
critic/dependency/workflow/pin edits. Tracked source whitespace check exit0.
Task notes, decisions, phase decision-log and session-log appended. No
unrequested feature; guards address lifecycle ownership/races/trust boundaries.

Next task3.4: bounded eager exposure, model-only search_tools activates deferred
matches for the next step, Hidden stays omitted. Read source and task/spec before
begin. Task3.1 HTTP is still incomplete; requested operator reqwest_mcp="0.13.4"
pin remains absent. Do not add alias or edit versions.toml until operator pin
exists. Original19/19 done; broad goal remains active, with no blocked audit.

## Subsequent checkpoint — 2026-09-03, task 3.4 complete

begin-task3.4 revision1853, end-task1855, both finished exit0. Actual MCP7/22,
ten-change checklist90/182 (92 remaining), canonical2/10, round4/5. kbd-status
readback matches position/waypoint revision1855. Null waypoint task/change and
stale fail-closed-tool-arguments next command persist; do not follow that hint.
Original19/19 done; broad goal active. This and previous turn made code progress.

New public mcp/exposure.rs: McpToolExposure(Arc<RwLock<DiscoveryState>>), Default
per chat; project(&BTreeMap<String,Arc<ToolDescriptor>>)->McpExposureSnapshot.
Snapshot visible() returns unchanged descriptor Arcs, has_deferred() controls
search advertisement, exposure(name) reports effective Eager/Deferred/Hidden/
ModelOnly separately from declared metadata. Max32 MCP tools per step; native
controls outside that bound. Search(query) matches all terms against tool
name/id/server/description, ranks exact/name matches first, selects max8, validates
1..512 nonblank Unicode chars. Most recent selections take priority, retained
selection list bounded32, removed/hidden/changed descriptors dropped at projection.
No server I/O, permission grants, descriptor rewriting or secret-bearing state.

New native_skills/search_tools.rs: SEARCH_TOOLS_NAME="search_tools" and
SearchToolsTool::new(McpToolExposure), BuiltIn/ModelOnly/ReadOnly, key
mcp_tool_discovery. Returns selected_for_next_step plus matching OpenAI tool
definitions. Both new modules exported. McpStepProjection.model_tools now caps32;
exposure(state) applies discovery to that exact eligible descriptor set.

Orchestrator::chat_with_history creates a fresh exposure state and copies the
native registry before stream execution. Every iteration (after skill activation)
assembles authoritative descriptors, applies exposure, registers the handler
only when deferred MCP tools exist, and freezes visible descriptors for both
model advertisement and serial/parallel execution. Reserved-name conflicts fail.
Handler remains only in private stream copy, never host/child shared registry;
steps with no deferred descriptors do not advertise it. Search cannot make a
new call in the same batch valid because its map does not change mid-batch.

ResolvedStep::new's only production caller is the orchestrator; it now accepts
that host exposure-projected map, retaining selected originally-Deferred tools
instead of removing them. Hidden still excluded. Immutable descriptor equivalence
for governance and future binding checks is unchanged. RunManager's initial
turn manifest uses bounded exposure and adds search_tools only when needed.
Existing skill activation and original run bindings remain otherwise unchanged.

T0 cargo check --locked --no-default-features --features server-full: clean
28.81s; integration17.14s had unused Exposure import, immediately removed; final
clean11.30s. All compiler/hook sessions terminal; no live handles. No tests,
test authoring, formatter, critic, dependencies, workflows or pins changed.
Source search confirms actual orchestrator/manager users. Task1.4/live behavior
remain phase-end verification. Notes/decision/session files appended.

Next task4.1: replace per-run from_config with immutable catalog resolution and
binding lookup in RunManager/activation, plus lifecycle forwarding. Read all
current source before designing integration; typed/legacy/child binding paths
must preserve exact policy and credentials. Task3.1 HTTP remains incomplete and
requires operator reqwest_mcp="0.13.4" pin (still absent) before direct alias.
Safe source work may proceed independently; do not claim stdio-only as full3.1.

## Subsequent checkpoint — 2026-09-03, task4.1 partial (revision1857)

Task4.1 began successfully at revision1857 (ordinal17/22). Do not begin it again
or end it yet. MCP remains7/22; total90/182;92 remaining; canonical2/10;
Execute round4/5. No tests/test authoring/fmt/critic until the phase boundary.

preflight.rs now dispatches governed calls by exact projected tool/server.
Orchestrator has optional Arc<McpPreflight>, paired descriptor assembly and
MCP dispatch with no legacy fallback; native registry tools keep native routing.
ResolvedStep validates/retains the exact prepared snapshot. Manager reads
registry/preflight/descriptors/active state in one lock and passes preflight to
Orchestrator. Skill-use outcomes now use those descriptors' server identities.

activation.rs ProjectedActivationHost captures runtime/catalog/policy/ActorOwner/
environment. new_projected prepares globals; activate prepares combined active
skills before publishing body and bindings atomically. Legacy/frozen branches
remain. new_projected and ProjectedActivationHost::new have no root caller yet.

McpPreflight::freeze_bindings(companion) retains only companion native tools,
waits for exact prepared MCP bindings, narrows descriptors, merges and freezes
concrete transports, retains leases through capture, then rechecks revocation.
ActivationContext::freeze_mcp_bindings dispatches prepared vs legacy/frozen.
No root delegation caller exists yet. registry.freeze_bindings now uses
list_all_tools under the existing discovery timeout, not first-page-only.
No frozen merge/reconnect permission was loosened. Explicit delegation may
start lazy bindings; ordinary preparation still does not.

T0 command cargo check --locked --no-default-features --features server-full:
earlier execution20.13s and activation31.94s clean. Session85921 was lost;
fresh recovery check88188 finished exit0 in19.55s, zero warnings. Outcome
check78538 finished exit0 in9.55s; freeze check50671 finished exit0 in7.35s,
both zero warnings. No live tool handles remain. Tracked diff check exit0.

Remaining task4.1 integration: root ActivationContext::new still live; legacy
from_config still in activate. Shared runtime/cache bootstrap and shutdown,
captured global definitions/auth metadata/environment, eligible skill catalog,
policy-universe discovery, graph and root delegation callers, lifecycle events.
Manager build_universe_and_conversation currently includes global MCP names
only; resolve this before enabling projected activation, without ad hoc widening
of an already-resolved tool policy. GraphContext/LlmNode/ToolNode still use the
legacy registry and must not bypass captured grants/governance on migration.
Do not mark a helper-only integration as task completion.

Operator reqwest_mcp="0.13.4" pin in versions.toml still absent at readback;
async question requested it again. No pin/manifest edits. HTTP remains3.1 scope.
No tests, acceptance review, commit, deletion or workflow edits. Histories and
task notes appended. kbd-status must report stale waypoint next-command and old
PR274/archive42/42 evidence separately from this unfinished ten-change delivery.

## Subsequent checkpoint — task4.1 ingress owner wiring

RunExecutionRequest adds verified_owner:Option<ActorOwner>, with_verified_owner
and with_user_context. The latter preserves anonymous as None, rejects inconsistent
principal/anonymous tenant, otherwise uses ActorOwner::from_verified_context.
Manager validates stamp vs user_id before session mutation and carries it into
actual/shadow ResolvedTurn (private field, public getter, crate setter).
McpPreflight now owns ActorOwner; ResolvedStep::with_mcp_preflight requires exact
Some(owner) equality before descriptor checks, else AssemblyError::McpOwnerMismatch.

Live adapters wired: uar/api/routes.rs create/resume/checkpoint resume, server.rs
api_chat_completion, actor/agent_actor.rs to thread/actor_host.rs. Actor session
now owns ActorOwner rather than String and passes it into each root request.
HTTP context failures return401. Checkpoint input None/history/state preserved.
No token parsing, credential lookup or serialized identity fields added.

Legacy positional APIs, embedded callers and continue_with_interaction still
lack the stamp. Do not fabricate one from user_id when enabling cache use;
provide explicit host identity/isolation. Anonymous MCP needs deliberate scoped
host ownership while preserving existing behavior. Root constructor stilllegacy;
new_projected/freeze_mcp_bindings still lack root callers. Existing4.1 remainder
and HTTP pin prerequisite unchanged. No task checkbox/end hook; revision1857.

T0 command cargo check --locked --no-default-features --features server-full:
core74851 clean18.47s; HTTP79551 clean9.87s; actor95597 clean8.14s;
owner-compare88783 clean12.90s. All exit0, no live handles. Tracked diff check0;
production call-site search recorded. No tests/fmt/critic, deps/pins/workflows,
commit/deletion. Counts90/182,92remaining, MCP7/22, canonical2/10, round4/5.

## Subsequent checkpoint — task4.1 supplied root MCP consumer

runtime.rs McpRunResources::new(owner,runtime,catalog,environment) captures exact
inputs; getters expose host-only references. Clone shares cache and supervisor.
RunExecutionRequest.mcp_resources:Option<McpRunResources> defaultsNone. Manager
checks resource-owner equality with verified_owner, forbids child injection,
requires resolved_policySome and matching cwd when separately supplied. Uses
captured cwd when request cwd absent. A captured graph request currently exits
projected_mcp_graph_unavailable before session mutation: temporary adapter gap.

Manager now calls ProjectedActivationHost::new and ActivationContext::new_projected
when resourcesSome; None uses existing constructor. Failure updates run status,
emits Error/RunDone or Cancelled and removes run cancellation registration.
Host constructor now requires CancellationToken; prepare races runtime.preflight
against it with biased cancellation, covering every activation. New preflight
error Cancelled. Shared runtime shutdown remains external host ownership.

T0 cargo check --locked --no-default-features --features server-full:
42607 exit0 clean34.12s;61075 exit0 clean13.40s. All handles terminal. No tests,
fmt, critic, dependency/pin/workflow edits, commit or deletion. Source search
confirms manager new_projected caller; still no McpRunResources::new caller in
bootstrap/API. This is not the completed default migration. Task4.1 unchecked,
MCP7/22, total90/182, canonical2/10, revision1857. Do not begin task again.

Next: standard bootstrap capture and catalog-aware policy universe; complete
HTTP after operator pin, governed graph/child handoff, lifecycle forwarding and
joined shutdown. Remove temporary captured-graph rejection before task closure.
Legacy embedded/interaction/anonymous identity work also remains. No scope cut.

## Subsequent checkpoint — 2026-09-03, thread4.1 actor shutdown

Inspection confirmed actor4.1 is not ready to close: Collaborate still invokes
ActorThreadSession::execute as an independent root. Do not relabel that a real
child. No repeated begin-task: thread4.1 began at1825, MCP4.1 at1857.

Source changes: actor/system.rs owns an actor-system child CancellationToken,
checks it under the spawn publication lock, cancels before shutdown snapshot,
and derives mailbox tokens from it. ActorHandle's JoinHandle is now an
Option behind a per-handle Tokio Mutex, retained in registry-owned Arc handles.
Join awaits by mutable reference and clears only after completion. Cancelling
a stop or shutdown future preserves ownership for later cleanup. stop_actor
removes only the matching Arc after join; shutdown_all snapshots without
removing, cancels all, joins then removes. No registry lock across joins.
server.rs async_resource_cleanup now calls actor_system.shutdown_all().await
before shared MCP/live-query cleanup. Existing hard shutdown deadline unchanged.

Compile-only T0 command cargo check --locked --no-default-features --features
server-full: 33005 exit0 17.42s,9478 exit0 25.05s,83030 exit0 15.73s; zero warnings.
All terminal. Source git diff --check exit0, no output. No tests, formatter,
critic, dependencies, pins, workflows, commits or deletion. Task notes and
append-only memory updated. Named guard scenarios: post-shutdown admission,
HTTP cancellation losing join ownership, and a stale stop removing a replacement.

Position remains revision1857, Execute round4/5, canonical2/10, checklist90/182,
92 remaining; thread5/25 and MCP7/22. No task completed and no end-task hook.
Full kbd-status refreshed, not acceptance. Next implementation still requires
root captures/attachment and a concrete ThreadExecutionHost, actual actor child
collaboration, graph/A2A adapters, full child budget/sandbox enforcement. MCP
HTTP alias still awaits the operator pin. No runtime shutdown/race tests ran;
retain their phase-end obligation and the existing hard-stop limitation.

## Subsequent checkpoint — 2026-09-03, captured child kernel entry

Concrete source progress, no completed numbered task. Revision1857; Execute
round4/5, implementation2/10, checklist90/182, thread5/25, MCP7/22. No repeat
begin-task or end-task, no goal status change, no phase-end test work.

New public thread/kernel.rs CapturedThreadKernel has private construction via
RunManager::capture_thread_kernel(owner, root record, persistence). Root resource
capture is automatic for verified ordinary roots after model/step assembly:
RunStreamState holds Weak<RunDelegationBindings>; root coroutine owns the
RunDelegationLifetime, which cancels a dedicated child token on Drop. Resource
Arcs include exact models/skills/native/activation/harness/cwd/approvals. The
canonical cwd accessor is WorldStateRuntime::directory. Borrower Arcs do not
extend authority beyond the root's lifetime; children/anonymous runs publish none.

Capture checks verified owner including tenant, root identity and exact live DB
record plus Running kernel before and after freezing MCP. This is now a caller
of ActivationContext::freeze_mcp_bindings. Snapshot native implementations at the
same capture. Artifact lookup uses resolve_registered_agent, never a default
substitute. history validates lineage and current record, allowing one next-turn
transition for a resumed child; canonical_thread_history returns per-run dialogue.
RunDialogue is a private redacted wrapper around an independent Session snapshot,
initialized after root input and updated alongside existing graph/assistant/tool/
interruption conversation writes. New generated world-state contributions remain
owned by WorldStateRuntime and are reconstructed on child checkpoint restoration.
This is in-process history, not a new persistent recovery guarantee.

CapturedThreadKernel::execute takes HostedThreadTurn, verifies committed child,
root and resource-subset selection, requires frozen servers and policy-qualified
captured model grants, splits the final MessageContent::Text input exactly once,
retains earlier child history unchanged, and sends Some(InheritedRunBindings)
through manager.execute_captured_thread. Verified ActorOwner, same root approvals,
cwd and snapshots reach the child. Explicit caller cancellation cancels the
derived token and awaits the same execution future; a local Drop guard cancels
if the caller discards it. The RunCompletionGuard supplies producer-unwind result.

ThreadService now has shutdown(): cancel/close admission atomically with job
registration, retain job slots through awaited joins, preserve failed receipts,
and only then reconcile/finish remaining child records as Cancelled. It never
writes the root's terminal state. Uncertain writes use confirm/readback, not a
blind retry. tracked operations reap finished jobs before pruning; panic closes
the host. There is still no root adapter caller of this shutdown method.

T0 command cargo check --locked --no-default-features --features server-full:
27857 failed E0308 on Option<&str>/String; fixed using the actual MessageContent
enum so image/other parts cannot silently disappear. Passing sessions:
42285=31.13s,17554=24.17s,6197=31.00s,67083=12.81s,94229=17.08s,29680=31.77s;
all exit0 and warning-free. No live build handles. Tracked diff check clean;
no-index added-file check reports file difference (exit1), no whitespace diagnostics.
No tests, authoring tests, fmt, strict validation, critic, dependencies, pins,
workflows, commits or deletes. Code behavior/phase acceptance remain unverified.

NEXT: do not claim ThreadExecutionHost exists. The concrete kernel capsule
supplies artifact/history/execute, but host check_admission still needs actual
sandbox/budget enforcement. No actor/graph/A2A caller invokes capture_thread_kernel.
Root permission capture, one ThreadService attachment, control installation,
joined root closure and real actor collaboration are still absent. Do not expose
children using fabricated physical grants or an admission no-op. Keep graph,
A2A, full root/child budgets and MCP bootstrap/HTTP integration in scope. The
operator-owned reqwest_mcp pin is still a separate prerequisite, not a reason
to stop all code progress. End-task only once the complete semantic task passes.

## Subsequent checkpoint — 2026-09-03, shared root budget admission

Continue task thread-native-subagents::5.1 (ordinal20/25), begun successfully at
waypoint1859, not completed. No duplicate begin or end hook. Checklists unchanged
90/182 overall and5/25 for this change; implementation2/10, executionround4/5.

Budget code now captures strict ThreadBudgets plus root start time. Descendants
reuse root keys/counters and min-intersect ceilings/deadlines. Model drivers
charge cumulative known token/cost updates before yielding, enforce turn/session
token and session/configured-scope cost limits, and use root request/token-rate
windows. Finite costs require a catalog price. Deadline races model start and
stream consumption. Tool gate consumes a root tool attempt after approval or
governance bypass. Failed execution does not refund authorized attempts.
RunModelBindings now stores unwrapped clients, wraps primary/fallback at use,
and applies narrowed child budgets exactly once. manager.rs removed the legacy
session-dollar ceiling assigned to Agent scope; strict budgets now use the root
session. Existing parser tests retain the old helper behind cfg(test).

ThreadService::attach now REQUIRES Arc<CapturedThreadKernel> after executor.
It verifies exact captured root and budget, retains the capture, and separately
checks budget before spawn/resume/child entry, including after preflight/history.
No callers of ThreadService::attach exist yet. Do not fabricate a capture or
replace sandbox admission with a no-op just to attach it. Concrete host and
actor/graph/A2A attachment remain the next implementation dependency.

T0 five passes zero warnings:35.14s,20.72s,15.31s,13.61s,12.11s. All terminal.
No tests/test authoring/fmt/strict validation/critic/dependencies/pins/workflows/
commits/deletes. Budget behavior is not proven by compilation. In-flight calls
can overshoot known-usage ceilings; absent usage and remote billing after local
cancel remain uncertain. Existing tools are not stopped at deadline by this edit.
Task5.1 stays open until the real child path is wired; task4.1 still owns the
missing adapter/attachment. Keep remaining graph/A2A/cancellation/MCP work in scope.

## Subsequent checkpoint — 2026-09-03, actual sandbox dispatch and backend capture

Task4.1 continued (before-hook already17/25), not completed. Waypoint1859 unchanged.
No tests at this point. T0 command cargo check --locked --no-default-features
--features server-full passed three times zero warnings24.77s/25.46s/23.50s; all
handles terminal. Tracked diff check clean. No dependency/pin/workflow/commit/
delete or external sandbox operations.

Source changes: Orchestrator receives artifact execution_mode from manager;
requires_sandbox applies Sandboxed to all calls, Auto to CodeExecution, and honors
descriptor-required isolation even in Direct. Parallel path uses the predicate.
No runner/adapter falls through to native/MCP. SandboxRunner::enforces_isolation
defaults false; RemoteRunner declares a trusted configured service, not an
attestation. WasmtimeRunner was inspected and runs raw host processes, so do not
wire it as isolation. The legacy backend implementation was not changed.

Explicit NativeSkill::sandbox_request replaces all heuristic code extraction.
TerminalExecTool implements sh,/bin/sh,bash,/bin/bash via shell-quoted exec in the
existing Bash language protocol; preserves env/cwd and clamps caller timeout to
the configured ceiling on both direct and sandbox paths. Other tools/shells do
not silently receive guessed execution semantics. Full adapters remain required.

configured_isolated_runner in sandbox/platform.rs is now called by server.rs
after constructing RunManager. It snapshots existing UAR_SANDBOX_* configuration,
supports remote only, validates HTTP(S) base URL without userinfo/query/fragment,
and returnsNone when nothing configured (no fake isolation fallback). Explicit
unsupported configuration returns startup error. Manager.with_sandbox_runner has
a real server caller. RunDelegationBindings and InheritedRunBindings each retain
the exact runner Arc; CapturedThreadKernel passes it to execute_request_inner.
The root/child backend no longer requires recapturing ambient credentials.

NEXT REQUIRED: owned sandbox-operation lifecycle before accepting this route.
Orchestrator currently creates/executes/destroys inline. Manager cancellation
drops stream.next, which can lose a created handle. Destroy errors are ignored;
lost create replies are uncertain remote state. Do not claim cancellation-safe
cleanup or silently retry unknown creation. Physical sandbox filesystem/env/network
binding capture/enforcement, concrete ThreadExecutionHost and root attachment,
actor collaboration, graph/A2A and full child cancellation are still open.
No checkbox changes:thread5/25,overall90/182,implementation2/10,round4/5.

## Subsequent checkpoint — 2026-09-03, owned sandbox lifecycle and binding integration

The preceding NEXT REQUIRED inline-lifecycle item is superseded by actual code.
New sandbox/execution.rs owns create/execute/destroy JoinHandles and preserves
unknown receipts without replay. A SandboxRunLease cancels on drop; supervisor
finish_run joins before releasing confirmed scopes, retains failures and exposes
content-free operations. Backend panics are caught; execution unwind still runs
destruction. RunManager finalizes on ordinary completion, cancellation and caught
unwind before dropping the actor completion guard. Server shutdown calls the
same supervisor after joining actors. cost_budget.rs supplies the captured root
deadline, including creation elapsed time. Create/destroy remain awaited because
dropping a remote request does not establish rollback.

New sandbox/bindings.rs retains backend/config and opaque host environment IDs.
RunManager still accepts its installed runner, then captures the existing default
configuration once per root. RunDelegationBindings/InheritedRunBindings now carry
`sandbox: Option<Arc<SandboxBinding>>`, not sandbox_runner. Child kernel entry calls
for_permissions; no environment/client reconstruction. SandboxSupervisor::open_run
now also takes that binding, and SandboxRun::execute accepts runner/request, not
caller-supplied config. It verifies the runner's Arc identity and blocks protected
environment replacement. Default config has no mounts/env and network=false;
nonempty mount configs/filesystem grants remain unsupported, not falsely mapped
to read-only using the untyped remote volumes map. Direct native tools are NOT
constrained by this sandbox-only binding.

Observed current checks: baseline1.96s, join cancellation fix24.60s, final integrated
binding20.31s, all T0 exit0 zero warnings. An intermediate standalone module pass
reported two unused warnings, fixed by production callers. Prior process73429
was unavailable; baseline was rerun. No tests/test authoring/fmt/strict validation/
critic/dependency/pin/workflow/commit/delete/remote calls. Tracked whitespace check
clean; new-file no-index checks produced no whitespace diagnostics (exit1 diff).

NEXT: concrete ThreadExecutionHost admission with real direct-tool permission
enforcement and root ThreadService attachment; then actor child collaboration,
graph/A2A adapters, remaining budgets/cancellation/MCP plumbing. Do not claim a
service is attached: no actor/graph/A2A caller yet captures/attaches it. Do not
count unconfirmed cleanup as Cancelled/success, or static remote isolation
declarations as attestation. Task4.1/5.1 open; no end hook/checklist changes.
Waypoint1859; thread5/25,total90/182,implementation2/10,round4/5. Tests at phase end.

## Subsequent checkpoint — 2026-09-03, concrete host and native permission seam

Supersedes the prior missing-concrete-host claim, NOT the missing adapter caller.
CapturedThreadKernel implements ThreadExecutionHost. It captures opaque MCP grants
for its exact frozen transport view; root_policy derives ThreadPermissions from
actual model/native/MCP/sandbox captures and the original run artifact. Admission
checks owner/live root, selected-resource subsets, captured limits, model routes/
budgets, MCP server/tool/credential binding equality and sandbox/adapter support.
ThreadService::attach now takes (Arc<CapturedThreadKernel>, events, user_grant),
not caller-provided root/policy/artifact/persistence/executor/cancellation. It gets
those from the capture. RunDelegationBindings retains original artifact and an
AtomicBool claimed by attach; repeated captures share the claim, so no second
zero-counter scheduler and no in-place retry after failed attachment.

NativeExecutionContext now has verified_owner and thread_policy. Orchestrator
supplies both from admitted state and uses execute_native for sequential and
parallel calls; it invokes NativeSkill::check_thread_policy before the overridable
contextual execution method. Default is rejection, not effect-label inference.
Echo/system_info have resource-free implementations; session_search enforces
memory enablement and owner-qualified lookup/returned-record identity. Its naked
execute now rejects missing verified context instead of reading anonymous data.
Terminal advertises supports_sandbox_execution only for its existing sh/bash
adapter. Legacy MCP-native calls are rejected for delegated turns until ported.

Turn-local agent controls check exact policy/authorization; activation and search
retain the exact policy used to create them. Child registry filtering now removes
search_tools too, preventing parent exposure-state reuse under equivalent-schema
dedup. Their fresh constructors receive the child-local policy.

T0 five passes zero warnings27.49s/22.50s/17.15s/24.94s/21.51s. All terminal.
No tests/test authoring/fmt/strict validation/critic/dependencies/pins/workflows/
commits/deletes/remote calls. Tracked diff check clean. Actor attachment is STILL
ABSENT: rg finds only capture_thread_kernel's definition, no ThreadService::attach
call. ActorCollaboration::collaborate still sends to a target actor independent
root. Root AGENT_TOOL_NAMES are not yet included in the actual policy universe;
do not expose tools by widening a resolved policy after governance.

NEXT SOURCE WORK: feed a host-owned committed actor root/persistence into manager
assembly before model launch, include real root control eligibility in normal
policy resolution, attach service once to the published executable capture,
install fresh root controls and retain/join service before root terminal events.
Then route actor collaboration through real child threads. Native file/patch/
web/compiler/A2UI and legacy MCP-native permission ports remain incomplete;
returning an unsupported error is temporary protection, not the requested final
feature. Keep graph/A2A, budgets/cancellation and MCP wiring in the full scope.
Uncomfortable limit: no behavioral claims from compilation; phase-end integration
tests must cover identity, policy checks, handler reuse and single attachment.
Task4.1/5.1 remain unchecked, no end hook. Waypoint1859,round4/5,2/10,90/182,thread5/25.

## Subsequent checkpoint — 2026-09-03, actor root attachment callers now exist

Supersedes the preceding NEXT SOURCE WORK paragraph. Actual actor root handoff,
normal-policy control eligibility, capture/ThreadService attachment, fresh root
handler registration before manifests, child-tree draining and producer joining
are now wired. Actor collaboration no longer sends a Collaborate mailbox command
that creates an independent root: it checks source readiness/owner/Cedar, records
explicit root-user delegation, spends root tool allowance and awaits a real child
on the target's registered artifact. Raw mailbox Collaborate now refuses.

ActorRootBinding has record, exact persistence, service OnceLock, readiness AtomicBool
and a retained ActorProducer join slot. It is shared by ActorThreadSession and
ActorHandle. Manager only calls shutdown (children); actor/registry calls finish
(join producer, then children). Do not call finish from the producer itself.
Registry retains failed roots after mailbox joining; next turn cannot discard an
unresolved receipt. Control handlers are installed only after snapshotting native
resources, preventing a service/kernel/handler ownership cycle. Child admission
still requires the stronger adapter contracts; root capture does not disable
existing direct root tools merely because their child ports are unfinished.

Final T0 cargo check --locked --no-default-features --features server-full exit0,
zero warnings23.18s. Earlier zero-warning13.29s/20.77s/23.22s; one unused-field
warning and one E0609 field typo corrected. All build handles terminal. No tests,
test authoring, fmt, strict validation, critic acceptance, dependencies/pins,
workflows, commits, deletion or external service operations. Tracked diff clean;
new-file no-index whitespace checks produce no diagnostics (diff exit1).

NEXT ACTUAL SOURCE WORK: finish native delegated permission support (file/patch/
web/compiler/A2UI and legacy MCP-native; direct terminal handling still unported).
Inspect existing implementations and capture their actual authority rather than
treating a ReadOnly label or an empty sandbox profile as filesystem/process rights.
Currently unsupported delegated calls are rejected, NOT delivered functionality.
Then graph4.2, A2A4.3, remaining5.1/5.2 and projected-MCP wiring. Actor collaboration
and cleanup now have actual source paths but no runtime evidence until phase end.
Keep task4.1 open for its incomplete permission integration. No status/end-task
hook fired because no complete task boundary occurred. Counts unchanged; goal active.

## Subsequent checkpoint — 2026-09-03, compiler/memory/web ports now exist

Task4.1 is still open, but do not repeat the superseded unported list above.
Four compiler tools now have host-scoped sessions/captured local-signing
contracts; six legacy native memory tools have contextual verified-owner ports;
web fetch binds its parsed authority to SSRF-checked addresses, disables ambient
proxies and caps streamed bytes. Details and file-by-file accounting are in
.prometheus/session-log.md, entry "Compiler, memory and web native delegation
ports (4.1 partial)". No checkbox/end-task hook; counts remain round4/5,
implementation2/10, checklist90/182, thread5/25, project103/120.

All current compile handles terminated: 30969 exit0/zero warnings47.45s;
91246 exit0/zero warnings32.56s; 30228 exit0/zero warnings15.11s. Targeted tracked
diff check exit0/no output. No tests/test authoring/fmt/strict verification/
acceptance critic/dependency/pin/workflow/commit/delete operation this checkpoint.

NEXT SOURCE WORK: file/patch, direct-terminal and A2UI delegated permission
support remains. Inspection confirmed file canonicalize-then-open/write races
and terminal output() under timeout with no retained child handle. Do not admit
these paths by adding a blanket policy Ok. No terminal/A2UI edits were made in
this checkpoint. Follow required UI routing if changing A2UI behavior. Then
graph4.2, A2A4.3, remaining5.1/5.2 and projected-MCP integration. Tests only at
phase end. Actor attachment/collaboration now exist; do not rebuild them.

Compatibility/risk: automatic HTTP proxies are disabled; memory uses existing
user_id ownership, deleted-history cannot use a missing record as authority,
and issued remote mutations are not rolled back by timeout. Compiler concurrent
get/update algorithms were not redesigned. DNS blocking work was not given a
new supervisor. These claims have compile evidence, not runtime acceptance.

## Subsequent checkpoint — 2026-09-03, bounded file I/O and new pin gate

Current goal turn made source progress: file_tools.rs now bounds reads on the
opened handle and enforces exact byte limits; file_patch.rs checks output
expansion before allocation and patches through its original handle. Write
success waits for flush. T0 session54886 terminal exit0/zero warnings12.46s;
targeted diff check exit0/no output. No tests/test authoring/fmt/strict checks,
manifest/lockfile/pin/workflow/commit/delete changes. Task4.1 and counts unchanged.

DIRECTORY CONFINEMENT STILL REQUIRED. cap-std4.0.2 is already in Cargo.lock
transitively, not a direct Cargo.toml dependency, and absent from versions.toml.
dependency-pin-discipline requires the pin; agents cannot edit versions.toml.
Asked operator to add cap_std = "4.0.2" under [pins] with a confined-file-access
rationale. Check the live file before proceeding. Exact version verified using
https://docs.rs/cap-std/4.0.2/cap_std/fs/struct.Dir.html ; Context7 returned no
relevant cap-std result in two searches. Do not introduce that dependency until
the operator pin exists; do not route around the gate via another crate.

Capture actual allowed directory handles and execute relative opens through
them once adoption is authorized. Single-handle read/write is not confinement
of the first open. Preserve permission filtering and fail-closed delegation.
Direct terminal/A2UI, then graph4.2/A2A4.3/budget/cancellation still remain.
This is the first pin-gate observation, not three blocked goal turns. All prior
compile handles are terminal. Goal stays active; no completion/end-task hook.

## Subsequent checkpoint — 2026-09-03, managed terminal process ownership

Source progress, not a blocked turn: cap_std manual pin still absent, but terminal
host lifetime work proceeded. New src/uar/tools/terminal_process.rs owns per-run
process jobs and exact Child handles with retained/borrowed joins, closed
admission, deadlines, cancellation and bounded stdout/stderr head/tail capture.
NativeExecutionContext and Orchestrator pass the real TerminalRun. TerminalExec
uses it for managed calls and rejects verified contexts without one. Raw callers
keep legacy capture with kill_on_drop fallback. Manager creates/drains scopes at
normal/cancel/graph/unwind paths; server shutdown calls shutdown_terminals.
Task4.1 not complete; direct delegated terminal execution remains denied.

T0 handles both terminal:98993 exit0/zero warnings39.43s;35925 exit0/zero warnings
19.92s. Targeted tracked diff check exit0/no output; new module no-index check
exit1/no whitespace diagnostics. No tests/test authoring/fmt/strict checks/
acceptance critic/dependencies/pins/workflows/commits/deletions. See the matching
session-log entry for file-by-file accounting and API evidence.

Remaining: the terminal Child handle does not own descendants/daemonized work;
no physical direct-host permission binding was added. Raw standalone callers
still lack supervised joining and bounded raw output. Do not claim either is
solved or enable delegated direct shell on this basis. File confinement still
awaits operator cap_std = "4.0.2" pin. A2UI permission work then graph4.2/A2A4.3,
budget/cancellation and projected MCP remain. Original scope/goal intact, counts
round4/5,2/10,90/182,thread5/25 unchanged. No task end/status completion boundary.

## Subsequent checkpoint — 2026-09-03, A2UI permission port

A2uiRenderSkill now explicitly permits its pure declarative validation under
the inherited child policy; governed dispatch still enforces tool selection,
approval and sandbox mode. Manager publication remains actual-run-owned. No UI
or protocol changes. T0 session31755 completed exit0, zero warnings13.55s;
targeted git diff --check exit0/no output. No tests or test authoring, fmt,
strict checks, acceptance review, dependency/pin/workflow/commit/deletion.

kbd-status fully rendered at turn start, revision1859: current phase2/10,
checklist90/182, round4/5, thread5/25, project103/120. No semantic task ended;
4.1 remains open. Do not repeat the A2UI permission-port work. Large-result
truncation can still prevent JSON publication; no delivery claim or transport
fix was made. Graph4.2 inspected only, no begin-task hook or source edits.

Next file-confinement implementation still awaits the operator adding
cap_std = "4.0.2" under [pins] in versions.toml (absent at this checkpoint).
Do not edit that file or add the dependency without its authoritative pin.
Direct delegated file/shell tools remain denied. Subsequent work: graph4.2,
A2A4.3, remaining budget/cancellation and projected MCP integration; preserve
the existing actor/root/kernel paths. Runtime acceptance remains phase-end work.

## Subsequent checkpoint — 2026-09-03, graph4.2 started; root caller still missing

Actual source progress: new graph/delegation.rs uses host controls plus the real
approval/budget gate; AgentNode no longer makes a fabricated local LLM call.
GraphContext carries the host delegate; manager supplies actor/inherited controls
and removes route prefixes. Defaults now define/seed general-purpose and
rust-reviewer without overwriting operator specialists; only orchestrator gains
explicit spawn authorization. ThreadService retains first-turn terminal receipts
so graph waits cannot return a resumed turn's result. See session-log matching
entry for file-by-file details and all constraints.

T0 sessions all terminal exit0/zero warnings:1392 29.80s;82990 16.54s;
98256 20.31s;37076 12.04s;52870 9.54s. Source-only fresh artifact critic completed:
P1 ordinary graph root attachment missing (OPEN), P2 later pending write hiding
first receipt (FIXED, compiled). No tests/test authoring/fmt/strict acceptance,
dependencies/pins/workflows/commits/deletions. Test constructors still require
GraphContext.thread_delegate at phase end; T0 does not compile test targets.

NEXT: finish4.2 by attaching a persisted, owned root for ordinary graph execution
and checkpoint entry points. They currently lack controls and now deny local
AgentNode instead of using the removed shim. Preserve verified owner provenance,
full RunExecutionRequest inputs, actual captured resources, joined cancellation,
terminal persistence and uncertain-write receipts. Do not infer verified owner
from a plain legacy user_id. Existing actor attachment and child kernel paths
are present; extend caller ownership rather than rebuilding them. Remote graph
A2A remains the old path and must be ported with4.3/5.2. Do not mark4.2 complete.

Canonical current waypoint revision1866, active chain unchanged, currentTask
stillnull in projection. Actual semantic task4.2 is in_progress with before hooks
at14:07:26Z. Do not repeat begin-task. Accidental ordinal record18 was cancelled
through typed API (not deleted); ledger6/26 includes this cancelled record.
Actual OpenSpec5/25 and90/182; canonical implementation2/10, project103/120,
round4/5 unchanged. No task end-hook. File4.1 still awaits cap_std manual pin;
there is other actionable implementation, so the goal is not blocked.

## Subsequent checkpoint — 2026-09-03, local graph task4.2 complete at Tier0

Supersedes the immediately preceding missing-ordinary-root caller checkpoint.
GraphRootSupervisor now owns persisted graph roots outside actor mailboxes;
ActorThreadSession preserves the complete verified request. RunCompletionCapture
has an observer-only mode so internal cleanup does not defeat last-SSE-viewer
cancellation. Actual ordinary graph callers are wired through RunManager;
discovery includes control names before policy resolution. Host identity cannot
be inferred from a legacy user_id. Preparation failure creates an isolated error
run without consulting another conversation's history. Worker/join/root session
ownership persists through cancelled waiters and unconfirmed writes.

Independent source critic found graph terminal status stuck Running and hidden
graph shutdown errors. Fixed manager graph exits to set Done/Error/Cancelled and
attempt all cleanup paths. Server retains graph/terminal/sandbox cleanup errors,
returns failure and does not report graceful success. Source-only re-review found
no remaining findings in those two paths. Intermediate private Session APIs
(E0624/E0603) replaced with public isolated SessionStore; unused cleanup now wired.

Tier0 cargo check --locked --no-default-features --features server-full final
passes:37547 exit0/no warnings1m57s;45000 exit0/no warnings21.23s;98430 exit0/no
warnings17.71s. All compiler handles terminal. Targeted tracked git diff --check
exit0/no output. No tests/test authoring/fmt/strict acceptance, dependencies/pins/
workflows/commits/deletions. Tests are phase-end work, NOT silently passed.

KBD end-task used semantic4.2 and exact canonical title, index18,total25.
Successful command session86181; task:after hooks14:30:56Z; waypoint revision1868,
updated14:31:01.238758Z. Full kbd-status rendered AFTER completion, with required
completion signal. Actual thread6/25; total91/182; canonical implementation2/10;
project103/120; round4/5. Ledger7/26 includes cancelled ordinal18; prompt ledger
10/20 also differs from actual9/18. Never count these as additional implementation.
Waypoint currentTask/change remain null and exactNextCommand remains stale.

NEXT: task4.3 A2A handler onto thread service and client cancellation propagation;
its begin-task has NOT run. Use canonical semantic ID and exact stored title.
Remote AgentNode still uses old A2A path and belongs with4.3/5.2. Captured-MCP
graph dispatch still explicitly rejects rather than using ambient bindings;
integration remains open. Direct file confinement4.1 awaits manual cap_std4.0.2
pin; direct delegated shell permissions remain unsupported. reqwest_mcp pin also
still absent; do not edit versions.toml. Other code remains actionable, goalactive.
Phase-end test work must update GraphContext constructors and replace old
synthetic AgentNode expectations with real child tests, then test lifecycle,
ownership, persistence recovery, cancellation, tools and full integration.

## Subsequent checkpoint — 2026-09-03, A2A4.3 ingress and client receipts

Task4.3 HAS begun, superseding the preceding next-task note. Begin-task used
semantic ID4.3/index19/25/exact title "A2A handler maps onto the thread service;
client propagates cancellation"; successful session4329, waypoint1870 at
14:34:46.887022Z. No end-task. Execute, Round4/5, implementation2/10, actual
91/182 tasks, thread6/25. Ledger7/26 includes cancelled ordinal18. Project103/120.
Waypoint currentTask/change remain null and exactNextCommand remains stale.

Implemented source: shared inbound A2AThreadService for HTTP/gRPC, owner-qualified
task/context bindings and actual named actor-root turns; exact ActorTurn receipts
and spawn_session capability; host-owned complete ActorThreadSession retained
after mailbox stop; failed joins/uncertain persistence survive cleanup retry;
actor shutdown propagates to server coordinator. Compiler-agent default selects
the four real compiler tools and is seeded only when absent. Named HTTP route
POST /a2a/agents/{agent_id}; gRPC x-uar-agent-id selects target, never identity.

client.rs fixes message/send wrapper and exact RPC/task receipts, redacts bearer
Debug and adds task_execution factory. New task_execution.rs owns borrowed
mutation futures across cancelled waiters, waits for task-ID receipt before
parent cancellation, polls to terminal, cleans up after poll failure and retains
unknown outcomes. No detached task or implicit mutation replay. The owner must
retain the object through cleanup; dropping it is not proof of remote stop.
Task::cleanup_unconfirmed is shared across direct cancellation and the driver.
Inbound publishes uncertainty BEFORE awaiting stop and clears it only on
confirmed settlement; Failed can mean unconfirmed cleanup, not remote stop.

All compiler sessions terminal. Current-turn results: resumed36184 exit0/zero
warnings19.26s;89572 exit0/zero warnings16.12s;91685 exit0/one redundant Future
import warning1m02s; removed warning,28949 exit0/zero warnings13.47s;63901
exit0/zero warnings13.83s;96545 exit0/zero warnings45.30s. Command for all:
cargo check --locked --no-default-features --features server-full.
Latest output: Finished `dev` profile [unoptimized + debuginfo] target(s) in 45.30s.
Tracked source diff check exit0/no diagnostics. No tests/test authoring/fmt/
strict acceptance/dependency/pin/workflow/commit/delete operations.

artifact_critic_a2a_ingress is idle after source-only review. Final review:
no unresolved findings in reviewed paths; not phase acceptance. It identified
the exact-capability/persistence retention, retry ordering, false terminal-cleanup
and dropped-waiter wire-marker issues, now fixed and compiled. Use fresh artifact
source only for further review; never supply generation history or self-review.

NEXT / UNCOMFORTABLE LIMITS:
- A2AClient::task_execution has NO actual graph/thread-host caller (rg confirmed).
  Remote AgentNode still uses old one-shot send. Do not mark4.3/5.2 complete.
- Operator question is outstanding: trusted UAR peers with explicit policy/
  root-budget enforcement, or arbitrary A2A with local tracking/cancellation.
  Current plan requires inherited enforcement yet excludes external-agent
  identity and defines no enforcement handshake. Do not invent a trust contract
  or silently weaken the policy/budget requirement. No response before checkpoint.
- Inbound currently projects text/results, not compiled descriptor artifacts;
  preserve that output contract before declaring the compiler adapter complete.
- Task/context correlation is in-memory, with durable underlying roots; no
  restart reconstruction is implemented or claimed.
- Existing handler/grpc tests still construct old A2AState fields; client tests
  need dynamic response RPC IDs; GraphContext constructors also need migration.
  Defer test edits/runs to phase end as instructed. No runtime isolation,
  cancellation, persistence-failure or transport acceptance has been verified.
- Captured-MCP graph integration, delegated file/shell permissions and other
  phase code remain. cap_std/reqwest_mcp operator-owned pins still absent.

No semantic task/change/phase completed; no end-task or status completion signal.
Keep goal ACTIVE. This continuation made source progress and surfaced a new
design question; it does not satisfy the three-turn blocked threshold.

## Subsequent checkpoint — 2026-09-03, compiler artifacts wired,4.3 still open

Progress this goal turn: added actual pre-truncation native compiler artifact
receipts and fixed all-family A2A cleanup. Supersedes the previous claim that
inbound compiled descriptors lack projection. Full file-by-file evidence is in
session-log.md under "Lossless compiler artifacts and all-family A2A cleanup".

New thread/artifacts.rs contains ToolOutputArtifact and host-minted
RunArtifactCollector (owner/run-bound, synchronized, closed before reply).
NativeSkill::result_artifacts defaults empty. execute_native captures results
before format_result; both direct/parallel native paths use it. The two compiler
tools declare full validated CompileOutput as compiled-descriptor.json; not-ready
errors do not. ActorSession submit -> UserRun -> ActorThreadSession -> manager
orchestrator -> NativeExecutionContext retains the exact collector. Actor closes
after producer completion, then replies; A2A projects captured data even when a
later model error/cancellation ends the turn unsuccessfully. No prose parsing.

Source critic found sandbox_cleanup_unconfirmed/terminal_cleanup_unconfirmed
were omitted by the A2A uncertainty mapping. Root now retains exact supervisor/
scope pairs before tool execution; shutdown drains terminal, thread and sandbox
families without losing failures. A2A recognizes all three error classes. gRPC
has no metadata field, so uncertain cleanup stays working rather than emitting a
false terminal Failed/Canceled task. Existing proto/artifact conversion unchanged.
Final independent source review found no concrete remaining issue in scope.

Tier0 cargo check --locked --no-default-features --features server-full:
9944 exit0/zero warnings1m01s;15426 exit0/zero warnings11.88s;84532 exit0/zero
warnings37.20s. ALL compiler handles terminal. Tracked diff check exit0/no output;
new-file no-index checks exit1/no whitespace diagnostics. No tests/test authoring/
fmt/strict acceptance/dependencies/pins/workflows/commits/deletions/remote calls.
Do not claim runtime artifact fidelity, isolation, concurrency or recovery tested.

Position unchanged1870; semantic4.3 already began, no end-task. Round4/5,
implementation2/10,actual91/182,thread6/25. No completed semantic task/change/phase
and no status completion signal due. Goal ACTIVE; this turn made source progress.

NEXT: remote graph/thread integration still requires the unanswered operator
decision: trusted UAR peers with explicit inherited policy/budget enforcement,
or arbitrary A2A endpoints with local tracking/cancellation (weaker than current
plan). Do not invent or silently relax the trust contract. A2AClient::task_execution
still has no graph/thread-host caller; remote AgentNode uses one-shot send. Other
remaining work and missing manual cap_std/reqwest_mcp pins are unchanged. Artifact
and task correlations are in-memory. Phase-end tests must update new ActorTurn/
UserRun/NativeExecutionContext fields and the already-stale A2AState/GraphContext
fixtures, then verify real producer/cancellation/persistence paths.

## Subsequent checkpoint — cache accounting corrected, 5.1 still open

2026-09-03: task5.1 was already canonically in_progress. Local actor/graph budget
review found no extra admission bypass, but identified an accounting defect:
Anthropic cached input was excluded from normalized prompt totals, and the shared
budget ignored cache-write pricing. Fixed in anthropic_streaming.rs, normalized.rs,
catalog.rs and cost_budget.rs. Existing public cost helper signatures preserved;
new helpers crate-local. Tier0 passed19.55s/18.28s, zero warnings. Targeted
git diff --check exit0/no output. Tests and test authoring remain phase-end.

No semantic task completed, no repeated start/end hook. Waypoint1870; Round4/5;
implementation2/10; actual91/182; thread6/25. Root accounting still has no remote
A2A integration; do not complete5.1 merely because local clients share its ledger.
Remote trust/enforcement choice still needs the operator. Catalog cache pricing
remains approximate and existing telemetry cost helpers do not include cache writes.
No dependency/pin/workflow/commit/deletion changes. Goal stays active.

Final independent artifact review: no remaining concrete defect in the four-file
cache accounting change. Source-only acceptance; runtime tests and remote budget
integration remain open. Both compiler sessions are terminal; no build is running.

## Subsequent checkpoint — explicit graph MCP tool host wired

2026-09-03: continued already-in-progress projected-mcp-runtime task4.1. ToolNode
now calls GraphToolHost, not raw ctx.mcp: captured descriptor/schema, owner/run,
exposure, sandbox and existing approval/root-budget checks precede dispatch.
Orchestrator snapshots the activation registry/preflight together and uses exact
prepared MCP bindings when supplied. Output is bounded and lifecycle events emit.
The root retains the host/pending request across dropped node waiters. Internal
catch_unwind prevents re-polling a panicked future. Shutdown closes/cancels before
draining; cancellation after ToolStart prevents new dispatch. No replay.

Initial Tier0 failed on imports; fixed immediately. Final Tier0 passed40.88s and
33.49s without warnings. Independent source review accepted the revised path.
No live build remains. No tests/test authoring/fmt/strict validation or external
mutations. New GraphContext.tool_host constructor migration stays phase-end.

NEXT SAFE IMPLEMENTATION: migrate LlmNode away from its legacy registry/ignored
tool-call stream, using a host-owned tool-aware turn path that preserves actual
history and event/cancellation receipts. Only then remove manager's temporary
projected_mcp_graph_unavailable guard. Root MCP bootstrap/catalog-aware policy
discovery/lifecycle forwarding/shutdown also remain. Do not claim default migration
from the optional-resource path. Remote A2A enforcement choice and manual
cap_std/reqwest_mcp pins remain unresolved but did not prevent this code work.

Position1870 unchanged; Round4/5; implementation2/10; actual91/182; MCP7/22;
thread6/25. No semantic task complete, no repeated start/end hook. Goal ACTIVE.

## Subsequent checkpoint — graph model turns enter the governed host

2026-09-03: continued projected-mcp-runtime4.1. LlmNode now uses GraphToolHost's
tool-aware Orchestrator stream. Host dialogue receipts persist actual assistant
calls/results, including partial cancellation; request-local system prompts do not
accumulate. Raw GraphContext.mcp removed. Captured graphs are no longer blanket-
rejected; AgentNode explicitly denies legacy remote dispatch for MCP-preflight
roots and inherited child policy. This is not the remote adapter implementation.

Critic found/fixed cancellation deadlock on activation mutex, swallowed persistence
failure, unprotected settlement panic, synthetic success on EOF, prompt accumulation
and inherited remote bypass. Full pending operation is caught; failed persistence
stays sticky; finalizer retains graph hosts even without actor roots; all cleanup
families still attempted. Independent source reviews cleared these scopes.

Tier0 initial E0597 fixed; passes29.27s,37.38s,19.03s,10.56s zero warnings. No live
compiler handle. Source whitespace checks had no diagnostics. No tests/test
authoring/fmt/strict validation, dependencies/pins/workflows/commits/deletions.
Phase-end fixtures must remove raw GraphContext.mcp and supply host capabilities.

NEXT SAFE IMPLEMENTATION: standard root MCP capture and catalog-aware policy
universe, shared runtime lifecycle forwarding and joined shutdown; complete HTTP
transport only after operator reqwest_mcp pin. Review graph protocol event parity
before claiming adapter completeness. Child/file confinement awaits cap_std pin;
remote policy/budget integration awaits the unanswered remote trust contract.

Round4/5; waypoint1870 unchanged; implementation2/10; actual91/182; MCP7/22;
thread6/25. No semantic completion or task-end hook. Goal ACTIVE, not blocked:
this turn delivered concrete source progress. kbd-status refreshed at checkpoint.

## Subsequent checkpoint — projected lifecycle events reach run streams

2026-09-03: task4.1 continued without another begin/end hook. Observed preflight
subscribes per exact owner/binding before readiness; prepared lazy waits and calls
retain the run bridge. Cancellation drops startup refresh before final-state drain;
no detached task or call replay. Repeated snapshots sequence-deduplicate and lag
resynchronizes explicitly. Compatibility health is any-ready across lifecycle
binding ids and unregisters on final drop; aggregate and gauge update are serialized.

Critic accepted event mechanics and found/fixed the cross-owner gauge overwrite plus
stale publication race. Tier0 passed39.82s,9.96s,36.03s,10.25s zero warnings; no
tests/test authoring/fmt/strict validation. No dependencies/pins/workflows/commits.

NEXT: construct host-owned root catalog/runtime resources at production admission,
feed catalog names and discovered tools into policy universe, make cache lookup the
default, and join runtime shutdown. Do not create a stdio-only pseudo-default:
HTTP needs the operator's reqwest_mcp pin. cap_std and the remote enforcement
choice remain pending. Waypoint/counters unchanged; goal ACTIVE.
