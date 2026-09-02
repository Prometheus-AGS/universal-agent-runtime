//! Transport-free tool enumeration.
//!
//! The embedded control plane returned a hardcoded `[]` for `tools` while both
//! registries sat right there on the kernel — a client could not tell "no tools
//! configured" from "tool listing is unavailable here".
//!
//! Tools come from TWO backends and a caller needs both to reason about what an
//! agent can actually do:
//!
//!   * NATIVE skills — in-process Rust (`web_fetch`, the diagnostics tool…).
//!     These work with no network and no child process, which is why they are
//!     the ones that function on a phone.
//!   * MCP tools — supplied by connected MCP servers, and therefore dependent
//!     on those servers being reachable.
//!
//! The distinction is carried in the output rather than flattened away: an
//! offline surface needs to show that its native tools still work while its MCP
//! tools do not, and a merged list with no `source` field cannot express that.

use std::sync::Arc;

use serde::Serialize;

use crate::mcp::registry::McpRegistry;
use crate::uar::runtime::native_skill::NativeSkillRegistry;

/// Which backend provides a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolSource {
    /// In-process Rust. No network, no child process — available offline.
    Native,
    /// Provided by a connected MCP server.
    Mcp,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolEntry {
    /// Name the model calls, unique across both backends.
    pub name: String,
    pub description: String,
    pub source: ToolSource,
    /// Owning MCP server, when `source` is `Mcp`. `None` for native tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
}

/// Every tool this runtime can currently dispatch.
///
/// Native tools are listed first because they are the ones that survive with no
/// network — an offline client reading this top-down sees what still works
/// before what does not.
pub async fn list(native: &Arc<NativeSkillRegistry>, mcp: &Arc<McpRegistry>) -> Vec<ToolEntry> {
    let mut entries = Vec::new();

    for name in native.names().await {
        let description = native
            .get(&name)
            .await
            .map(|skill| skill.description().to_string())
            .unwrap_or_default();
        entries.push(ToolEntry {
            name,
            description,
            source: ToolSource::Native,
            server: None,
        });
    }

    for (name, tool) in mcp.tools() {
        // MCP tool names are namespaced `server__tool`, which is how the
        // orchestrator routes a call back to its server. Split it so a client
        // can group by server without re-parsing the convention itself.
        let server = name.split("__").next().map(str::to_string);
        entries.push(ToolEntry {
            name: name.clone(),
            description: tool.description.as_deref().unwrap_or_default().to_string(),
            source: ToolSource::Mcp,
            server,
        });
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::runtime::native_skill::NativeSkill;

    #[derive(Debug)]
    struct StubTool;

    #[async_trait::async_trait]
    impl NativeSkill for StubTool {
        fn name(&self) -> &str {
            "stub_tool"
        }
        fn description(&self) -> &str {
            "a stand-in for a real native tool"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({ "type": "object", "properties": {} })
        }
        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
    }

    #[tokio::test]
    async fn lists_native_tools_with_their_source() {
        let native = Arc::new(NativeSkillRegistry::new());
        native
            .register(StubTool)
            .await
            .expect("stub descriptor registers");
        let mcp = Arc::new(McpRegistry::new_empty());

        let listed = list(&native, &mcp).await;
        let stub = listed
            .iter()
            .find(|entry| entry.name == "stub_tool")
            .expect("the registered native tool must appear");

        // The whole point: not an empty array, and identifiably native.
        assert_eq!(stub.source, ToolSource::Native);
        assert!(!stub.description.is_empty());
        // Native tools carry no server, because there is none.
        assert!(stub.server.is_none());
    }

    #[tokio::test]
    async fn an_empty_runtime_lists_nothing_rather_than_failing() {
        // "No tools" is a legitimate state and must not be an error — the
        // `unavailable` contract is what distinguishes it from "cannot list".
        let listed = list(
            &Arc::new(NativeSkillRegistry::new()),
            &Arc::new(McpRegistry::new_empty()),
        )
        .await;
        assert!(listed.is_empty());
    }

    #[test]
    fn the_source_field_survives_serialization() {
        // A client that cannot tell native from MCP cannot explain why some
        // tools still work offline and others do not.
        let entry = ToolEntry {
            name: "web_fetch".to_string(),
            description: "fetch a URL".to_string(),
            source: ToolSource::Native,
            server: None,
        };
        let json = serde_json::to_value(&entry).expect("serializes");
        assert_eq!(json["source"], "native");
        assert!(json.get("server").is_none(), "absent, not null");
    }
}
