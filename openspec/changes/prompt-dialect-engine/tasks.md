## 1. Engine (shipped prior turn)

- [x] 1.1 `PromptDialectEngine`/`PromptDialect`/`DialectRequest`
      (`src/llm/prompt_dialect.rs`) — dialect detection + per-model
      `request_params`. 6 unit tests green.

## 2. Wire into request assembly (this pass)

- [x] 2.1 `LlmRequest.extra_params: Option<serde_json::Value>` seam added.
- [x] 2.2 `Orchestrator`'s tool-loop request construction calls
      `PromptDialectEngine::request_params` and populates `extra_params`.
- [x] 2.3 `LiterLlmDriver::stream` merges `extra_params` into
      `ChatCompletionRequest::extra_body`.
- [x] 2.4 All other `LlmRequest` literal construction sites updated
      (`server.rs`, `llm_node.rs`, `router_node.rs`, `summarizer.rs`,
      `xml_tool_injector.rs` test helper) — non-tool-loop paths pass `None`.

## 3. Verify

- [x] 3.1 `cargo check --lib` green.
- [x] 3.2 `cargo test --lib llm::` — 72/72 green, including all 6
      pre-existing `prompt_dialect` tests (now exercised via a real call
      path, not just in isolation).
- [x] 3.3 Full-suite batch checkpoint: 318/318 lib tests green.
