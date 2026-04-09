## Context

UAR's `/v1/messages` endpoint currently receives Anthropic-shaped requests, converts them to OpenAI format, runs through liter-llm, then converts the NormalizedEvent stream back to Anthropic SSE format. This round-trip loses prompt caching semantics, extended thinking blocks, and native content types. The existing `api_messages()` handler in `src/server.rs` (lines 2206-2550+) already has Anthropic request/response types defined — the refactoring replaces the middle section (liter-llm driver) with a native Anthropic client while keeping the request parsing and response formatting.

The tool normalization layer sits between the `LlmDriver` trait and any non-Anthropic backend. It makes Groq/Ollama/Mistral/candle-vllm tool calls look identical to Claude's, so downstream consumers never need per-model logic.

**Current flow:** Request → convert to OpenAI → liter-llm → NormalizedEvent → convert to Anthropic SSE
**New flow:** Request → AnthropicDriver (native reqwest) → NormalizedEvent → Anthropic SSE (passthrough)

## Goals / Non-Goals

**Goals:**
- Claude Code can use UAR's `/v1/messages` as a drop-in Anthropic API replacement
- Full prompt caching with `cache_control` annotations (90% cost reduction on repeated agent turns)
- Extended thinking with streaming `ThinkingDelta` events
- Any backend model's tool calls normalized to Claude Sonnet 4.6 format
- Zero breaking changes to existing `/v1/chat/completions` OpenAI path
- Anthropic streaming SSE protocol matched exactly (event ordering, field names, types)

**Non-Goals:**
- Replacing liter-llm for non-Anthropic providers (it stays for 142+ providers)
- Building a separate `uar-anthropic` crate (implementation lives in `src/llm/` modules for now)
- Custom Anthropic SDK wrapper (we use raw reqwest for simplicity)
- Batch API support (streaming-only for now)
- Admin UI changes (existing provider/model pages work as-is)

## Decisions

### D1: AnthropicDriver as an LlmDriver Implementation
**Decision**: Implement `AnthropicDriver` as a new `LlmDriver` trait implementation in `src/llm/anthropic_driver.rs` using raw `reqwest` with SSE streaming.

**Rationale**: The `LlmDriver` trait is the existing abstraction. By implementing it, the `AnthropicDriver` plugs into the Orchestrator's tool loop without changes. Using reqwest directly (instead of an Anthropic SDK) gives full control over `cache_control` annotation, thinking config, and streaming behavior.

**Alternative considered**: Using the `anthropic-sdk` Rust crate — rejected because it may lag behind API changes, and we need fine-grained control over cache_control placement and SSE parsing.

### D2: SSE Decoder as a State Machine
**Decision**: The Anthropic SSE decoder maintains a `StreamState` tracking current content block index and type, accumulated tool use input JSON, and usage counters. It maps directly to existing `NormalizedEvent` variants.

**Rationale**: Anthropic's streaming model emits `content_block_start` → `content_block_delta` (×N) → `content_block_stop` per block, with blocks potentially being text, tool_use, or thinking. The state machine pattern handles interleaved blocks cleanly.

**Event mapping:**
| Anthropic SSE Event | NormalizedEvent |
|---------------------|-----------------|
| `message_start` | `StreamStart { request_id }` |
| `content_block_start` (type: text) | (internal state change, no event) |
| `content_block_delta` (type: text_delta) | `MessageDelta { text }` |
| `content_block_start` (type: thinking) | (internal state change) |
| `content_block_delta` (type: thinking_delta) | `ThinkingDelta { text }` |
| `content_block_start` (type: tool_use) | (internal state, extract id + name) |
| `content_block_delta` (type: input_json_delta) | `ToolCallDelta { call_index, id, name, arguments_delta }` |
| `content_block_stop` (tool_use) | `ToolCallComplete { call_index, id, name, arguments_json }` |
| `message_delta` | `Usage { ... }` (if usage present) |
| `message_stop` | `Done` |

### D3: CacheStrategy Annotation
**Decision**: `CacheStrategy` annotates request blocks with `cache_control: {"type": "ephemeral"}` before sending. Configurable per: system prompt, tools array (last tool), and N most recent conversation turns.

**Rationale**: Anthropic requires minimum 1024 tokens in a cacheable block. The strategy checks block size before annotating. System prompt and tools are nearly always worth caching (they repeat every turn). Recent conversation turns benefit from caching in multi-turn agents.

**Configuration:**
```rust
pub struct CacheStrategy {
    pub cache_system_prompt: bool,      // default: true
    pub cache_tools: bool,              // default: true
    pub cache_conversation_turns: usize, // default: 4 (last N user/assistant pairs)
    pub min_tokens_to_cache: u32,       // default: 1024 (Anthropic minimum)
}
```

### D4: ToolNormalizerDriver Wraps Non-Anthropic Drivers
**Decision**: `ToolNormalizerDriver<D: LlmDriver>` wraps any driver and intercepts the stream, applying tool-call extraction/normalization based on the model's capability profile.

**Rationale**: Different models have different tool-call capabilities: native (OpenAI, Anthropic, Gemini), grammar-constrained (candle-vllm), instruction-tuned (Qwen, Llama), or text-only. The normalizer makes them all produce identical `ToolCallDelta`/`ToolCallComplete` events.

**Capability detection:** Static profiles compiled into `ModelCapabilityRegistry` for known models, with glob pattern matching (e.g., `"qwen2.5*"` → InstructionTuned). Unknown models default to InstructionTuned with generic XML format.

### D5: Provider Dispatch at Orchestrator Construction
**Decision**: When building an `Orchestrator`, detect the provider from the model string (`anthropic/*` → `AnthropicDriver`, else → `ToolNormalizerDriver<LiterLlmDriver>`).

**Rationale**: The Orchestrator owns the tool loop. By selecting the driver at construction time, all tool iterations use the same driver consistently. The dispatch happens in `start_server()` when building the default Orchestrator and in `api_messages()` for per-request model overrides.

### D6: Extend LlmRequest for Anthropic-Specific Fields
**Decision**: Add optional fields to `LlmRequest`: `cache_strategy: Option<CacheStrategy>`, `thinking_config: Option<ThinkingConfig>`, `anthropic_system: Option<Vec<serde_json::Value>>`. These are ignored by `LiterLlmDriver`.

**Rationale**: The `LlmDriver` trait takes `LlmRequest`. Rather than creating a separate Anthropic request type, extending the existing struct keeps the interface uniform. Non-Anthropic drivers simply ignore the new fields.

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|------|----------|------------|
| Anthropic API changes break wire format | Medium | Pin to known API version (`2023-06-01` header), add version detection |
| XML tool extraction misparses complex tool call JSON | Medium | Robust regex + JSON validation; retry with error feedback (budget: 2 retries) |
| Cache strategy wastes cache writes on short prompts | Low | `min_tokens_to_cache` threshold (1024 tokens) prevents unprofitable caching |
| Two driver paths (Anthropic vs liter-llm) means double the maintenance | Medium | Share `NormalizedEvent` model; both paths are testable independently |
| Extended thinking produces very long responses | Low | `budget_tokens` cap; surface ThinkingDelta in UI for transparency |
| reqwest SSE parsing fragile on chunked responses | Low | Use `reqwest-eventsource` or manual line-based SSE parser with reconnect |

## Migration Plan

### Phase 1: AnthropicDriver + Streaming (ship independently)
1. Create `anthropic_types.rs` with full Anthropic request/response serde types
2. Create `anthropic_streaming.rs` SSE decoder → NormalizedEvent
3. Create `anthropic_cache.rs` CacheStrategy
4. Create `anthropic_driver.rs` implementing LlmDriver
5. Wire into `api_messages()` handler for `anthropic/*` models
6. Test with Claude Code against `/v1/messages`

### Phase 2: Tool Normalization (ship as second milestone)
7. Create `capability_registry.rs` with static model profiles
8. Create `tool_extractor.rs` streaming state machine
9. Create `tool_normalizer.rs` wrapper driver
10. Wire into Orchestrator for non-Anthropic models
11. Test with Qwen/Llama/Groq models producing tool calls

### Phase 3: Cache Accounting + Policy
12. Track cache usage in Prometheus metrics (`uar_cache_*` counters)
13. Add Cedar policy rules for caching authorization
14. Add admin UI cache analytics (optional)

### Rollback Strategy
- Feature-gated: `ANTHROPIC_NATIVE_DRIVER=true` env var enables the new path
- Default: false (existing liter-llm path unchanged)
- Rollback: set env var to false, restart
