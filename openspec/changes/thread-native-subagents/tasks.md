# Tasks — thread-native-subagents

scope: src/uar/runtime/thread/**, src/uar/runtime/actor/**, src/uar/runtime/graph/**, src/uar/api/a2a/**, src/uar/api/actors.rs, src/uar/persistence/**, src/uar/domain/events.rs, src/uar/api/adapters.rs, src/uar/api/sse.rs, src/uar/runtime/cost_budget.rs, src/uar/runtime/manager.rs (graph branch, budget), src/uar/runtime/native_skills/agents/**, tests/agent_threads.rs, tests/agent_policy_intersection.rs, tests/a2a_thread_service.rs

## 1. Failing tests first

- [ ] 1.1 `tests/agent_threads.rs`: a spawned child persists an `AgentThread` and `AgentEdge` in memory, SurrealDB, and PostgreSQL providers with identical ordering on list
- [ ] 1.2 A child spawned with `HistoryForkMode::None` sees only its delegated prompt; `LastTurns(2)` sees two turns and no tool outputs
- [ ] 1.3 `tests/agent_policy_intersection.rs`: a child artifact granting a tool the parent policy denies yields a child policy without it; an unsupported policy shape fails closed
- [ ] 1.4 A child cannot cause an approval to be granted; approval requests bubble to the root run's approval channel
- [ ] 1.5 The fifth concurrent child, the fourth nesting level, and the seventeenth total child are refused with typed limit errors
- [ ] 1.6 Root cancellation cancels every child; a remote A2A child receives `tasks/cancel`
- [ ] 1.7 Child model calls are recorded against the root run's budget; an exceeded budget refuses new spawns and new model calls
- [ ] 1.8 `send_agent_message` delivers a typed message whose sender identity is metadata; the child's user turn contains no prepended identity text
- [ ] 1.9 `tests/a2a_thread_service.rs`: inbound `message/send` starts a run on the named agent artifact; `tasks/get` reflects thread status; `tasks/cancel` cancels the thread; wire shapes unchanged
- [ ] 1.10 Lifecycle events carry parent id, child id, canonical path, status, and terminal outcome and no prompt text; AG-UI subagent events are emitted from them
- [ ] 1.11 Actor endpoints without user context return 401

## 2. Kernel

- [ ] 2.1 Add `thread/mod.rs` (`AgentThread`, `AgentEdge`, `AgentHandle`), `thread/spawn.rs` (`AgentSpawnRequest`, `HistoryForkMode`, fork filtering that keeps only final assistant messages and user turns), `thread/messages.rs` (`InterAgentMessage`), `thread/limits.rs`
- [ ] 2.2 Add `thread/policy_intersection.rs`: intersect parent and child artifact policies for skills, MCP servers, tools, credentials, sandbox, budgets; fail closed
- [ ] 2.3 Persistence trait methods and three provider implementations with stable ordering

## 3. Tools and events

- [ ] 3.1 Descriptors for `spawn_agent`, `send_agent_message`, `wait_agents`, `list_agents`, `interrupt_agent`; spawn description states that spawning requires explicit user or artifact authorization
- [ ] 3.2 Lifecycle event variants; AG-UI subagent mapping; live `RuntimeStep` for graph runs

## 4. Adapters

- [ ] 4.1 `AgentActor` as a mailbox over the kernel; actor API requires user context
- [ ] 4.2 `AgentNode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix
- [ ] 4.3 A2A handler maps onto the thread service; client propagates cancellation

## 5. Budget and cancellation

- [ ] 5.1 Record child usage against the root run; enforce `Exceeded` on spawn and model call
- [ ] 5.2 Child cancellation tokens from the run token; A2A cancel on parent cancel

## 6. Verification

- [ ] 6.1 Tier 1: the three new test targets
- [ ] 6.2 Tier 2: fmt check and full test run
- [ ] 6.3 Local multi-agent cancellation smoke with a real model (record command and output)
- [ ] 6.4 `openspec validate thread-native-subagents --strict`
