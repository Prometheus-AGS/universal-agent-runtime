## ADDED Requirements

### Requirement: Tool Definition to XML Conversion
XmlToolInjector SHALL convert OpenAI-format tool definitions into an XML schema block suitable for injection into a system prompt.

#### Scenario: Single tool conversion
- **WHEN** XmlToolInjector receives a tool definition with name `"search"`, description `"Search the web"`, and a JSON schema for parameters `{"query": {"type": "string"}, "limit": {"type": "integer"}}`
- **THEN** it MUST produce an XML block containing `<tool name="search">` with the description and parameter schema encoded as XML child elements.

#### Scenario: Multiple tool definitions
- **WHEN** XmlToolInjector receives three tool definitions
- **THEN** it MUST produce a single `<tools>` XML block containing three `<tool>` child elements, one for each definition, preserving all parameter schemas.

### Requirement: System Prompt Injection
XmlToolInjector SHALL inject the XML tools block into the system prompt without destroying existing system prompt content.

#### Scenario: Existing system prompt preserved
- **WHEN** the system prompt is `"You are a helpful assistant."` and tool definitions are provided
- **THEN** XmlToolInjector MUST produce a system prompt that contains both the original text `"You are a helpful assistant."` and the generated `<tools>` XML block.

#### Scenario: Empty system prompt
- **WHEN** the system prompt is empty and tool definitions are provided
- **THEN** XmlToolInjector MUST produce a system prompt containing the `<tools>` XML block and instruction text only.

### Requirement: Tool Call Instruction Text
XmlToolInjector SHALL include instruction text directing the model to use `<tool_call>` tags for tool invocations.

#### Scenario: Instruction text included
- **WHEN** the XML tools block is generated
- **THEN** the injected system prompt section MUST include instruction text that tells the model to respond with `<tool_call>{"name": "<tool_name>", "arguments": {...}}</tool_call>` when it wants to invoke a tool.

#### Scenario: Instruction format matches extractor expectations
- **WHEN** the instruction text describes the tool call format
- **THEN** the format described MUST match what the ToolCallExtractor expects to parse: an opening `<tool_call>` tag, a JSON object with `"name"` and `"arguments"` keys, and a closing `</tool_call>` tag.

### Requirement: Parameter Schema Fidelity
XmlToolInjector SHALL preserve the full JSON schema of each tool's parameters in the XML representation.

#### Scenario: Nested object parameters
- **WHEN** a tool definition has parameters with nested objects and arrays
- **THEN** XmlToolInjector MUST represent the full schema including types, descriptions, required fields, and nested structures in the XML output.

#### Scenario: Required parameters marked
- **WHEN** a tool definition specifies required parameters
- **THEN** the XML representation MUST indicate which parameters are required so the model can produce valid invocations.
