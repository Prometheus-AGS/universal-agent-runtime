# CH-04 prompt-dialect-engine

## Why

`PromptDialectEngine` (`src/llm/prompt_dialect.rs`) already existed, fully
implemented and unit-tested — per-model dialect detection (Anthropic XML,
OpenAI JSON, Kimi Markdown, GLM thinking, Qwen hybrid, MiniMax structured)
plus `request_params()` producing the exact web-verified extra parameters
each dialect wants (thinking budgets, reasoning-persistence toggles). But it
had zero call sites outside its own module — the engine was dead code. The
`thinking_budget` config knob (the natural "this deployment wants reasoning"
signal) was similarly dead.

## What changed

- `LlmRequest` (`src/llm/mod.rs`) gained `extra_params: Option<serde_json::Value>`
  — a generic seam for provider-dialect-specific extra request-body
  parameters, since the existing fields (`cache_strategy`, `thinking_config`,
  `anthropic_system`) are Anthropic-driver-specific and unused by
  `LiterLlmDriver` (the universal 142+-provider driver actually used by
  `Orchestrator`).
- `Orchestrator`'s tool-loop request construction now calls
  `PromptDialectEngine::request_params(model_id, DialectRequest {
  wants_reasoning: thinking_budget.is_some(), multi_turn: message_json.len() >
  1, hard: thinking_budget > 4096 })` and populates `extra_params` when the
  result is non-empty.
- `LiterLlmDriver::stream` merges `req.extra_params` into liter-llm's
  `ChatCompletionRequest::extra_body` — a field liter-llm already documents
  as "Provider-specific extra parameters merged into the request body," so
  no upstream change was needed.
- All 6 other `LlmRequest` construction sites (server.rs, graph nodes,
  summarizer, xml_tool_injector tests) updated with `extra_params: None`.

## Scope notes

- `wants_reasoning` is keyed off `thinking_budget.is_some()` rather than a
  richer per-request signal (e.g. task complexity) — this repo has no
  existing "this request needs reasoning" classifier to consult, and
  `thinking_budget` was already the closest existing config knob for it.
- `hard` (drives Anthropic's larger thinking-token budget / GLM's `max`
  effort) uses a simple `thinking_budget > 4096` heuristic threshold rather
  than a computed problem-difficulty score — no such score exists elsewhere
  in the runtime to reuse.
