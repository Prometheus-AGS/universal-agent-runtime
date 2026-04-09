Good context. Now I have a clear picture of the existing `NormalizedEvent` stream, `LiterLlmDriver`, and where the seams are. Let me lay out a comprehensive plan.

---

## UAR: Anthropic Native Driver + Universal Tool Semantics Normalization

### Strategic framing

The goal has two distinct but tightly coupled parts:

**Part A — Anthropic as a first-class driver.** Right now Anthropic calls go through liter-llm, which speaks OpenAI-compatible wire format. That means you lose prompt caching, extended thinking, `cache_control` blocks, image/document content types, and the Anthropic streaming event model. You need a native `AnthropicDriver` that speaks the Messages API directly.

**Part B — Tool semantic normalization.** Any backend (candle-vllm / Groq / Ollama / Mistral / anything) should emit tool calls that are structurally indistinguishable from what Claude Sonnet 4.6 produces, so downstream consumers — OpenFang tool runner, AG-UI events, gen_ui_core — never need to know what the real backend was.

---

### New crate structure

```
uar/crates/
  uar-anthropic/          ← NEW: native Anthropic Messages API client
    src/
      client.rs           ← reqwest-based, streaming SSE
      types.rs            ← request/response models (serde)
      cache.rs            ← CacheControlBlock, CacheStrategy
      streaming.rs        ← AnthropicStreamDecoder → NormalizedEvent
      error.rs

  uar-tool-normalizer/    ← NEW: semantic shim layer
    src/
      capability.rs       ← ModelCapabilityRegistry
      injector.rs         ← PromptToolInjector (XML schema in system prompt)
      extractor.rs        ← ToolCallExtractor (regex + JSON parse from text)
      wrapper.rs          ← ToolNormalizerDriver (wraps any DriverTrait)
      shim.rs             ← ClaudeSonnet46Shim (the public surface)

  uar-driver-core/        ← EXISTING: extend DriverTrait
```

---

### Phase 1 — `uar-anthropic`: native Messages API client

**Goal:** Replace liter-llm for all `anthropic/*` model addresses. Own the wire format entirely.

**`types.rs` — key structures:**

```rust
// Request-side
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: Vec<SystemBlock>,      // supports cache_control
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolDef>>,
    pub tool_choice: Option<ToolChoice>,
    pub stream: bool,
    pub thinking: Option<ThinkingConfig>,  // extended thinking
    pub metadata: Option<RequestMetadata>,
}

pub struct SystemBlock {
    pub r#type: String,          // "text"
    pub text: String,
    pub cache_control: Option<CacheControl>,   // {"type": "ephemeral"}
}

pub struct CacheControl {
    pub r#type: CacheControlType,   // Ephemeral | Persistent (future)
}

// Tool definition with cache_control on the tools array itself
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub cache_control: Option<CacheControl>,  // put on last tool to cache the list
}

// Response-side
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    Thinking { thinking: String, signature: String },
}

pub struct MessagesResponse {
    pub id: String,
    pub model: String,
    pub stop_reason: StopReason,
    pub usage: Usage,
    pub content: Vec<ContentBlock>,
}

pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}
```

**`streaming.rs` — SSE decoder → NormalizedEvent:**

The Anthropic streaming protocol emits these event types:
```
message_start → content_block_start → content_block_delta (×N) 
→ content_block_stop → message_delta → message_stop
```

The decoder must maintain a `StreamState` that tracks:
- Current `content_block` index and type (`text` vs `tool_use`)
- Accumulated `tool_use` input JSON string (partial JSON across deltas)
- Usage accumulation (cache tokens land in `message_delta`)

Output maps to existing `NormalizedEvent` variants — but you'll need two new variants:
```rust
pub enum NormalizedEvent {
    // existing
    MessageDelta { text: String },
    ToolCallDelta { index: usize, id: String, name: String, input_fragment: String },
    ToolCallComplete { index: usize, id: String, name: String, input: serde_json::Value },
    Done { stop_reason: String },
    // new
    ThinkingDelta { text: String },           // extended thinking stream
    UsageUpdate { usage: NormalizedUsage },   // cache hit/miss accounting
}

pub struct NormalizedUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_write_tokens: u32,   // cache_creation_input_tokens
    pub cache_read_tokens: u32,    // cache_read_input_tokens
    pub estimated_cost_usd: Option<f64>,
}
```

**`cache.rs` — CacheStrategy:**

```rust
pub struct CacheStrategy {
    pub cache_system_prompt: bool,
    pub cache_tools: bool,
    pub cache_conversation_turns: usize,  // N most recent turns to mark cacheable
    pub min_tokens_to_cache: u32,          // don't annotate blocks < 1024 tokens (API minimum)
}

impl CacheStrategy {
    pub fn apply(&self, req: &mut MessagesRequest) { ... }
    // Annotates the last SystemBlock, last ToolDef, and up to N message turns
    // with cache_control: {type: "ephemeral"}
}
```

This is what lets you get 90% cost reduction on repeated agent runs with the same system prompt + tools.

---

### Phase 2 — `ModelCapabilityRegistry`

Every backend model has a different tool calling capability level. You need a registry that maps model strings to a capability profile:

```rust
pub enum ToolCallCapability {
    /// Native structured tool calls (Anthropic, OpenAI, Gemini, Mistral Large)
    Native,
    /// Grammar-constrained JSON output mode (Hermes, Functionary, candle-vllm with constrained decoding)
    GrammarConstrained,
    /// Instruction-tuned to follow XML tool call format (Qwen2.5, DeepSeek, Llama-3.1 instruct)
    InstructionTuned,
    /// No tool call support — pure text, must parse aggressively
    TextOnly,
}

pub struct ModelCapabilityProfile {
    pub tool_call_capability: ToolCallCapability,
    pub supports_system_prompt: bool,
    pub supports_streaming: bool,
    pub max_context_tokens: u32,
    pub supports_parallel_tool_calls: bool,
    pub tool_call_format: ToolCallFormat,  // Hermes / Llama / generic XML
}

pub struct ModelCapabilityRegistry {
    profiles: HashMap<String, ModelCapabilityProfile>,
}
```

Static profiles for known models are compiled in. Unknown models default to `InstructionTuned` with generic XML format. The registry exposes:

```rust
impl ModelCapabilityRegistry {
    pub fn get(&self, model: &str) -> &ModelCapabilityProfile;
    pub fn register(&mut self, pattern: &str, profile: ModelCapabilityProfile);
    // supports glob patterns: "qwen2.5*", "candle-vllm/*"
}
```

---

### Phase 3 — `ToolNormalizerDriver` (the shim)

This is the core of Part B. It wraps any `DriverTrait` implementor and makes it behave like Claude Sonnet 4.6 from the caller's perspective.

```rust
pub struct ToolNormalizerDriver<D: DriverTrait> {
    inner: D,
    registry: Arc<ModelCapabilityRegistry>,
    model: String,
}

impl<D: DriverTrait> DriverTrait for ToolNormalizerDriver<D> {
    async fn stream(&self, req: NormalizedRequest) -> impl Stream<Item = NormalizedEvent> {
        let profile = self.registry.get(&self.model);
        
        match profile.tool_call_capability {
            ToolCallCapability::Native => {
                // Pass through — model speaks tool calls natively
                // Only normalize response format if needed (e.g. OpenAI → Anthropic shape)
                self.inner.stream(req).await
                    .map(|e| self.normalize_event(e))
            }
            ToolCallCapability::GrammarConstrained => {
                // Inject JSON schema as constrained grammar parameter
                // Parse structured JSON output back into ToolCallComplete events
                let transformed = self.inject_grammar_constraints(req);
                self.inner.stream(transformed).await
                    .map(|e| self.extract_grammar_tool_calls(e))
            }
            ToolCallCapability::InstructionTuned => {
                // Inject XML tool schema into system prompt
                // Parse <tool_call> blocks from text stream
                let transformed = self.inject_xml_tool_prompt(req);
                self.inner.stream(transformed).await
                    .via(self.xml_tool_call_extractor())
            }
            ToolCallCapability::TextOnly => {
                // Full prompt engineering — include examples + schema
                // Aggressive XML/JSON extraction from raw text
                let transformed = self.inject_few_shot_tool_prompt(req);
                self.inner.stream(transformed).await
                    .via(self.aggressive_tool_extractor())
            }
        }
    }
}
```

**`injector.rs` — XML tool injection for InstructionTuned models:**

```
<tool_definitions>
  <tool name="search_memory">
    <description>Search the agent's long-term memory store.</description>
    <parameters type="object">
      <property name="query" type="string" required="true">
        The search query.
      </property>
      <property name="limit" type="integer" required="false" default="5">
        Maximum results to return.
      </property>
    </parameters>
  </tool>
  ...
</tool_definitions>

To call a tool, emit ONLY this exact format and nothing else before or after:
<tool_call>
{"name": "search_memory", "input": {"query": "user preferences"}}
</tool_call>

After receiving tool results, continue your response normally.
```

**`extractor.rs` — streaming XML extractor:**

The tricky part is streaming. You can't wait for `</tool_call>` to close before emitting events — you need to emit `ToolCallDelta` events as the JSON inside the tag streams in. The extractor is a state machine:

```rust
enum ExtractorState {
    Text,                          // emitting MessageDelta events
    InToolCallTag { buffer: String }, // accumulating JSON
    Done,
}
```

When `<tool_call>` is detected in the text stream:
1. Suppress the tag text from `MessageDelta` output
2. Buffer everything until `</tool_call>`
3. Parse the buffered JSON
4. Emit `ToolCallComplete` with a synthetic `id` (uuid v4)
5. Resume `MessageDelta` emission

For streaming fidelity, emit `ToolCallDelta` fragments as the JSON accumulates inside the tag.

---

### Phase 4 — `ClaudeSonnet46Shim`

This is the public-facing facade. It takes any `(provider, model)` pair and wraps it in the normalizer, presenting a surface that always behaves like Claude Sonnet 4.6:

```rust
pub struct ClaudeSonnet46Shim {
    inner: Box<dyn DriverTrait>,
    cache_strategy: CacheStrategy,
    capability_registry: Arc<ModelCapabilityRegistry>,
}

impl ClaudeSonnet46Shim {
    /// Create from any provider/model string
    pub fn for_model(provider: &str, model: &str, config: ShimConfig) -> Self;
    
    /// The shim always reports itself as claude-sonnet-4-6 to callers
    pub fn reported_model(&self) -> &'static str { "claude-sonnet-4-6" }
    
    /// Full conversation turn with automatic caching annotation
    pub async fn invoke(
        &self,
        system: &str,
        tools: &[ToolDef],
        messages: &[Message],
    ) -> Result<MessagesResponse>;
    
    /// Streaming variant
    pub async fn stream(
        &self,
        system: &str, 
        tools: &[ToolDef],
        messages: &[Message],
    ) -> impl Stream<Item = NormalizedEvent>;
}
```

The `ShimConfig` controls:
- `cache_strategy: CacheStrategy` — how aggressively to apply `cache_control` annotations
- `thinking_budget: Option<u32>` — enable extended thinking if backend supports it
- `max_tokens: u32`
- `tool_call_retry_budget: u32` — if extracted tool call JSON fails to parse, retry with error feedback

---

### Phase 5 — Integration into UAR's request pipeline

Right now UAR exposes an OpenAI-compatible endpoint. You need to add a native Anthropic-compatible endpoint AND use the shim internally for all agent turns.

**New Axum routes in UAR:**

```
POST /v1/messages                    ← Anthropic Messages API (pass-through + caching)
POST /v1/messages/stream             ← Anthropic streaming
GET  /v1/models                      ← augmented with shim-advertised models
POST /v1/shim/{provider}/{model}/messages  ← explicit shim endpoint
```

**Internal agent turn loop changes:**

```rust
// Before: LiterLlmDriver speaks OpenAI wire format
// After: dispatch based on provider

let driver: Box<dyn DriverTrait> = match route_provider(model_addr) {
    Provider::Anthropic => {
        Box::new(AnthropicDriver::new(config, cache_strategy))
    }
    Provider::Other(p) => {
        let base_driver = LiterLlmDriver::new(p, config);
        let profile = capability_registry.get(model_addr);
        Box::new(ToolNormalizerDriver::new(base_driver, profile, cache_strategy))
    }
};
```

The `NormalizedEvent` stream coming out is always in Anthropic shape — `ContentBlockStart`, `ContentBlockDelta`, `ContentBlockStop`, `MessageDelta`, `MessageStop` — regardless of what emitted it.

---

### Phase 6 — Prompt cache accounting in SurrealDB

Cache hits/misses need to be tracked for cost accounting and to inform future caching decisions:

```rust
// surreal-memory-server schema addition
pub struct CacheUsageRecord {
    pub session_id: Uuid,
    pub agent_id: String,
    pub model: String,
    pub timestamp: DateTime<Utc>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_write_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_savings_usd: f64,
}
```

The `UsageUpdate` `NormalizedEvent` from Phase 1 feeds this record. Over time UAR can adapt cache strategy based on observed hit rates per agent.

---

### Phase 7 — Cedar policy surface for caching

Not every tenant should be able to use prompt caching (it has cost implications for cache writes). Add Cedar policies:

```cedar
// Allow premium tier to use prompt caching
permit(
    principal in Role::"premium",
    action == Action::"invoke_with_cache",
    resource is Agent
);

// Restrict extended thinking to enterprise tier  
permit(
    principal in Role::"enterprise",
    action == Action::"invoke_with_thinking",
    resource is Agent
);
```

---

### Implementation sequence

| Phase | Crate | Effort | Dependency |
|---|---|---|---|
| 1 | `uar-anthropic` types + client | 2 days | none |
| 2 | `uar-anthropic` streaming decoder | 2 days | Phase 1 |
| 3 | `uar-anthropic` cache strategy | 1 day | Phase 1 |
| 4 | `ModelCapabilityRegistry` | 1 day | none |
| 5 | `ToolNormalizerDriver` + XML injector | 3 days | Phase 4 |
| 6 | `ToolCallExtractor` streaming state machine | 2 days | Phase 5 |
| 7 | `ClaudeSonnet46Shim` facade | 1 day | all above |
| 8 | UAR Axum routes + dispatch | 2 days | all above |
| 9 | SurrealDB cache accounting | 1 day | Phase 2 |
| 10 | Cedar policy extension | 0.5 day | Phase 8 |

**Total: ~16 dev days.** Phases 1–3 (native Anthropic with caching) can ship independently as immediate value. Phases 4–7 (tool normalization shim) ship as the second milestone.

---

### Key decision: liter-llm stays for non-Anthropic routing

Don't rip out liter-llm — it handles 142+ provider connections you don't want to reimplement. The `ToolNormalizerDriver` wraps it. Only Anthropic gets a native driver because it's the normalization target AND the most latency/cost-sensitive use case.

---

Want me to start with Phase 1 and produce the full `uar-anthropic` crate skeleton, or draft the `ToolNormalizerDriver` + `ExtractorState` machine first?