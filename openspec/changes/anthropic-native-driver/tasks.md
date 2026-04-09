## 1. Anthropic Types (Wire Protocol)

- [x] 1.1 Create `src/llm/anthropic_types.rs` with Anthropic Messages API request types: `MessagesRequest`, `SystemBlock`, `Message`, `ContentBlock`, `ToolDef`, `CacheControl`, `ThinkingConfig`
- [x] 1.2 Add Anthropic response types: `MessagesResponse`, `ContentBlockResponse` (Text, ToolUse, Thinking), `Usage` (with cache_creation_input_tokens, cache_read_input_tokens), `StopReason`
- [x] 1.3 Add Anthropic streaming SSE event types: `StreamEvent` enum with `MessageStart`, `ContentBlockStart`, `ContentBlockDelta`, `ContentBlockStop`, `MessageDelta`, `MessageStop`, `Error`
- [x] 1.4 Add `mod anthropic_types;` to `src/llm/mod.rs`
- [x] 1.5 Verify all types serialize/deserialize correctly with unit tests against known API payloads

## 2. Anthropic SSE Streaming Decoder

- [x] 2.1 Create `src/llm/anthropic_streaming.rs` with `StreamState` struct tracking: current block index, block type, accumulated tool JSON, usage counters
- [x] 2.2 Implement `decode_sse_line()` that parses `event:` and `data:` SSE fields into `StreamEvent`
- [x] 2.3 Implement `StreamState::process_event()` mapping each `StreamEvent` to zero or more `NormalizedEvent`s
- [x] 2.4 Handle text blocks: `content_block_start(text)` → state change, `content_block_delta(text_delta)` → `MessageDelta`
- [x] 2.5 Handle tool_use blocks: `content_block_start(tool_use)` → capture id+name, `content_block_delta(input_json_delta)` → `ToolCallDelta`, `content_block_stop` → `ToolCallComplete` with assembled JSON
- [x] 2.6 Handle thinking blocks: `content_block_start(thinking)` → state change, `content_block_delta(thinking_delta)` → `ThinkingDelta`
- [x] 2.7 Handle interleaved blocks (text → tool_use → text) with correct block index tracking
- [x] 2.8 Map `message_delta` with usage to `Usage` event (extract cache_read/write tokens)
- [x] 2.9 Map `message_stop` to `Done` event
- [x] 2.10 Handle `error` SSE events by emitting `Error` NormalizedEvent
- [x] 2.11 Add `mod anthropic_streaming;` to `src/llm/mod.rs`
- [x] 2.12 Add unit tests: decode a full streaming conversation with text + tool_use + thinking blocks

## 3. Cache Strategy

- [x] 3.1 Create `src/llm/anthropic_cache.rs` with `CacheStrategy` struct: `cache_system_prompt`, `cache_tools`, `cache_conversation_turns`, `min_tokens_to_cache`
- [x] 3.2 Implement `CacheStrategy::apply()` that annotates `MessagesRequest` blocks with `cache_control: {"type": "ephemeral"}`
- [x] 3.3 Annotate the last `SystemBlock` if `cache_system_prompt` is true and block exceeds min token threshold
- [x] 3.4 Annotate the last `ToolDef` if `cache_tools` is true
- [x] 3.5 Annotate the last N user/assistant message pairs if `cache_conversation_turns > 0`
- [x] 3.6 Add `Default` impl with production defaults: system=true, tools=true, turns=4, min=1024
- [x] 3.7 Add `mod anthropic_cache;` to `src/llm/mod.rs`
- [x] 3.8 Add unit test: verify cache_control annotations on a sample request

## 4. AnthropicDriver Implementation

- [x] 4.1 Create `src/llm/anthropic_driver.rs` with `AnthropicDriver` struct holding: `reqwest::Client`, `api_key`, `base_url`, `model`, `cache_strategy`, `thinking_config`
- [x] 4.2 Implement `AnthropicDriver::new()` constructor from `LlmConfig` + optional `CacheStrategy`
- [x] 4.3 Implement `LlmDriver::stream()`: build `MessagesRequest` from `LlmRequest`, apply cache strategy, POST to `/v1/messages` with `stream: true`
- [x] 4.4 Convert `LlmRequest.messages` (JSON values) to Anthropic `Message` format (handle role mapping, content blocks, tool results)
- [x] 4.5 Convert `LlmRequest.tools` (OpenAI function schema) to Anthropic `ToolDef` format
- [x] 4.6 Set headers: `anthropic-version: 2023-06-01`, `x-api-key`, `content-type: application/json`
- [x] 4.7 Parse SSE response stream using `anthropic_streaming::StreamState` decoder
- [x] 4.8 Return `Pin<Box<dyn Stream<Item = Result<NormalizedEvent>>>>` matching `LlmDriver` trait signature
- [x] 4.9 Handle HTTP error responses (400, 401, 429, 500) with appropriate `Error` events
- [x] 4.10 Implement rate limit backoff on 429 responses using `retry-after` header
- [x] 4.11 Add `mod anthropic_driver;` to `src/llm/mod.rs`
- [x] 4.12 Add integration test: mock Anthropic SSE response, verify NormalizedEvent stream

## 5. Extended Thinking Support

- [x] 5.1 Add `ThinkingConfig` struct to `anthropic_types.rs`: `enabled: bool`, `budget_tokens: u32`
- [x] 5.2 When `ThinkingConfig` is provided, include `thinking` field in `MessagesRequest`
- [x] 5.3 Verify `ThinkingDelta` events stream correctly through the decoder (already handled in task 2.6)
- [x] 5.4 Add config option `UAR_LLM__THINKING_BUDGET` for global thinking budget default

## 6. Model Capability Registry

- [x] 6.1 Create `src/llm/capability_registry.rs` with `ToolCallCapability` enum: Native, GrammarConstrained, InstructionTuned, TextOnly
- [x] 6.2 Define `ModelCapabilityProfile` struct: tool_call_capability, supports_system_prompt, supports_streaming, max_context_tokens, supports_parallel_tool_calls, tool_call_format
- [x] 6.3 Create `ModelCapabilityRegistry` with HashMap<String, Profile> and glob pattern matching
- [x] 6.4 Populate static profiles for known models: anthropic/* (Native), openai/* (Native), groq/* (Native), mistral/* (Native), qwen* (InstructionTuned), llama* (InstructionTuned), deepseek* (InstructionTuned)
- [x] 6.5 Implement `get()` with glob pattern matching and `register()` for custom profiles
- [x] 6.6 Default unknown models to InstructionTuned with generic XML format
- [x] 6.7 Add `mod capability_registry;` to `src/llm/mod.rs`
- [x] 6.8 Add unit test: verify known models resolve to correct profiles

## 7. XML Tool Injector

- [x] 7.1 Create `src/llm/xml_tool_injector.rs` with `inject_xml_tools()` function
- [x] 7.2 Convert OpenAI tool definitions to XML `<tool_definitions>` block with `<tool>`, `<description>`, `<parameters>` elements
- [x] 7.3 Add instruction text: "To call a tool, emit ONLY: `<tool_call>{JSON}</tool_call>`"
- [x] 7.4 Prepend XML block to existing system prompt (preserve original content)
- [x] 7.5 Handle nested JSON Schema properties → XML `<property>` elements with type, required, description
- [x] 7.6 Add `mod xml_tool_injector;` to `src/llm/mod.rs`
- [x] 7.7 Add unit test: convert sample tools to XML, verify structure

## 8. Tool Call Extractor (Streaming State Machine)

- [x] 8.1 Create `src/llm/tool_extractor.rs` with `ExtractorState` enum: Text, InToolCallTag { buffer: String }, Done
- [x] 8.2 Implement `ToolCallExtractor` struct with `process_delta()` method that takes text input and returns Vec<NormalizedEvent>
- [x] 8.3 Detect `<tool_call>` opening tag in text stream, transition to InToolCallTag state
- [x] 8.4 Buffer JSON content inside tag, emit `ToolCallDelta` events for each chunk
- [x] 8.5 On `</tool_call>` close tag: parse buffered JSON, emit `ToolCallComplete` with synthetic UUID
- [x] 8.6 Handle tag split across multiple deltas (partial `<tool_` at end of one delta, `call>` at start of next)
- [x] 8.7 Handle malformed JSON: emit `Error` event with parse failure message
- [x] 8.8 Resume Text state after tool call extraction, suppressing tag text from MessageDelta output
- [x] 8.9 Add `mod tool_extractor;` to `src/llm/mod.rs`
- [x] 8.10 Add unit tests: extract tool call from streaming text, handle split tags, handle malformed JSON

## 9. Tool Normalizer Driver

- [x] 9.1 Create `src/llm/tool_normalizer.rs` with `ToolNormalizerDriver` struct wrapping `Arc<dyn LlmDriver>`
- [x] 9.2 Accept `ModelCapabilityProfile` for the target model
- [x] 9.3 For `Native` capability: pass through stream unchanged
- [x] 9.4 For `InstructionTuned`: inject XML tools into system prompt via `xml_tool_injector`, wrap stream with `ToolCallExtractor`
- [x] 9.5 For `GrammarConstrained`: inject JSON schema as constrained output parameter, parse structured JSON responses into ToolCallComplete events
- [x] 9.6 For `TextOnly`: inject few-shot examples + XML schema, use aggressive extraction with retry
- [x] 9.7 Implement `LlmDriver::stream()` that dispatches based on capability profile
- [x] 9.8 Add `mod tool_normalizer;` to `src/llm/mod.rs`
- [x] 9.9 Add unit test: verify InstructionTuned path injects XML and extracts tool calls

## 10. Provider Dispatch Integration

- [x] 10.1 Add `detect_provider()` function that extracts provider from `"provider/model"` string
- [x] 10.2 Refactor Orchestrator construction to select driver: `anthropic/*` → `AnthropicDriver`, else → `ToolNormalizerDriver<LiterLlmDriver>`
- [x] 10.3 Add `ANTHROPIC_NATIVE_DRIVER` env var feature gate (default: true)
- [x] 10.4 When feature gate is off, fall back to existing `LiterLlmDriver` for all models
- [x] 10.5 Refactor `api_messages()` handler to use `AnthropicDriver` when resolved model is Anthropic
- [x] 10.6 Preserve existing `convert_anthropic_messages_to_openai()` path as fallback when feature gate is off
- [x] 10.7 Add `CacheStrategy` resolution from global config + per-request `prompt_caching_enabled` flag
- [x] 10.8 Add integration test: verify Anthropic model uses AnthropicDriver, non-Anthropic uses ToolNormalizerDriver

## 11. Extend LlmRequest

- [x] 11.1 Add optional fields to `LlmRequest`: `cache_strategy: Option<CacheStrategy>`, `thinking_config: Option<ThinkingConfig>`, `anthropic_system: Option<Vec<serde_json::Value>>`
- [x] 11.2 Ensure `LiterLlmDriver` ignores new fields (no behavior change)
- [x] 11.3 `AnthropicDriver` reads new fields when building `MessagesRequest`

## 12. Prometheus Cache Metrics

- [x] 12.1 Add metrics to `src/uar/telemetry/metrics.rs`: `uar_cache_write_tokens_total`, `uar_cache_read_tokens_total`, `uar_cache_savings_estimated_usd`
- [x] 12.2 Instrument `AnthropicDriver` to record cache metrics from `Usage` events
- [x] 12.3 Add cache hit rate gauge: `uar_cache_hit_rate` (read / (read + write) ratio)

## 13. End-to-End Verification

- [ ] 13.1 Test `/v1/messages` with Claude Sonnet streaming + tool use → verify correct Anthropic SSE format
- [ ] 13.2 Test `/v1/messages` with prompt caching → verify cache_control in request and cache tokens in response
- [ ] 13.3 Test `/v1/messages` with extended thinking → verify ThinkingDelta events stream correctly
- [ ] 13.4 Test tool normalization with a non-native model (Groq/Ollama) → verify tool calls look like Claude output
- [ ] 13.5 Test Claude Code connection to `/v1/messages` as Anthropic proxy → verify full compatibility
