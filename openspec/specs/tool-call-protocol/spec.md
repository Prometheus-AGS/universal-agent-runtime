# tool-call-protocol Specification

## Purpose

Define strict tool argument validation, declared effects, governed concurrency and name collision handling.

## Requirements

### Requirement: Tool arguments are validated before execution
The runtime SHALL parse tool-call arguments strictly and validate them against the tool's declared input schema, and SHALL NOT execute a tool whose arguments fail to parse or validate.

#### Scenario: Malformed JSON arguments
- **WHEN** the model emits a tool call whose argument string is not valid JSON
- **THEN** the runtime records a tool result of type `invalid_arguments` containing the parse error, the tool is not executed, and the loop continues so the model can retry

#### Scenario: Schema-invalid arguments
- **WHEN** the argument object violates the tool's input schema
- **THEN** the runtime records an `invalid_arguments` result containing the validator's message and the tool is not executed

#### Scenario: Validator compiled once
- **WHEN** a tool set is assembled for a run
- **THEN** each descriptor's schema validator is compiled once and reused for every call in that run

### Requirement: Every tool carries a declared effect
Each tool exposed to a model SHALL be described by one descriptor carrying an effect of `ReadOnly`, `ExternalMutation`, `CodeExecution`, or `Unknown`, and the runtime SHALL NOT infer effect, parallel safety, or approval need from the tool's name.

#### Scenario: Legacy tool without a declared effect
- **WHEN** a native or MCP tool declares no effect
- **THEN** its descriptor effect is `Unknown`, it executes sequentially, and it is subject to the approval policy

#### Scenario: MCP read-only annotation
- **WHEN** an MCP tool advertises `readOnlyHint: true`
- **THEN** its effect is `ReadOnly` for parallel scheduling only, and the approval decision is unchanged by the annotation

### Requirement: Parallel execution is gated by effect
The runtime SHALL execute tool calls concurrently only when every call in the batch is `ReadOnly`, has a compatible concurrency key, and requires neither approval nor a sandbox, and SHALL preserve call order in the recorded results. Concurrency keys are compatible when they differ or when either is absent; two calls with the same key are incompatible and run in order; any call whose effect is not `ReadOnly` takes an exclusive lock and runs alone.

#### Scenario: Mixed batch
- **WHEN** a batch contains one `ReadOnly` call and one `Unknown` call
- **THEN** the `Unknown` call runs alone and the results are recorded in the model's call order

#### Scenario: Same concurrency key
- **WHEN** two `ReadOnly` calls carry the same concurrency key
- **THEN** they run one after the other in call order

#### Scenario: Distinct or absent keys
- **WHEN** two `ReadOnly` calls carry distinct keys, or one carries no key
- **THEN** they run concurrently

### Requirement: Tool name collisions fail assembly
The runtime SHALL reject a tool set in which two non-identical descriptors share a provider-visible name, and SHALL keep MCP tools namespaced by server.

#### Scenario: Conflicting native descriptors
- **WHEN** two native tools with different schemas share a name
- **THEN** assembly fails with a typed collision error before any model call

#### Scenario: Same tool on two servers
- **WHEN** two MCP servers expose a tool with the same base name
- **THEN** both are exposed under their server namespace and no duplicate entry appears in the tool list
