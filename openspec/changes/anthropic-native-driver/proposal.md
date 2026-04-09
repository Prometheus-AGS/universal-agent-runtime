## Why

UAR currently routes all Anthropic requests through liter-llm's OpenAI-compatible wire format, losing native Anthropic capabilities: prompt caching (`cache_control` blocks), extended thinking, native streaming event model (`content_block_start/delta/stop`), and Anthropic-specific content types (images, documents, thinking blocks). This makes it impossible for tools like Claude Code to use UAR as a transparent Anthropic proxy — they detect missing capabilities and fall back. To present UAR as a fully native Anthropic endpoint, we need a direct Messages API driver that preserves the complete Anthropic wire protocol, plus a tool-call normalization layer that makes ANY backend model's tool calls structurally identical to Claude Sonnet/Opus 4.6 output.

## What Changes

### Native Anthropic Driver
- Add `AnthropicDriver` implementing `LlmDriver` trait, speaking the Messages API directly via reqwest (no liter-llm intermediary)
- Full SSE streaming decoder mapping Anthropic events (`message_start`, `content_block_start/delta/stop`, `message_delta`, `message_stop`) to `NormalizedEvent`
- Native `cache_control` annotation on system blocks, tool definitions, and conversation turns via configurable `CacheStrategy`
- Extended thinking support (`thinking` content blocks with configurable `budget_tokens`)
- Native image/document content block support (base64 + URL source)
- Prompt cache accounting (cache write/read tokens tracked in `Usage` events)

### Tool Call Normalization Layer
- Add `ModelCapabilityRegistry` mapping model strings to tool-call capability profiles (Native, GrammarConstrained, InstructionTuned, TextOnly)
- Add `ToolNormalizerDriver` wrapper that makes any backend produce Claude Sonnet 4.6-shaped tool calls
- XML tool schema injection for instruction-tuned models (Qwen, DeepSeek, Llama)
- Streaming `ToolCallExtractor` state machine that parses `<tool_call>` blocks from text streams
- Grammar-constrained JSON extraction for structured output models

### `/v1/messages` Endpoint Upgrade
- Refactor existing `/v1/messages` handler to use `AnthropicDriver` when the resolved model is an Anthropic model (bypass liter-llm entirely)
- Preserve current liter-llm path for non-Anthropic models with `ToolNormalizerDriver` wrapping
- Response SSE format matches Anthropic wire protocol exactly (Claude Code compatible)

### Provider Dispatch in Orchestrator
- Add provider-aware driver selection in `Orchestrator` construction — Anthropic models get `AnthropicDriver`, others get `ToolNormalizerDriver<LiterLlmDriver>`
- Cache strategy applied automatically for Anthropic models based on global + per-request configuration

## Capabilities

### New Capabilities
- `anthropic-driver`: Native Anthropic Messages API client with reqwest-based streaming SSE, full content block support, and direct wire protocol handling
- `anthropic-cache-strategy`: Configurable prompt caching with `cache_control` annotation on system blocks, tools, and conversation turns
- `anthropic-extended-thinking`: Extended thinking support with configurable budget_tokens and thinking content block streaming
- `anthropic-streaming-decoder`: SSE decoder translating Anthropic streaming events to NormalizedEvent stream
- `model-capability-registry`: Registry mapping model identifiers to tool-call capability profiles for normalization decisions
- `tool-normalizer-driver`: Wrapper driver that normalizes any backend's tool calls to Claude Sonnet 4.6 format
- `xml-tool-injector`: System prompt injection of XML tool schemas for instruction-tuned models
- `tool-call-extractor`: Streaming state machine extracting tool calls from text output (XML and JSON formats)
- `provider-dispatch`: Provider-aware driver selection in Orchestrator and `/v1/messages` handler

### Modified Capabilities
- None (all changes are additive; existing OpenAI path through liter-llm is preserved)

## Impact

### Backend (Rust)
- `src/llm/mod.rs`: Extend `LlmRequest` with optional Anthropic-specific fields (cache_control, thinking config)
- `src/llm/anthropic_driver.rs`: New file — `AnthropicDriver` implementing `LlmDriver`
- `src/llm/anthropic_types.rs`: New file — Anthropic Messages API request/response serde types
- `src/llm/anthropic_streaming.rs`: New file — SSE decoder state machine
- `src/llm/anthropic_cache.rs`: New file — `CacheStrategy` implementation
- `src/llm/tool_normalizer.rs`: New file — `ToolNormalizerDriver` wrapper
- `src/llm/capability_registry.rs`: New file — `ModelCapabilityRegistry`
- `src/llm/tool_extractor.rs`: New file — streaming `ToolCallExtractor` state machine
- `src/llm/orchestrator.rs`: Provider-aware driver selection
- `src/server.rs`: Refactor `api_messages()` to use native driver for Anthropic models
- `src/normalized.rs`: No changes needed — existing variants cover all Anthropic events

### Dependencies
- No new crates needed — `reqwest` (with streaming), `serde`, `async-stream`, `futures` already available
- `liter-llm` stays for non-Anthropic provider routing (142+ providers)

### Compatibility
- Existing `/v1/chat/completions` (OpenAI) path is unchanged
- Existing `/v1/messages` path gets upgraded but maintains backward compatibility
- All `NormalizedEvent` consumers (AG-UI, MCP tool runner, session store) are unaffected
