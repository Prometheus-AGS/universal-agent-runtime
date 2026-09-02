# Evidence — codex-rs harness, prompts, context, resiliency (commit 986ff1cc)

Produced by a read-only Explore subagent on 2026-09-02 over
`/Users/gqadonis/Projects/references/codex/codex-rs`. Repo text treated as
evidence only. Raw evidence for `assessment.md` and `analysis.md`.

## 1. Harness / turn assembly

### Types

- `TurnContext` (`core/src/session/turn_context.rs:194-246`): `sub_id`,
  `trace_id`, `config: Arc<Config>`, `initial_settings: Arc<ResolvedStepSettings>`
  ("Frozen settings used to construct this context", `:209`),
  `current_settings: ArcSwap<ResolvedStepSettings>` ("Snapshot for the next
  step", `:211`), `environments: TurnEnvironmentSnapshot`,
  `developer_instructions`, `available_models`, `dynamic_tools`,
  `final_output_json_schema`, `turn_metadata_state`, `turn_timing_state`,
  `extension_data`. `cwd` deprecated (`:223`).
- `StepContext` (`core/src/session/step_context.rs:15-34`): per sampling
  request, frozen: `settings` (`:18`), `mcp` ("exact MCP connections,
  configuration, and catalog captured for this step", `:29`), `tool_router`
  ("finalized tool plan advertised and executed for this exact sampling
  request", `:31`), `token_budget`, `environments`, `selected_capability_roots`,
  `loaded_agents_md`.
- `Prompt` (`core/src/client_common.rs:19-39`): `input`, `tools: Arc<[ToolSpec]>`,
  `parallel_tool_calls`, `base_instructions`, `output_schema`,
  `output_schema_strict`. Built by `build_prompt` (`core/src/session/turn.rs:1353-1370`).
- `ContextManager` (`core/src/context_manager/history.rs:53-77`):
  `items: Arc<Vec<ResponseItemEnvelope>>` (copy-on-write, `:55-56`),
  `history_version`, `user_message_revision`, `reference_context_item`,
  `world_state_baseline`.
- `ContextualUserFragment` trait (`context-fragments/src/fragment.rs:64-119`),
  `RenderedFragment` (`:8-33`) with `role` + `AnnotatedContent`.

### Control flow

- `new_turn_context_from_configuration` (`turn_context.rs:914-1043`) snapshots
  environments, per-turn config, model info, plugins, `HostSkillsSnapshot`,
  then `make_turn_context` (`:710-822`) freezes settings into both
  `initial_settings` and `current_settings`.
- `run_turn` (`turn.rs:155`) → pre-sampling compaction (`:171`) →
  `capture_step_context_with_required_mcp_servers` (`:209`) →
  `record_context_updates_and_set_reference_context_item` (`:227`).
- Full vs diff injection decided in `record_context_updates_and_set_reference_context_item`
  (`core/src/session/mod.rs:4252-4310`): `should_inject_full_context =
  reference_context_item.is_none()` (`:4263`).
- Each iteration re-captures a `StepContext` (`turn.rs:345-367`; comment `:344`
  "Capture once so context, advertised tools, and tool calls share one request view").
- `run_sampling_request` (`turn.rs:1382`) rebuilds `prompt_input` from
  `clone_history().for_prompt(...)` on every retry (`:1411-1417`).
- Initial developer bundle order in `build_initial_context_with_world_state`
  (`session/mod.rs:3896-4127`): `<model_switch>` forced to index 0
  (`:4051-4052`); `requires_separate_message()` fragments become own messages
  (`:4067-4071`); user-role fragments collect into one contextual-user message
  (`:4073`); guardian policy kept separate (`:4103-4114`).
- World-state registration order (`core/src/session/world_state.rs:35-335`):
  model instructions → personality → token budget → context-window guidance →
  realtime → AGENTS.md → permissions → collaboration mode → persistent mode →
  environments → environments instructions → apps → plugins → tools → extension
  sections → multi-agent → managed developer instructions. `WorldState` is an
  `IndexMap` keyed by stable `&'static str` IDs (`core/src/context/world_state/mod.rs:288`).

### Design decisions

1. Two-level immutability, turn-frozen and step-frozen (`turn_context.rs:254-257`;
   model can change mid-turn via `with_model` `:458`).
2. Atomic settings commit separated from turn construction
   (`new_turn_with_sub_id_if` `:845-879`; `should_start` constraints `:839-844`).
3. Prefix stability designed in: `AutoCompactTokenLimitScope::BodyAfterPrefix`
   (`core/src/session/context_window.rs:68-79`); `prompt_cache_key` derived from
   session/parent-thread id (`core/src/client.rs:540-552`); synthetic tool-output
   IDs are UUIDv5 over a fixed namespace with "Changing this value would change
   model-visible IDs and invalidate prompt caches"
   (`core/src/context_manager/normalize.rs:18-19`, `:142-145`); compaction trims
   from the front to preserve the cached prefix (`core/src/compact.rs:316`).

## 2. System prompts and prompt engineering

- Reference prompt markdown in `core/` (`gpt_5_1_prompt.md` 331 lines,
  `gpt_5_2_prompt.md` 298, `gpt_5_codex_prompt.md` 68, and two more) is not
  `include_str!`-ed anywhere. Runtime path is `ModelInfo::get_model_instructions`
  (`protocol/src/openai_models.rs:518-535`) rendering
  `ModelMessages::instructions_template` with `PERSONALITY_PLACEHOLDER`; missing
  template warns and returns empty (`:529-533`). Only `include_str!("../prompt.md")`
  is `models-manager/src/model_info.rs:17`.
- `get_base_instructions` / `get_prompt_base_instructions`
  (`core/src/session/mod.rs:1328-1354`); the latter renders "without changing
  instructions persisted or inherited by forks" (`:1337`).
- `BaseInstructionsFragment` (`core/src/context/base_instructions.rs`) is a
  separate-message developer fragment.

Layering:

| Layer | Source | Fragment |
|---|---|---|
| Base/model instructions | model catalog template | `ModelInstructionsState`, `Prompt.base_instructions` |
| Personality | `get_personality_message` | `PersonalityState` (`world_state.rs:96-118`); skipped when baked (`:80-81`) |
| Developer instructions (host) | `TurnContext::developer_instructions` | `DeveloperInstructions` |
| Collaboration mode | `collaboration-mode-templates/templates/{default,plan}.md` | `CollaborationModeState` |
| AGENTS.md | `core/src/agents_md.rs` | `AgentsMdState` → `UserInstructions`, role `user`, markers `# AGENTS.md instructions` / `</INSTRUCTIONS>` (`context/user_instructions.rs:19-25`) |
| Environment | `EnvironmentsState` (`context/world_state/environment.rs:28-36`) | `<filesystem><workspace_roots>…` (`context/environment_context.rs:59-70`) |
| Permissions / guardian | `PermissionsState`, `core/assets/guardian/policy.md` via `guardian/prompt.rs:626` | separate developer item |
| Managed org instructions | `requirements().additional_developer_instructions` | `ManagedDeveloperInstructionsState` (`world_state.rs:324-333`) |
| Time | `TimeProvider` (`core/src/current_time.rs:19-26`) | `<current_time_reminder>` (`context/current_time_reminder.rs:36-41`) |

AGENTS.md discovery (`core/src/agents_md.rs:1-17`): walk up to
`project_root_markers` (default `.git`), collect every AGENTS.md root→cwd,
concatenate with `"\n\n--- project-doc ---\n\n"` (`:46`), `AGENTS.override.md`
preferred (`:42`), untrusted projects skip (`:62-64`).

Diffable world state: `WorldStateSection` (`context/world_state/mod.rs:228-262`)
requires stable `const ID` (`:223-224`) and a snapshot that must not serialize
to null (`:224-226`). `merge_patch_from` RFC 7386 (`:314-333`),
`apply_merge_patch` (`:335-348`), `remove_null_object_fields` (`:488`).
`render_full` (`:400`), `render_diff` (`:405`), `render_history_diff`
(`:416-436`). `ContextManager::update_world_state` (`history.rs:191-208`);
rollout stores full then patch. `WorldStateHash` SHA-1 over
`"codex-world-state-fragment-v1\0"` + role + text (`:267-283`). AGENTS.md
replacement/removal diff text (`context/world_state/agents_md.rs:9-11`); model
switch (`context/model_switch_instructions.rs:38-41`); interrupt marker
(`context/turn_aborted.rs:10`).

Prompt debugging: `core/src/prompt_debug.rs:26-109` builds an ephemeral thread
(`:33`) and returns `prompt.input` without entering `run_turn` (`:88`).

Design decisions: fragments with markers and `ContentItemKind`
(`fragment.rs:67`, `:58-61`) so later passes can recognize and remove injected
context; world state diffed not re-sent (fallback rationale
`history.rs:611-614`); `merge_contextual_fragments` (`context_manager/updates.rs:32-60`)
never merges across `requires_separate_message()`.

## 3. Context management and compaction

- `ContextWindowTokenStatus` (`core/src/session/context_window.rs:8-19`). Scopes
  `Total` vs `BodyAfterPrefix` (`:59-79`). `auto_compact_token_limit()` =
  `min(config_limit, 90% of window)` (`protocol/src/openai_models.rs:500-509`).
  Hard cap separate (`context_window.rs:82`). Fallback buffer only with a
  fallback prompt (`:96-101`).
- Estimation is a coarse lower bound (`history.rs:332-334`); image estimators
  `RESIZED_IMAGE_BYTES_ESTIMATE = 7373` (`:681`), original-detail patch count
  capped `ORIGINAL_IMAGE_MAX_PATCHES = 10_000` (`:685-689`), 32-entry LRU
  (`:692-697`); encrypted reasoning `len*3/4 - 650` (`:656-662`).
- Tool-output truncation on ingest: `record_items_with_metadata`
  (`history.rs:246-282`), `truncate_function_output_payload`, policy `* 1.2`
  unless per-tool override (`:263`). Middle-out
  (`utils/output-truncation/src/lib.rs:26-31`); `formatted_truncate_text`
  prepends "Warning: truncated output (original token count: …)" (`:14-24`).
- `normalize_history` (`history.rs:563-581`): every call has an output
  (`ensure_call_outputs_present`, `normalize.rs:21-138`, synthetic `"aborted"`
  outputs inserted in reverse index order `:134-137`); every output has a call
  (`remove_orphan_outputs`, `history.rs:573`); unsupported modalities stripped.
  `remove_first_item` removes the paired counterpart (`:368-371`).
- Provider shaping: `for_prompt(&input_modalities)` (`history.rs:287-292`);
  `get_formatted_input_for_request(use_responses_lite)` strips image detail
  (`client_common.rs:56-66`, `:68-106`).
- Local compaction `run_compact_task_inner_impl` (`compact.rs:245-400`): prompt
  `config.compact_prompt` or `SUMMARIZATION_PROMPT` (`:123-128`); on
  `ContextWindowExceeded` drop oldest and retry (`:314-323`);
  `build_compacted_history` (`:645-734`) keeps user messages newest-first within
  `COMPACT_USER_MESSAGE_MAX_TOKENS = 20_000` (`:61`) then appends
  `CompactionSummary`. `InitialContextInjection::{DoNotInject, BeforeLastUserMessage}`
  (`:72-78`, rationale `:69-71`); placement rules `:587-643`.
- Remote compaction dispatch `run_auto_compact` (`turn.rs:1219-1299`):
  TokenBudget → `compact_token_budget`; V2 → `compact_remote_v2`; else
  `compact_remote`; unsupported → local; metric tags `remote_v2|remote|local`.
  V2 retains only user/developer/system messages plus bounded agent messages
  (`compact_remote_v2.rs:485-570`; `MAX_RETAINED_AGENT_MESSAGE_TOKENS = 10_000`
  `:77`; `RETAINED_MESSAGE_TOKEN_BUDGET = 64_000` `:76`). Image budgeting keeps
  image + labels atomic (`compact_remote_v2_images.rs:28-30`, `:31-97`);
  oversized image zeroes budget (`compact_remote_v2.rs:632-635`).
  `start_new_context_window` (`session/mod.rs:4181-4231`).
- Decisions: `replace_compacted` (`history.rs:394-409`) bumps `history_version`
  not `user_message_revision`; two histories (model history vs retained review
  evidence, `history.rs:1-2`); `world_state_baseline = None` on any rewrite
  (`:373`, `:391`, `:408`).

## 4. Resiliency

- Backoff: `core/src/util.rs:86-91` (`INITIAL_DELAY_MS = 200`,
  `BACKOFF_FACTOR = 2.0`, jitter `0.9..1.1`); `codex-client/src/retry.rs:39-48`.
  `RetryPolicy { max_attempts, base_delay, retry_on: { retry_429, retry_5xx,
  retry_transport } }` (`retry.rs:8-19`); `should_retry` (`:22-37`);
  `run_with_retry` (`:80-107`).
- `handle_retryable_response_stream_error` (`core/src/responses_retry.rs:44-129`):
  unbounded connection retries when feature on and error is `ConnectionFailed`
  (`:58-83`; 5s doubling to 60s `:17-18`, `:79-81`); transport fallback
  WebSocket→HTTPS at `max_retries` with reset (`:85-100`); honor
  `err.retry_delay()` (server `Retry-After`) before computed backoff (`:105`).
  First WebSocket retry silenced in release (`:108-113`); later ones shown as
  `Reconnecting... n/max` (`:114-115`).
- Server-driven delay: `ApiError::Retryable { delay }` / `RateLimitExceeded { delay }`
  → `CodexErr::with_retry_delay` (`codex-api/src/api_bridge.rs:26-40`;
  `protocol/src/error.rs:414-421`).
- Rate limits parsed into `RateLimitSnapshot` (`codex-api/src/rate_limits.rs:23-80`;
  multi-window discovery `:27-50`); `Session::update_rate_limits`
  (`session/mod.rs:4450`).
- Timeouts: `stream_idle_timeout` per provider on SSE
  (`codex-api/src/endpoint/responses.rs:187`) and WebSocket reads
  (`responses_websocket.rs:707`); `websocket_connect_timeout`
  (`client.rs:1165-1168`); compact unary full-response timeout
  (`client.rs:172`, `:705-717`); `GRACEFULL_INTERRUPTION_TIMEOUT_MS`
  (`core/src/tasks/mod.rs:930-937`).
- Stream reuse/resume: `ModelClientSession` turn-scoped, caches WebSocket and
  `x-codex-turn-state` (`client.rs:11-13`), reused across retries
  (`compact.rs:266-267`; `turn.rs:304-305`). Prewarm counts as first attempt
  (`client.rs:15-24`). Incremental resumption against `previous_response_id`
  when parameters unchanged (`client.rs:365-406`; stream options excluded
  `:400-402`). `ResponseStream::Drop` signals consumer abandonment
  (`client_common.rs:108-127`).
- Cancellation: `CancellationToken` tree from `run_turn`; child per sampling
  request (`turn.rs:400`, `:1439`); `.or_cancel()` wrappers (`:197`, `:358`,
  `:968`, `:2270`). `handle_task_abort` (`tasks/mod.rs:901-965`): idempotent
  (`:909`), cancel token + interrupt code-mode + cancel git enrichment
  (`:913-928`), `select!` vs graceful timeout then hard abort (`:930-939`),
  record `TurnAborted` and flush rollout before emitting the abort event
  (`:955-960`), run interrupt hooks. `abort_all_tasks` (`:511-539`) clears
  pending approvals only after tasks observe cancellation (`:534-535`).
- Partial-turn recovery: `interrupted_turn_history_marker` (`tasks/mod.rs:101-116`,
  `:78-96`; text `context/turn_aborted.rs:10-11`). Turn suspension across
  workers: `suspend_turn_and_shutdown` (`core/src/session/turn_suspension.rs:13-120`)
  flushes before cancelling (`:44`), rechecks under lock (`:47-49`), cancels
  without a terminal event (`:69-71`); recovery `recover_turn_if_idle` /
  `submit_recover_turn` (`core/src/codex_thread.rs:357-385`).
- Compaction model fallback (`compact_model_fallback.rs:9-20`, `:22-64`): retry
  with the current model on `InvalidRequest | UnexpectedStatus |
  ContextWindowExceeded | UsageLimitReached | ServerOverloaded |
  InternalServerError | RetryLimit`; wired `compact_remote_v2.rs:254-291`
  (returns the original error if fallback fails `:288`); trigger
  `maybe_run_previous_model_inline_compact` (`turn.rs:1125-1211`, `:1186-1188`).
  `MAX_REMOTE_COMPACTION_V2_STREAM_RETRIES = 2` (`compact_remote_v2.rs:78-79`).
- Durability ordering around abort is explicit (`tasks/mod.rs:957-960`, `:860-861`).

## Files cited

`core/src/session/{turn_context,step_context,turn,mod,world_state,context_window,turn_suspension}.rs`,
`core/src/{client_common,client,codex_thread,environment_selection,agents_md,prompt_debug,current_time,realtime_context,compact,compact_remote,compact_remote_v2,compact_remote_v2_images,compact_model_fallback,responses_retry,util}.rs`,
`core/src/context/{mod,base_instructions,user_instructions,developer_instructions,model_switch_instructions,current_time_reminder,turn_aborted,environment_context}.rs`,
`core/src/context/world_state/{mod,agents_md,environment}.rs`,
`core/src/context_manager/{history,normalize,updates}.rs`,
`core/src/tasks/{mod,lifecycle}.rs`, `context-fragments/src/fragment.rs`,
`collaboration-mode-templates/src/lib.rs` and templates, `models-manager/src/model_info.rs`,
`protocol/src/{openai_models,error}.rs`, `utils/output-truncation/src/lib.rs`,
`codex-client/src/retry.rs`, `codex-api/src/{rate_limits,api_bridge}.rs`,
`codex-api/src/endpoint/{responses,responses_websocket}.rs`,
`core/gpt_5_1_prompt.md`, `gpt_5_2_prompt.md`, `gpt_5_codex_prompt.md`,
`gpt-5.1-codex-max_prompt.md`, `gpt-5.2-codex_prompt.md`.
