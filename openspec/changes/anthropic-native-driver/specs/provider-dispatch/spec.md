## ADDED Requirements

### Requirement: Provider-Based Driver Selection
The Orchestrator SHALL select the appropriate LlmDriver implementation based on the model's provider prefix.

#### Scenario: Anthropic model dispatched to AnthropicDriver
- **WHEN** the Orchestrator is constructed with a model string matching `anthropic/*` (e.g., `"anthropic/claude-sonnet-4-20250514"`)
- **THEN** it MUST use an `AnthropicDriver` instance as its LlmDriver.

#### Scenario: Non-Anthropic model dispatched to ToolNormalizerDriver
- **WHEN** the Orchestrator is constructed with a model string like `"openai/gpt-4o"` or `"groq/llama-3.3-70b-versatile"`
- **THEN** it MUST use a `ToolNormalizerDriver<LiterLlmDriver>` instance as its LlmDriver.

### Requirement: Per-Request Model Override Dispatch
The `api_messages()` handler SHALL perform the same provider dispatch for per-request model overrides.

#### Scenario: Request-level model override to Anthropic
- **WHEN** the `api_messages()` handler receives a request with `model: "anthropic/claude-sonnet-4-20250514"` that differs from the default Orchestrator model
- **THEN** it MUST dispatch that request to an `AnthropicDriver` instance, regardless of the default Orchestrator's driver.

#### Scenario: Request-level model override to non-Anthropic
- **WHEN** the `api_messages()` handler receives a request with `model: "openai/gpt-4o"` while the default Orchestrator uses AnthropicDriver
- **THEN** it MUST dispatch that request to a `ToolNormalizerDriver<LiterLlmDriver>` instance.

### Requirement: Feature Gate via Environment Variable
The AnthropicDriver path SHALL be feature-gated via the `ANTHROPIC_NATIVE_DRIVER` environment variable.

#### Scenario: Feature gate enabled
- **WHEN** the environment variable `ANTHROPIC_NATIVE_DRIVER` is set to `"true"` and the model is `anthropic/*`
- **THEN** the Orchestrator MUST use the `AnthropicDriver` for that model.

#### Scenario: Feature gate disabled
- **WHEN** the environment variable `ANTHROPIC_NATIVE_DRIVER` is unset or set to `"false"` and the model is `anthropic/*`
- **THEN** the Orchestrator MUST fall back to `ToolNormalizerDriver<LiterLlmDriver>` (the existing liter-llm path) for backward compatibility.

#### Scenario: Feature gate irrelevant for non-Anthropic models
- **WHEN** the model is `"openai/gpt-4o"` regardless of the `ANTHROPIC_NATIVE_DRIVER` value
- **THEN** the Orchestrator MUST always use `ToolNormalizerDriver<LiterLlmDriver>`.

### Requirement: Zero Breaking Changes to OpenAI Path
Provider dispatch SHALL not alter the behavior of existing `/v1/chat/completions` OpenAI-compatible endpoints.

#### Scenario: OpenAI chat completions unaffected
- **WHEN** a request is sent to `/v1/chat/completions` with any model
- **THEN** the request MUST be handled by the existing liter-llm path without any involvement of the AnthropicDriver, regardless of the `ANTHROPIC_NATIVE_DRIVER` setting.

#### Scenario: Existing session behavior preserved
- **WHEN** a session was created using the `/v1/chat/completions` endpoint
- **THEN** subsequent requests to that session MUST continue using the same driver path without interruption.
