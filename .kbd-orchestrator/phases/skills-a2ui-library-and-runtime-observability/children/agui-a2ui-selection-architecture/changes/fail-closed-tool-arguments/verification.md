<!-- mirror of openspec/changes/fail-closed-tool-arguments/tasks.md -->
# Tasks — fail-closed-tool-arguments

scope: src/llm/orchestrator.rs (tool loop), src/uar/runtime/native_skill.rs, src/mcp/registry.rs (merge, tool listing), src/uar/runtime/manager.rs (approval heuristic and gate), src/uar/tools/descriptor.rs, src/uar/tools/validate.rs, tests/tool_call_protocol.rs, versions.toml (operator)

## 0. Precondition

- [ ] 0.1 Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task

## 1. Failing tests first

- [ ] 1.1 `tests/tool_call_protocol.rs`: a tool call with arguments `{"path": ` (truncated JSON) produces an `invalid_arguments` result containing the parse error and the tool's execute function is never invoked
- [ ] 1.2 A tool call with `{"path": 42}` against a schema requiring a string produces `invalid_arguments` with the validator message
- [ ] 1.3 Two `ReadOnly` descriptors with distinct keys execute concurrently; two with the same key run in call order; a `ReadOnly` descriptor with no key runs concurrently with either; one `Unknown` descriptor serializes against all of them
- [ ] 1.4 An MCP tool with `readOnlyHint: true` and `ToolApprovalPolicy::Ask` still pauses for approval
- [ ] 1.5 Two MCP servers exposing the same tool name produce two namespaced descriptors; two native skills with the same name fail assembly with `ToolCollision`
- [ ] 1.6 A descriptor's validator is compiled once per assembly (count `validator_for` calls across ten invocations)

## 2. Descriptor and validation

- [ ] 2.1 Add `src/uar/tools/descriptor.rs`: `ToolDescriptor`, `ToolEffect`, `ApprovalClass`, `Exposure` (`Eager | Deferred | Hidden | ModelOnly`, only `Eager` used here)
- [ ] 2.2 Add `src/uar/tools/validate.rs`: `compile(&Value) -> Result<Validator>` via `jsonschema::validator_for`; `validate(&Validator, &str) -> Result<Value, InvalidArguments>` covering parse and schema errors
- [ ] 2.3 Adapt `NativeSkill` into a descriptor with `effect: Unknown` by default and an optional `effect()` trait method; adapt MCP tools with `readOnlyHint` mapping; adapt built-in runtime tools

## 3. Orchestrator

- [ ] 3.1 Replace the two `unwrap_or_else(|_| {})` sites with `validate`; on error push the typed `invalid_arguments` result and continue the loop
- [ ] 3.2 Replace `is_parallel_safe_tool` and `is_code_execution_tool` with descriptor fields; delete the prefix lists
- [ ] 3.3 Implement the read/write-lock gate; keep `.buffered` collection in call order
- [ ] 3.4 Replace `tool_requires_approval` substring scan with `ApprovalClass` from the descriptor; Cedar decision precedence unchanged

## 4. Registry

- [ ] 4.1 `McpRegistry::merge` dedupes `tools` by namespaced name; identical descriptors collapse, conflicting ones return `ToolCollision`
- [ ] 4.2 Tool list assembly sorts by name and is built from descriptors, not from two ad hoc JSON lists

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test tool_call_protocol`
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 `openspec validate fail-closed-tool-arguments --strict`
