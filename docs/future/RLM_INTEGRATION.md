# RLM Integration into Universal Agent Runtime

> **Status:** Future / Experimental — Not Yet Implemented  
> **Source:** MIT CSAIL paper [arXiv:2410.01855](https://arxiv.org/abs/2410.01855)  
> **Related Project:** [`prometheus/rlm`](../../../rlm/README.md)

---

## What Is RLM?

**Recursive Language Model (RLM)** is an *inference-time prompting strategy* — not a new model. It works with any existing LLM. The core idea:

> Instead of stuffing millions of tokens into the LLM's context window, RLM offloads the full context into an external **REPL** (e.g., Rhai or Python). The LLM writes and executes code to explore, decompose, and query that context via recursive `llm_query()` calls — processing only small, relevant chunks at a time.

### Three Stages

1. **Context Offloading** — Large context lives in a REPL variable, not the prompt
2. **Recursive Decomposition** — The LLM spawns sub-calls to process context chunks
3. **Aggregation** — Sub-results are merged in the REPL and a final answer is synthesized

### Benchmark Results (MIT Paper)

| Task | Context Size | Baseline Acc | RLM Acc | Complexity |
|------|-------------|-------------|---------|------------|
| S-NIAH (Needle-in-Haystack) | 8K–128K | 95.0% | 97.5% | O(1) |
| OOLONG | 32K–131K | 72.3% | 89.4% | O(n) |
| OOLONG-Pairs | 8K–32K | 45.2% | 78.6% | O(n²) |
| BrowseComp (100 docs) | — | 61.8% | 83.2% | O(n log n) |
| Code Repos | ~50K tokens | 68.5% | 84.1% | O(n) |

---

## The `prometheus/rlm` Project

The `rlm` workspace is a planned production Rust implementation of this paper, comprising:

| Crate | Role |
|-------|------|
| `rlm-core` | Core types, `RlmExecutor`, ports |
| `rlm-repl-rhai` | Sandboxed Rhai REPL backend |
| `rlm-server` | HTTP/SSE server (`/v1/rlm/execute`) |
| `rlm-ffi` | WASM bindings for Cherry Studio |
| **`rlm-uar-adapter`** | UAR integration adapter ← key integration point |

The `rlm-uar-adapter` crate maps `RlmEvent` → `UarEvent` and exposes RLM as an execution strategy inside UAR's `RunManager`. This is a first-class item in the RLM roadmap (Phase 2).

---

## Should We Do This?

### Reasons To Integrate

- **Already planned** — `rlm-uar-adapter` is a named crate in the `rlm` workspace roadmap
- **Solves a real scaling problem** — UAR's current `ContextStrategy` (SlidingWindow, ProgressiveSummarization) caps out around 128K tokens; RLM scales to millions
- **No retraining required** — Works with any LLM provider already supported (OpenAI, Groq, Anthropic, etc.)
- **Zero risk when disabled** — 100% opt-in via feature flags; existing users are unaffected
- **Natural fit** — `RunManager` already has `provider_registry`, `context_manager`, `skill_service` — all hook points for RLM routing
- **Cost savings** — For sparse long-context tasks (e.g. "find X in 200 documents"), RLM only pays for relevant chunks, not the full token count

### Honest Caveats

- **Latency increases** — RLM makes multiple serial LLM calls (O(n) minimum). For typical short-context queries it is strictly worse than direct chat
- **rlm crates not yet complete** — Layer 2 wiring depends on `rlm-core` and `rlm-repl-rhai` being implemented
- **Experimental** — Running a Rhai REPL with LLM-generated code in a sandboxed agent has security surface area; verify sandboxing before enabling in production
- **Prompt sensitivity** — Smaller or non-instruction-tuned models may not reliably follow REPL code generation patterns; best results with GPT-4-class models

---

## Integration Architecture

### Precedence Model (Three Scopes)

```
Per-request `rlm` field         ← HIGHEST (on/off per conversation)
        ↓
Agent artifact `runtime.rlm`    ← per-agent opt-in/out
        ↓
Global `config.yaml [rlm]`      ← server-wide default
        ↓
Default: disabled                ← LOWEST (safe default)
```

A master kill switch (`global.enabled = false`) prevents all lower scopes from enabling RLM, regardless of what they request.

An `auto_threshold_tokens` setting lets RLM self-activate only when estimated input exceeds a token count — so agents don't need to manually toggle it per request.

---

## Proposed Changes

### Layer 1 — Feature Flag Infrastructure (UAR only, do first)

#### `src/config.rs` — New `RlmConfig` struct

```rust
/// Configuration for the Recursive Language Model (RLM) execution strategy.
/// Experimental feature — disabled by default.
#[derive(Debug, Deserialize, Clone)]
pub struct RlmConfig {
    /// Master kill switch. Default: false (opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// Maximum REPL iterations per execution. Default: 50.
    #[serde(default = "RlmConfig::default_max_iterations")]
    pub max_iterations: u32,
    /// Maximum recursive call depth. Default: 1.
    #[serde(default = "RlmConfig::default_recursion_depth")]
    pub recursion_depth: u32,
    /// Context chunk size in tokens. Default: 4096.
    #[serde(default = "RlmConfig::default_chunk_size")]
    pub chunk_size: u32,
    /// Execution timeout in seconds. Default: 300.
    #[serde(default = "RlmConfig::default_timeout_seconds")]
    pub timeout_seconds: u64,
    /// Base URL of an external rlm-server. None = in-process adapter.
    #[serde(default)]
    pub server_url: Option<String>,
    /// Only activate RLM when estimated input tokens exceed this threshold.
    /// 0 = always use RLM when enabled.
    #[serde(default)]
    pub auto_threshold_tokens: u64,
}
```

Add to `AppConfig`:
```rust
#[serde(default)]
pub rlm: RlmConfig,
```

#### `src/uar/domain/artifact.rs` — Agent-level override

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub entry: String,
    #[serde(default)]
    pub protocols: HashMap<String, ProtocolConfig>,
    /// Per-agent RLM override. None = inherit global config.
    #[serde(default)]
    pub rlm: Option<AgentRlmConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRlmConfig {
    pub enabled: Option<bool>,
    pub max_iterations: Option<u32>,
    pub recursion_depth: Option<u32>,
    pub chunk_size: Option<u32>,
    pub auto_threshold_tokens: Option<u64>,
}
```

#### API layer — Conversation-level override (run request body)

```rust
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    // ... existing fields ...
    /// Optional per-conversation RLM override. Highest precedence.
    #[serde(default)]
    pub rlm: Option<ConversationRlmOverride>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConversationRlmOverride {
    pub enabled: Option<bool>,
    pub max_iterations: Option<u32>,
    pub recursion_depth: Option<u32>,
}
```

#### `example.config.yaml` — New `rlm` section

```yaml
# =============================================================================
# RLM (Recursive Language Model) - EXPERIMENTAL
# =============================================================================
rlm:
  enabled: false                  # Master switch (default: off)
  max_iterations: 50
  recursion_depth: 1
  chunk_size: 4096
  timeout_seconds: 300
  # server_url: "http://localhost:8090"   # External rlm-server (optional)
  auto_threshold_tokens: 0        # 0 = always use RLM when enabled
```

#### Agent artifact YAML example

```yaml
runtime:
  entry: "main"
  rlm:
    enabled: true                 # Opt-in at agent level
    max_iterations: 100
    auto_threshold_tokens: 50000  # Only engage RLM for inputs > 50K tokens
```

#### Precedence resolver (new helper in `RunManager`)

```rust
fn resolve_rlm_config(
    global: &RlmConfig,
    agent: Option<&AgentRlmConfig>,
    conversation: Option<&ConversationRlmOverride>,
) -> EffectiveRlmConfig { ... }
```

---

### Layer 2 — Execution Wiring (requires `rlm-core` + `rlm-repl-rhai`)

In `RunManager::start_run()`, after context management, before spawning the `Orchestrator`:

```rust
let effective_rlm = resolve_rlm_config(
    &self.global_rlm_config,
    artifact.runtime.rlm.as_ref(),
    rlm_override.as_ref(),
);

if effective_rlm.enabled {
    let estimated_tokens = estimate_tokens(&optimized_messages);
    if effective_rlm.auto_threshold_tokens == 0
        || estimated_tokens > effective_rlm.auto_threshold_tokens
    {
        // Route through RlmUarAdapter instead of standard Orchestrator
        return self.start_rlm_run(
            run_id, artifact, optimized_messages,
            effective_rlm, emitter, execution_session
        ).await;
    }
}
// Default path: Orchestrator (unchanged)
```

The `start_rlm_run` method:
1. Builds an `RlmRequest` from messages
2. Calls `RlmUarAdapter::execute()` (from `rlm-uar-adapter` crate)
3. Maps `RlmEvent` stream → `NormalizedEvent` stream
4. Emits events through the same `RunEventEmitter` — transparent to all consumers

---

## Implementation Phases

| Phase | Work | Depends On |
|-------|------|-----------|
| **1 (Now)** | Layer 1: Config structs, `AgentArtifact` extension, API param, precedence resolver, YAML example | Nothing — pure config/struct work |
| **2** | `rlm-core` + `rlm-repl-rhai` implementation in `prometheus/rlm` | Phase 1 of RLM project |
| **3** | Layer 2: Wire `rlm-uar-adapter` into `RunManager` | Phase 2, Layer 1 |
| **4** | Enable `auto_threshold_tokens` auto-routing in production | Phase 3 + load testing |

---

## Security Considerations

- The Rhai REPL executes LLM-generated code. Verify `rlm-repl-rhai` sandboxing and memory limits before production use.
- API keys must never be accessible from within the REPL environment.
- Consider rate-limiting RLM runs separately from standard runs due to higher LLM call volume.

---

## References

- [RLM Paper (arXiv:2410.01855)](https://arxiv.org/abs/2410.01855)
- [`prometheus/rlm` README](../../../rlm/README.md)
- [`RunManager` source](../../src/uar/runtime/manager.rs)
- [`AgentArtifact` source](../../src/uar/domain/artifact.rs)
- [`AppConfig` source](../../src/config.rs)
