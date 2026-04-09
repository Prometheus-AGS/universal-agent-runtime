## ADDED Requirements

### Requirement: LlmDriver Wrapper
ToolNormalizerDriver SHALL wrap any type implementing LlmDriver and delegate streaming to the inner driver.

#### Scenario: Wrapping a LiterLlmDriver
- **WHEN** `ToolNormalizerDriver<LiterLlmDriver>` is constructed
- **THEN** it MUST implement the LlmDriver trait and delegate the underlying HTTP communication to the wrapped LiterLlmDriver.

### Requirement: Native Capability Passthrough
For models with Native tool_call_capability, ToolNormalizerDriver SHALL pass the stream through without modification.

#### Scenario: Native model stream passthrough
- **WHEN** the model's capability profile has `tool_call_capability: Native` and the inner driver emits `ToolCallDelta` and `ToolCallComplete` events
- **THEN** ToolNormalizerDriver MUST forward those events unmodified to the output stream.

#### Scenario: Native model with no tool calls
- **WHEN** the model's capability profile has `tool_call_capability: Native` and the inner driver emits only `MessageDelta` events
- **THEN** ToolNormalizerDriver MUST forward those events unmodified.

### Requirement: GrammarConstrained JSON Schema Injection
For models with GrammarConstrained capability, ToolNormalizerDriver SHALL inject JSON schema constraints into the request.

#### Scenario: Grammar-constrained tool definitions
- **WHEN** the model's capability profile has `tool_call_capability: GrammarConstrained` and the LlmRequest contains tool definitions
- **THEN** ToolNormalizerDriver MUST transform the tool definitions into JSON schema grammar constraints in the format expected by the underlying provider and pass the modified request to the inner driver.

#### Scenario: Grammar-constrained stream normalization
- **WHEN** the inner driver produces JSON-formatted tool call output constrained by the injected schema
- **THEN** ToolNormalizerDriver MUST parse the output and emit `NormalizedEvent::ToolCallDelta` and `NormalizedEvent::ToolCallComplete` events.

### Requirement: InstructionTuned XML Injection
For models with InstructionTuned capability, ToolNormalizerDriver SHALL inject XML tool schemas into the system prompt and extract tool calls from the response.

#### Scenario: XML tool schema injected
- **WHEN** the model's capability profile has `tool_call_capability: InstructionTuned` and the LlmRequest contains tool definitions
- **THEN** ToolNormalizerDriver MUST use the XmlToolInjector to convert tool definitions to XML format and prepend them to the system prompt before passing the request to the inner driver.

#### Scenario: Tool call extraction from response
- **WHEN** the inner driver emits `MessageDelta` events containing `<tool_call>` tags in the text
- **THEN** ToolNormalizerDriver MUST use the ToolCallExtractor to detect and parse tool calls, emitting `NormalizedEvent::ToolCallDelta` and `NormalizedEvent::ToolCallComplete` events instead of raw text.

### Requirement: TextOnly Few-Shot Injection
For models with TextOnly capability, ToolNormalizerDriver SHALL inject few-shot examples into the prompt to demonstrate tool call format.

#### Scenario: Few-shot examples added to prompt
- **WHEN** the model's capability profile has `tool_call_capability: TextOnly` and the LlmRequest contains tool definitions
- **THEN** ToolNormalizerDriver MUST inject the XML tool schema plus one or more few-shot example exchanges demonstrating proper `<tool_call>` tag usage into the system prompt.

#### Scenario: TextOnly tool call extraction
- **WHEN** the inner driver emits `MessageDelta` events and the text contains `<tool_call>` tags
- **THEN** ToolNormalizerDriver MUST extract and normalize tool calls identically to the InstructionTuned path.

### Requirement: Uniform Output Normalization
All capability levels SHALL produce identical ToolCallDelta and ToolCallComplete event shapes.

#### Scenario: Event shape consistency across capability levels
- **WHEN** a tool call is produced by any capability level (Native, GrammarConstrained, InstructionTuned, or TextOnly)
- **THEN** the emitted `ToolCallDelta` events MUST contain `call_index`, `id`, `name`, and `arguments_delta` fields, and the `ToolCallComplete` events MUST contain `call_index`, `id`, `name`, and `arguments_json` fields, regardless of which capability path produced them.
