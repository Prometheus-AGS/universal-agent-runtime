//! Cookbook: register and call an in-process native MCP-style tool.

use std::sync::Arc;
use universal_agent_runtime::mcp::registry::{McpRegistry, NativeTool};

#[derive(Debug)]
struct EchoTool;

#[async_trait::async_trait]
impl NativeTool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echoes the provided message back to the caller."
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        Ok(serde_json::json!({"echo": args.get("message").cloned().unwrap_or_default()}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = McpRegistry::empty().with_native_tool(Arc::new(EchoTool))?;
    let result = registry
        .call_namespaced_tool("native__echo", serde_json::json!({"message": "Hello, UAR"}))
        .await?;
    println!("Tool result: {result}");
    Ok(())
}
