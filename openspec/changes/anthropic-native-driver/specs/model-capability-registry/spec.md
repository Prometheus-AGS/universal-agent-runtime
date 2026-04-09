## ADDED Requirements

### Requirement: Static Model-to-Capability Mapping
The ModelCapabilityRegistry SHALL provide a static registry that maps model identifier strings to capability profiles.

#### Scenario: Known Anthropic model lookup
- **WHEN** the registry is queried with model string `"anthropic/claude-sonnet-4-20250514"`
- **THEN** the registry MUST return a capability profile with `tool_call_capability: Native`.

#### Scenario: Known OpenAI model lookup
- **WHEN** the registry is queried with model string `"openai/gpt-4o"`
- **THEN** the registry MUST return a capability profile with `tool_call_capability: Native`.

### Requirement: Capability Profile Fields
Each capability profile SHALL include: tool_call_capability, supports_system_prompt, supports_streaming, max_context_tokens, and supports_parallel_tool_calls.

#### Scenario: Full profile for a known model
- **WHEN** the registry returns a profile for `"anthropic/claude-sonnet-4-20250514"`
- **THEN** the profile MUST contain `tool_call_capability: Native`, `supports_system_prompt: true`, `supports_streaming: true`, `max_context_tokens: 200000`, and `supports_parallel_tool_calls: true`.

#### Scenario: Profile for a text-only model
- **WHEN** the registry returns a profile for a model classified as TextOnly
- **THEN** the profile MUST contain `tool_call_capability: TextOnly`, `supports_parallel_tool_calls: false`.

### Requirement: Glob Pattern Matching
The registry SHALL support glob patterns for matching model families to capability profiles.

#### Scenario: Glob match for Qwen model family
- **WHEN** the registry is queried with model string `"ollama/qwen2.5-coder:32b"`
- **THEN** the registry MUST match the glob pattern `"*/qwen2.5*"` and return a profile with `tool_call_capability: InstructionTuned`.

#### Scenario: Glob match for Llama model family
- **WHEN** the registry is queried with model string `"groq/llama-3.3-70b-versatile"`
- **THEN** the registry MUST match the appropriate glob pattern and return the corresponding capability profile.

#### Scenario: Specific model overrides glob
- **WHEN** the registry contains both a glob pattern `"*/claude-*"` and an exact entry `"anthropic/claude-sonnet-4-20250514"`
- **THEN** the exact entry MUST take precedence over the glob pattern.

### Requirement: Unknown Model Default
Unknown models SHALL default to the InstructionTuned capability profile.

#### Scenario: Completely unknown model
- **WHEN** the registry is queried with model string `"custom/my-finetuned-model-v3"`
- **THEN** the registry MUST return a default profile with `tool_call_capability: InstructionTuned`.

#### Scenario: Unknown model has safe defaults
- **WHEN** the registry returns the default profile for an unknown model
- **THEN** the profile MUST have `supports_system_prompt: true`, `supports_streaming: true`, and `supports_parallel_tool_calls: false`.

### Requirement: Four Capability Levels
The registry SHALL classify models into exactly four tool_call_capability levels: Native, GrammarConstrained, InstructionTuned, and TextOnly.

#### Scenario: Native capability models
- **WHEN** the registry is queried with any `anthropic/*`, `openai/*`, or `google/*` model
- **THEN** the registry MUST return `tool_call_capability: Native`.

#### Scenario: GrammarConstrained capability models
- **WHEN** the registry is queried with a model served via candle-vllm or a provider supporting grammar constraints
- **THEN** the registry MUST return `tool_call_capability: GrammarConstrained`.

#### Scenario: InstructionTuned capability models
- **WHEN** the registry is queried with models like `"ollama/qwen2.5*"` or `"ollama/llama*"` that support tool use via instruction following
- **THEN** the registry MUST return `tool_call_capability: InstructionTuned`.

#### Scenario: TextOnly capability models
- **WHEN** the registry is queried with a model that has no tool call support
- **THEN** the registry MUST return `tool_call_capability: TextOnly`.
