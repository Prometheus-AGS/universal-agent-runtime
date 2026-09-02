# Evidence — codex-rs subagents, threads, app-server, governance, observability, testing (commit 986ff1cc)

Produced by a read-only Explore subagent on 2026-09-02 over
`/Users/gqadonis/Projects/references/codex/codex-rs`. Repo text treated as
evidence only. The explorer read `control_tests.rs`, `guardian/review.rs`,
`guardian/review_session.rs`, and `session/mod.rs` only in targeted excerpts
and did not open `multi_agents/wait.rs`; claims there rest on the cited lines.

## 1. Sub-agent architecture and threading

### Types

- `AgentControl` (`core/src/agent/control.rs:118-134`): `session_id` (= root
  thread id `:120`), `manager: Weak<ThreadManagerState>` (cycle break
  `:121-123`), `thread_id_generator` (`:125-126`), `state: Arc<AgentRegistry>`,
  `v2_residency`, `agent_execution_limiter`, `rollout_budget`,
  `root_service_tier`.
- `SpawnAgentOptions` (`control.rs:87-96`): `fork_parent_spawn_call_id`,
  `fork_mode`, `parent_thread_id`, `parent_turn_id`, `root_turn_id`,
  `environments`, `multi_agent_v2_usage_hints`, `cyber_access_program`.
- `SpawnAgentForkMode::{FullHistory, LastNTurns(usize)}` (`control.rs:81-84`).
- `AgentMetadata` (`core/src/agent/registry.rs:53-58`); `SubAgentSource::ThreadSpawn
  { parent_thread_id, depth, agent_path, agent_nickname, agent_role }`
  (`protocol/src/protocol.rs:2821-2830`, alias `agent_type` `:2828`).
- `AgentPath` newtype `/root/child/...` (`protocol/src/agent_path.rs:15-22`,
  `ROOT = "/root"` `:18`); model-facing identifier in V2.
- `InterAgentCommunication { id, author: AgentPath, recipient, other_recipients,
  content, encrypted_content, internal_chat_message_metadata_passthrough,
  trigger_turn }` (`protocol/src/protocol.rs:803-819`).
- `ThreadConfigSnapshot` (`core/src/codex_thread.rs:82-106`).

### Spawn flow (V2)

1. Model calls `spawn_agent{task_name, message, agent_type?, model?,
   reasoning_effort?, fork_turns?}` (`core/src/tools/handlers/multi_agents_v2/spawn.rs:93`);
   `fork_turns` parses to None/FullHistory/LastNTurns (`:291-324`);
   `fork_context` rejected (`:292-296`).
2. Child config from the live turn: `build_agent_shared_config`
   (`multi_agents_common.rs:195-218`) then `apply_spawn_agent_runtime_overrides`
   (`:235-262`) copying approval policy, reviewer, cwd, permission profile;
   doc `:170-176`.
3. Role overrides (`apply_spawn_agent_role` `:352-390` → `agent/role.rs:51-67`),
   service tier intersect (`:322-350`), then runtime overrides applied again
   (`spawn.rs:145`) so a role cannot re-widen.
4. `spawn_agent_internal` (`control/spawn.rs:586`): execution slot, residency
   slot, registry slot (`:604-624`), inheritance (`:625-632`), source + path
   (`control.rs:691-719`), fork or fresh (`spawn.rs:657-697`).
5. Initial prompt delivered as `InterAgentCommunication` with `trigger_turn: true`
   (`spawn.rs:171-178`, `:760-769`).
6. Spawn edge persisted to `agent-graph-store` (`control.rs:817-846`) unless
   ephemeral; `notify_thread_created` (`spawn.rs:740`).
7. V1 completion watcher (`spawn.rs:771-783` → `control.rs:573-663`); V2
   completion arrives as `InterAgentCommunication` `trigger_turn: false`
   (`control.rs:634-651`).

### Inheritance

| Thing | Inherited | Where |
|---|---|---|
| Model/provider/reasoning | yes, overridable | `multi_agents_common.rs:198-204`, `264-320` |
| Approval policy, cwd, permission profile | forced, twice | `:235-262`; `spawn.rs:145` |
| Exec policy manager | only if `child_uses_parent_exec_policy` | `control.rs:750-770` |
| Environments | snapshot | `control.rs:727-748` |
| MCP manager, plugins, skills, extensions | shared Arc (delegate path) | `codex_delegate.rs:100-105` |
| Service tier | root's, intersected | `:322-350`, `control/service_tier.rs:8-17` |
| Rollout budget | shared across tree | `control.rs:131`, `rollout_budget.rs:19-21` |
| Token usage history | no | `spawn.rs:99-100` |
| Guardian evidence in forked checkpoints | scrubbed | `spawn.rs:999-1001` |

### Fork filtering

`keep_forked_rollout_item` (`spawn.rs:63-103`): assistant messages only when
`phase == FinalAnswer` (`:68`); reasoning, function calls, tool outputs,
realtime, inter-agent comms, risk scores dropped; `TurnContext`/`WorldState`
only for `FullHistory` (`:95-97`, cache-prefix rationale). `retain_forked_item`
(`:919-975`) swaps parent developer instructions for the child's; also over
compaction `replacement_history` (`:1002-1011`, `:916-918`).

### Model-facing tools

V1 namespace `multi_agent_v1`: `spawn_agent`, `send_input`, `wait_agent`,
`resume_agent`, `close_agent` (`multi_agents_spec.rs:65-98, 143-179, 242-278,
313-333`). V2 namespace `collaboration`: `spawn_agent`, `send_message`,
`followup_task`, `wait_agent`, `interrupt_agent`, `list_agents`
(`:100-141, 181-240, 280-311, 335-353`). `spawn_agent` V2 requires
`task_name` + `message` (`:134`), returns only `task_name` under
`hide_spawn_agent_metadata` (default true, `config/mod.rs:1296`). Messages
`.with_encrypted()` (`:194`, `:627`). `send_message` does not trigger a turn
(`:200`); `followup_task` does (`:233`) and is refused against root
(`message_tool.rs:75-84`). V2 `wait_agent` returns only which agents have
updates (`:283`). `interrupt_agent` refuses self (`interrupt_agent.rs:61-63`).
`close_agent` note on concurrency (`:324`). Spawn description hard-gates
spawning on explicit user/AGENTS.md/skill authorization (`:688`, `:696-698`).

### Limits

1. Total spawned threads: `AgentRegistry::reserve_spawn_slot` CAS
   (`registry.rs:96-115`, `337-353`); V1 default 6 (`config/mod.rs:232`); V2
   `max_concurrent_threads_per_session - 1`, default 4 → 3 children
   (`:1562-1575`, `:233`); `SpawnReservation` drop-release (`registry.rs:393-402`).
2. Concurrent running turns: `AgentExecutionLimiter` (`control/execution.rs:13-89`),
   V2 subagents only (`:91-97`), checked per turn (`codex_thread.rs:461-474`).
3. Depth: V1 only, `agent_max_depth = 1` (`config/mod.rs:242`, `888`;
   `spec_plan.rs:650-654`; `multi_agents/spawn.rs:69-70`). V2 has no depth cap;
   gates on V2-capable model (`spec_plan.rs:655-658`).
4. V2 residency LRU eviction of idle children (`control/residency.rs:117-158`,
   `:139-153`, unloadable states `:233-239`).

### Cancellation

`close_agent` marks edge Closed then `shutdown_agent_tree`
(`control/legacy.rs:48-116`); `shutdown_live_agent` flushes rollout before
`Op::Shutdown` (`legacy.rs:8-44`); delegates use child `CancellationToken`
(`codex_delegate.rs:151-153`); `shutdown_delegate` Interrupt then Shutdown with
500ms drain (`:322-337`); `InternalAgentDied` triggers GC (`control.rs:373-388`).

### Persistence

`agent-graph-store` trait (`agent-graph-store/src/store.rs:17-59`), edges with
`ThreadSpawnEdgeStatus::{Open, Closed}` (`types.rs:5-12`), SQLite
`LocalAgentGraphStore` (`local.rs:11-28`); stable ordering requirement
(`store.rs:14-16`); descendants filter on every edge (`:51-54`). Resume: V1 BFS
reopen (`spawn.rs:1110-1159`); V2 `restore_v2_agent_metadata` without reopening
(`:157-225`).

### Identity and roles

`agent-identity` is Ed25519/JWT workload identity against a backend
(`agent-identity/src/lib.rs:40, 266, 315, 358, 425, 497`). `agent-roles` loads
TOML roles (`agent_role_config.rs:38-115`, validation `:78-87`, `:129-155`,
`:157-198`). Built-ins `default`, `explorer`, `worker`
(`core/src/agent/role.rs:343-401`; worker prompt `:377-378`).

Decisions: roles narrow but never widen (`role.rs:1-4`; feature overrides only
false `:91-106`; skills only disable `:107-118`; V2 resume captures/restores
runtime policy `spawn.rs:386-418`); `SpawnInitialInput` pairs communication
with context (`spawn.rs:29-39`); `AgentPath` addressing via
`resolve_agent_target` (`agent/agent_resolver.rs:12-38`).

## 2. Thread and session model

- Rollouts JSONL under `sessions/` (`rollout/src/recorder.rs:37-38, 74-76`);
  `is_persisted_rollout_item` (`rollout/src/policy.rs:9-24`, `:42-62`, `:17`,
  `:16`). `ThreadHistoryMode::{Legacy, Paginated}`; `load_latest_model_context`
  (`control/spawn.rs:128-153`). `thread-store` crate with SQLite backend
  (`codex_thread.rs:687-726`).
- Fork/resume: `spawn_forked_thread` (`spawn.rs:792-1086`; materialize+flush
  before snapshot `:855-858`); `resume_agent_from_rollout` (`:1089-1162`);
  `ensure_v2_agent_loaded` (`:298-584`).
- `CodexThread` (`codex_thread.rs:185-193`). Turn input modes:
  `start_or_steer_turn` (`:321-327`), `start_turn_if_idle` (`:333-351`,
  `unreachable!` `:348`), `steer_turn(request, expected_turn_id)` (`:442-459`),
  `recover_turn_if_idle` (`:357-398`), `suspend_turn_and_shutdown`
  (`:411-439`, doc `:400-410`, oneshot owned by session `:420-437`).
- app-server protocol: not true JSON-RPC 2.0 (`app-server-protocol/src/rpc.rs:1-2`);
  macros `client_request_definitions!` (`common.rs:495`),
  `server_request_definitions!` (`:1686`), `server_notification_definitions!`
  (`:1841`). Methods: `thread/start` (`:514`), `thread/resume` (`:521`),
  `thread/fork` (`:527`), archive/delete/unsubscribe/rollback/revert/
  compact/start/settings/update/inject_items/read/turns/list/items/list;
  `turn/start`, `turn/steer`, `turn/interrupt`, `turn/settings/update`;
  notifications thread/turn/item lifecycle, deltas, tokenUsage,
  autoApprovalReview, guardianWarning; server→client requests
  (`:1686-1757`): command/fileChange/permissions approval, `item/tool/requestUserInput`,
  `mcpServer/elicitation/request`, `item/tool/call`, `attestation/generate`,
  `currentTime/read`. `ThreadStartParams` (`v2/thread.rs:62-122`),
  `ThreadForkParams` (`:518-553`, `:530`). `#[experimental("…")]` fields
  (`thread.rs:69, 83, 96, 116`).
- Multi-client: `ThreadStateManager` (`app-server/src/thread_state.rs:333-339`),
  one listener per thread with `listener_generation` (`:106`, `:132`), many
  subscribers (`:396-404`), `wait_for_thread_subscriber` (`:378-394`),
  `ThreadListenerCommand` (`:335-338`), ordered server-request resolution (`:79`).
  `app-server-daemon` supervisor (`app-server-daemon/src/lib.rs:28-53`).

Decisions: three-mode turn submission enum with `unreachable!` arms
(`codex_thread.rs:321-474`); suspend-for-handoff distinct from interrupt
(`:400-402`, `:418-419`); one listener, many subscribers, generation counters
(`thread_state.rs:101-106`).

## 3. Governance intersections

- Guardian (`core/src/guardian/mod.rs:1-13`): compact transcript, dedicated
  review session, strict JSON, fail closed (`:12`), inherits parent proxy
  (`:10-11`). `GuardianAssessment` (`:161-167`). Routing
  (`guardian/review.rs:209-217`). Circuit breaker 3 consecutive / 10 of 50
  (`mod.rs:196-246`), CyberModel 1/1 (`:64-68`, `:210-219`). Reviewer sessions
  get empty extensions (`codex_delegate.rs:84-88`), `ThreadSource::GuardianReview`
  (`:112-116`), zero tools (`spec_plan.rs:601-605`).
- Delegates require approval policy `Never`, pinned via `Constrained::allow_only`
  (`codex_delegate.rs:63-68`).
- V2 reload revalidation (`spawn.rs:450-527`): environment match (`:460-469`),
  non-permission config equality (`:482-488`), remote executor fails closed on
  permission diff (`:491-496`), `intersect_effective_permission_profiles`
  (`:517-523`; `protocol/src/permission_profile_intersection.rs:37-`, fail-closed
  cases `:42-46`, `:57-63`, `:64-68`, doc `:31-36`). Changed parent exec policy
  is a hard error (`spawn.rs:445-449`).
- `root_user_authorization` (`control/user_authorization.rs:22-94`): reads root
  history, drops summaries and `<user_action>` (`:47-49`), keeps FinalAnswer
  (`:58`), 900 tokens each, last 8 (`:17-18`, `:78`);
  `GuardianAuthorizationVersion` (`codex_thread.rs:159-175`). Test
  `core/tests/suite/guardian_subagent_authorization.rs:35-38`. Guardian
  history stripped from forked checkpoints (`spawn.rs:999-1001`).
- `attestation.rs` trait (`:7`, `:24-26`); `attestation/generate` request
  (`common.rs:1731-1735`; `thread_state.rs:358-375`). `cyber_access_program.rs:5-12`
  gate; propagates parent→child only (`multi_agents_v2/spawn.rs:224`,
  `control/spawn.rs:752`).
- Ownership validation on cold reload three ways (`spawn.rs:365-376`) plus
  `Arc::ptr_eq` on the registry (`:279-295`).

## 4. Observability

- `otel_init.rs:16-95`: three exporters `None | Statsig | OtlpHttp | OtlpGrpc`
  (`:22-66`); metrics disabled when analytics off (`:70-77`); service name
  (`:79-80`); SQLite telemetry (`:103-110`).
- Events (`otel/src/events/session_telemetry.rs`): `codex.conversation_starts`
  (`:573`; MCP names in log, count in trace `:589-594`), `codex.api_request`
  (`:668`), `codex.turn_ttft` (`:269`), `codex.turn_cost` (`:326`),
  `codex.sse_event` (`:950-1015`), `codex.user_prompt` (`:1061/1067`),
  `codex.tool_decision` (`:1086/1095`), `codex.sandbox_outcome` (`:1117/1126`),
  websocket (`:724`, `:775`), `codex.auth_recovery` (`:810`);
  `codex.agent_communication` (`core/src/agent_communication.rs:53`);
  `codex.multi_agent.spawn` (`multi_agents_v2/spawn.rs:253-257`);
  `core.threads.live` gauge (`codex_thread.rs:79`, `:226`). GenAI semconv usage
  attributes (`:540-552`).
- Redaction: `log_user_prompts` (`:109`, `:1053-1056`); trace carries only
  lengths/counts (`:1058-1072`); inter-agent bodies never logged
  (`agent_communication.rs:59-62`, `:40-42`; `control.rs:315-316`);
  `decision.to_opaque_string()` (`:1090`).
- analytics crate batched client (`analytics/src/client.rs:80-84`);
  `track_collab_tool_call` (`:357-381`); `CollabAgentToolCallItem`
  (`protocol/src/items.rs:318-326`, tools `:297-307`);
  `emit_subagent_session_started` (`session/mod.rs:4617-4656`, `:4630-4635`).
- `event_mapping.rs`: `CONTEXTUAL_DEVELOPER_PREFIXES` (`:40-62`),
  `parse_user_message` hides contextual content (`:99-101`), media wrapper
  elision (`:108-122`), append-only list (`:57`).

## 5. Testing practices

- `core_test_support` at `core/tests/common` (Cargo.toml `:2`; workspace
  `Cargo.toml:296`). wiremock with real SSE bodies: `responses.rs`
  `start_mock_server` (`:1288`), `sse()` (`:717`), event constructors
  (`:732-900+`), `mount_sse_once` (`:1116`), `mount_sse_once_match` (`:1103`),
  `mount_sse_sequence` (`:1547`), `mount_response_sequence` (`:1588`),
  `mount_models_once` (`:1238`), `mount_compact_*` (`:1125-1237`);
  `ResponsesRequest` helpers (`:188-414`); `strip_metadata` (`:122-186`).
- `validate_request_body_invariants` (`responses.rs:1675-1740+`, invoked `:711`)
  panics on orphaned call/output pairs on every POST in every test.
- `TestCodex` builder (`test_codex.rs:325-337`, terminals `:470, 493, 502, 519`);
  `#[ctor]` determinism (`lib.rs:51-79`); `wait_for_event*`,
  `wait_for_mcp_server`; skip macros (`skip_if_sandbox!`, `skip_if_no_network!`, …).
- Tool-loop e2e pattern (`core/tests/suite/agent_execution.rs:53-104`,
  content-matched mounts, `mount_completed_worker` `:87`, `:96`;
  `v2_nested_spawn_checks_shared_active_execution_capacity` `:108`,
  multi_thread flavor `:107`). Suites: `multi_agent_mode.rs`,
  `multi_agent_resume.rs`, `subagent_notifications.rs`,
  `subagent_service_tier.rs`, `spawn_agent_description.rs`,
  `guardian_subagent_authorization.rs`, `codex_delegate.rs`, `fork_thread.rs`,
  `compact_resume_fork.rs`.
- insta snapshots in `core/tests/suite/snapshots/`, including `_request_diff`
  snapshots of successive request bodies. Unit tests via `#[path]` sibling
  files (`control.rs:918-920`, `registry.rs:404-406`, `role.rs:418-420`).
- Schema fixtures `app-server-protocol/schema/{typescript,json}`;
  `schema_fixtures.rs` (`:32`, `:8` experimental filter);
  `precomputed_exports_tests.rs:13-42`; `EXPERIMENTAL_CLIENT_METHODS` tables
  (`common.rs:410-427`). app-server integration `app-server/tests/suite/v2/`
  (~60 files), `app-server/tests/common/`, `app-server-test-client`.

Practices worth copying: protocol invariant checker inside the mock;
content-matched mock responses; committed generated schema diffed in CI with
experimental-field filtering.
