# Architecture Evolution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the architecture evolution: tool sandboxing via microsandbox 0.3.12, graph-based orchestration, state checkpointing, multi-agent A2A delegation, Cherry Studio context management, skill-level model overrides, admin UI guardrails, and 100% integration test coverage.

**Architecture:** Event-sourced graph orchestration layer built on petgraph, with microsandbox VM isolation for tool execution, checkpoint persistence via SurrealDB/Postgres, 5 context management strategies, and Playwright E2E tests guaranteeing all chains work end-to-end.

**Tech Stack:** Rust (Axum, petgraph, microsandbox 0.3.12, tokio, serde), React 18 (Playwright E2E), SurrealDB/Postgres persistence, tiktoken-rs for token counting

**Design doc:** `docs/plans/2026-04-12-architecture-evolution-design.md`

---

## Phase 0: Test Infrastructure Foundation

Before building features, set up the test infrastructure that guarantees correctness.

### Task 0.1: Create MockLlmDriver for Integration Tests

**Files:**
- Create: `src/llm/mock_driver.rs`
- Modify: `src/llm/mod.rs` (add `pub mod mock_driver;`)

**Step 1: Create the mock driver**

```rust
// src/llm/mock_driver.rs
use crate::llm::{LlmDriver, LlmRequest};
use crate::normalized::NormalizedEvent;
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Canned responses for deterministic testing.
#[derive(Clone)]
pub struct MockLlmDriver {
    responses: Arc<Mutex<Vec<Vec<NormalizedEvent>>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLlmDriver {
    pub fn new(responses: Vec<Vec<NormalizedEvent>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a mock that always returns a simple text response.
    pub fn echo() -> Self {
        Self::new(vec![vec![
            NormalizedEvent::MessageDelta { text: "Hello from mock!".to_string() },
            NormalizedEvent::Done,
        ]])
    }

    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl std::fmt::Debug for MockLlmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockLlmDriver").finish()
    }
}

#[async_trait]
impl LlmDriver for MockLlmDriver {
    async fn stream(
        &self,
        _req: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        let mut count = self.call_count.lock().unwrap();
        let idx = *count;
        *count += 1;
        drop(count);

        let responses = self.responses.lock().unwrap();
        let events = responses.get(idx % responses.len()).cloned().unwrap_or_default();

        let stream = async_stream::stream! {
            for event in events {
                yield Ok(event);
            }
        };

        Ok(Box::pin(stream))
    }
}
```

**Step 2: Register module**

Add `pub mod mock_driver;` to `src/llm/mod.rs`.

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add src/llm/mock_driver.rs src/llm/mod.rs
git commit -m "feat: add MockLlmDriver for deterministic integration tests"
```

---

### Task 0.2: Create Integration Test Harness

**Files:**
- Create: `tests/common/mod.rs`
- Create: `tests/common/test_server.rs`

**Step 1: Create test harness that starts the server with in-memory SurrealDB**

```rust
// tests/common/mod.rs
pub mod test_server;

// tests/common/test_server.rs
use std::net::TcpListener;

/// Find an available port for test server.
pub fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Build a test AppConfig with in-memory SurrealDB.
pub fn test_config(port: u16) -> universal_agent_runtime::config::AppConfig {
    let mut config = universal_agent_runtime::config::AppConfig::default();
    config.server.port = port;
    config.persistence.provider = "surreal".to_string();
    config.persistence.database_url = "mem://test".to_string();
    config.llm.model = "openai/gpt-4o".to_string();
    config.llm.api_key = Some("test-key".to_string());
    config.llm.base_url = Some("http://127.0.0.1:0".to_string()); // won't be called with mock
    config
}
```

**Step 2: Verify**

Run: `cargo test --test common` (should compile even if no tests yet)

**Step 3: Commit**

```bash
git add tests/common/
git commit -m "feat: add integration test harness with in-memory SurrealDB"
```

---

### Task 0.3: Install Playwright for Frontend E2E Tests

**Files:**
- Modify: `frontend/package.json`
- Create: `frontend/playwright.config.ts`
- Create: `frontend/e2e/smoke.spec.ts`

**Step 1: Install Playwright**

```bash
cd frontend && bun add -D @playwright/test && npx playwright install chromium
```

**Step 2: Create Playwright config**

```typescript
// frontend/playwright.config.ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL: "http://localhost:3002",
    headless: true,
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "cargo run --bin universal-agent-runtime",
    url: "http://localhost:3002/healthz",
    reuseExistingServer: true,
    timeout: 120_000,
    cwd: "..",
  },
});
```

**Step 3: Create smoke test**

```typescript
// frontend/e2e/smoke.spec.ts
import { test, expect } from "@playwright/test";

test("admin page loads", async ({ page }) => {
  await page.goto("/admin");
  await expect(page.locator("text=Providers")).toBeVisible();
});

test("chat page loads", async ({ page }) => {
  await page.goto("/threads");
  await expect(page.locator("text=New thread")).toBeVisible();
});
```

**Step 4: Verify**

Run: `cd frontend && npx playwright test --reporter=list` (against running server)

**Step 5: Commit**

```bash
git add frontend/package.json frontend/playwright.config.ts frontend/e2e/
git commit -m "feat: add Playwright E2E test infrastructure with smoke tests"
```

---

## Phase 1: Context Management (Cherry Studio-Inspired)

This is the highest-impact improvement for making chat actually usable with long conversations.

### Task 1.1: Implement Context Strategy Types

**Files:**
- Create: `src/uar/context/mod.rs`
- Create: `src/uar/context/strategy.rs`
- Modify: `src/uar/mod.rs` (add `pub mod context;`)

**Step 1: Define context strategy types**

```rust
// src/uar/context/mod.rs
pub mod strategy;
pub use strategy::*;

// src/uar/context/strategy.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextStrategy {
    None,
    SlidingWindow {
        #[serde(default = "default_max_messages")]
        max_messages: usize,
    },
    Summarize {
        #[serde(default = "default_summarize_threshold")]
        threshold: usize,
        #[serde(default = "default_summary_max_tokens")]
        summary_max_tokens: usize,
        model: Option<String>,
    },
    TruncateMiddle {
        #[serde(default = "default_keep_first")]
        keep_first: usize,
        #[serde(default = "default_keep_last")]
        keep_last: usize,
    },
    Hierarchical {
        #[serde(default = "default_short_term_turns")]
        short_term_turns: usize,
        #[serde(default = "default_mid_term_tokens")]
        mid_term_summary_tokens: usize,
        #[serde(default = "default_long_term_tokens")]
        long_term_facts_tokens: usize,
    },
}

fn default_max_messages() -> usize { 20 }
fn default_summarize_threshold() -> usize { 6 }
fn default_summary_max_tokens() -> usize { 500 }
fn default_keep_first() -> usize { 2 }
fn default_keep_last() -> usize { 4 }
fn default_short_term_turns() -> usize { 5 }
fn default_mid_term_tokens() -> usize { 2000 }
fn default_long_term_tokens() -> usize { 500 }

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::SlidingWindow { max_messages: default_max_messages() }
    }
}

/// Quick token estimation: ~4 chars per token.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Apply context strategy to a message list.
/// Returns the filtered message list.
pub fn apply_strategy(
    messages: &[serde_json::Value],
    strategy: &ContextStrategy,
) -> Vec<serde_json::Value> {
    match strategy {
        ContextStrategy::None => messages.to_vec(),

        ContextStrategy::SlidingWindow { max_messages } => {
            if messages.len() <= *max_messages {
                messages.to_vec()
            } else {
                messages[messages.len() - max_messages..].to_vec()
            }
        }

        ContextStrategy::TruncateMiddle { keep_first, keep_last } => {
            let total_keep = keep_first + keep_last;
            if messages.len() <= total_keep {
                messages.to_vec()
            } else {
                let mut result = messages[..*keep_first].to_vec();
                result.extend_from_slice(&messages[messages.len() - keep_last..]);
                result
            }
        }

        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => {
            // These require LLM calls for summarization.
            // For V1, fall back to sliding window with generous limit.
            let max = 50;
            if messages.len() <= max {
                messages.to_vec()
            } else {
                messages[messages.len() - max..].to_vec()
            }
        }
    }
}
```

**Step 2: Register module in `src/uar/mod.rs`**

**Step 3: Write unit test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sliding_window_trims() {
        let msgs: Vec<serde_json::Value> = (0..10)
            .map(|i| serde_json::json!({"role": "user", "content": format!("msg-{i}")}))
            .collect();
        let result = apply_strategy(&msgs, &ContextStrategy::SlidingWindow { max_messages: 3 });
        assert_eq!(result.len(), 3);
        assert_eq!(result[0]["content"], "msg-7");
    }

    #[test]
    fn truncate_middle_keeps_ends() {
        let msgs: Vec<serde_json::Value> = (0..8)
            .map(|i| serde_json::json!({"role": "user", "content": format!("msg-{i}")}))
            .collect();
        let result = apply_strategy(&msgs, &ContextStrategy::TruncateMiddle { keep_first: 2, keep_last: 3 });
        assert_eq!(result.len(), 5);
        assert_eq!(result[0]["content"], "msg-0");
        assert_eq!(result[1]["content"], "msg-1");
        assert_eq!(result[2]["content"], "msg-5");
    }
}
```

**Step 4: Verify**

Run: `cargo test context::strategy`

**Step 5: Commit**

```bash
git add src/uar/context/ src/uar/mod.rs
git commit -m "feat: context management strategies (none, sliding_window, truncate_middle, summarize, hierarchical)"
```

---

### Task 1.2: Wire Context Strategy into Chat Completion

**Files:**
- Modify: `src/server.rs` — `api_chat_completion` function

**Step 1: Import and apply context strategy**

In `api_chat_completion`, after message history is loaded but before it's passed to the orchestrator, apply the context strategy. The strategy comes from:
1. Agent's context_strategy field (if set in extensions)
2. Global default from config
3. Fallback to SlidingWindow { max_messages: 20 }

Add the strategy application between the memory context injection and the `run_manager.start_run()` call. The messages need to be filtered before constructing the run.

**Step 2: Add `context_strategy` to AppConfig**

In `src/config.rs`, add a new field to `AppConfig`:
```rust
pub context_strategy: ContextStrategy,
```
Default: `ContextStrategy::SlidingWindow { max_messages: 20 }`

**Step 3: Integration test**

Create `tests/test_context_strategy.rs`:
```rust
// Verify that sending 30 messages with sliding_window(10) results in only 10 messages reaching the LLM
```

**Step 4: Commit**

```bash
git add src/server.rs src/config.rs tests/test_context_strategy.rs
git commit -m "feat: wire context strategy into chat completion pipeline"
```

---

## Phase 2: Tool Sandboxing via Microsandbox

### Task 2.1: Complete Microsandbox Runner Implementation

**Files:**
- Modify: `src/sandbox/microsandbox_runner.rs`

**Step 1: Replace placeholder execute() with real microsandbox API**

Replace `tokio::process::Command` in `execute()` with `sandbox.exec()`. Replace `tokio::fs` in `write_file()`/`read_file()` with `sandbox.write_file()`/`sandbox.read_file()`. Replace stub `destroy()` with `sandbox.stop_and_wait()`.

Use microsandbox 0.3.12 API:
```rust
let output = self.sandbox.exec(
    ExecOptions::builder()
        .args(&command_parts)
        .timeout(Duration::from_secs(self.config.timeout_secs))
        .build()
).await?;
```

**Step 2: Add network policy configuration**

Default to `NetworkPolicy::none()` (airgapped). Allow per-agent `tool_execution` policy to open network.

**Step 3: Integration test**

Create `tests/test_sandbox.rs` (only runs when microsandbox feature is enabled):
```rust
#[cfg(feature = "sandbox-microsandbox")]
#[tokio::test]
async fn test_sandbox_executes_command() {
    // Create sandbox, execute "echo hello", verify output
}
```

**Step 4: Commit**

```bash
git add src/sandbox/microsandbox_runner.rs tests/test_sandbox.rs
git commit -m "feat: complete microsandbox 0.3.12 tool execution integration"
```

---

### Task 2.2: Wire Sandbox into Tool Execution Path

**Files:**
- Modify: `src/llm/orchestrator.rs` — tool execution section
- Modify: `src/uar/domain/artifact.rs` — add `tool_execution` policy

**Step 1: Add tool_execution policy to AgentPolicy**

```rust
pub struct ToolPolicy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub max_concurrent: usize,
    #[serde(default)]
    pub execution_mode: ToolExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ToolExecutionMode {
    #[default]
    Direct,
    Sandboxed,
    Auto, // sandbox code execution tools, direct for data tools
}
```

**Step 2: In orchestrator tool execution, check policy and route**

**Step 3: Commit**

```bash
git add src/llm/orchestrator.rs src/uar/domain/artifact.rs
git commit -m "feat: wire sandbox into tool execution with per-agent policy"
```

---

## Phase 3: Graph-Based Orchestration

### Task 3.1: Create Graph Core Types

**Files:**
- Create: `src/uar/runtime/graph/mod.rs`
- Create: `src/uar/runtime/graph/types.rs`
- Create: `src/uar/runtime/graph/engine.rs`
- Modify: `src/uar/runtime/mod.rs`

**Step 1: Define GraphState, GraphNode trait, GraphEdge**

```rust
// src/uar/runtime/graph/types.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphState {
    pub data: HashMap<String, serde_json::Value>,
    pub messages: Vec<serde_json::Value>,
    pub iteration: u32,
}

impl GraphState {
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), v);
        }
    }
}

pub struct GraphContext {
    pub run_id: String,
    pub session_id: Option<String>,
    pub mcp: std::sync::Arc<crate::mcp::McpRegistry>,
    pub llm_config: crate::config::LlmConfig,
}

pub enum NodeResult {
    Continue(GraphState),
    Finished(GraphState),
    Error(String),
}

#[async_trait]
pub trait GraphNode: Send + Sync {
    fn id(&self) -> &str;
    async fn execute(&self, state: &mut GraphState, ctx: &GraphContext) -> NodeResult;
}

pub enum GraphEdge {
    Direct { from: String, to: String },
    Conditional {
        from: String,
        condition: Box<dyn Fn(&GraphState) -> String + Send + Sync>,
    },
}
```

**Step 2: Implement AgentGraph**

```rust
// src/uar/runtime/graph/engine.rs
pub struct AgentGraph {
    nodes: HashMap<String, Box<dyn GraphNode>>,
    edges: Vec<GraphEdge>,
    entry_node: String,
}

impl AgentGraph {
    pub fn builder(entry: &str) -> AgentGraphBuilder { ... }
    pub async fn execute(&self, initial_state: GraphState, ctx: &GraphContext) -> GraphState { ... }
}
```

The execution engine traverses the graph using petgraph, following edges based on conditions.

**Step 3: Unit tests for graph traversal**

**Step 4: Commit**

```bash
git add src/uar/runtime/graph/ src/uar/runtime/mod.rs
git commit -m "feat: graph orchestration core types and execution engine"
```

---

### Task 3.2: Implement Built-in Graph Nodes

**Files:**
- Create: `src/uar/runtime/graph/nodes/mod.rs`
- Create: `src/uar/runtime/graph/nodes/llm_node.rs`
- Create: `src/uar/runtime/graph/nodes/tool_node.rs`
- Create: `src/uar/runtime/graph/nodes/router_node.rs`
- Create: `src/uar/runtime/graph/nodes/checkpoint_node.rs`

**Step 1: LlmNode — calls the LLM driver**
**Step 2: ToolNode — executes a specific tool**
**Step 3: RouterNode — LLM-based conditional routing**
**Step 4: CheckpointNode — saves state to persistence**
**Step 5: Tests for each node type**
**Step 6: Commit**

---

### Task 3.3: Integrate Graph into RunManager

**Files:**
- Modify: `src/uar/runtime/manager.rs`

**Step 1: Add optional graph parameter to start_run**

When a graph is provided, execute it instead of the simple tool loop. When no graph is provided, preserve current behavior.

**Step 2: The default tool loop as a graph**

Create a helper that builds the standard tool loop as a degenerate graph for backward compatibility testing.

**Step 3: Integration test**

```rust
#[tokio::test]
async fn test_simple_graph_execution() {
    // Build a 2-node graph: LlmNode -> FinishNode
    // Execute with MockLlmDriver
    // Verify state flows through correctly
}
```

**Step 4: Commit**

---

## Phase 4: State Checkpointing

### Task 4.1: Add Checkpoint Persistence

**Files:**
- Create: `src/uar/runtime/checkpoint.rs`
- Modify: `src/uar/persistence/mod.rs`
- Modify: `src/uar/persistence/providers/surreal.rs`

**Step 1: Define Checkpoint struct and persistence trait methods**

```rust
pub struct Checkpoint {
    pub id: String,
    pub run_id: String,
    pub thread_id: String,
    pub node_id: String,
    pub iteration: u32,
    pub state: serde_json::Value,
    pub messages: Vec<serde_json::Value>,
    pub created_at: String,
}
```

Add to PersistenceLayer:
```rust
async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
async fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
async fn list_checkpoints(&self, run_id: &str) -> Result<Vec<Checkpoint>>;
```

**Step 2: Implement in SurrealDB provider**
**Step 3: Integration test for checkpoint save/load/list**
**Step 4: Commit**

---

### Task 4.2: Add Checkpoint API Endpoints

**Files:**
- Modify: `src/server.rs`

**Step 1: Add endpoints**
```
GET  /api/uar/runs/{run_id}/checkpoints
POST /api/uar/runs/{run_id}/resume
POST /api/uar/runs/{run_id}/resume/{checkpoint_id}
```

**Step 2: Integration test with curl**
**Step 3: Commit**

---

### Task 4.3: Wire Checkpoints into Orchestrator

**Files:**
- Modify: `src/llm/orchestrator.rs`

**Step 1: After each tool loop iteration, save checkpoint**

Pass a checkpoint callback into the orchestrator that saves state after each iteration. This doesn't break the existing flow — it's additive.

**Step 2: Integration test: verify checkpoints are created during tool loop**
**Step 3: Commit**

---

## Phase 5: Multi-Agent Orchestration

### Task 5.1: Create A2A Client

**Files:**
- Create: `src/uar/api/a2a/client.rs`

**Step 1: HTTP client for A2A JSON-RPC 2.0**

```rust
pub struct A2AClient {
    http: reqwest::Client,
}

impl A2AClient {
    pub async fn send_message(&self, url: &str, message: &str) -> Result<TaskResult> {
        // POST to url with JSON-RPC 2.0 message/send
    }
    pub async fn get_task(&self, url: &str, task_id: &str) -> Result<Task> {
        // POST to url with JSON-RPC 2.0 tasks/get
    }
}
```

**Step 2: Test with mock HTTP server**
**Step 3: Commit**

---

### Task 5.2: Create AgentNode for Graph Orchestration

**Files:**
- Create: `src/uar/runtime/graph/nodes/agent_node.rs`

**Step 1: AgentNode resolves local vs remote**

```rust
pub struct AgentNode {
    agent_id_or_url: String,
}

impl GraphNode for AgentNode {
    async fn execute(&self, state: &mut GraphState, ctx: &GraphContext) -> NodeResult {
        // Check: is this a local agent ID or remote URL?
        // Local: spawn nested run via RunManager
        // Remote: use A2AClient
    }
}
```

**Step 2: Integration test: local agent delegation**
**Step 3: Commit**

---

## Phase 6: Skill-Level Model Override

### Task 6.1: Add SkillExecutionConfig to Skill Domain

**Files:**
- Modify: `src/uar/domain/skills.rs`
- Modify: `src/uar/runtime/manager.rs`

**Step 1: Add config to Skill struct**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillExecutionConfig {
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub max_tokens: Option<usize>,
}
```

**Step 2: In RunManager, after skill matching, override model if skill specifies one**

**Step 3: Frontend: add ModelSelector to skill editor (agent-editor.tsx Capabilities tab)**
**Step 4: Commit**

---

## Phase 7: Admin UI Guardrails

### Task 7.1: Agent Editor Model Validation

**Files:**
- Modify: `frontend/src/admin/components/agent-editor.tsx`

**Step 1: Add warning badge when no model is set**

In the Identity tab, below the ModelSelector, show a warning if the field is empty AND no global default exists. On save, validate that either the agent has a model OR a global default exists.

**Step 2: Add warning icon to agent list for agents without models**

**Step 3: Commit**

---

### Task 7.2: Chat No-Model Guard

**Files:**
- Modify: `frontend/src/pages/chat-page.tsx`

**Step 1: Before starting a chat, verify model resolution will succeed**

Call a new lightweight endpoint `GET /api/uar/resolve-model` that returns the resolved model or an error. If error, show inline message with CTA instead of streaming a broken error.

**Step 2: Backend endpoint**

```rust
// GET /api/uar/resolve-model — returns { provider_id, model_id } or error
```

**Step 3: Commit**

---

### Task 7.3: Onboarding Default Model Check

**Files:**
- Modify: frontend onboarding component

**Step 1: "Configure provider" step stays incomplete until a default model is also confirmed**

**Step 2: Commit**

---

## Phase 8: Comprehensive Integration Tests

### Task 8.1: Rust Integration Tests

**Files:**
- Create: `tests/test_chat_completion.rs`
- Create: `tests/test_provider_resolution.rs`
- Create: `tests/test_agent_crud.rs`
- Create: `tests/test_session_management.rs`
- Create: `tests/test_knowledge_base.rs`
- Create: `tests/test_mcp_optional.rs`

Each test file:
1. Starts a test server with in-memory SurrealDB
2. Makes HTTP requests to API endpoints
3. Verifies responses
4. Cleans up

**Key tests:**

```rust
// test_chat_completion.rs
#[tokio::test]
async fn test_chat_returns_streamed_response() {
    // Start server with MockLlmDriver
    // POST /api/chat/completion with message
    // Verify SSE stream contains text delta events
    // Verify stream ends with [DONE]
}

// test_provider_resolution.rs
#[tokio::test]
async fn test_provider_registers_and_resolves() {
    // POST /api/uar/providers to register openai
    // POST /api/chat/completion (no model specified)
    // Verify the resolved model is the provider's default
}

// test_mcp_optional.rs
#[tokio::test]
async fn test_server_starts_without_mcp_json() {
    // Start server in temp dir (no mcp.json)
    // Verify server starts and responds to health check
    // Verify chat still works (just no MCP tools available)
}
```

**Step: Write all tests, verify they pass**
**Commit after each test file passes**

---

### Task 8.2: Playwright E2E Tests

**Files:**
- Create: `frontend/e2e/admin-providers.spec.ts`
- Create: `frontend/e2e/admin-agents.spec.ts`
- Create: `frontend/e2e/admin-tools.spec.ts`
- Create: `frontend/e2e/admin-skills.spec.ts`
- Create: `frontend/e2e/admin-knowledge.spec.ts`
- Create: `frontend/e2e/chat-basic.spec.ts`
- Create: `frontend/e2e/chat-agent-selection.spec.ts`
- Create: `frontend/e2e/chat-no-provider.spec.ts`
- Create: `frontend/e2e/chat-session-config.spec.ts`

Each test:
1. Navigates to the relevant page
2. Interacts with UI elements (click, type, select)
3. Verifies expected outcomes (text visible, elements present, navigation works)

**Key tests:**

```typescript
// admin-providers.spec.ts
test("search filters providers list", async ({ page }) => {
  await page.goto("/admin");
  await page.click("text=Providers");
  await page.fill('[placeholder="Search providers..."]', "openai");
  await expect(page.locator("text=OpenAI")).toBeVisible();
  await expect(page.locator("text=Anthropic")).not.toBeVisible();
});

// chat-basic.spec.ts
test("sending a message shows response", async ({ page }) => {
  await page.goto("/threads");
  await page.click("text=New thread");
  await page.fill('[placeholder="Send a message"]', "Hello");
  await page.click('[aria-label="Send"]');
  // Wait for agent response to appear
  await expect(page.locator('[data-testid="agent-message"]')).toBeVisible({ timeout: 15000 });
});

// chat-no-provider.spec.ts
test("shows warning when no provider configured", async ({ page }) => {
  // This test needs a server with no providers configured
  await page.goto("/threads");
  await expect(page.locator("text=No LLM Provider Configured")).toBeVisible();
  await page.click("text=Configure Provider");
  await expect(page).toHaveURL(/\/admin/);
});
```

**Step: Write all tests, verify they pass against running server**
**Commit after each test file**

---

### Task 8.3: Final Verification — Full Build + All Tests

**Step 1: Run complete Rust test suite**
```bash
cargo test --all
```
Expected: ALL tests pass.

**Step 2: Run Playwright E2E suite**
```bash
cd frontend && npx playwright test --reporter=html
```
Expected: ALL tests pass.

**Step 3: Build release binary and verify it starts**
```bash
cargo build --release
./target/release/universal-agent-runtime &
sleep 10
curl -s http://localhost:3002/healthz | grep ok
curl -s -X POST http://localhost:3002/api/chat/completion \
  -H 'Content-Type: application/json' \
  -d '{"messages":[{"role":"user","content":"Hello"}],"stream":false}'
```
Expected: Server starts, health check passes, chat responds.

**Step 4: Commit and tag**
```bash
git tag v0.2.0-alpha
git push origin main --tags
```

---

## Dependency Graph

```
Phase 0 (Test Infra) ─── no deps
Phase 1 (Context Mgmt) ─── no deps
Phase 2 (Sandboxing) ─── no deps
Phase 3 (Graph) ──────── Phase 0 (needs MockLlmDriver for tests)
Phase 4 (Checkpoints) ── Phase 3 (checkpoints stored per graph node)
Phase 5 (Multi-Agent) ── Phase 3 (AgentNode is a graph node)
Phase 6 (Skill Model) ── no deps
Phase 7 (Admin UI) ───── Phase 1, Phase 6 (needs context + skill model to be wired)
Phase 8 (Tests) ──────── ALL phases (tests verify everything)
```

**Execution order:**
1. Phase 0 + Phase 1 + Phase 2 + Phase 6 (parallel, independent)
2. Phase 3 (depends on Phase 0)
3. Phase 4 + Phase 5 (depend on Phase 3, can be parallel)
4. Phase 7 (depends on Phase 1 + 6)
5. Phase 8 (after everything)
