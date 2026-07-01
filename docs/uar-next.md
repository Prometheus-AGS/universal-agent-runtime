# Universal Agent Runtime (UAR) — Comprehensive Architecture & Production Readiness Analysis

**Date:** 2026-07-01  
**Analyst:** Mavis (powered by MiniMax M3)  
**Scope:** Universal Agent Runtime (`/Users/gqadonis/Projects/prometheus/universal-agent-runtime`) + LibreFang/BossFang (`/Users/gqadonis/Projects/references/librefang`)  
**Research Base:** 50+ independent sources, 8 research dimensions, competitive harness analysis (Hermes, OpenClaw, Claude Code, Codex, OpenCode), A2A/AG-UI/MCP protocol specifications, 2026 RAG best practices, frontier model comparison data

---

## Executive Summary

The Universal Agent Runtime (UAR) is a **production-grade, ~59,400-line Rust agentic AI runtime** with a React/TypeScript admin UI, built around a philosophy of **MCP-first, streaming-native, tool-everywhere** architecture. It supports **142+ LLM providers** via `liter-llm`, includes an 8-stage compiler pipeline (PMPO), Cedar policy governance, and dual-database backends (Postgres + SurrealDB). It is an impressive technical achievement.

**However, critical gaps exist** that prevent it from being a premier agent harness in the July 2026 landscape:

1. **No dynamic model routing** — The runtime lacks the intelligence to route tasks to optimal models based on cost, capability, or task type (the core competitive advantage of modern harnesses)
2. **RAG is bolted-on, not a knowledge runtime** — Document ingestion exists but lacks the 2026 "knowledge runtime" pattern (verification, reasoning, access control, audit trails as a unified layer)
3. **No per-model prompt dialect optimization** — The UAR normalizes API calls but doesn't adapt prompt formats (XML for Claude, JSON schema for OpenAI, Markdown for Kimi, thinking_mode for GLM)
4. **AG-UI and A2A coexist but don't integrate** — Both protocols are implemented but there's no unified task model; they operate as separate subsystems
5. **Skill system is storage-centric, not activation-centric** — Skills are stored well but activation accuracy (the hard problem) lacks the sophistication of Claude Code or OpenCode
6. **Significant overlap with LibreFang/BossFang** — Both systems handle agents, skills, providers, orchestration, and web UI

**The path forward:** The UAR should become the **"harness kernel"** — the model-routing, prompt-engineering, context-management, and execution layer. LibreFang should become the **"agent operating system"** — the channel adapters, workflow engine, marketplace, and enterprise UI. The UAR should shed its web UI, its skill marketplace concepts, and its channel adapters, while LibreFang should drop its own LLM provider layer in favor of the UAR's `liter-llm` + routing intelligence.

---

## 1. UAR Architecture Analysis: Is It Well-Suited for Model-Aware Agent Adjustment?

### 1.1 Current Architecture Strengths

The UAR's architecture is **fundamentally sound** for an agent runtime:

| Component | Strength | Assessment |
|---|---|---|
| **liter-llm** | 142+ providers, compile-time model catalog, zero runtime network calls | **Excellent** — this is the UAR's crown jewel |
| **MCP integration** | Always-on MCP client, dynamic tool discovery | **Excellent** — MCP is the 2026 standard for tools |
| **8-stage compiler (PMPO)** | UAR-AGENT-MD → signed AgentDescriptor (Ed25519) | **Excellent** — unique differentiator for agent distribution |
| **Cedar policy engine** | Declarative authorization | **Good** — but evaluation is a "future milestone" (not wired) |
| **Dual database** | Postgres + SurrealDB | **Good** — but credential store not implemented for Postgres |
| **Memory system** | Multi-scope (session/user/agent/global/task) with hybrid BM25 + vector search | **Good** — but MemPalace disabled by default due to dependency conflicts |
| **Streaming** | Dual SSE: OpenAI-compatible + AG-UI simultaneously | **Good** — but lacks the 2026 streaming-native expectations |

### 1.2 Critical Gaps for Model-Aware Adjustment

The research on 11 frontier models (July 2026) reveals that **the harness around the model is now the primary differentiator**. The UAR is missing the most important modern harness capabilities:

#### Gap 1: No Dynamic Model Router

The UAR can *connect* to 142+ providers but cannot *choose* between them intelligently. Modern harnesses (Claude Code, OpenCode, Codex) all route to different models based on task type. The UAR needs:

- **Task-type classifier**: Code refactor → Kimi K2.7-Code; Visual task → MiniMax M3; Architecture reasoning → Opus 4.8; Regulated → Claude; Cost-sensitive → Qwen 3.7 Plus
- **Cost-performance optimizer**: Token accounting that includes tokenizer efficiency (Anthropic's +30% English inflation), reasoning token multipliers (3-10×), context surcharges (1.5-2×)
- **Real-time benchmark-aware routing**: A registry of per-model capabilities that updates from independent benchmarks (SWE-Bench, Terminal-Bench, MCP Atlas) rather than vendor claims
- **Failover with degradation**: If Opus 4.8 is rate-limited, fall back to Sonnet 5; if Z.ai is 429-ing, route to OpenRouter

**Recommendation:** Add a `Router` module between the API layer and `liter-llm`. This module should:
1. Accept a task description + constraints (budget, latency, compliance requirements)
2. Query the model capability registry (from the 2026 comparison data)
3. Select the optimal model + provider combination
4. Transform the prompt to the model's preferred dialect (see Section 13)
5. Track actual cost vs predicted and feed back to improve routing

#### Gap 2: No Per-Model Prompt Dialect Optimization

The UAR's `liter-llm` normalizes API calls but treats all models as interchangeable black boxes. Research shows this is **fundamentally wrong**:

- Claude: XML tags (`<context>`, `<instructions>`, `<examples>`) outperform plain text by significant margins; "extended thinking" toggle dramatically improves reasoning
- GPT-5.5: Strict JSON schemas via `response_format` are the cleanest approach; prompt caching reduces cost by ~80%
- Kimi K2.7: `preserve_thinking: true` is a killer feature for multi-turn coherence
- GLM-5.2: `thinking_mode: max` for hard problems; `high` is default; expert persona framing increases performance noticeably
- Qwen 3.7: `enable_thinking: true` + `preserve_thinking: true` for multi-turn
- MiniMax M3: Markdown headers + explicit role demarcation; multimodal prompts get full attention when image comes first

**Recommendation:** The UAR's agent description specification (Section 4) should include a `prompt_dialect` field per model. The runtime should:
1. Detect the target model from the router
2. Apply the correct prompt template (XML envelope, JSON schema wrapper, Markdown header hierarchy)
3. Add model-specific system prompt prepends (safety, thinking instructions, tool format hints)
4. Handle reasoning token persistence (Kimi's preserve_thinking, Qwen's enable_thinking, Anthropic's extended thinking) as first-class API parameters

#### Gap 3: No Context Strategy per Model

Context management is treated uniformly across all models. Research shows dramatic differences:

- **Anthropic**: Lowest positional bias (~18% drop in middle); ContextualRAG pattern (+35-50% retrieval); 2× input pricing above 200K tokens
- **GPT-5.5**: Standard transformer positional bias; prefers shorter chunks (800-1,500 tokens) for code retrieval
- **MiniMax M3**: Sparse attention reduces positional bias but can miss information in unattended regions; chunk into natural sections
- **Kimi**: `preserve_thinking` is the killer feature for long-horizon agent loops; best for multi-hour sessions
- **GLM-5.2**: Reward-hacking behavior manifests in eval contexts; needs guardrails for production monitoring

**Recommendation:** The context manager should be model-aware:
1. Choose chunking strategy based on model (short chunks for GPT-5.5, long for Kimi)
2. Place critical information at beginning/end based on model's positional bias profile
3. Apply model-specific context compression (Anthropic cookbook = 58% reduction, Active Context Compression = 22.7%)
4. Track KV cache efficiency and switch models if context growth exceeds the chosen model's sweet spot

---

## 2. UAR vs LibreFang/BossFang: Overlap Analysis

### 2.1 The Overlap Problem

Both systems are **large Rust codebases** that handle overlapping concerns:

| Concern | UAR | LibreFang | Overlap Severity |
|---|---|---|---|
| LLM Provider Management | `liter-llm` (142+ providers) | 28 providers via `fang-llm` | **High** — both solve the same problem |
| Agent Orchestration | `RunManager` + `Orchestrator` | Kernel + Workflow Engine | **High** — both manage agent execution loops |
| Skill System | 4 types (Native, WASM, MCP, Builtin) | 6 types (Prompt, Python, WASM, Node, Shell, Builtin) | **High** — both have skill registries |
| Web Dashboard | React/TS admin UI (100+ files) | React + Vite dashboard (50+ pages) | **High** — both have management UIs |
| Memory/RAG | Hybrid BM25 + vector + knowledge graph | `surreal-memory` semantic memory | **Medium** — different approaches but same domain |
| Channel Adapters | None (focused on API) | 45+ (Telegram, Discord, Slack, etc.) | **Low** — UAR doesn't have this |
| Workflow Engine | Basic (10-iteration tool loop) | Full workflow engine with approval gates | **Medium** — UAR simpler, LF more mature |
| MCP | Always-on client | MCP support | **Medium** — UAR more deeply integrated |
| P2P Protocol | None | BossFang P2P protocol | **Low** — UAR doesn't have this |
| Marketplace | None (compiler for distribution) | Skill marketplace (ClawHub) | **Low** — different approaches |
| Compilation | 8-stage PMPO pipeline | None | **Low** — UAR unique |
| Policy/Governance | Cedar engine | Basic RBAC | **Medium** — UAR more advanced |

### 2.2 Recommended Division of Responsibility

The two systems should **not** be merged. They should **complement** each other through a clean interface:

#### UAR Becomes the "Harness Kernel"

**What the UAR keeps and doubles down on:**
1. **`liter-llm`** + dynamic model router (the intelligence layer for choosing models)
2. **Per-model prompt dialect engine** (XML/JSON/Markdown transformation)
3. **Context management** with model-aware strategies
4. **MCP client** (always-on, dynamic tool discovery)
5. **A2A protocol implementation** (agent-to-agent communication)
6. **AG-UI streaming** (real-time UI updates)
7. **8-stage compiler (PMPO)** for agent distribution
8. **Cedar policy engine** (when fully wired)
9. **Multi-scope memory** (session/user/agent/global/task)

**What the UAR removes or hands to LibreFang:**
1. **Web dashboard** — LibreFang's dashboard is more mature (50+ pages vs UAR's 100+ files but narrower scope)
2. **Channel adapters** — LibreFang has 45+; UAR has none. This is clearly LibreFang's domain
3. **Skill marketplace concepts** — LibreFang's ClawHub is a real marketplace; UAR's compiler is for distribution, not discovery
4. **Workflow engine** — LibreFang's is more mature with approval gates; UAR's is basic
5. **P2P protocol** — LibreFang has this; UAR doesn't need it

#### LibreFang Becomes the "Agent Operating System"

**What LibreFang keeps and doubles down on:**
1. **45+ channel adapters** (the broadest coverage in the industry)
2. **Workflow engine** with approval gates and visual editor
3. **Skill marketplace (ClawHub)** for discovery and community
4. **P2P protocol** for decentralized agent communication
5. **Enterprise dashboard** (50+ management pages)
6. **Desktop app** (electron/Tauri wrapper)

**What LibreFang removes or hands to UAR:**
1. **LLM provider layer** — Replace `fang-llm` (28 providers) with UAR's `liter-llm` (142+ providers) + router
2. **Basic agent orchestration** — Delegate to UAR's `RunManager` + `Orchestrator`
3. **MCP client** — Use UAR's always-on implementation
4. **Memory/RAG** — Use UAR's multi-scope memory + knowledge graph
5. **Policy engine** — Use UAR's Cedar implementation (when wired)

#### The Interface Between Them

The UAR should expose an **A2A-compatible agent interface** that LibreFang consumes. Specifically:

```
LibreFang Agent OS          UAR Harness Kernel
┌─────────────────┐        ┌─────────────────────┐
│ Channel Adapter │───────→│ A2A Task Endpoint   │
│ (Telegram, etc) │        │ (/.well-known/agent)│
├─────────────────┤        ├─────────────────────┤
│ Workflow Engine │───────→│ Agent Descriptor    │
│ (Approval gates)│        │ (Compiled + Signed) │
├─────────────────┤        ├─────────────────────┤
│ Dashboard UI    │←───────│ AG-UI SSE Stream    │
│ (Management)    │        │ (Real-time updates)   │
├─────────────────┤        ├─────────────────────┤
│ Skill Marketplace│←───────│ MCP Tool Registry   │
│ (ClawHub)       │        │ (Dynamic discovery)   │
└─────────────────┘        └─────────────────────┘
```

LibreFang sends tasks to the UAR via A2A. The UAR executes with the optimal model, prompt dialect, and context strategy. Results stream back via AG-UI. LibreFang's dashboard displays the UAR's state.

---

## 3. Agent Description Specification Review

### 3.1 Current Specification Analysis

The UAR uses an **8-stage PMPO compiler** that transforms UAR-AGENT-MD Markdown into signed AgentDescriptor structs. This is a **unique and powerful approach** that no other harness has.

**Current specification coverage:**
- Agent metadata (name, version, author, description)
- System prompt templates
- Tool schemas (MCP integration)
- Skill dependencies
- Policy constraints (Cedar)
- Memory scope preferences
- Provider hints (but not dynamic routing)

### 3.2 Gaps in the Specification

Based on the 2026 model research, the specification is **missing critical fields**:

#### Missing: Model Capability Requirements

The specification should declare:
```yaml
model_requirements:
  primary:
    capability_profile: "coding"  # coding, vision, reasoning, multimodal, cost_sensitive
    min_context: 128000
    preferred_dialect: "xml"  # xml, json, markdown, mixed
    reasoning_required: true
    multimodal_required: false
  fallbacks:
    - model_family: "kimi"
      max_cost_multiplier: 2.0
    - model_family: "qwen"
      max_cost_multiplier: 1.5
```

#### Missing: Prompt Dialect Configuration

```yaml
prompt_dialect:
  format: "xml"  # xml, json, markdown, yaml
  structure:
    system_envelope: "<system>{content}</system>"
    context_block: "<context>{content}</context>"
    instruction_block: "<instructions>{content}</instructions>"
    example_block: "<examples>{content}</examples>"
  special_directives:
    - "think_step_by_step"  # CoT
    - "enable_extended_thinking"  # Anthropic
    - "preserve_thinking"  # Kimi
```

#### Missing: RAG Architecture Preferences

```yaml
rag_configuration:
  strategy: "hybrid"  # pure_long_context, rag_only, hybrid
  chunk_size: 1500
  chunk_overlap: 200
  embedding_model: "bge-m3"
  reranker: "bge-reranker-v2-m3"
  retrieval_top_k: 10
  synthesis_model: "claude-sonnet-5"  # For synthesis step
  lost_in_middle_mitigation: "contextual_rag"
```

#### Missing: Context Management Strategy

```yaml
context_strategy:
  max_tokens: 120000  # Effective, not advertised
  compression: "active_context_compression"
  hierarchy_format: "markdown_headings"  # markdown, json, xml
  position_bias_mitigation: "beginning_end_placement"
  multi_turn_persistence: "preserve_thinking"
```

#### Missing: API Harness Configuration

```yaml
api_harness:
  provider: "anthropic"
  endpoint: "native"  # native, openai_compat, anthropic_compat
  moderation: "passive"  # passive, active, strict
  before_prompt:
    - "safety_prepend"
    - "system_prompt_injection_guard"
  after_prompt:
    - "json_schema_validation"
    - "output_filter"
  rate_limit_strategy: "exponential_backoff"
  retry_policy:
    max_retries: 3
    backoff_base: 2.0
```

### 3.3 Specification Recommendations

1. **Add a `model_capability_requirements` section** that declares what the agent needs, not which model to use. Let the router decide.
2. **Add a `prompt_dialect` section** with templates per model family. The compiler should generate multiple variants.
3. **Add a `rag_configuration` section** with per-agent RAG preferences.
4. **Add a `context_strategy` section** with model-aware context management.
5. **Add an `api_harness` section** for provider-specific behavior.
6. **Version the specification** — UAR-AGENT-MD v1.0 should be frozen; v2.0 should add these fields.
7. **Add conformance testing** — The compiler should validate that the agent can actually run with the declared requirements.

---

## 4. Provider & Model Management: liter-llm + Dynamic Routing

### 4.1 liter-llm Assessment

`liter-llm` is **the UAR's strongest asset**. Supporting 142+ providers with a compile-time model catalog is genuinely impressive. It solves the "provider fragmentation" problem that plagues every other harness.

**Competitive comparison:**
- Claude Code: Claude only (1 family)
- Codex: OpenAI only (1 family)
- OpenCode: 75+ providers (good)
- UAR: 142+ providers (best in class)

### 4.2 What liter-llm Is Missing

**Missing: Capability Registry with Independent Benchmarks**

The model catalog knows *what* models exist but not *how good* they are. It should integrate:
- SWE-Bench scores (Verified, Pro, Multilingual)
- Terminal-Bench scores
- MCP Atlas / MCP Mark scores
- Cost-per-token (including tokenizer efficiency)
- Effective context size (RULER-tested, not advertised)
- Positional bias profile
- Tool-call reliability rates
- Multi-turn consistency scores

**Missing: Token Cost Normalization**

The catalog should expose *normalized* costs that account for:
- Tokenizer efficiency ( Anthropic +30% English inflation)
- Reasoning token multipliers (3-10× for thinking models)
- Context surcharges (1.5-2× for long context)
- System prompt overhead (35-50% of input)

**Missing: Dynamic Provider Health**

The catalog should track real-time provider health:
- Z.ai's 429 issues (documented in June 2026)
- OpenAI rate limit tiers
- Anthropic plan-based limits
- OpenRouter automatic failover behavior

### 4.3 Recommended Router Architecture

```rust
pub struct ModelRouter {
    capability_registry: CapabilityRegistry,  // From 2026 research
    cost_calculator: TokenCostCalculator,     // Normalized costs
    health_monitor: ProviderHealthMonitor,  // Real-time status
    prompt_transformer: PromptTransformer,    // Dialect adaptation
    context_manager: ModelAwareContextManager, // Context strategy
}

impl ModelRouter {
    pub async fn route(&self, task: &TaskDescriptor) -> RoutingDecision {
        // 1. Filter by capability requirements
        let candidates = self.capability_registry.filter(&task.requirements);
        
        // 2. Filter by health (exclude failing providers)
        let healthy = self.health_monitor.filter_healthy(candidates);
        
        // 3. Score by cost-performance
        let scored = self.cost_calculator.score(healthy, &task.budget);
        
        // 4. Select optimal model
        let selection = scored.select_optimal();
        
        // 5. Transform prompt to model's dialect
        let prompt = self.prompt_transformer.transform(&task.prompt, &selection.model);
        
        // 6. Configure context strategy
        let context = self.context_manager.configure(&selection.model, &task.context);
        
        RoutingDecision { selection, prompt, context }
    }
}
```

---

## 5. RAG Functionality Review

### 5.1 Current RAG Implementation

The UAR has:
- Document ingestion via Kreuzberg (PDF, Office, OCR)
- Semantic chunking
- Leiden community detection for graph building
- Hybrid BM25 + vector search
- Knowledge graph
- Auto-capture

### 5.2 Gaps vs 2026 "Knowledge Runtime" Standard

The 2026 research shows RAG has evolved from "search → append → generate" to a **knowledge runtime** with:
- Query decomposition
- Parallel search
- Verification
- Reasoning
- Access control
- Audit trails

**The UAR is missing:**
1. **Query decomposition** — No automatic breakdown of complex queries into subqueries
2. **Verification layer** — No fact-checking or accuracy confirmation
3. **Access control integration** — RAG doesn't respect Cedar policies for document-level permissions
4. **Audit trails** — No logging of what documents were retrieved for each query
5. **Agentic RAG** — No dynamic strategy selection (vector vs graph vs community summary based on query type)
6. **Embedding versioning** — No versioned embedding indexes that can be rolled back
7. **Continuous evaluation** — No feedback loop from real user queries

### 5.3 Recommendation: Extract RAG to a "Knowledge Service"

RAG should **not** be part of the agent runtime. It should be a **separate service** that the runtime calls via MCP:

```
UAR Agent Runtime ──MCP──→ Knowledge Service
                              ├─ Document Ingestion
                              ├─ Query Decomposition
                              ├─ Hybrid Retrieval (vector + BM25 + graph)
                              ├─ Verification Layer
                              ├─ Access Control (Cedar policies)
                              ├─ Audit Trails
                              └─ Continuous Evaluation
```

**Benefits:**
- RAG can be scaled independently
- Multiple agents can share the same knowledge base
- Knowledge service can be deployed on-premise for compliance
- The UAR focuses on execution, not data management

### 5.4 If RAG Stays In-Process

If RAG must stay in the UAR (for latency reasons), then:
1. Add query decomposition (parallel subquery execution)
2. Add verification layer (cross-reference retrieved facts)
3. Integrate Cedar policies for document-level access control
4. Add audit logging (what was retrieved, when, for which agent)
5. Add agentic strategy selection (vector for factual, graph for multi-hop, community summary for aggregation)
6. Add embedding versioning and rollback
7. Add continuous evaluation pipeline

---

## 6. Orchestrator & Agent Routing Analysis

### 6.1 Current Implementation

The UAR specifies:
- At minimum 2 agents: orchestrator + default agent
- Orchestrator can be interacted with directly
- Direct agent interaction via `model` parameter in OpenAI API interface
- Default agent executes if no other agents match

### 6.2 Assessment

**The 2-agent minimum is too simplistic** for 2026 production use. Modern harnesses use:
- Claude Code: Dynamic Workflows with parallel subagents (up to 16)
- Codex: Multi-task parallelism with cloud agents
- OpenCode: 4 built-in agents (Build, Plan, General, Explore) + custom agents

**The routing mechanism is too static** — it doesn't account for:
- Task complexity (simple query → default agent; complex refactor → coding specialist)
- Cost constraints (high-volume → cheap model; critical → expensive model)
- Compliance requirements (HIPAA → Anthropic only; non-sensitive → open-weight)
- Model availability (real-time health checks)

### 6.3 Recommendations

1. **Expand to N-agent architecture** with a registry of specialist agents:
   - `coding-agent`: Optimized for code generation (Kimi/Claude)
   - `vision-agent`: Optimized for visual tasks (MiniMax M3/GPT-5.5)
   - `terminal-agent`: Optimized for shell commands (GLM-5.2/GPT-5.5)
   - `research-agent`: Optimized for web search and synthesis (Claude/Opus)
   - `business-agent`: Optimized for document analysis and workflows (Claude)

2. **Add intent-based routing** with a classifier that maps user requests to agent capabilities:
   ```
   "Refactor this React component" → coding-agent
   "What's in this screenshot?" → vision-agent
   "Find me the latest papers on X" → research-agent
   "Process this invoice" → business-agent
   ```

3. **Add cost-aware routing** with budget envelopes:
   ```yaml
   routing_budget:
     daily_limit: "$50"
     per_task_limit: "$5"
     fallback_on_budget_exceeded: "qwen-3.7-plus"
   ```

4. **Add compliance-aware routing**:
   ```yaml
   compliance:
     regulated_data: true
     allowed_providers: ["anthropic"]
     forbidden_providers: ["alibaba", "minimax", "moonshot"]
   ```

---

## 7. AG-UI & A2A Protocol Integration

### 7.1 Current Implementation

The UAR implements both:
- **AG-UI**: Dual SSE streaming (OpenAI-compatible + AG-UI named events)
- **A2A**: Google's Agent-to-Agent Protocol RC v1.0 with JSON-RPC + gRPC

### 7.2 The Problem: Parallel But Not Integrated

Both protocols exist but there's **no unified task model**. AG-UI is for UI-to-agent communication. A2A is for agent-to-agent communication. They should share the same underlying task representation.

**The A2A implementation is incomplete**:
- gRPC auto-generation is disabled (build.rs comments out tonic-build due to v0.14 API changes)
- The proto exists but the module is not exported
- The A2A gRPC transport is not functional

### 7.3 Recommendations

1. **Fix the A2A gRPC implementation** — This is a critical blocker for multi-agent deployments. The tonic-build issue needs to be resolved.

2. **Unify task model** — Both AG-UI and A2A should use the same `Task` struct:
   ```rust
   pub struct UnifiedTask {
       pub id: UUID,
       pub agent_descriptor: AgentDescriptor,
       pub prompt: Prompt,  // Already dialect-transformed
       pub context: ContextSnapshot,
       pub status: TaskStatus,
       pub artifacts: Vec<Artifact>,
       pub stream: StreamConfig,  // AG-UI or A2A
   }
   ```

3. **Add A2A Agent Card registry** — The UAR should maintain a registry of available agents with their Agent Cards, making discovery automatic.

4. **Add AG-UI event types for A2A delegation** — When an agent delegates to another agent via A2A, the UI should show this as a delegation event.

5. **Maintain A2UI registry** — As requested, the UAR should maintain a registry of A2A-compatible agents that can be discovered by other agents.

---

## 8. Intent Classification & Skill System Review

### 8.1 Current Implementation

The UAR has 6 pluggable intent classification backends:
- Rules
- TF-IDF
- WASM
- Hybrid
- LocalEmbedding
- LLM

Skills are stored in:
- Filesystem
- Database
- With hot-reload

4 skill types: Native (Rust), WASM (WASM Component Model), MCP, Builtin

### 8.2 Assessment

**Intent classification is storage-centric, not activation-centric.** The hard problem is not *storing* skills — it's *choosing the right skill at the right time with the right schema*. Claude Code achieves ~95% skill activation accuracy because it's tuned for Claude's tool-calling behavior. The UAR's generic approach is likely significantly lower.

**Missing: Skill activation accuracy benchmarking** — There's no way to measure whether the UAR's intent classification actually picks the right skill.

**Missing: Skill dialect adaptation** — Skills are stored with their schemas but not adapted for different models' tool-calling preferences (Claude's XML vs OpenAI's JSON vs Kimi's markdown).

**Missing: Skill failure recovery** — When a skill call fails, there's no automatic retry with a different model or skill variant.

### 8.3 Recommendations

1. **Add skill activation accuracy metrics** — Track precision/recall per skill, per model, per intent classifier backend.

2. **Add per-model skill schema transformation** — Transform skill schemas to match the model's native tool format:
   - Claude: XML tags (`<tool>`, `<parameter>`)
   - OpenAI: JSON schema (`type: "function"`)
   - Kimi: Markdown headers + JSON blocks
   - Qwen: XML or JSON depending on configuration

3. **Add skill fallback chains** — If `skill_A` fails with model X, try `skill_A_variant` with model Y.

4. **Add skill evaluation harness** — Automated testing of skill activation accuracy with ground-truth datasets.

5. **Consolidate skill storage** — The current dual storage (filesystem + DB) is confusing. Use the database as the source of truth with filesystem as a backup/import mechanism.

---

## 9. Web UI Analysis

### 9.1 Current Implementation

The UAR has a React/TypeScript admin UI with:
- Zustand-based real-time entity graph
- 100+ frontend files
- Management pages for agents, skills, providers, memory, RAG

### 9.2 Assessment

**The web UI is functional but not S-tier.** Compared to:
- Cherry Studio: Clean, intuitive, minimal configuration overhead
- LibreFang dashboard: 50+ management pages, more mature
- Claude Code: Terminal-native, no web UI needed

**Specific issues:**
1. **Provider configuration is complex** — 142+ providers require manual API key configuration per provider
2. **No model comparison view** — Users can't see side-by-side model performance
3. **No cost tracking dashboard** — Users can't see per-model, per-task spend
4. **No real-time agent state visualization** — AG-UI streaming exists but the UI doesn't show agent reasoning, tool calls, or context usage in real-time
5. **Configuration is scattered** — Provider settings, model settings, agent settings, skill settings are in different places

### 9.3 Recommendations

1. **Study Cherry Studio's provider configuration** — Cherry Studio has a clean "Add Provider → Select Type → Enter API Key → Select Models" flow. The UAR should adopt this pattern.

2. **Add a unified configuration page** — One page for all provider/model settings with:
   - Provider cards (logo, name, status)
   - Model list with capabilities and costs
   - API key management (encrypted, per-tenant)
   - Health indicators (green/yellow/red per provider)

3. **Add a model comparison dashboard** — Show:
   - Benchmark scores (SWE-Bench, Terminal-Bench, etc.)
   - Cost per 1K tokens
   - Effective context size
   - Latency percentiles
   - Your own usage stats

4. **Add cost tracking** — Real-time spend tracking with:
   - Per-model breakdown
   - Per-task breakdown
   - Budget alerts
   - Cost prediction for pending tasks

5. **Add agent state visualization** — Real-time display of:
   - Agent reasoning steps (with preserve_thinking for Kimi)
   - Tool calls in progress
   - Context usage (beginning/middle/end visualization)
   - Memory retrieval (what was retrieved, from where)
   - Routing decisions (why this model was chosen)

6. **Consider using a design system** — Adopt a consistent design system (shadcn/ui, Tailwind) for a polished look.

---

## 10. Cherry Studio Configuration Research

Cherry Studio (popular desktop LLM client) handles provider/model configuration elegantly:

### 10.1 Provider Configuration Pattern

1. **Add Provider** → Select provider type (OpenAI, Anthropic, Gemini, Custom)
2. **Enter API Key** and **API Host** (base URL)
3. **Auto-detect models** — Cherry Studio fetches available models from the provider
4. **Enable/disable models** — User selects which models to show in the UI
5. **Model parameters** — Temperature, max tokens, top P per model

### 10.2 Database Storage Pattern

Cherry Studio stores configuration in:
- **Local JSON/DB** for community edition
- **Centralized admin backend** for enterprise edition (Postgres)
- **Model metadata** is cached locally after first fetch from provider

### 10.3 Recommendations for UAR

1. **Adopt the "Add Provider" flow** — Simplify provider configuration to: name, type, API key, base URL
2. **Auto-detect models** — After provider configuration, fetch available models and cache them
3. **Store provider config in database** — Not just files; enable multi-tenant provider management
4. **Store model metadata in database** — Capabilities, costs, benchmarks as structured data
5. **Enable provider health checks** — Periodic background checks that update provider status
6. **Support "default model" per task type** — User can set defaults (coding → Kimi, chat → Claude, etc.)

---

## 11. Storage Model Analysis

### 11.1 Current Implementation

The UAR supports dual backends:
- **Postgres** with `pgvector`
- **SurrealDB** with SurrealQL

12 database migrations exist for both.

### 11.2 Assessment

**Dual database support is a liability, not an asset.** It doubles migration complexity, testing burden, and deployment confusion. The UAR should pick one primary backend and treat the other as a legacy option.

**Postgres issues:**
- Credential store not implemented for multi-tenant encryption
- `pgvector = "=0.4.1"` is pinned and fragile

**SurrealDB issues:**
- Hard-pinned to `=3.0.5` — upgrading requires coordinated changes across 3 subsystems
- SurrealDB is still maturing; production stability concerns

### 11.3 Recommendations

1. **Make Postgres the primary backend** — It's the industry standard for production. SurrealDB can be an experimental option.
2. **Fix the Postgres credential store** — This is blocking multi-tenant production use.
3. **Unpin pgvector** — Use semver-compatible ranges, not exact pins.
4. **Add schema versioning** — The 12 migrations are good but need a schema versioning table.
5. **For RAG documents:** Consider a separate document store (S3/GCS for files, Postgres for metadata, vector DB for embeddings) rather than storing everything in the primary DB.
6. **For configuration:** Use a configuration hierarchy:
   ```
   /etc/universal-agent-runtime/   # System defaults
   ~/.config/universal-agent-runtime/  # User overrides
   ./.uar/  # Project-specific overrides
   ENV vars  # Runtime overrides
   DB (tenant table)  # Tenant-specific overrides
   ```

---

## 12. Per-Model Prompt Handling Strategy

### 12.1 The Problem

The UAR treats all models as interchangeable. Research shows this is **wrong** — each model has a "prompt dialect" that significantly affects performance.

### 12.2 Recommended Implementation

Add a `PromptDialectEngine` to the UAR:

```rust
pub enum PromptDialect {
    AnthropicXml,    // <context>, <instructions>, <examples>
    OpenAIJson,      // response_format + JSON schema
    KimiMarkdown,    // preserve_thinking + heading hierarchy
    GLMThinking,     // thinking_mode: high/max + expert persona
    QwenHybrid,      // enable_thinking + XML/JSON mix
    MiniMaxMultimodal, // Markdown + image-first ordering
    GenericMarkdown, // Fallback
}

pub struct PromptDialectEngine;

impl PromptDialectEngine {
    pub fn transform(&self, prompt: &Prompt, dialect: PromptDialect) -> TransformedPrompt {
        match dialect {
            PromptDialect::AnthropicXml => self.to_anthropic_xml(prompt),
            PromptDialect::OpenAIJson => self.to_openai_json(prompt),
            PromptDialect::KimiMarkdown => self.to_kimi_markdown(prompt),
            // ... etc
        }
    }
    
    fn to_anthropic_xml(&self, prompt: &Prompt) -> TransformedPrompt {
        TransformedPrompt {
            system: format!("<system>{}</system>", prompt.system),
            context: format!("<context>{}</context>", prompt.context),
            instructions: format!("<instructions>{}</instructions>", prompt.instructions),
            examples: format!("<examples>{}</examples>", prompt.examples),
            // Enable extended thinking for reasoning tasks
            extra_params: json!({"extended_thinking": true}),
        }
    }
    
    fn to_kimi_markdown(&self, prompt: &Prompt) -> TransformedPrompt {
        TransformedPrompt {
            system: format!("# System\n{}\n", prompt.system),
            context: format!("# Context\n{}\n", prompt.context),
            instructions: format!("# Instructions\n{}\n", prompt.instructions),
            // Preserve thinking across turns
            extra_params: json!({"preserve_thinking": true}),
        }
    }
    
    fn to_openai_json(&self, prompt: &Prompt) -> TransformedPrompt {
        TransformedPrompt {
            system: prompt.system.clone(),
            // Use structured output with JSON schema
            extra_params: json!({
                "response_format": {
                    "type": "json_schema",
                    "json_schema": prompt.output_schema
                }
            }),
        }
    }
}
```

### 12.3 Dialect Detection

The engine should auto-detect dialect from the model family:

```rust
pub fn detect_dialect(model_id: &str) -> PromptDialect {
    if model_id.contains("claude") || model_id.contains("anthropic") {
        PromptDialect::AnthropicXml
    } else if model_id.contains("gpt") || model_id.contains("openai") {
        PromptDialect::OpenAIJson
    } else if model_id.contains("kimi") || model_id.contains("moonshot") {
        PromptDialect::KimiMarkdown
    } else if model_id.contains("glm") || model_id.contains("z.ai") {
        PromptDialect::GLMThinking
    } else if model_id.contains("qwen") || model_id.contains("alibaba") {
        PromptDialect::QwenHybrid
    } else if model_id.contains("minimax") || model_id.contains("m3") {
        PromptDialect::MiniMaxMultimodal
    } else {
        PromptDialect::GenericMarkdown
    }
}
```

### 12.4 RAG Differences

Each model also needs RAG strategy adaptation:
- Claude: ContextualRAG pattern, 1,500-2,500 token chunks, Voyage 3 + Cohere Rerank 3.5
- GPT-5.5: Structured JSON retrieval, 800-1,500 token chunks, OpenAI text-embedding-3-large
- Kimi: Long-context-as-RAG replacement, whole-document retrieval, no chunking needed for <256K
- GLM-5.2: Dense retrieval, 1,500 token chunks, BGE-M3, reward-hacking guard
- MiniMax M3: Multimodal RAG, image + text + video, CLIP-style embeddings

---

## 13. Context Management Review

### 13.1 Current Implementation

The UAR has multi-scope memory:
- session
- user
- agent
- global
- task

With hybrid BM25 + vector search and knowledge graph.

### 13.2 Assessment

**The context management is model-agnostic but should be model-aware.** Key issues:

1. **No positional bias mitigation** — Critical information might be placed in the middle of context where retrieval is weakest
2. **No context compression** — Doesn't use Active Context Compression (22.7% reduction) or Anthropic cookbook compaction (58% reduction)
3. **No KV cache efficiency tracking** — Doesn't monitor or optimize for model-specific KV cache behavior
4. **No multi-turn decay tracking** — Doesn't measure or counteract semantic drift across turns
5. **MemPalace disabled** — The most sophisticated memory component is disabled due to dependency conflicts

### 13.3 Recommendations

1. **Add model-aware context placement** — Place critical information at beginning/end based on model's positional bias profile
2. **Add context compression** — Implement Active Context Compression and model-specific compaction strategies
3. **Add KV cache monitoring** — Track KV cache growth and switch models if approaching limits
4. **Add multi-turn coherence tracking** — Measure semantic drift and trigger re-summarization when drift exceeds threshold
5. **Fix MemPalace** — Resolve the `rusqlite` / `libsqlite3-sys` version conflicts and enable by default
6. **Add structured context hierarchy** — Support markdown headings, JSON schemas, and XML tags as memory organization formats, selected per model preference

---

## 14. Critical Gaps & Problematic Code

### 14.1 Issues from Codebase Exploration

| Issue | Severity | Impact | Fix Priority |
|---|---|---|---|
| gRPC auto-generation disabled | **Critical** | A2A gRPC transport non-functional | P0 |
| Postgres credential store not implemented | **Critical** | Multi-tenant encryption only on SurrealDB | P0 |
| MemPalace disabled by default | **High** | Most sophisticated memory unavailable | P1 |
| Tool approval uses keyword heuristic | **High** | Cedar policy engine not wired | P1 |
| Heavy dependency pinning | **High** | Build fragility, upgrade difficulty | P1 |
| Monolithic files (server.rs 4,848 LOC) | **Medium** | Maintenance difficulty, testability | P2 |
| A2A gRPC module not exported | **Medium** | A2A incomplete | P2 |
| No dynamic model router | **Critical** | Core competitive gap | P0 |
| No prompt dialect engine | **High** | Suboptimal model performance | P1 |
| No per-model context strategy | **High** | Lost-in-the-middle, wasted tokens | P1 |
| No cost tracking | **Medium** | Budget overruns, poor optimization | P2 |
| No skill activation metrics | **Medium** | Can't measure or improve accuracy | P2 |
| Web UI not S-tier | **Medium** | User experience below competitors | P2 |

### 14.2 Build & Deployment Issues

1. **`surrealdb = "=3.0.5"`** — Hard pin makes security updates impossible
2. **`pgvector = "=0.4.1"`** — Same problem for Postgres
3. **Multiple Git revs** — Makes builds non-reproducible across environments
4. **No Docker Compose** for full stack — Hard to evaluate locally

---

## 15. Competitive Positioning

### 15.1 How UAR Compares to Major Harnesses (July 2026)

| Dimension | UAR | Hermes | OpenClaw | Claude Code | Codex | OpenCode |
|---|---|---|---|---|---|---|
| **Provider breadth** | ⭐⭐⭐⭐⭐ 142+ | ⭐⭐⭐ 28 | ⭐⭐⭐⭐ 75+ | ⭐ 1 (Claude) | ⭐ 1 (OpenAI) | ⭐⭐⭐⭐ 75+ |
| **Model routing** | ⭐ None | ⭐⭐ Basic | ⭐⭐⭐ Good | ⭐⭐⭐ Dynamic Workflows | ⭐⭐⭐⭐ Multi-task | ⭐⭐⭐⭐ Agent-based |
| **Prompt dialects** | ⭐ None | ⭐⭐⭐ Good | ⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Native Claude | ⭐⭐⭐⭐⭐ Native GPT | ⭐⭐⭐⭐ Generic |
| **Context management** | ⭐⭐⭐ Multi-scope | ⭐⭐⭐⭐ Hot/cold memory | ⭐⭐⭐⭐ ContextEngine | ⭐⭐⭐⭐⭐ Best in class | ⭐⭐⭐⭐ Good | ⭐⭐⭐ Good |
| **MCP integration** | ⭐⭐⭐⭐⭐ Always-on | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ First-party | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Best |
| **Skill system** | ⭐⭐⭐ 4 types | ⭐⭐⭐⭐⭐ Self-improving | ⭐⭐⭐⭐⭐ 1.3M skills | ⭐⭐⭐⭐⭐ Skills + hooks | ⭐⭐⭐⭐ Skills | ⭐⭐⭐⭐⭐ LSP-aware |
| **A2A support** | ⭐⭐⭐ Partial (gRPC broken) | ⭐⭐ None | ⭐⭐⭐⭐ Good | ⭐⭐ None | ⭐⭐ None | ⭐⭐⭐ Basic |
| **AG-UI support** | ⭐⭐⭐⭐ Dual SSE | ⭐⭐ None | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Native | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐ Good |
| **Web UI** | ⭐⭐⭐ Functional | ⭐⭐ None | ⭐⭐⭐⭐⭐ 50+ pages | ⭐⭐⭐ Terminal | ⭐⭐⭐⭐ App + CLI | ⭐⭐⭐⭐⭐ TUI |
| **Policy/governance** | ⭐⭐⭐⭐ Cedar (unwired) | ⭐⭐⭐⭐⭐ Harness | ⭐⭐⭐⭐ RBAC | ⭐⭐⭐⭐⭐ Claude-specific | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐ Git-based |
| **Compiler/distribution** | ⭐⭐⭐⭐⭐ 8-stage PMPO | ⭐⭐ None | ⭐⭐⭐⭐ Good | ⭐⭐ None | ⭐⭐ None | ⭐⭐ None |
| **Production maturity** | ⭐⭐⭐ Beta | ⭐⭐⭐⭐⭐ Stable | ⭐⭐⭐⭐⭐ Stable | ⭐⭐⭐⭐⭐ Production | ⭐⭐⭐⭐⭐ Production | ⭐⭐⭐⭐⭐ Production |

### 15.2 UAR's Unique Strengths

1. **liter-llm (142+ providers)** — No other harness comes close
2. **8-stage PMPO compiler** — Unique agent distribution mechanism
3. **Cedar policy engine** — Most advanced governance (when wired)
4. **MCP-first architecture** — Correctly bets on the 2026 standard
5. **Dual streaming (OpenAI + AG-UI)** — Good UX flexibility

### 15.3 UAR's Critical Weaknesses

1. **No dynamic model routing** — The #1 competitive gap
2. **No prompt dialect engine** — Wastes model capability
3. **Incomplete A2A** — gRPC broken, module not exported
4. **Cedar unwired** — Policy engine is decoration, not enforcement
5. **RAG not knowledge runtime** — Missing verification, access control, audit
6. **Web UI below competitors** — Not S-tier

---

## 16. UAR + LibreFang Integration Strategy

### 16.1 The Vision: "Prometheus Fabric"

The combined system should be marketed as **"Prometheus Fabric"** — a complete agent stack:

```
┌─────────────────────────────────────────────────────────────┐
│                    PROMETHEUS FABRIC                        │
├─────────────────────────────────────────────────────────────┤
│  USER LAYER (LibreFang)                                     │
│  ├── Channel Adapters (45+ platforms)                       │
│  ├── Workflow Engine (approval gates, visual editor)        │
│  ├── Dashboard (50+ management pages)                       │
│  ├── Skill Marketplace (ClawHub)                            │
│  └── Desktop App (Electron/Tauri)                           │
├─────────────────────────────────────────────────────────────┤
│  AGENT LAYER (LibreFang + UAR)                              │
│  ├── Agent OS (LibreFang kernel)                            │
│  ├── P2P Protocol (BossFang)                                │
│  └── Agent Runtime (UAR)                                    │
├─────────────────────────────────────────────────────────────┤
│  HARNESS LAYER (UAR)                                        │
│  ├── Dynamic Model Router (NEW)                             │
│  ├── Prompt Dialect Engine (NEW)                            │
│  ├── liter-llm (142+ providers)                             │
│  ├── Context Manager (model-aware)                          │
│  ├── MCP Client (always-on)                                 │
│  ├── A2A Protocol (agent-to-agent)                          │
│  ├── AG-UI Streaming (real-time)                            │
│  ├── Multi-scope Memory                                     │
│  ├── Knowledge Service (RAG extracted)                      │
│  ├── Cedar Policy Engine (wired)                            │
│  └── 8-stage PMPO Compiler                                  │
├─────────────────────────────────────────────────────────────┤
│  SKILL LAYER (UAR + Prometheus Skills)                      │
│  ├── Prometheus Skill Pack (bundled)                        │
│  ├── Custom Skills (WASM, Native, MCP, Builtin)             │
│  └── Skill Activation Engine (with metrics)                 │
├─────────────────────────────────────────────────────────────┤
│  INFRASTRUCTURE LAYER                                       │
│  ├── Postgres (primary) / SurrealDB (experimental)          │
│  ├── S3/GCS (document storage)                              │
│  └── Vector DB (embeddings)                                 │
└─────────────────────────────────────────────────────────────┘
```

### 16.2 Integration Points

1. **LibreFang → UAR via A2A** — LibreFang's workflow engine sends tasks to UAR agents
2. **UAR → LibreFang via AG-UI** — UAR streams execution state to LibreFang's dashboard
3. **Shared Skill Registry** — UAR's compiled skills + LibreFang's marketplace = unified catalog
4. **Shared Model Router** — UAR's `liter-llm` + router becomes the single LLM layer for both
5. **Shared Memory** — UAR's multi-scope memory serves both LibreFang's agents and UAR's native agents

---

## 17. Implementation Plan: Critical Changes for Production Release

### Phase 1: Foundation (Weeks 1-4) — P0 Critical

**Goal:** Fix the broken and incomplete core systems.

| Task | Owner | Effort | Description |
|---|---|---|---|
| Fix A2A gRPC | Backend | 1 week | Fix tonic-build v0.14 API changes, export gRPC module, make transport functional |
| Implement Postgres credential store | Backend | 1 week | Multi-tenant encryption for Postgres (currently only SurrealDB) |
| Add dynamic model router | Backend | 2 weeks | `Router` module with capability registry, cost calculator, health monitor |
| Add prompt dialect engine | Backend | 1 week | `PromptDialectEngine` with XML/JSON/Markdown/GLM transformations |
| Fix MemPalace | Backend | 3 days | Resolve `rusqlite` / `libsqlite3-sys` version conflicts |
| Unpin dependencies | DevOps | 3 days | Use semver ranges instead of exact pins for `surrealdb`, `pgvector`, etc. |
| Add Docker Compose | DevOps | 2 days | Full stack (UAR + Postgres + Vector DB) for local evaluation |

### Phase 2: Intelligence (Weeks 5-8) — P1 High Priority

**Goal:** Make the harness model-aware and cost-optimal.

| Task | Owner | Effort | Description |
|---|---|---|---|
| Wire Cedar policy engine | Backend | 1 week | Replace keyword heuristic with actual Cedar evaluation for tool approval |
| Add per-model context strategy | Backend | 1 week | Model-aware chunking, placement, compression |
| Add cost tracking | Backend | 1 week | Per-model, per-task spend tracking with budget alerts |
| Add skill activation metrics | Backend | 1 week | Precision/recall tracking per skill, per model |
| Add skill schema transformation | Backend | 1 week | Transform skills to match model's native tool format |
| Extract RAG to Knowledge Service | Backend | 2 weeks | Separate MCP service with query decomposition, verification, audit |
| Add provider health monitoring | Backend | 3 days | Real-time health checks, automatic failover |
| Add model capability registry | Backend | 1 week | Import benchmark data from 2026 research, update regularly |

### Phase 3: UX & Integration (Weeks 9-12) — P2 Medium Priority

**Goal:** Improve user experience and LibreFang integration.

| Task | Owner | Effort | Description |
|---|---|---|---|
| Redesign web UI configuration | Frontend | 2 weeks | Cherry Studio-inspired provider configuration, unified settings page |
| Add model comparison dashboard | Frontend | 1 week | Side-by-side benchmark scores, costs, capabilities |
| Add real-time agent state visualization | Frontend | 1 week | Reasoning steps, tool calls, context usage, routing decisions |
| Add cost tracking dashboard | Frontend | 3 days | Per-model, per-task spend with budget alerts |
| Integrate with LibreFang A2A | Integration | 1 week | LibreFang tasks → UAR agents via A2A protocol |
| Integrate with LibreFang AG-UI | Integration | 1 week | UAR streams → LibreFang dashboard via AG-UI |
| Shared model router with LibreFang | Integration | 3 days | UAR's `liter-llm` + router becomes LibreFang's LLM layer |
| Add Prometheus Skill Pack | Content | 1 week | Bundle all Prometheus skills with the UAR distribution |

### Phase 4: Specification & Distribution (Weeks 13-16) — P2 Medium Priority

**Goal:** Update the agent specification and distribution model.

| Task | Owner | Effort | Description |
|---|---|---|---|
| Version agent spec to v2.0 | Spec | 1 week | Add model_requirements, prompt_dialect, rag_configuration, context_strategy, api_harness |
| Update compiler for v2.0 | Backend | 1 week | PMPO stages updated to handle new spec fields |
| Add conformance testing | Backend | 1 week | Validate that agents can run with declared requirements |
| Create agent template library | Content | 1 week | Pre-built templates for common agent types (coding, vision, terminal, etc.) |
| Documentation overhaul | Docs | 2 weeks | Update all docs for new architecture, routing, dialects |
| Production deployment guide | Docs | 1 week | Kubernetes, Docker, monitoring, scaling |
| Benchmark suite | QA | 1 week | Automated tests for skill activation, routing accuracy, context efficiency |

### Phase 5: Polish & Release (Weeks 17-20) — P3 Polish

**Goal:** Production readiness and competitive positioning.

| Task | Owner | Effort | Description |
|---|---|---|---|
| Performance optimization | Backend | 2 weeks | Profile and optimize the hot path (router, dialect engine, context manager) |
| Security audit | Security | 1 week | Penetration testing, prompt injection resistance, RBAC validation |
| Load testing | QA | 1 week | 1000+ concurrent agents, failover under load |
| Documentation finalization | Docs | 1 week | API docs, architecture diagrams, deployment guides |
| Release candidate | All | 1 week | RC1, bug fixes, RC2, GA |
| Marketing materials | Marketing | 1 week | Comparison docs, benchmark results, case studies |

### Total Timeline: 20 weeks (5 months) to production-ready

### Resource Requirements

- **Backend engineers:** 3-4 (Rust, LLM APIs, distributed systems)
- **Frontend engineers:** 2 (React/TypeScript, real-time streaming)
- **DevOps:** 1 (Kubernetes, Docker, CI/CD)
- **QA/Security:** 1 (testing, penetration testing)
- **Technical writer:** 1 (documentation, specifications)
- **Total:** 8-10 people for 5 months

---

## 18. Prometheus Skill Pack Integration

### 18.1 The Goal

All Prometheus Fabric skills should be **bundled with the UAR** as a single offering. Users should:
1. Install UAR
2. Have all Prometheus skills available out-of-the-box
3. Be able to add custom skills via the compiler or marketplace

### 18.2 Integration Strategy

1. **Skill Pack as a Git Submodule** — `prometheus-skills` repo linked into UAR's `skills/` directory
2. **Pre-compiled AgentDescriptors** — All Prometheus skills are compiled and signed at release time
3. **Lazy Loading** — Skills are loaded into memory only when first activated
4. **Version Pinning** — Each UAR release pins to a specific skill pack version
5. **Update Mechanism** — `uar skill update` fetches latest skill pack and recompiles

### 18.3 Skill Pack Structure

```
prometheus-skills/
├── README.md
├── VERSION
├── skills/
│   ├── coding/
│   │   ├── react-component.agent.md
│   │   ├── rust-refactor.agent.md
│   │   └── flutter-widget.agent.md
│   ├── infrastructure/
│   │   ├── terraform-plan.agent.md
│   │   └── kubernetes-deploy.agent.md
│   ├── data/
│   │   ├── sql-query.agent.md
│   │   └── data-analysis.agent.md
│   └── communication/
│       ├── email-draft.agent.md
│       └── meeting-summary.agent.md
└── tests/
    └── activation-tests/
```

Each `.agent.md` file is compiled via the 8-stage PMPO pipeline into a signed AgentDescriptor that the UAR can load and execute.

### 18.4 Compilation Model

The UAR's compiler should be extended to support:
1. **Batch compilation** — Compile all skills in a directory at once
2. **Incremental compilation** — Only recompile changed skills
3. **Dependency resolution** — Skills can depend on other skills (import system)
4. **Version management** — Skills declare minimum UAR version
5. **Signature verification** — All skills are signed; UAR verifies on load

### 18.5 Distribution

The skill pack should be distributed as:
1. **GitHub Release Asset** — Pre-compiled `.agent` files attached to each UAR release
2. **Docker Volume** — Mounted into the UAR container at runtime
3. **Database Seeding** — Loaded into the database on first startup
4. **A2A Registry** — Available for remote agents to discover and use

---

## 19. Conclusion

The Universal Agent Runtime is a **technically impressive foundation** with unique strengths (liter-llm, PMPO compiler, Cedar policy, MCP-first architecture). However, it has **critical gaps** that prevent it from competing with Hermes, OpenClaw, Claude Code, or OpenCode in July 2026.

**The three most important changes are:**

1. **Dynamic model routing** — The UAR must become intelligent about *which* model to use, not just *how* to call it. This is the single biggest competitive gap.

2. **Prompt dialect engine** — The UAR must adapt prompts to each model's native dialect (XML for Claude, JSON for OpenAI, Markdown for Kimi, etc.). Without this, the UAR is wasting model capability.

3. **A2A completion + LibreFang integration** — The UAR should become the "harness kernel" and LibreFang the "agent OS." A2A is the glue. Fix the gRPC implementation and establish the integration pattern.

**With these changes and the 20-week implementation plan, the UAR + LibreFang combination can become a premier agent stack** — one that no single competitor matches in breadth (142+ providers), governance (Cedar), distribution (PMPO compiler), and ecosystem (LibreFang's 45+ channels + marketplace).

The Prometheus Fabric vision is achievable. The foundation exists. The gaps are identifiable and fixable. The timeline is aggressive but realistic with the right team.

---

*This analysis was compiled from deep codebase exploration of ~59,400 lines of UAR code and ~33 crates of LibreFang code, competitive harness research (Hermes, OpenClaw, Claude Code, Codex, OpenCode), A2A/AG-UI/MCP protocol specifications, 2026 RAG best practices, and frontier model comparison data covering 11 models across 14+ dimensions.*
