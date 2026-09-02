<!-- mirror of openspec/changes/fail-closed-tool-arguments/proposal.md and specs/*/spec.md -->
# fail-closed-tool-arguments

Rank 2 of the codex-harness-comparative-analysis change set. Source: gap G2 in the phase `analysis.md`.

## Why

A malformed tool-call argument string executes the tool with `{}` at two sites (`src/llm/orchestrator.rs:982-983`, `:1032-1033`) with no log and no feedback to the model. For a tool whose parameters are optional (a list, a search, a delete-by-filter), that is a silent semantic change. `parameters_schema()` is only ever serialized into the prompt (`src/uar/runtime/native_skill.rs:186`); nothing validates against it, although `jsonschema` 0.49.4 is already a dependency (`Cargo.toml:418`). Tool effects are inferred from name substrings: `is_parallel_safe_tool` (`orchestrator.rs:138-148`), `tool_requires_approval` (`src/uar/runtime/manager.rs:366-370`), `is_code_execution_tool` (`orchestrator.rs:115-134`). A tool named `get_and_purge_records` is parallel-safe and escapes the approval heuristic. `McpRegistry::merge` duplicates the tools vector (`src/mcp/registry.rs:733-734`).

Codex keeps arguments as a raw string until the handler parses them, returns any parse or schema error to the model as a failed output (`core/src/tools/handlers/mod.rs:83-90`; `core/src/tools/parallel.rs:219-243`), classifies every failure as model-visible or fatal (`tools/src/function_call_error.rs:5-10`), and gates parallelism with one `RwLock` where read-only tools take the read lock (`parallel.rs:155-159`) with `false` as the default (`tools/src/tool_executor.rs:122-124`).

## What changes

- One immutable `ToolDescriptor`: id, provider-safe name, source, server, input schema, effect, approval class, sandbox requirement, concurrency key, exposure, output limit. Native skills, MCP tools, and built-in runtime tools all adapt into it.
- Argument parsing is strict and validated with a `jsonschema::Validator` compiled once per descriptor at assembly. Parse or schema failure produces a typed `invalid_arguments` tool result returned to the model; the tool never executes.
- `ToolEffect::{ReadOnly, ExternalMutation, CodeExecution, Unknown}`. Legacy tools default to `Unknown`. MCP `readOnlyHint` maps to `ReadOnly` for parallelism only and never bypasses approval, because MCP 2026-07-28 declares annotations untrusted.
- Parallel execution allowed only for `ReadOnly` descriptors with compatible concurrency keys and no approval or sandbox requirement, implemented as a read/write lock; deterministic result ordering preserved.
- Name collisions are detected at assembly: identical descriptors dedupe, MCP tools keep their namespace, conflicting descriptors fail assembly with a typed error.
- Codex paths cited above are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## Scope

- `src/llm/orchestrator.rs` (tool loop `:498-512`, `:974-1094`)
- `src/uar/runtime/native_skill.rs`
- `src/mcp/registry.rs` (`merge`, `openai_tools_json`)
- `src/uar/runtime/manager.rs` (`tool_requires_approval` and the gate closure `:1712-1826`)
- new: `src/uar/tools/descriptor.rs`, `src/uar/tools/validate.rs`
- tests: `tests/tool_call_protocol.rs`

Out of scope: deferred exposure and `search_tools` (projected-mcp-runtime), Cedar policy content, sandbox runners, and `versions.toml`, which this change does not edit.

## Dependencies

None on other changes in this set. Precondition, external to this change: the operator records `jsonschema = "0.49.4"` in `versions.toml` before this change starts; task 0.1 checks for the entry and stops if it is absent.

## Verification

Tier 0 per edit; Tier 1 the new tests; Tier 2 at the boundary.

## The uncomfortable thing

Every tool that today is auto-classified parallel-safe by prefix becomes `Unknown` and therefore sequential until its descriptor says otherwise. Throughput drops for read-heavy MCP servers that do not set `readOnlyHint` until operators annotate them. That is the correct default and the change says so in the proposal rather than preserving the inference.


## Spec delta: tool-approval-workflow

## MODIFIED Requirements

### Requirement: Tool calls matching approval policy pause for user confirmation
When a tool call matches a Cedar policy rule marked as `requires_approval`, or the tool's descriptor approval class requires it under the effective run policy, the run SHALL pause and emit an approval request event to the frontend. The approval decision SHALL be derived from Cedar and the descriptor's approval class, never from the tool's name.

#### Scenario: High-risk tool requires approval
- **WHEN** the agent calls a tool tagged as high-risk in Cedar policy (e.g., `filesystem__delete`)
- **THEN** the run pauses, an `agui.tool_call.approval_required` event is emitted with tool name, arguments, and risk reason

#### Scenario: User approves tool call
- **WHEN** the user clicks "Approve" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": true}` resumes the tool execution

#### Scenario: User rejects tool call
- **WHEN** the user clicks "Reject" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": false}` cancels the tool call and the agent receives a rejection message

#### Scenario: Approval timeout
- **WHEN** no approval response is received within the configured approval timeout
- **THEN** the tool call is automatically rejected and the agent is notified

#### Scenario: Name does not decide approval
- **WHEN** a tool named `get_and_purge_records` is called with an `ExternalMutation` effect under `ToolApprovalPolicy::Auto`
- **THEN** the run pauses for approval because of its effect and policy, regardless of the `get_` prefix

## Spec delta: tool-call-protocol

## ADDED Requirements

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
