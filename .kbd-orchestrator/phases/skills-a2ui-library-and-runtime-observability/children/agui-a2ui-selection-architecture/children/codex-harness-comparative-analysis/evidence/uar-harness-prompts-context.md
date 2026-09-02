# Evidence — UAR harness, prompts, context, model-path resiliency (commit dce44e78)

Produced by a read-only Explore subagent on 2026-09-02. File:line citations are
against commit `dce44e78`. Raw evidence for `assessment.md`.

## 1. Harness / turn assembly

Entry points in `src/uar/runtime/manager.rs`: `start_run` (`:983`),
`start_run_with_history` (`:998`), `continue_with_interaction` (`:1022`),
`start_run_with_policy` (`:1067`) all delegate to
`start_run_with_policy_and_history` (`:1094-2604`), one function of about
1,510 lines with `#[allow(clippy::too_many_arguments)]` at `:1092`. File is
3,087 lines; `RunManager` declared `:154`; impl `:461-2757`.

Pipeline, straight-line inside that function (no trait, stage list, or registry):

| Step | Lines |
|---|---|
| Run id, broadcast channel cap 256, `EVENT_HISTORY_LIMIT = 512` (`:40`) | 1105-1116 |
| Session resolve/create | 1128-1134 |
| Seed prior turns only if `message_count() == 0`; `"system"` seeds dropped (`:1152`) | 1143-1156 |
| Effective policy (`resolve_effective_policy` `:866`) | 1158-1164 |
| Model backfill (`:2739`) | 1166 |
| User message | 1169 |
| Run record, registration | 1175-1198 |
| `effective_run_policy` artifact | 1200-1207 |
| Cancellation child token | 1219-1225 |
| `system_prompt = artifact.prompt.system.clone()` | 1229 |
| RAG: KB resolve, retrieve, `push_str(citation_stream.prompt_block())`; top-3, threshold 0.7 hardcoded (`:1268`) | 1232-1301 |
| Skills: SkillService or legacy classifier | 1306-1433 |
| Skill allowlist filter | 1435 |
| Skill overlays `push_str("\n\n[SKILL: …]")` | 1448-1451 |
| MCP registries per skill | 1454-1466 |
| System message at index 0, then `session.messages()` | 1471-1477 |
| Context strategy 1: `resolve_effective_strategy` + `trim_with_summarization` | 1489-1523 |
| Context strategy 2: `context_manager.apply(messages, context_limit)` | 1526-1539 |
| `SkillActivated` events | 1541-1550 |
| MCP merge + `SelectionMode` filtering | 1554-1580 |
| Native tools filter | 1581 |
| Per-agent provider resolve | 1584-1607 |
| Per-skill model override, first wins | 1610-1626 |
| Credential layer session→agent→user→system (`apply_credential_layer` `:249`) | 1628-1642 |
| Orchestrator build | 1652-1660 |
| Failover driver | 1665-1690 |
| Health monitor, resilience policy, cache strategy | 1691-1699 |
| Tool approval gate closure | 1702-1824 |
| `tokio::spawn` execution task | 1888 |

Memory is not in the list. Memory context is built in `src/server.rs:4892`
(`context_builder::build_context_with_hits`) and prepended to the user message
text (`server.rs:4941-4947`, `[MEMORY CONTEXT]…[/MEMORY CONTEXT]`).
`memory_hits` reaches `RunManager` only as a `MemoryRecall` SSE payload
(`manager.rs:1901-1914`). A direct `start_run` caller gets no memory.

Prompt type: `src/uar/domain/artifact.rs:118` `AgentPrompt { system: String,
instructions: Vec<String> }`. No assembly type, section list, or ordering type.

`src/uar/llm/`: `mod.rs` (6 lines), `router.rs` (73), `schema.rs` (34), off the
hot path.

`src/uar/prompt_cache.rs` (139 lines) is dead: constructed at `server.rs:1148`,
stored `lib.rs:135`, `get/set/delete/clear` never called outside its tests.
Real caching is `src/llm/anthropic_cache.rs` (`CacheStrategy`), applied
`manager.rs:1697`, consumed only by `anthropic_driver.rs:136-138`;
`external_driver.rs:101` hardcodes `cache_strategy: None`.

`src/llm/prompt_dialect.rs` (251 lines): `request_params` called
`orchestrator.rs:590-601`, gated on `thinking_budget.is_some()`. Detection is
substring sniffing on model id (`:41-59`). `prefers_xml_envelope()` (`:64`) and
`markdown_averse()` (`:71`) have no callers.

`src/llm/xml_tool_injector.rs`: `inject_tools_into_system_prompt` called from
`tool_normalizer.rs:55, 87, 116`; prepends the XML block (`:75-79`) and clears
`req.tools`. (The other explorer found `ToolNormalizerDriver` itself is never
constructed by `build_driver` `orchestrator.rs:47-76`; both facts hold.)

## 2. System prompts

Composition: one `String` in fixed source order: `artifact.prompt.system`, RAG
citation block (`:1291`), skill overlays (`:1448-1451`); at driver time the XML
tool block is prepended in front of all of it (`xml_tool_injector.rs:77`).

Ordering stability: section order fixed; content order within skill overlays is
nondeterministic. `SkillRegistry.skills` is `HashMap<String, Skill>`
(`skills/registry.rs:16`); `list()` iterates `.values()` (`:209-215`); Hybrid
merge builds from that (`service.rs:643-663`); legacy fallback collects into a
`HashMap` then `.into_values()` (`manager.rs:1414-1424`). Identical turns can
emit overlays in different orders, changing the prefix byte-for-byte and
defeating the Anthropic `cache_control` prefix match (`anthropic_cache.rs:55-73`).
Tools were sorted deliberately (`orchestrator.rs:511-516`); prompt sections were
not. RAG content varies per turn, invalidating any prefix past section 1.

Authority/provenance: none. RAG chunk text (`:1291`) and skill overlays
(`:1451`) are concatenated into the operator's system prompt with no marker
beyond `[SKILL: …]`. Grep for `authority|provenance|trust_level|PromptFragment|
SystemSection` in `src/uar/`, `src/llm/` returns only unrelated hits.

Host/project instructions and world state: absent. `AgentPrompt.instructions`
never read. Grep for `AGENTS.md|CLAUDE.md|project_instructions|host_instructions|
<env>|environment_info` in `src/` returns zero. `Utc::now()` in manager.rs only
at `:2365` (artifact metadata) and `:3019` (test).

## 3. Context management

Two independent subsystems applied back to back every run:

- Path A, `src/uar/context/strategy.rs` (680 lines): `ContextStrategy::{None,
  SlidingWindow{max_messages default 20 (:66)}, Summarize{threshold default 6
  (:69)…}, TruncateMiddle{keep_first 2, keep_last 4 (:74,77)}, Hierarchical
  {5/2000/500}, Auto}` (`:12`).
- Path B, `src/uar/runtime/context/manager.rs` (423 lines) with a different
  enum `crate::uar::domain::context::ContextStrategy::{SlidingWindow,
  KeepFirstLast, ProgressiveSummarization}`.
- Called at `manager.rs:1517` (A) then `manager.rs:1538` (B).

Six trimming paths: `trim_count` (`strategy.rs:110`), `apply_strategy` (`:161`),
`trim_with_summarization` (`:282`), `apply_sliding_window`
(`context/manager.rs:86`), `apply_keep_first_last` (`:143`),
`apply_progressive_summarization` (`:230`).

Tokenizers: Path A `estimate_tokens = (len+3)/4` (`strategy.rs:100-103`); Path B
`TokenService` tiktoken `cl100k_base` (`context/token_service.rs:9-13`), the only
one counting tool-call tokens (`:33-39`), applied regardless of provider.

Tool-call/result pairs: Path A slices by index (`strategy.rs:119`, `:135`;
`split_at` at `:298`, `:331`; fallbacks to `trim_count` at `:293, 308, 333,
361`). No inspection of `MessageRole::Tool` or `tool_call_id`.

System message can be trimmed away: the vector passed at `manager.rs:1517`
includes the system message at index 0 (`:1471`, `:1477`); `SlidingWindow`
keeps only the tail (`strategy.rs:119`). Path B re-preserves system messages
(`context/manager.rs:96-105`, `:242-252`) but after Path A already dropped it.

Repeated identical messages: `apply_keep_first_last` dedups tail candidates by
content equality against the head (`context/manager.rs:193-198`); comment at
`:181-186` calls it imperfect.

Output truncation: only `src/uar/tools/fetch_guard.rs` (`DEFAULT_MAX_LENGTH =
5000` `:52`, `paginate` `:202-209`, `truncated` flag `:86`). `file_tools.rs:75`
rejects by size. `terminal_exec.rs:75-81` returns full stdout/stderr unbounded.
MCP results (`orchestrator.rs:1010-1021`) and native results not truncated.

Checkpoints: `src/uar/runtime/checkpoint.rs` (58 lines). `Checkpoint { id,
run_id, thread_id, node_id, iteration, state, messages, created_at }` (`:10`).
`restore_state()` (`:50-57`) clones `messages` and `unwrap_or_default()` on
state; only used by the graph branch (`manager.rs:1932-2042`).

## 4. Claim verification

- (a) manual sequencing at `manager.rs:1229`: TRUE.
- (b) malformed JSON executes as `{}`: TRUE at two sites, `orchestrator.rs:982-983`
  and `:1032-1033`. No log, no feedback to the model.
- (c) two strategies sequential, repeated messages and tool pairs damaged: TRUE
  (`manager.rs:1517`, `:1538`; `strategy.rs:119/135`; `context/manager.rs:193-198`).
  Plus: Path A can drop the system message.
- (d) `AgentPrompt` is a system string plus unused instructions: TRUE.
  `instructions` written at `defaults.rs:100`, hardcoded empty at
  `compiler/to_artifact.rs:148`, never read into a prompt.
- (e) extension surfaces do not compose the hot path: TRUE. `PluginLoader`
  trait (`wasm/plugin_loader.rs:109`) has zero implementors and callers; doc
  comment `:108` says wiring is a follow-up. `WasmSandbox` constructed
  `server.rs:1196`, stored `lib.rs:148`, never read. WIT files
  `wit/uar-plugin.wit`, `wit/uar-skill.wit` have no host binding on the run
  path. `src/uar/tools/` (7 modules) is a real tool-only extension point on the
  hot path (`orchestrator.rs:986`, `1010`).

## 5. Resiliency on the model path

Retries: stream creation only, `orchestrator.rs:625-697`, wrapped in
`timeout(stream_start_timeout_ms)`. Backoff `:640-643`
`base * multiplier^exponent`, capped `retry_max_delay_ms`, bounded by
`retry_budget_ms` (`:644`, `:671`). Retryability by substring match on the
error string (`:150-175`).

Unconsumed knobs: `ResiliencePolicy` (`settings/resilience_policy.rs:6-23`)
declares 16 fields. `retry_jitter_mode` validated (`:97-100`), surfaced
(`settings/manager.rs:1089, 1184, 1951`), applied at `server.rs:2142`, but the
backoff at `orchestrator.rs:640-643` applies no jitter. `retry_respect_retry_after`
default true (`:39`), plumbed (`settings/manager.rs:1096, 1190`), but no
`Retry-After` header parsed anywhere in `src/llm/`. `rate_limit_*` governs HTTP
ingress (`security/rate_limit.rs:71`, `server.rs:1091-1094`), not outbound calls.

Failover: one hop, creation-time only. `manager.rs:1665-1690` builds a single
fallback from `fallback_models.first()`; `orchestrator.rs:712-765` tries it once
on primary creation failure. `fallback_models[1..]` unused.

Health: `src/llm/health.rs` (190 lines) `ProviderHealthMonitor` (`:69-85`,
`:89-94`, `:123-137`), consulted by `ModelRouter::route` (`router.rs:54`, `119`),
recorded by the orchestrator (`:708, 714, 733, 745, 782`). `ModelRouter` is
constructed `server.rs:1178`, exposed as `AppState.model_router` (`lib.rs:141`)
for `POST /api/uar/route`; `start_run_with_policy_and_history` never calls it;
the model comes from policy (`manager.rs:1584-1607`).

Mid-stream failure: `Err(e)` yields `Error` and returns (`orchestrator.rs:858-864`
region). `accumulated_content` is appended to the session on every exit path
including error (`manager.rs:2397-2399` after the `Err` arm `:2386-2395`), no
truncation marker.

Stream resumption: `GET /api/uar/runs/{id}/stream` (`routes.rs:76-143`) supports
`last_event_id` query or `Last-Event-ID` header (`:94-99`), replay
(`:118-122`), live filter (`:128-130`), AG-UI snapshot (`:102-116`). Main chat
SSE (`server.rs:5019`) calls `history_since(&run_id, None)`, no header read.
Replay bounded by `EVENT_HISTORY_LIMIT = 512` with `pop_front()`
(`manager.rs:75`, `:2645`). `src/uar/realtime/{surreal_bus.rs:159,
postgres_bus.rs:130}` reconnect the DB-notify bus, not the model stream.

Cancellation: root token → per-run child (`manager.rs:1219-1225`); `cancel_run`
(`:728`), `cancel_run_for_user` (`:750`), `cancel_session_run` (`:763/769`);
biased `select!` in tool loop (`:2064-2076`) and graph (`:1966-1979`);
`RunDisconnectGuard` (`:417-449`); `cancel_run_if_no_subscribers` (`:2623`);
terminal `Cancelled` (`:2423-2430`). Cooperative only; in-flight MCP call or
`terminal_exec` subprocess is not killed.

Timeouts: `stream_start_timeout_ms` default 15s (`orchestrator.rs:628`) is the
only model-path timeout; `request_timeout_ms` (30s) applies to HTTP ingress
(`server.rs:1656`). No idle/stall timeout on an established stream.

Loop bound: `MAX_TOOL_ITERATIONS = 10` (`orchestrator.rs:102`, enforced
`:555-568`). Approval wait 300s (`manager.rs:1804`).

## 6. Open OpenSpec changes touching these areas

| Change | Unchecked / total | Note |
|---|---|---|
| `uar-scoped-chat-control-plane` | 17 / 21 | Unchecked: 3.1 typed execution context; 3.2 scoped retrieval; 3.3 scoped skill matching; 3.4 scoped MCP + tool approval; 3.5 model capabilities and context budgets without hard-coded 128K; 3.6 event replay; all of §4, §5 |
| `central-uar-error-enum` | 5 / 24 | explicit deferrals with rationale |
| `add-run-cancellation` | 4 / 31 | live-env manual checks, badge, validate |
| `repair-activate-prompt-caching` | 1 / 18 | 5.2 Playwright; HashMap ordering defect defeats it anyway |
| `resumable-streaming-client` | 1 / 17 | 5.3 manual mid-stream drop |
| `per-model-context-strategy` | 0 / 9 | implemented, not archived |
| `prompt-dialect-engine` | 0 / 8 | implemented, not archived |
| `add-configurable-resilience-policies` | 0 / 21 | complete but jitter/Retry-After unconsumed; archived duplicate exists |
| `provider-health-failover` | 0 / 12 | complete; router off hot path |
| `progressive-summarization` | no change dir | feature exists in code with no change record |

Also unarchived: `plugin-loader-wit-contract`, `wasm-component-skill-runtime`,
`wasmtime-disposition`.

## 7. Defects, ranked by the explorer

1. System prompt can be trimmed away (`manager.rs:1471` + `:1517` + `strategy.rs:119`).
2. Malformed tool-call JSON executes with empty args (`orchestrator.rs:982`, `:1032`).
3. Tool-call/result pairs severable (`strategy.rs:119, 135, 298, 331`).
4. Skill overlay order nondeterministic (`skills/registry.rs:16, 209`).
5. Identical repeated messages deduplicated (`context/manager.rs:193-198`).
6. Two tokenizers disagree (`strategy.rs:100` vs `token_service.rs:9`).
7. Tool output unbounded (`terminal_exec.rs:75-81`; `orchestrator.rs:1010-1021`).
8. `retry_jitter_mode` / `retry_respect_retry_after` dead config.
9. Health monitor does not gate run model selection (`manager.rs:1584-1607`).
10. Partial streamed turns persist as complete (`manager.rs:2397`).
11. Main chat SSE cannot resume (`server.rs:5019`).
12. `AgentPrompt.instructions` write-only.
13. `src/uar/prompt_cache.rs` dead.
14. `WasmSandbox` constructed, never read.
15. `PluginLoader` no implementors.
16. `Checkpoint::restore_state` swallows deserialization failure (`checkpoint.rs:51`).
17. `"system"` seed messages dropped (`manager.rs:1152`).
18. No stall timeout on established streams.
19. Retryability by error-string substring (`orchestrator.rs:150-175`).
20. No world state, no prompt provenance.
