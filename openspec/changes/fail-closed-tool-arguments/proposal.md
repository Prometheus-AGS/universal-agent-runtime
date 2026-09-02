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
