# Evidence — UAR subagents, runtime resiliency, UI protocols, prior harness work, versions.toml (commit dce44e78)

Produced by a read-only Explore subagent on 2026-09-02. File:line citations are
against commit `dce44e78`. Raw evidence for `assessment.md`.

## 1. Sub-agent architecture

### Types

- `AgentActorState` (`src/uar/runtime/actor/agent_actor.rs:25-34`): `agent_id`,
  `status`, `history: Vec<Message>` (in-memory), `orchestrator: Arc<Orchestrator>`.
- `AgentActorArgs` (`:50-61`): `agent_id`, `llm_config`, `mcp`, `native_skills`,
  `system_prompt: Option<String>`. No artifact, SkillService, policy,
  persistence, cancellation token, or cost budget.
- `AgentMessage` (`actor/messages.rs:10-42`): `UserPrompt`, `Collaborate{from_agent_id,
  task, reply}`, `ToolResult`, `Shutdown`. `AgentReply` (`:46-54`).
- `GraphContext` (`src/uar/runtime/graph/types.rs:49-64`): `run_id`, `session_id`,
  `mcp`, `llm_config`, `driver`, `cache_strategy`, `persistence`. No artifact,
  skills, policy, cancellation token, cost budget, emitter.
- `GraphState` (`types.rs:18-25`); `AgentNode` (`graph/nodes/agent_node.rs:35-39`)
  = `id` + `agent_id_or_url`.

### Three execution paths

Path A, actor mailbox (`agent_actor.rs:103-267`, `actor/system.rs:91-132`):
`spawn_agent` (`system.rs:91`) spawns a `ractor` actor; `pre_start`
(`agent_actor.rs:108`) builds a fresh `Orchestrator` (`:115-119`) from the
server-wide `llm_config` (`server.rs:1098-1102`). Inherits MCP, native skills,
LLM config. Does not inherit artifact, prompt (unless REST body,
`api/actors.rs:50`), skills matching, RAG, memory, Cedar, guardrails, budget,
cancellation. `Collaborate` is flattened to text
`"[Collaboration request from agent {from}]: {task}"` (`agent_actor.rs:201-202`)
pushed as `User` (`:204-209`); identity survives only in reply
`metadata.collaboration_from` (`:232-234`). `collect_stream_response`
(`:68-97`) drops all events except `MessageDelta`/`Error`/`Done`. Cancellation
only cooperative `Shutdown` (`:259-263`), best-effort (`system.rs:214`, `:224`);
no root token. No depth/concurrency limit (`system.rs:99-103` only name dedup);
mutual collaborate deadlocks on paired oneshots (`system.rs:178-188`). Routes
`POST /api/uar/actors`, `/{id}/message`, `/{id}/collaborate`, `DELETE /{id}`
(`server.rs:1360-1361`, `api/actors.rs:22-28`). Handlers take no
`Extension<UserContext>` (contrast `routes.rs:327`).

Path B, local graph node (`agent_node.rs:93-160`, dispatched `:190-198`):
triggered when `agent_id_or_url` is not http(s) (`:54-56`). `execute_local`
builds a two-message request (`:99-118`) with synthesized system prompt
`"You are the '{id}' sub-agent…"` (`:104-105`) and `tools: Vec::new()` (`:109`).
Inherits only `ctx.driver` and `cache_strategy`. No artifact, skills, tools,
MCP, memory, RAG, policy, history (`:180-188` passes one string).

Path C, remote A2A (`agent_node.rs:200-235`): fresh `A2AClient::new()` per
execution (`:211`), `send_message` → JSON-RPC `message/send`
(`api/a2a/client.rs:107-111`); `Message::user_text` (`:210`) bare text; no auth,
timeout, or cancellation propagation. Writes `_agent_result_{id}`,
`_agent_task_id_{id}`, `_agent_output_{id}` (`:218-222`). Inbound A2A server
(`api/a2a/handler.rs:41-47`) is bound to `CompilerService`, not `RunManager`:
`handle_message_send` creates a `CompilerSession` (`:142`) and returns a canned
welcome (`:154-161`); `continue_task` pattern-matches `"# Agent:"` /
`"## Metadata"` (`:198`) else echoes a stub (`:222-227`). AgentCard `:100-102`.

### Dispatch gating and events

Graph path only for `artifact.id == "orchestrator-agent"` (`manager.rs:1864-1868`).
Graph is `uar::defaults::orchestrator_graph()` (`server.rs:1055`,
`defaults.rs:110-121`): `RouterNode` + `AgentNode("general-purpose")` +
`AgentNode("rust-reviewer")`, neither a persisted artifact (`defaults.rs:135`
seeds only `default-agent`, `orchestrator-agent`). Graph branch
`manager.rs:1933-2055`: `RunStart` (`:1891-1896`), `StatePatch` (`:1916-1929`),
then synthesized `RuntimeStep` pairs replayed from `_graph_trace` after
completion (`:1993-2009`); output prefixed `"[{route}]\n\n"` (`:2023`);
`RunDone` (`:2045-2049`). No `ToolStart/ToolEnd/Usage/BudgetAlert/SkillActivated`.

`LlmNode` passes `tools: ctx.mcp.openai_tools_json()` (`graph/nodes/llm_node.rs:98`,
comment `:84-97` dated 2026-09-01) but ignores every non-`MessageDelta` event
(`:125`) and has no tool-result loop (`:132-143`).

### Budget

`CostBudgetTracker` (`runtime/cost_budget.rs:89-91`), scopes `:21-28`, limits
`:45-50`, status `:63-70`, in-memory (`:80-85`, doc `:9-12`). Recorded only in
the non-graph tool loop at `manager.rs:2454-2520` (Run/Session/Agent
`:2457-2464`, Global `:2499-2500`; Task never `:2454-2455`). Graph branch returns
at `:2054`; actors have no tracker (`system.rs:54-63`). `Exceeded`
(`cost_budget.rs:130-139`) never aborts. Agent-scope limits read from
`artifact.extensions["budgets"]` (`manager.rs:1845-1863`).

### Cancellation

Root token `manager.rs:196`, per-run `:200`, `cancel_run` `:728`,
`RunDisconnectGuard` `:417-449`. Graph: biased `select!` wrapping
`graph.execute` (`:1975-1988`); `GraphContext` carries no token so an in-flight
A2A HTTP request is dropped without `tasks/cancel`; `A2AClient::cancel_task`
(`client.rs:121-125`) never called from `AgentNode`. Actors: no token.

### Persistence of parent/child

None. `PersistenceLayer` (`persistence/mod.rs:43-310`) has no run/parent/child
methods; only `parent_id` is the settings hierarchy (`mod.rs:238`,
`providers/postgres.rs:1048-1125`, `providers/memory.rs:330-336`). `Checkpoint`
has no parent-run field. `NormalizedEvent`/domain events (`domain/events.rs:6-203`,
27 variants) have no delegation variant.

### Verdict on the claim

"Actors, graph nodes, and A2A take different execution paths; local graph
children bypass artifacts, skills, and tools." TRUE and understated: also
bypass history, memory, RAG, per-agent provider policy, credential layer
(`manager.rs:249`), Cedar, guardrails, cost, usage.

## 2. Runtime resiliency

| Capability | Status | Evidence |
|---|---|---|
| Graceful shutdown | Implemented | `server.rs:1990`, `2002`, `1916-1922`, `104`, `181-182`; tests `6254-6406`; spec `graceful-shutdown`; `fix-graceful-shutdown-deadline-semantics` 29/29 |
| Worker pool | Implemented, change open | `rag/ingestion_worker.rs:362-493`, `:367`, `:433`, `:490-493`, `:180`; `server.rs:1218`; `fix-worker-pool-graceful-shutdown` 19/22 |
| Run cancellation | Implemented, change open | `manager.rs:196, 200, 728-786, 417-449`; `events.rs:87`; `add-run-cancellation` 27/31; does not reach sub-agents |
| Checkpoint/resume | PARTIAL, resume broken | `resume_run_from_checkpoint` loads (`api/routes.rs:346`) and starts a new run whose input is a prose string (`:371-386`); `restore_state()` (`checkpoint.rs:50-57`) has zero call sites; response still reports `checkpoint_iteration` (`:388-395`) |
| Resumable streaming server | Implemented | `manager.rs:2652`, `2604-2610`; `api/sse.rs:121, 261-283` |
| Resumable streaming client | Implemented | `resumable-streaming-client` 16/17 |
| Provider failover | Implemented, opt-in off | `llm/health.rs`; `with_failover_config` `manager.rs:630`; `enabled: false` default (`:212-215`); `provider-health-failover` 12/12 |
| MCP reconnect | Implemented | `mcp/registry.rs:64-161`, driven `:972-973`; spec `mcp-reconnect-recovery` |
| Circuit breakers | Not implemented | no `CircuitBreaker` type; `governor` (`Cargo.toml:419`) is ingress throttling (`server.rs:1092-1095`) |
| Configurable resilience policies | Implemented | `settings/resilience_policy.rs:6-23`, defaults `:26-45`, validate `:71-114`; `with_resilience_policy` `manager.rs:637`; 21/21 |
| Error enum | PARTIAL | `uar/error.rs:20-310`; doc `:4-7` says incremental; `central-uar-error-enum` 19/24 |
| Deep health probes | PARTIAL, spec not met | `server.rs:1291-1293`, `2275-2315`; SurrealDB reported ok on handle presence (`:2299-2303`); Redis never checked; spec `deep-health-probes` requires both |

Related open: `certify-operational-resilience` 3/12, `perf-security-load` 23/27.

## 3. UI protocol surfaces

### A2UI — v0.9.1

`src/uar/a2ui/protocol.rs:12-17`: `PROFILE = "uar.a2ui/1"`, `VERSION = "v0.9.1"`,
`COMPAT_VERSION = "v0.9"`, `CATALOG_ID = "urn:uar:a2ui:catalog:1"`,
`BASIC_CATALOG_ID = https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json`.
Validation `:260-262`. DTOs `:29-58` with `deny_unknown_fields`. Emitters
`policy_surface.rs:71-119`; test `:158`. Frontend packages: `a2ui-core` 0.10.4,
`a2ui-react` 0.10.1, `a2ui-lit` 0.1.0, `a2ui-svelte` 0.1.0, `a2ui-uar` 0.1.0,
`a2ui-inspector` 0.1.0 (package versions, not protocol versions; no mapping
asserted). Specs: `a2ui-uar-renderer` (9 reqs), `a2ui-ux-2026` (6),
`a2ui-devtools` (4), `a2ui-testing-ui` (4), `a2ui-cross-renderer-conformance`
(2), `a2ui-lit-renderer` (2), `a2ui-svelte-renderer` (2), `a2ui-live-testing`
(2), `a2ui-react-conformance` (1). Open changes:
`a2ui-migrate-design-systems-embedder-from-flint-forge` 31/40,
`a2ui-realtime-backbone-from-flint-realtime-fabric` 18/24,
`a2ui-migrate-entity-components-from-prometheus-entity-management` 19/23,
`a2ui-vendor-google-core-react` 22/24. `versions.toml` pins Zod 3.25.76 for the
`@a2ui/web_core` catalog boundary.

### AG-UI — profile `uar.agui/1`

`src/uar/api/adapters.rs:68` profile constant; stamped `:74`, `:389`;
`sse.rs:215, 233, 264, 279`. Official events: `RUN_STARTED` (`adapters.rs:87`),
`TEXT_MESSAGE_CONTENT` (`:95`), `REASONING_MESSAGE_CONTENT` (`:104`),
`TOOL_CALL_ARGS` (`:145`), `TOOL_CALL_END` (`:159`), `TOOL_CALL_RESULT` (`:172`),
`RUN_ERROR` (`:208`, `:223`), `RUN_FINISHED` (`:216`, `:361`), `STATE_DELTA`
(`:278`), `STATE_SNAPSHOT`/`MESSAGES_SNAPSHOT` (`sse.rs:264`, `279`). Legacy
dotted vocabulary retained in parallel (`sse.rs:345-444`), selected by
`agui_spec` flag (`sse.rs:295`, `310-328`). `NormalizedEvent` enum
`src/normalized.rs:50-221`. Frontend `frontend/src/platform/agui/*`. Spec
`ag-ui-chat-conformance` (4 reqs) has placeholder Purpose (`spec.md:4`). No
`@ag-ui/*` npm dependency; conformance is self-hosted.

### A2A — JSON-RPC RC v1.0; gRPC v0.3

`api/a2a/mod.rs:1`; methods `message/send`, `tasks/get`, `tasks/cancel`
(`handler.rs:76-85`); mounted `POST /a2a/compiler`, `GET /.well-known/agent.json`
(`mod.rs:61-68`). gRPC `api/a2a/grpc.rs` (559 lines) behind `a2a-transport`
(`mod.rs:24-25`; `Cargo.toml:202`); not in `default = ["minimal"]`
(`Cargo.toml:136-139`), only via `server-full` (`:140`). tonic 0.14. Both
transports front the compiler, not the agent runtime. Registries in-memory,
Surreal, Postgres (`mod.rs:48-53`).

### MCP-UI / MCP Apps — absent

Grep for `mcp-ui|mcp_ui|mcpUi|mcp_apps|mcp-apps|"ui://` across `src/`,
`frontend/`, `openspec/`, `Cargo.toml`: zero. `rmcp = "=3.1.2"` (`Cargo.toml:286`).

### `select-and-observe-presentations`

Not an OpenSpec change. Tracked only in the agui child `progress.json`:
`IN_PROGRESS`, `tasks_total: 0`, change 3 of 3; siblings
`establish-presentation-catalog-workspace` and `scope-presentation-capabilities`
DONE. The agui child directory holds only `progress.json`, `model-routing.log`,
`children/`: no goals, plan, assessment, or tasks.

## 4. Prior harness work

`uar-harness-parity` (2026-06-02): parity framing vs Mastra/VoltAgent/LangGraph.
Assessment overturned seed assumptions (observability half-built; cancellation
P0; resumable streaming frontend-only; evals+guardrails the real gap). Explicit
kill decision: do not wire the parking-lot `HookBus`; deliver `RuntimeStep` on
`NormalizedEvent`, not a second bus. Outcome 5 MET/1 PARTIAL/2 NOT MET; H3
`emit-runtime-step-events` skipped by execution drift. Carried debt: sandbox +
MCP-status recorders dead; guardrails detect-only; `tool_requires_approval`
keyword heuristic; config-liveness snapshots; in-repo `ModelCost` catalog.

`uar-next-harness` (2026-07-04): mandate to implement every `docs/uar-next.md`
recommendation. G1-G3 MET (a2a-grpc-enable, postgres-credential-store,
provider-health-failover, prompt-dialect-engine, capability-registry-benchmarks,
rag-hardening in-process, per-model-context-strategy, cost-budgets-backend,
skill-activation-metrics, skill-pack-bundling, librefang-a2a-agui-bridge,
agui-spec-alignment, cost-dashboard, model-comparison-dashboard). G4/G5 NOT MET,
carried to `uar-spec-v2-and-polish`. Cedar `is_tool_allowed` confirmed called
from the orchestrator gate. Decisions not to contradict: durable cost roll-up
out of scope; per-agent budget config surface absent; CH-08 outcome correlation
scope-cut; artifact-refiner QA gate unautomated 4+ phases. Lessons: verify
"committed" against `git log` and call-site grep (dead-code pattern); check
`git submodule status`.

## 5. versions.toml

28 lines of pins plus empty `[decisions]`. Header: authoritative, agents must
not edit. Pins with rationale: `vitest 4.1.10`, `zod 3.25.76` (A2UI catalog
boundary), `loro_crdt 1.13.9`, `liter_llm 1.18.2` (+commit), `surreal_memory_server`
commits, `prometheus_skill_system` commits, `surrealdb 3.2.4` lockstep
(SDK/types/server/image digest), `dompurify 3.4.14`, `js_yaml 3.15.1/4.3.1`,
`nanoid 3.3.18`.

Not recorded in versions.toml but pinned in Cargo.toml: `rmcp =3.1.2` (`:286`),
`tiktoken-rs 0.12.0` (`:378`), `tokenizers 0.23` optional (`:377`), `jsonschema
0.49.4` (`:418`), `wasmtime 47` / `wasmtime-wasi 47` optional behind
`wasm-runtime` (`:270-271`, in `server-full` `:140`), `sqlx 0.8.6` (`:382`),
`pgvector =0.4.2` (`:394`), tonic 0.14 (`:241-242`). LLM access funnels through
liter-llm (`Cargo.toml:489`, `features = ["full"]`; `.claude/rules/rust.md`)
with a native Anthropic driver alongside (`anthropic-native-driver` 89/94).
A2A/AG-UI protocol versions live only as Rust constants and spec text.

## 6. Defects observed

1. Checkpoint resume discards state (`routes.rs:346`, `:371-386`, `:388-395`; `restore_state` zero callers).
2. `LlmNode` sends tools it cannot execute (`llm_node.rs:98`, `:125`, `:132-143`).
3. `AgentNode` local delegation is a bare two-message call (`agent_node.rs:99-118`); sub-agent ids have no artifacts (`defaults.rs:116-117`, `:135`).
4. Graph runs bypass cost accounting (`manager.rs:2054` vs `:2454`).
5. `BudgetStatus::Exceeded` never stops anything (`cost_budget.rs:130-139`).
6. Actor system orphaned from runtime (`server.rs:1098`; `system.rs:221-227`).
7. No depth/concurrency limit on delegation (`system.rs:99-103`; `engine.rs:15` `MAX_GRAPH_ITERATIONS = 1_000`).
8. A2A children not cancelled (`manager.rs:1975-1988`; `client.rs:121`).
9. `/readyz` reports SurrealDB healthy without probing (`server.rs:2299-2303`).
10. Graph `RuntimeStep` events fabricated after the fact (`manager.rs:1993-2009`).
11. `select-and-observe-presentations` has no written plan.
12. `ag-ui-chat-conformance` spec Purpose placeholder (`spec.md:4`).
13. Dual AG-UI vocabularies with no deprecation path (`sse.rs:295`, `345-444`).
14. Actor API unauthenticated relative to run APIs (`actors.rs`, `:50`).
