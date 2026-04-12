# Architecture Evolution Design — Tool Sandboxing, Graph Orchestration, Multi-Agent, Checkpoints, Context Management

**Date:** 2026-04-12
**Status:** Approved
**Scope:** Rust backend core systems, admin UI improvements, integration testing

---

## 1. Tool Sandboxing via Microsandbox 0.3.12

### Problem
MCP tools execute with full system access. No isolation boundary.

### Solution
Complete the existing microsandbox_runner.rs stub by wiring in microsandbox 0.3.12 APIs.

### Implementation
- Replace `tokio::process::Command` in `execute()` with `sandbox.shell(command)` / `sandbox.exec(ExecOptions)`
- Replace `tokio::fs` in `write_file()`/`read_file()` with `sandbox.write_file()`/`sandbox.read_file()`
- Replace stub `destroy()` with `sandbox.stop_and_wait()`
- Use `ExecOptions` for timeout, env, cwd control
- Network policy: `NetworkPolicy::none()` by default (airgapped), configurable per agent
- Secret injection via `secret_env()` for API keys needed by tools

### Tool execution routing
```
Tool Call -> Governance Gate -> Is sandboxed?
  YES -> microsandbox.exec(command) -> ExecOutput -> ToolResult
  NO  -> Direct execution (current path)
```

### Configuration
Per-agent `tool_execution` policy field: `"sandboxed" | "direct" | "auto"`.

---

## 2. Graph-Based Orchestration Layer

### Problem
Single-agent, flat tool loop. No graph-based routing, conditional branching, or parallel execution.

### Solution
Build `AgentGraph` module using petgraph for directed graph execution with typed state.

### Architecture

```rust
pub struct AgentGraph {
    nodes: HashMap<String, Box<dyn GraphNode>>,
    edges: Vec<GraphEdge>,
    state: GraphState,
    checkpoints: Vec<Checkpoint>,
}

pub enum GraphEdge {
    Direct { from: String, to: String },
    Conditional { from: String, condition: Box<dyn Fn(&GraphState) -> String> },
}

pub trait GraphNode: Send + Sync {
    async fn execute(&self, state: &mut GraphState, ctx: &GraphContext) -> NodeResult;
    fn id(&self) -> &str;
}
```

### Built-in node types
- `LlmNode` — calls the orchestrator's LLM driver
- `ToolNode` — executes a specific tool via MCP registry
- `RouterNode` — LLM-based routing to select next node
- `AgentNode` — delegates to another agent (local or A2A)
- `CheckpointNode` — saves state to persistence layer

### Integration
- `RunManager::start_run` gains optional `graph: Option<AgentGraph>`
- With graph: execution follows graph nodes
- Without graph: current tool loop preserved (backward compatible)
- The simple tool loop is a degenerate graph: LlmNode -> ToolNode -> LlmNode (loop)

---

## 3. State Checkpointing via Event Sourcing

### Problem
No state persistence between tool loop iterations. Long runs restart from scratch on failure.

### Solution
Checkpoint state after each significant step. Enable resume from any checkpoint.

### Data model

```rust
pub struct Checkpoint {
    id: String,
    run_id: String,
    thread_id: String,
    node_id: String,
    iteration: u32,
    state: serde_json::Value,
    messages: Vec<serde_json::Value>,
    pending_tool_calls: Vec<ToolCall>,
    created_at: String,
}
```

### When checkpoints are created
1. After each tool loop iteration (before next LLM call)
2. After each graph node execution
3. Before human-in-the-loop approval gates
4. On error (for debugging/replay)

### Persistence
New methods on PersistenceLayer: `save_checkpoint`, `load_checkpoint`, `list_checkpoints`.

### API
```
GET  /api/uar/runs/{run_id}/checkpoints
POST /api/uar/runs/{run_id}/resume
POST /api/uar/runs/{run_id}/resume/{checkpoint_id}
```

### A2A integration
InputRequired state saves checkpoint. Resume on next `message/send`.

---

## 4. Multi-Agent Orchestration via A2A

### Problem
Single agent per request. No delegation to other agents.

### Solution
Internal multi-agent via nested runs. External via A2A JSON-RPC 2.0 protocol.

### Architecture
```
Agent A (graph execution)
  -> AgentNode("agent-b")
    -> Is agent-b local?
      YES -> RunManager.start_nested_run(agent_b, input, parent_run_id)
      NO  -> A2AClient.send_message(remote_url, input)
```

### New components
- `A2AClient` — HTTP client speaking A2A JSON-RPC 2.0
- `AgentNode` in graph — resolves by ID (local) or URL (remote)
- Extend `AgentRegistry` with `resolve(id_or_url)` -> `Local(AgentArtifact) | Remote(url)`

### Context passing
- Parent passes relevant context (not full history) to child
- Child result injected as tool result into parent's messages
- Parent checkpoint includes child run reference

---

## 5. Context Management (Cherry Studio-Inspired)

### Problem
No context window management. Long conversations hit token limits.

### Solution
Port Cherry Studio's 5-strategy system to Rust.

### Strategies

| Strategy | Behavior |
|----------|----------|
| `none` | Send all messages |
| `sliding_window` | Keep last N messages (default) |
| `summarize` | LLM-summarize older messages |
| `truncate_middle` | Keep first K + last M, drop middle |
| `hierarchical` | 3-tier: recent + summaries + facts |

### Implementation

```rust
pub enum ContextStrategy {
    None,
    SlidingWindow { max_messages: usize },
    Summarize {
        threshold: usize,
        summary_max_tokens: usize,
        model: Option<String>,
    },
    TruncateMiddle {
        keep_first: usize,
        keep_last: usize,
    },
    Hierarchical {
        short_term_turns: usize,
        mid_term_summary_tokens: usize,
        long_term_facts_tokens: usize,
    },
}
```

### Where it plugs in
Between message history load and LLM request in api_chat_completion:
1. Load full message history
2. Apply context strategy -> filtered/compressed messages
3. Prepend system prompt + memory context
4. Send to orchestrator

### Token estimation
Quick: `ceil(text.len() / 4)`. Accurate: tiktoken-rs (already a dependency).

### Configuration hierarchy
1. Global default (config.yaml)
2. Agent override (AgentArtifact.context_strategy)
3. Session override (AgentSession.context_strategy)

---

## 6. Skill-Level Provider/Model Override

### Problem
Skills can't specify preferred models. Summarization skills waste expensive tokens.

### Solution
Add `SkillExecutionConfig` with optional provider/model preference.

```rust
pub struct SkillExecutionConfig {
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub max_tokens: Option<usize>,
}
```

### Resolution chain (most specific wins)
1. Session override
2. Active skill preference
3. Agent default model
4. Global default model

### Frontend
Skill editor gains optional ModelSelector dropdown per skill.

---

## 7. Admin UI Improvements

### 7a. Provider/Model Clarity
- Persistent banner showing current default model
- "Set Default Model" dropdown within provider detail
- Star icon on default model

### 7b. Agent Editor Guardrails
- Warning badge if no model set: "Will use global default"
- Error on save if no model AND no global default
- Warning icon on agents without model in list

### 7c. Chat Failure Prevention
- No-provider guard (already done)
- No-model guard: validate before starting run, show inline error with CTA
- Model label next to agent in dropdown (already done)

### 7d. Onboarding Enhancement
- "Configure provider" step incomplete until default model is also set

---

## 8. Integration Testing Strategy

### 8a. Rust Integration Tests

| Test Suite | Validates |
|-----------|-----------|
| test_chat_completion | Full chat POST -> streamed response (mocked LLM) |
| test_provider_resolution | Registration -> resolution -> correct key/URL |
| test_tool_execution | Tool call -> MCP -> result -> injected into LLM |
| test_agent_crud | Create/read/update/delete via API |
| test_session_management | Session -> agent config -> effective config merge |
| test_context_strategy | All 5 strategies applied correctly |
| test_checkpoint_lifecycle | Save -> list -> resume |
| test_skill_import | Parse SKILL.md -> validate -> persist |
| test_mcp_json_optional | Server starts without mcp.json |
| test_knowledge_base_crud | KB create -> doc upload -> search -> delete |

### Mocking
- `MockLlmDriver`: canned responses for deterministic tests
- `MockMcpServer`: responds to known tool calls
- Real SurrealDB `mem://` for persistence

### 8b. Frontend E2E Tests (Playwright)

| Test | Validates |
|------|-----------|
| admin-providers.spec.ts | Search, configure, set default |
| admin-agents.spec.ts | Create with model, edit, delete |
| admin-tools.spec.ts | Click tool, fill form, execute |
| admin-skills.spec.ts | Create, import, toggle |
| admin-knowledge.spec.ts | Create KB, upload doc |
| chat-basic.spec.ts | Select agent, send message, verify response |
| chat-agent-selection.spec.ts | Switch agents, verify model changes |
| chat-no-provider.spec.ts | Guard -> CTA -> navigate |
| chat-session-config.spec.ts | Open config, change model, save |

### Playwright setup
- `bun add -D @playwright/test` in frontend
- Tests against live server with mem:// SurrealDB + mock LLM
- `globalSetup`: start server, `globalTeardown`: kill server
