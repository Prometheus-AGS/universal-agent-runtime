ASSESSMENT: codex-harness-comparative-analysis
Project: Universal Agent Runtime (UAR)
Date: 2026-09-02
Codebase baseline: UAR at `dce44e78` compared against Codex CLI `codex-rs` at `986ff1cc`, plus a web survey of fourteen other harnesses and four UI/agent protocols fetched 2026-09-01.
Cross-tool progress: none recorded for this child; parent `agui-a2ui-selection-architecture` has 2 of 3 changes done with `select-and-observe-presentations` in progress and no written plan.

Evidence files under `evidence/` in this directory hold every file:line citation. This document states judgments; the evidence files state facts. Citation convention: paths beginning with `src/`, `tests/`, `openspec/`, `wit/`, `Cargo.toml`, or `versions.toml` are relative to this repository at `dce44e78`; every other path (`core/`, `tools/`, `ext/`, `skills/`, `codex-mcp/`, `rmcp-client/`, `protocol/`, `hooks/`, `context-fragments/`, `utils/`, `codex-api/`, `codex-client/`, `otel/`, `agent-graph-store/`, `app-server-protocol/`, `sandboxing/`) is relative to the Codex CLI workspace at `/Users/gqadonis/Projects/references/codex/codex-rs` at `986ff1cc`, which is outside this repository.

## Framing

The operator's instruction is to take Codex's reliability and the parts of its design that fit UAR's goals, not to turn UAR into Codex. UAR's mission is a provider-neutral universal runtime with artifacts, scoped policy, Cedar governance, normalized events, A2A, A2UI, AG-UI, liter-llm routing, and sandboxing. Codex is a single-provider product. Every comparison below is filtered through that difference, and a final section names the Codex behaviors that must not be copied.

The operator also supplied a prior "Codex-Derived UAR Runtime Kernel" analysis. This assessment verifies its claims rather than adopting them. Where a claim was wrong, the correction changes what should be built.

## IMPLEMENTATION STATUS

Status is UAR's current state on each axis. "Codex reference" is the pattern Codex uses, cited in the evidence files.

### 1. Harness and turn assembly: PARTIAL

UAR runs every turn through one function, `start_run_with_policy_and_history`, about 1,510 lines in `src/uar/runtime/manager.rs`, with prompt, RAG, skills, MCP, policy, context, credentials, failover, and approval sequenced imperatively. There is no turn type, no step type, and no immutable snapshot; the tool list is frozen before the loop and cannot change mid-run. Memory is not even in this function: it is prepended to the user message text in `src/server.rs`, so a direct `start_run` caller gets no memory.

Codex reference: a turn-frozen `TurnContext` (`core/src/session/turn_context.rs:194-246`) and a step-frozen `StepContext` (`core/src/session/step_context.rs:15-34`) capture settings, MCP catalog, tool router, token budget, and environments once per sampling request (`core/src/session/turn.rs:344-367`), so context, advertised tools, and executed tools share one view. Twelve contributor traits (`ext/extension-api/src/contributors.rs:77-380`) let extensions add prompt fragments, tools, MCP servers, and lifecycle observation without touching the core loop. UAR counterpart: `src/uar/runtime/manager.rs:1094-2604`, `src/server.rs:4941-4947`.

Gap: the supplied analysis's diagnosis here is correct. The absence of a typed per-step snapshot is the root of several defects below (tool list frozen, skills matched once, prompt sections unordered).

### 2. System prompts and prompt engineering: STUB

UAR's prompt type is `AgentPrompt { system, instructions }`; `instructions` is written by defaults and the compiler but never read. The prompt is one string built by `push_str`: artifact system text, RAG citations, skill overlays, with the XML tool block prepended at driver time. Skill overlay order comes from `HashMap` iteration, so identical turns can produce byte-different prefixes, which defeats the Anthropic cache controls that `repair-activate-prompt-caching` was meant to deliver. There is no authority or provenance marker separating operator text from retrieved documents or skill bodies, and no world state (cwd, time, environment, project instructions) is injected at all.

Codex reference: every prompt section is a `ContextualUserFragment` with markers, a role, and a stable content kind (`context-fragments/src/fragment.rs:64-119`); world state is an `IndexMap` of sections with stable IDs (`core/src/context/world_state/mod.rs:228-262`, `:288`) diffed with RFC 7386 merge patches (`:314-348`) so only changed sections are re-sent (`core/src/context_manager/history.rs:191-208`); base instructions come from the model catalog (`protocol/src/openai_models.rs:518-535`), not from the binary. AGENTS.md is discovered root-to-cwd with an override file and skipped for untrusted projects (`core/src/agents_md.rs:1-64`). UAR counterpart: `src/uar/domain/artifact.rs:118-122`, `src/uar/runtime/manager.rs:1229-1477`, `src/uar/runtime/skills/registry.rs:16, 209-215`, `src/llm/anthropic_cache.rs:55-73`.

Cross-harness: Claude Code, Gemini CLI, Amp, OpenCode, Cursor, and Kimi all layer project instruction files by ancestor walk and load subtree files lazily. UAR has none of this, which matters directly for the code-generation goal.

### 3. Skill activation: PARTIAL

Discovery, scoped enablement (global, agent, conversation), effective-policy intersection, provenance, and collision handling are real and tested. Matching is not: the keyword threshold parameter is never read, LLM matching falls back to keyword with a warning, and the legacy classifier path injects every scored skill even when classification is below threshold. `max_active` and `prefer` are defined and never enforced. There is no catalog mode; every matched skill's full SKILL.md body is appended to the system prompt with no token budget. There is no model-facing or API-facing explicit activation.

Codex reference: a one-line-per-skill catalog (`ext/skills/src/render.rs:252-261`) under a budget of 2% of context with a 10,000-token configured cap and 8,000-char fallback (`render.rs:17-22`, `:127-153`), round-robin description truncation before any omission (`render.rs:325-366`, `:408-447`), full body only for explicitly mentioned skills (`skills/src/selection.rs:42-109`) capped at 8,000 bytes (`render.rs:19`, `:1176-1178`), ambiguous `$name` mentions ignored rather than guessed (`selection.rs:178-190`). The six-selector dynamic chooser runs in shadow mode and never changes the prompt (`ext/skills/src/shadow_selection_experiment/mod.rs:1, 50-68`; `ext/skills/src/config.rs:14-15`). UAR counterpart: `src/uar/runtime/skills/service.rs:625-731`, `src/uar/runtime/manager.rs:1358-1454`, `src/uar/domain/artifact.rs:96-105`.

Cross-harness: catalog-then-load is universal across the surveyed harnesses. Gemini and OpenCode use a model-callable activation tool; Claude Code, Codex, Kimi, and Cursor use explicit mentions. UAR is the only runtime in the survey that injects full bodies for every match.

### 4. Skill use after activation: STUB

Once matched, the overlay persists only because the system message is never regenerated, and it is matched against the first user input only. Skill-declared MCP servers are the sole capability a skill can contribute; native tools, WASM components, and scripts declared by skills are not callable (the WASM runtime is orphaned, script execution is advertised in a doc comment and absent). `preferred_tools` is parsed, persisted, and never applied. Attribution of tool use back to a skill is server-granular and excludes overlay-only skills, which is the common case. The overlay sits in the system message that both context trimmers can drop, and `SkillActivated` is emitted after trimming, so telemetry can report an activation whose body is gone.

Codex reference: the body is a first-class conversation item recorded once (`core/src/session/turn.rs:289-290`, `:926-976`) that persists across turns and is the first thing reclaimed under context pressure (`ext/skills/src/lib.rs:50-54`; `core/src/context_manager/history.rs:600-628`); bundled scripts run through ordinary exec under the full policy stack and are attributed after the fact (`skills/src/invocation.rs:26-43`, `:111-127`; `core/src/skills.rs:121-209`); a `skills.read` tool pages resources under a byte budget with a fail-closed sandbox check (`ext/skills/src/tools/read.rs:135-154`, `:294-331`; `ext/skills/src/tools/mod.rs:56`). Skills expand context, never authority (`ext/skills/src/extension.rs:289-322`; `core/src/mcp_skill_dependencies.rs:40-83`). UAR counterpart: `src/uar/runtime/manager.rs:1439-1477`, `:1516-1550`, `src/uar/runtime/skills/wasm_runtime.rs:94-296`, `src/uar/runtime/skills/storage/filesystem.rs:139`.

Cross-harness: Claude Code re-attaches the most recent skill invocations after compaction under a 25,000-token budget and tags every token and cost metric with `skill.name`; Amp hides tools from a skill-bundled MCP server until the skill loads. Only Claude Code documents skill telemetry attribution.

### 5. MCP server activation and execution: PARTIAL

Global servers connect eagerly with per-server failure isolation, namespacing, generation-guarded reconnect, metrics, and shutdown. Two real problems: skill-contributed servers are re-spawned by `McpRegistry::from_config` for every run with no connection reuse, paying up to 30 seconds of connect and list on the critical path; and `Stdio.sandboxed` is stored and echoed but never applied, so the spawn is an unisolated `Command`. `merge` does not deduplicate the tools vector. Tool exposure is all-or-nothing; there is no deferred loading or tool search.

Codex reference: lazy startup gated on a cached, model-visible catalog (`codex-mcp/src/connection_manager.rs:258-259`, `:559-576`, `:700-705`); connection reuse keyed on config and credential identity (`:94-129`); per-turn projection with `Direct`/`Deferred`/`Hidden` exposure (`tools/src/tool_executor.rs:51-80`; `core/src/mcp_tool_exposure.rs:75-147`; `core/src/tools/spec_plan.rs:196-268`) and a `tool_search` tool registered only when deferred tools exist (`spec_plan.rs:371-406`); skill MCP dependencies admitted through the policy catalog twice (`core/src/mcp_skill_dependencies.rs:76-113`). Codex does not sandbox MCP child processes either (`rmcp-client/src/stdio_server_launcher.rs`; `connection_manager.rs:324-325`); that is one of the two behaviors the supplied analysis correctly says not to copy. UAR counterpart: `src/mcp/registry.rs:281-366`, `:423-428`, `:733-734`, `src/mcp/config.rs:21`, `src/uar/runtime/manager.rs:1448-1470`, `:1556-1583`.

Cross-harness: Claude Code turned tool search on by default and reports an 85% token reduction; MCP 2026-07-28 makes `tools/list` ordering deterministic for cache hits and deprecates Dynamic Client Registration. UAR pins `rmcp =3.1.2` and every surveyed client targets 2025-11-25 semantics, so this is a shared drift, not a UAR-specific one.

### 6. Tool calling: PARTIAL

The approval gate is real: Cedar is consulted first, deny wins, approval-required emits an event and waits 300 seconds. Parallel execution exists but is gated by name-prefix inference (`get_`, `list_`, …). Three defects the supplied analysis got right: malformed tool-call JSON executes as `{}` at two sites with no log and no feedback to the model; tool effects are inferred from name substrings; there is no output truncation anywhere on the tool path, so one verbose command can fill the window. Two claims it got wrong: native and MCP registries are combined early and frozen, not late, and `jsonschema` is pinned with a caret in Cargo.toml, absent from `versions.toml`, and unused on the tool path. The `ToolNormalizerDriver` and the XML tool injector are dead code, so models needing XML tool injection are unsupported despite 820 lines existing for it.

Codex reference: a two-variant `FunctionCallError` (`RespondToModel` vs `Fatal`, `tools/src/function_call_error.rs:5-10`) forces every failure to be classified; parse errors go back to the model as a failed output (`core/src/tools/handlers/mod.rs:83-90`; `core/src/tools/parallel.rs:219-243`); parallelism is one `RwLock` where read-only tools take the read lock and everything else the write lock (`parallel.rs:42-62`, `:155-159`), with `false` as the default (`tools/src/tool_executor.rs:122-124`) and MCP `read_only_hint` as the opt-in (`core/src/tools/handlers/mcp.rs:128-139`); middle-out truncation with a warning header applies on ingest (`core/src/context_manager/history.rs:246-282`; `utils/output-truncation/src/lib.rs:14-31`); name collisions are recorded and can fail assembly (`core/src/tools/registry.rs:353-384`; `spec_plan.rs:416-470`). UAR counterpart: `src/llm/orchestrator.rs:47-76`, `:138-148`, `:498-512`, `:982-983`, `:1032-1033`, `src/uar/runtime/manager.rs:366-370`, `:1712-1826`, `src/uar/tools/terminal_exec.rs:75-81`, `Cargo.toml:418`.

### 7. Context management: PARTIAL, with correctness defects

Two independent subsystems with different enums and different tokenizers (`len/4` and `cl100k_base`) run back to back on every run. The first slices by index with no awareness of tool-call pairs, so a window boundary can orphan a tool result and produce a provider 400. The first also receives the system message at index 0 and keeps only the tail, so a long conversation drops the agent's identity, RAG block, and every skill overlay. The second deduplicates identical repeated user messages. Tool outputs enter history verbatim. Checkpoints exist for the graph branch only, and the resume endpoint loads a checkpoint then starts a new run with a prose string as input; `restore_state` has zero callers.

Codex reference: `normalize_history` guarantees every call has an output and every output a call, inserting synthetic aborted outputs (`core/src/context_manager/history.rs:563-581`; `core/src/context_manager/normalize.rs:21-138`); compaction keeps user messages newest-first under a budget (`core/src/compact.rs:61`, `:645-734`), keeps a separate review-evidence history (`history.rs:1-2`, `:394-409`), invalidates the world-state baseline on any rewrite (`history.rs:373`, `:391`, `:408`), and falls back from a previous model to the current one with a smaller retry budget (`core/src/compact_model_fallback.rs:9-64`; `core/src/compact_remote_v2.rs:78-79`, `:254-291`). Token counting is declared a coarse lower bound (`history.rs:332-334`) and the compaction budget excludes the cached prefix (`core/src/session/context_window.rs:59-79`). UAR counterpart: `src/uar/context/strategy.rs:100-153`, `:282-361`, `src/uar/runtime/context/manager.rs:86-252`, `src/uar/runtime/context/token_service.rs:9-39`, `src/uar/runtime/manager.rs:1471-1538`, `src/uar/runtime/checkpoint.rs:50-57`, `src/uar/api/routes.rs:346-395`.

### 8. Sub-agent architecture: STUB

Three execution paths share no kernel: actors build their own `Orchestrator` from the server-wide config with no artifact, policy, Cedar, budget, cancellation, or events; local graph nodes make a two-message call with `tools: Vec::new()`; remote A2A nodes send bare text with no auth, timeout, or cancellation, and UAR's own inbound A2A endpoint is bound to the compiler, so it cannot run an agent. The graph path is hardwired to one agent id and two sub-agent ids that have no artifacts. Inter-agent identity is text prepended to the user message. There is no parent/child persistence, no depth or concurrency limit, no budget sharing (graph runs bypass cost accounting entirely), and `BudgetStatus::Exceeded` never stops anything. The actor REST API takes no user context. The supplied analysis's claim here is true and understated.

Codex reference: every child is a real thread through the same kernel (`core/src/agent/control.rs:118-134`; `core/src/agent/control/spawn.rs:586-697`) with runtime policy forced from the live turn and applied again after roles (`core/src/tools/handlers/multi_agents_common.rs:235-262`; `core/src/tools/handlers/multi_agents_v2/spawn.rs:145`); roles can only narrow (`core/src/agent/role.rs:1-4`, `:91-118`); children never ask for approval (`core/src/codex_delegate.rs:63-68`) and cannot widen permissions because the intersection fails closed (`control/spawn.rs:450-527`; `protocol/src/permission_profile_intersection.rs:31-68`); the human's authorization is read from the root thread, not from anything a child says (`core/src/agent/control/user_authorization.rs:22-94`); a typed `InterAgentCommunication` with `trigger_turn` distinguishes a note from a wake-up (`protocol/src/protocol.rs:803-819`); spawn is persisted to a graph store (`control.rs:817-846`; `agent-graph-store/src/store.rs:17-59`); caps exist for total children (`core/src/agent/registry.rs:96-115`; `core/src/config/mod.rs:232-233`, `:1562-1575`), concurrently running turns (`core/src/agent/control/execution.rs:13-97`), and V1 depth (`config/mod.rs:242`; `spec_plan.rs:650-658`), with LRU residency eviction (`core/src/agent/control/residency.rs:117-158`). UAR counterpart: `src/uar/runtime/actor/agent_actor.rs:25-61`, `:201-209`, `src/uar/runtime/actor/system.rs:91-132`, `:178-227`, `src/uar/runtime/graph/types.rs:49-64`, `src/uar/runtime/graph/nodes/agent_node.rs:99-118`, `:200-235`, `src/uar/api/a2a/handler.rs:41-47`, `:142-161`, `src/uar/runtime/manager.rs:1864-1868`, `:1975-1988`, `:2054`, `:2454`, `src/uar/runtime/cost_budget.rs:130-139`, `src/uar/api/actors.rs:22-50`.

Cross-harness: every surveyed harness that has subagents gives them fresh context by default, returns a final summary, and caps depth or forbids nesting.

### 9. Resiliency: PARTIAL

Strong: run cancellation with disconnect guard, graceful shutdown, worker pool, MCP reconnect with generation guard, resumable streaming on the run stream endpoint, configurable resilience policy, provider health monitor and failover, graceful shutdown tests. Weak: retries cover stream creation only, retryability is decided by substring-matching error strings, `retry_jitter_mode` and `retry_respect_retry_after` are validated and persisted but never read, failover is one hop from a `Vec`, the health monitor does not gate run model selection, there is no stall timeout on an established stream, a half-streamed turn is persisted as a complete assistant message, the main chat SSE path ignores `Last-Event-ID`, cancellation does not reach subagents or kill in-flight subprocesses, no circuit breaker exists, and `/readyz` reports SurrealDB healthy on handle presence without probing.

Codex reference: honors server `Retry-After` before computed backoff (`core/src/responses_retry.rs:105`; `codex-api/src/api_bridge.rs:26-40`), unbounded connection retries with capped doubling and a transport fallback (`responses_retry.rs:17-18`, `:58-100`), jittered exponential backoff (`core/src/util.rs:6-7`, `:86-91`; `codex-client/src/retry.rs:8-48`), per-provider idle timeouts on SSE and WebSocket (`codex-api/src/endpoint/responses.rs:187`; `responses_websocket.rs:707`), flushes the rollout before emitting an abort event (`core/src/tasks/mod.rs:955-960`), records an interrupted-turn marker the model can see (`tasks/mod.rs:101-116`; `core/src/context/turn_aborted.rs:10-11`), and supports turn suspension so another worker can recover a turn under its original id (`core/src/session/turn_suspension.rs:13-120`; `core/src/codex_thread.rs:357-439`). UAR counterpart: `src/llm/orchestrator.rs:150-175`, `:625-697`, `:712-765`, `:858-864`, `src/uar/settings/resilience_policy.rs:6-45`, `src/llm/router.rs:54, 119`, `src/uar/runtime/manager.rs:196-200`, `:417-449`, `:1584-1607`, `:2386-2399`, `src/server.rs:2299-2303`, `:5019`, `src/uar/api/routes.rs:76-143`.

Cross-harness: Gemini's own tracker calls its fallback fragile for exactly the string-matching reason UAR has. Pydantic AI's `FallbackModel` and Claude Code's partial-output-on-overload are the provider-neutral versions of what UAR needs.

### 10. Extensions and hooks: STUB

The only functioning extension point adds tools (`src/uar/tools/mod.rs`; `src/uar/runtime/native_skill.rs:49-61`). `PluginLoader` has no implementors (`src/uar/runtime/wasm/plugin_loader.rs:108-109`), `WasmSandbox` is constructed and never read (`src/server.rs:1196`; `src/lib.rs:148`), the WIT contracts (`wit/uar-plugin.wit`, `wit/uar-skill.wit`) have no host binding on the run path, and the parking-lot `HookBus` was explicitly killed in `uar-harness-parity` in favor of `RuntimeStep` events on the existing bus. Codex has twelve lifecycle hook events (`hooks/src/lib.rs:23-35`; dispatch points `core/src/hook_runtime.rs:124-593`) with a stable tool-agnostic pre/post payload contract (`core/src/tools/registry.rs:95-143`) and twelve contributor traits (`ext/extension-api/src/contributors.rs:77-380`). Claude Code, Gemini, and Cursor all expose pre/post tool hooks.

### 11. Observability: PARTIAL

OTLP tracing, latency and cost metrics, normalized events, and `RuntimeStep` exist. Graph runs fabricate `RuntimeStep` pairs after completion (`src/uar/runtime/manager.rs:1993-2009`). Nothing attributes tokens or tools to a skill (`src/uar/runtime/manager.rs:136-151`, `:1441-1445`). Codex separates logs from traces as a privacy boundary, prompt text and MCP names only in logs, lengths and counts in traces (`otel/src/events/session_telemetry.rs:585-596`, `:1053-1072`), never logs inter-agent message bodies (`core/src/agent_communication.rs:59-62`), and emits a `codex.skill.injected` counter with skill, plugin, and model tags (`core/src/skills.rs:64-75`).

### 12. Testing: PARTIAL

UAR has scoped-governance (`tests/skill_scoped_governance.rs:159, 352, 471`), wasm (`tests/wasm_skill_execution.rs:41, 85`), graph, checkpoint, and chat-completion tests. There is no end-to-end test from skill activation through a skill-contributed tool call to outcome correlation, and none for tool-loop protocol well-formedness. Codex puts an orphaned-call validator inside its wiremock so every model-backed test is also a wire-format conformance test (`core/tests/common/responses.rs:711`, `:1675-1740`), matches mock responses on request content so concurrent agents can be scripted (`responses.rs:1103`; `core/tests/suite/agent_execution.rs:53-108`), snapshots the diff between successive requests to catch prefix-stability regressions (`core/tests/suite/snapshots/*_request_diff.snap`), and commits its generated protocol schema with experimental fields filtered (`app-server-protocol/src/schema_fixtures.rs:8, 32`; `precomputed_exports_tests.rs:13-42`).

### 13. UI and agent protocols: PARTIAL

A2UI is pinned to production v0.9.1 in Rust with `deny_unknown_fields` (`src/uar/a2ui/protocol.rs:12-17`, `:29-58`, `:260-262`), nine specs, and four open migration changes; v1.0 remains a candidate upstream (`actionResponse`, `surfaceProperties`), so the pin is correct today. AG-UI uses a self-defined `uar.agui/1` profile (`src/uar/api/adapters.rs:68`) with the official event names (`adapters.rs:87-278`) plus a legacy dotted vocabulary kept in parallel (`src/uar/api/sse.rs:295`, `:345-444`) with no deprecation path; upstream added `SubagentStarted/Finished/Error` events, which UAR cannot emit because it has no subagent lifecycle. A2A is JSON-RPC RC v1.0 (`src/uar/api/a2a/mod.rs:1`, `:61-68`) and gRPC v0.3 (`src/uar/api/a2a/grpc.rs`) behind `server-full` (`Cargo.toml:136-140`), both fronting the compiler (`src/uar/api/a2a/handler.rs:41-47`); upstream is v1.0.1 with `tasks/list` and a standardized "canceled" state. MCP-UI / MCP Apps is entirely absent (zero matches for `ui://`, `mcp-ui`, `mcp_apps` across `src/`, `frontend/`, `openspec/`), and upstream has finalized it as `io.modelcontextprotocol/ui` (spec 2026-01-26, SDK 1.7.5). The in-progress `select-and-observe-presentations` change is not an OpenSpec change and has no written plan (only `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/progress.json`).

## VERIFICATION OF THE SUPPLIED ANALYSIS

| Claim | Verdict | Consequence |
|---|---|---|
| Prompt, skills, RAG, MCP, policy, tools manually sequenced at `manager.rs:1229` | TRUE | Typed assembly is justified |
| Malformed JSON executes as `{}` at `orchestrator.rs:985` | TRUE, two sites | Fail-closed argument parsing is justified |
| Two context strategies run sequentially; repeated messages and tool pairs damaged | TRUE, plus system prompt can be dropped | One history path is justified and more urgent than stated |
| Every selected full overlay injected; threshold ignored; LLM matching falls back | TRUE | Catalog mode is justified |
| Skill MCP registries recreated and merged each run | TRUE | Projected MCP runtime is justified |
| Actors, graph nodes, A2A diverge; local children bypass artifacts, skills, tools | TRUE, understated | Thread-native subagents are justified |
| `AgentPrompt` is a system string plus unused instructions | TRUE | Typed fragments are justified |
| Extension surfaces do not compose the hot path | TRUE | Contributor registry is justified |
| Native and MCP registries combined late | FALSE, combined early and frozen | The fix is per-step re-projection, not earlier merging |
| Name-prefix inference of tool effects | TRUE | Effect classification is justified |
| `jsonschema` already pinned | PARTIAL, caret pin, not in `versions.toml`, unused on tool path | Pin must be recorded before relying on it |
| Codex dynamic skill selector must not become the correctness gate | TRUE, Codex itself runs it shadow-only | Agreed |
| Codex MCP child processes are unsandboxed | TRUE | UAR must not copy; UAR's own `sandboxed` flag is also inert, which the analysis did not say |

The supplied analysis omitted: skill-overlay ordering breaking prompt caching, the system prompt being trimmable, checkpoint resume discarding state, dead resilience knobs, the health monitor being off the hot path, the inbound A2A endpoint not running agents, MCP-UI being absent, and MCP 2026-07-28 drift. Its subagent limits (four concurrent, depth three, sixteen total) match the surveyed range (Codex 3 children default, Claude Code 20 concurrent and depth 3, Cursor depth 2).

## WHAT NOT TO COPY FROM CODEX

- Responses API `previous_response_id` incremental resumption, `x-codex-turn-state` sticky routing, and WebSocket-to-HTTPS transport fallback are OpenAI-transport specific. UAR's equivalent must live behind liter-llm and the driver trait.
- ChatGPT-auth gating (`cyber_access_program`, agent-identity JWTs against a vendor backend, Codex Apps entitlement metadata) is product identity, not runtime design. UAR already has Cedar, JWT hardening, and tenant isolation.
- The guardian reviewer as a model-based approval oracle with a circuit breaker is a judgment call UAR should evaluate, not adopt. UAR's governance is policy-first (Cedar); a model reviewer would be an additional tier behind it, and the survey shows every classifier tier is documented as best-effort.
- Unsandboxed MCP child processes.
- The shadow skill selector as a correctness gate.
- Base instructions sourced from a vendor model catalog. UAR's equivalent is the artifact plus the prompt-dialect engine, which already exists and is partly unwired (`prefers_xml_envelope`, `markdown_averse` have no callers).

## CROSS-TOOL PROGRESS

- NONE recorded in this child's `progress.json`.
- Parent child phase: `establish-presentation-catalog-workspace` DONE, `scope-presentation-capabilities` DONE, `select-and-observe-presentations` IN_PROGRESS with zero tasks and no plan file.
- Open changes touching the axes above with unchecked tasks: `uar-scoped-chat-control-plane` 17/21 unchecked (typed execution context, scoped retrieval/skills/MCP, context budgets without hard-coded 128K), `wasm-component-skill-runtime` 6/15, `central-uar-error-enum` 5/24, `add-run-cancellation` 4/31, `skill-activation-metrics` 3/7, `add-skill-kind-and-origin` 3/11, `add-skill-system-submodule` 3/16, `microsandbox-mcp-isolation` 1/80 (the sandboxed-stdio test), `tool-loop-cedar-gating` 1/14, `resumable-streaming-client` 1/17, `repair-activate-prompt-caching` 1/18, four A2UI migration changes (31/40, 18/24, 19/23, 22/24), `certify-operational-resilience` 3/12, `perf-security-load` 23/27.
- Complete but unarchived: `ch08-activation-outcome-correlation`, `wire-mcp-server-provisioning`, `rmcp-pin-bump`, `wasmtime-disposition`, `per-model-context-strategy`, `prompt-dialect-engine`, `add-configurable-resilience-policies` (with two unconsumed knobs), `provider-health-failover` (router off hot path).
- `progressive-summarization` exists in code and as a spec with no change record.

## SPEC GAP SUMMARY

- `multi-agent-orchestration` spec requires router→agent traversal with attributed sub-agent contribution and step events; implemented only for one hardwired agent id, with fabricated step events and no tool use in children.
- `deep-health-probes` spec requires Postgres, Redis, and SurrealDB probes; SurrealDB is asserted from handle presence and Redis is never checked.
- `ag-ui-chat-conformance` spec Purpose is an unedited archive placeholder.
- `progressive-summarization` spec's "summary preserves key decisions, tool results, and user preferences" is not verifiable because the trimming path can remove the system prompt and orphan tool results before summarization runs.
- No spec covers: prompt fragment authority and ordering, skill catalog budgets, explicit skill activation, tool-argument validation, tool-output limits, tool effect classification, parent/child thread persistence, subagent limits, MCP lazy startup, MCP Apps, project-instruction discovery, world-state injection, stall timeouts, or `Retry-After` handling. Each of these needs a new capability or an extended requirement before any change can pass `openspec validate`.
- Unplanned scope present in code without a spec: actor REST API, `src/uar/prompt_cache.rs`, `ToolNormalizerDriver` family, `WasmSandbox` wiring.

## BUILD HEALTH

- build check: PASS — `cargo check --locked --no-default-features --features server-full` finished clean in 7m 52s (Tier 0). No tests were run at this stage; that is correct for assess.
- known violations: `bun run typecheck` had 17 pre-existing errors as of the `uar-next-harness` reflection; not re-measured here.
- test coverage: PARTIAL — no activation-through-use test, no tool-loop conformance test, no resume-from-checkpoint test that asserts restored state.

## CONSTRAINT CHECK

- AGENTS.md / CLAUDE.md violations: capability inversion says mutating actions are gated in the trusted host layer, never in an agent kernel. The actor REST API accepts an arbitrary system prompt and spawns agents without user context, and graph children run with no policy at all. Both are gaps against that rule, not violations introduced by this phase. `versions.toml` is authoritative for pins, and `jsonschema`, `rmcp`, `tiktoken-rs`, `wasmtime`, `tonic`, and the A2A/AG-UI protocol versions are not recorded there.
- constraints.md violations: N/A — `.kbd-orchestrator/constraints.md` does not exist in this checkout (`ls` returns "No such file or directory"), so no project-level constraints file was available to check.
- GitHub Actions policy: no workflow changes are implied by this assessment.

## GOAL PROGRESS

- Goal 1, compare codex-rs against UAR on the eight seed axes plus resiliency and skill use: MET — each axis above cites both codebases and the survey.
- Goal 2, gap analysis citing concrete files in both repos including practices outside the seed list: MET — evidence files cover hooks, extensions, observability, testing, governance, and thread persistence, which were not on the seed list.
- Goal 3, rank findings and write an ordered plan with spec deltas: PARTIAL at this stage. The three goals in `goals.md` are phase goals shared by assess, analyze, and plan. Assess supplies the ranking below and the evidence behind it; the ordered change list with spec deltas is the plan stage's artifact, because the KBD phase order forbids planning before analyze has weighed build-versus-adopt for each item. The ranking here is therefore an input to plan, not a substitute for it.

### Preliminary value ranking (input to analyze and plan)

Immediate value, correctness defects with a named failure and a small blast radius:

1. Context correctness: tool-call/result pairs severable, system prompt trimmable, unbounded tool output, repeated-message dedup (`strategy.rs:119, 135`; `manager.rs:1471, 1517`; `terminal_exec.rs:75-81`; `context/manager.rs:193-198`). Fails with a provider 400 or a run that has forgotten its own identity.
2. Fail-closed tool arguments and effect classification (`orchestrator.rs:982, 1032, 138-148`; `manager.rs:366-370`). Fails by executing a tool with empty arguments or running a mutating tool in parallel.
3. Deterministic prompt section ordering (`skills/registry.rs:16, 209-215`). Fails by defeating the prompt cache the operator turned on.
4. Skill catalog with budget, threshold enforcement, `max_active`, and explicit activation (`service.rs:698`; `manager.rs:1383-1394, 1448-1454`; `artifact.rs:96-105`). Fails by injecting every scored skill body and by never being able to prove which skill was used.
5. Resilience knobs that exist but are unread, health gating on the hot path, stall timeout, partial-turn marker, chat SSE resume (`orchestrator.rs:640-643`; `manager.rs:1584-1607, 2397`; `server.rs:5019`).

Later value, structural and larger:

6. Typed turn and step assembly with contributor traits; the prerequisite for items 7 and 8 to be testable.
7. Projected MCP runtime with lazy start, connection reuse, deferred exposure, and a real stdio sandbox.
8. Thread-native subagents with policy intersection, typed inter-agent messages, persistence, limits, and cancellation; unblocks AG-UI subagent events and A2A v1.0.1 alignment.
9. Project-instruction discovery and world-state injection for the code-generation goal.
10. MCP Apps support and MCP 2026-07-28 alignment; upstream-driven, not a UAR defect.

Not applicable to UAR: the Codex-specific items listed under "What not to copy".

## THE UNCOMFORTABLE THINGS

- Three prior phases (`uar-harness-parity`, `uar-next-harness`, and the supplied analysis) each restated the same harness gaps. The recurring failure mode in this repo is committed-but-unwired code: `prompt_cache.rs`, `ToolNormalizerDriver`, `WasmSandbox`, `PluginLoader`, `restore_state`, `retry_jitter_mode`, `preferred_tools`, `max_active`. A plan that adds new abstractions without a "zero external call sites" check will produce more of it.
- Several changes are marked complete whose behavior is not delivered: `add-configurable-resilience-policies` (jitter, Retry-After), `provider-health-failover` (router off hot path), `repair-activate-prompt-caching` (ordering defeats the cache), `ch08-activation-outcome-correlation` (overlay-only skills excluded). Marking them complete was not wrong under their task lists; the task lists were too narrow.
- The supplied analysis's `RunManager` migration risk is real and the mitigation (shadow assembly) is expensive. The 1,510-line function has no seams to shadow behind today.
- The research-server job for the deep-research pipeline never left initialization; its worker process is defunct. The survey in `evidence/harness-survey-2026-09-01.md` was produced by a subagent using primary sources and carries its own manifest and uncertainty section, but it did not pass the pipeline's credibility scoring or Feynman gate. Treat its harness claims as high-credibility where the manifest says so and unverified where it says 404.

## UNVERIFIED

- Tier 1 and Tier 2 test results (not run at assess).
- Live behavior of any defect above; all are static readings of the code at `dce44e78`.
- Codex claims rest on targeted reads; the explorer did not fully read `control_tests.rs`, `guardian/review.rs`, `guardian/review_session.rs`, `session/mod.rs`, or `multi_agents/wait.rs`.
- Goose tool router and its subagent cap, Amp subagent details, Cursor summarization internals.

## UNRESOLVED REVIEW FINDINGS

Two rounds of isolated cross-model adversarial review were run (`review/assess/findings.round1.json`, `review/assess/findings.json`; judge `kbd-judge` via the REST gateway, producer `claude-fable-5-1`, cross-model check verified-distinct). Round 1's two CRITICALs (missing inline Codex citations, goal 3 wording) were fixed. Round 2 leaves:

- CRITICAL, accepted as a tooling limit: the packet builder marks every Codex path as `MISSING` because it validates citations against this repository's file tree, and the Codex workspace lives outside it. The citations are real; the four `evidence/codex-*.md` files record the lines an explorer read at `986ff1cc`. The analyze stage should spot-check a sample of Codex citations directly, since the packet cannot.
- WARNING, partly accepted: cross-harness survey claims come from a subagent's primary-source reads with a per-URL credibility column, not from the deep-research pipeline, whose worker died at initialization. The immediate-value ranking (items 1 to 5) rests on UAR code reads alone. Only items 9 and 10 and the "cross-harness" sentences lean on the survey, and each of those cites a URL rated high in the manifest. Analyze must not promote a survey-only claim into a change without re-fetching its source.
- WARNING, carried to the handoff: the phase goal text asks for an ordered plan with spec deltas. That is the plan stage's artifact under the KBD phase order. The handoff records this so plan owns it explicitly rather than it being lost between stages.

## SYCOPHANCY SELF-CHECK

S-02: UAR's existing patterns were evaluated against code, not accepted; three "complete" changes are reported as not delivering their behavior. S-03: twenty-plus defects surfaced. S-06: no "clearly" or "obviously" used. The `detect_sycophancy` result is stored under `sycophancy/`.

ASSESSMENT COMPLETE
