# thread-native-subagents

Rank 8 of the codex-harness-comparative-analysis change set. Source: gap G8 and the lifecycle-event items of G11 in the phase `analysis.md`.

## Why

Three sub-agent paths share no kernel. Actors build their own `Orchestrator` from the server-wide config with no artifact, policy, Cedar, budget, cancellation, persistence, or events (`src/uar/runtime/actor/agent_actor.rs:115`; `src/uar/runtime/actor/system.rs:54-63`). Local graph nodes make a two-message call with `tools: Vec::new()` (`src/uar/runtime/graph/nodes/agent_node.rs:99-118`). Remote A2A nodes send bare text with no auth, timeout, or cancellation (`agent_node.rs:200-235`), and UAR's inbound A2A endpoint is bound to the compiler so it cannot run an agent (`src/uar/api/a2a/handler.rs:41-47`). Inter-agent identity is text prepended to the user message (`agent_actor.rs:201-202`). There is no parent/child persistence, no depth or concurrency limit, graph runs bypass cost accounting (`src/uar/runtime/manager.rs:2054` versus `:2454`), `BudgetStatus::Exceeded` never stops anything (`src/uar/runtime/cost_budget.rs:130-139`), and the actor REST API takes no user context (`src/uar/api/actors.rs:22-50`). The `multi-agent-orchestration` spec is met only for one hardwired agent id with fabricated step events (`manager.rs:1864-1868`, `:1993-2009`).

Codex runs every child as a real thread through the same kernel, forces runtime policy from the live turn and re-applies it after roles (`core/src/tools/handlers/multi_agents_common.rs:235-262`; `multi_agents_v2/spawn.rs:145`), lets roles only narrow (`core/src/agent/role.rs:1-4`), intersects permissions fail-closed (`protocol/src/permission_profile_intersection.rs:31-68`), reads user authorization from the root thread only (`core/src/agent/control/user_authorization.rs:22-94`), types inter-agent messages (`protocol/src/protocol.rs:803-819`), persists the spawn graph (`agent-graph-store/src/store.rs:17-59`), and caps children, running turns, and depth. These are provider-neutral safety properties and are adopted as requirements. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- Durable `AgentThread` and `AgentEdge` records (owner, root, parent, canonical path, artifact id, status, history revision, timestamps) on `PersistenceLayer` for memory, SurrealDB, and PostgreSQL, with stable ordering.
- Internal contracts: `AgentSpawnRequest`, `HistoryForkMode::{None, Full, LastTurns(u32)}`, `AgentHandle`, typed `InterAgentMessage` with `trigger_turn`. Default fork mode is `None`; the delegated prompt and child artifact are explicit.
- Child effective policy is the intersection of the parent policy and the child artifact policy; skills, MCP servers, tools, credentials, sandbox permissions, and budgets can only narrow; intersection fails closed on unsupported shapes. Children never widen approval; user authorization is read from the root run.
- Shared across the tree: root cancellation (A2A children receive `tasks/cancel`), cost and token budgets (exceeded budget stops new spawns and new model calls), limits (four concurrent children, depth three, sixteen total per root run), lifecycle subscriptions.
- Model tools through the descriptor registry: `spawn_agent`, `send_agent_message`, `wait_agents`, `list_agents`, `interrupt_agent`.
- Adapters: `AgentActor` becomes a mailbox over the kernel; `AgentNode` spawns and waits on a real child thread; A2A task, status, cancel, and result map onto the thread service with the wire contract unchanged; the actor API requires user context.
- Additive lifecycle events with parent and child ids, canonical path, status, and terminal outcome, never child prompts or hidden reasoning; live `RuntimeStep` for graph runs replaces post-hoc replay; AG-UI `SubagentStarted`, `SubagentFinished`, `SubagentError` emitted from the same events.

## Scope

- `src/uar/runtime/actor/**`, `src/uar/runtime/graph/**`
- `src/uar/api/a2a/{handler.rs,client.rs,mod.rs}`, `src/uar/api/actors.rs`
- `src/uar/persistence/{mod.rs,providers/memory.rs,providers/surreal.rs,providers/postgres.rs}`
- `src/uar/domain/events.rs`, `src/uar/api/{adapters.rs,sse.rs}`
- `src/uar/runtime/{cost_budget.rs,manager.rs}` (graph branch and budget recording)
- new: `src/uar/runtime/thread/{mod.rs,spawn.rs,policy_intersection.rs,messages.rs,limits.rs}`, `src/uar/runtime/native_skills/agents/*.rs`
- tests: `tests/agent_threads.rs`, `tests/agent_policy_intersection.rs`, `tests/a2a_thread_service.rs`

Out of scope: A2A v1.0.1 wire alignment (deferred, trigger is this change), guardian-style model reviewer, agent identity against an external backend.

## Dependencies

typed-turn-assembly (children are turns), fail-closed-tool-arguments (agent tools are descriptors), model-path-resiliency (cancellation and budget seams).

## Verification

Tier 0 per edit; Tier 1 the three new test targets; Tier 2 at the boundary; local multi-agent cancellation smoke with a real model recorded with output.

## The uncomfortable thing

Making actors, graph nodes, and A2A adapters over one kernel deletes behavior some clients may depend on, including the `[route]` text prefix on graph output and unauthenticated actor endpoints. Both are defects by the spec's own standard, and the change says so instead of preserving them.
