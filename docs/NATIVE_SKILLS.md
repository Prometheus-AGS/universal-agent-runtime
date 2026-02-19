# Native Skills

The native skill system lets you embed high-performance Rust tool implementations directly
in the runtime binary, bypassing MCP serialization for critical hot-path operations.

## Architecture

```
┌──────────────┐    tool call    ┌──────────────────────┐
│  LLM Engine  │ ──────────────▶│     Orchestrator     │
└──────────────┘                │                      │
                                │  1. Check native     │
                                │     skill registry   │──▶ In-process execution
                                │  2. Fall back to MCP │──▶ JSON-RPC over stdio
                                └──────────────────────┘
```

Native skills have **priority** over MCP tools: when the orchestrator receives a tool call,
it first checks the `NativeSkillRegistry`. If a match is found, the skill runs in-process.
Otherwise, the call is forwarded to MCP as usual.

## Built-in Skills

| Name                  | Description                           |
|-----------------------|---------------------------------------|
| `native_echo`         | Returns input unchanged (testing)     |
| `native_system_info`  | Returns OS, arch, family, CPU count   |

## Creating a Custom Native Skill

Implement the `NativeSkill` trait and register it:

```rust
use universal_agent_runtime::uar::runtime::native_skill::NativeSkill;

#[derive(Debug)]
struct MyTool;

#[async_trait::async_trait]
impl NativeSkill for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Does something useful" }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input = args["input"].as_str().unwrap_or_default();
        Ok(serde_json::json!({ "result": input.to_uppercase() }))
    }
}
```

Register in `src/uar/runtime/native_skills/mod.rs`:

```rust
pub mod my_tool;

pub async fn register_builtins(registry: &NativeSkillRegistry) {
    registry.register(echo::EchoSkill).await;
    registry.register(system_info::SystemInfoSkill).await;
    registry.register(my_tool::MyTool).await;  // ← add here
}
```

## Key Files

| File | Purpose |
|------|---------|
| `src/uar/runtime/native_skill.rs` | `NativeSkill` trait + `NativeSkillRegistry` |
| `src/uar/runtime/native_skills/mod.rs` | Built-in registration |
| `src/uar/runtime/native_skills/echo.rs` | Echo skill |
| `src/uar/runtime/native_skills/system_info.rs` | System info skill |
