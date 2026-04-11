//! Native session_search tool — keyword search within a conversation session.
//!
//! Loads a session from the persistence layer and filters messages by keyword.
//! Cross-session full-text search requires a `session_id` to be provided;
//! without one the tool returns a helpful error explaining the limitation.

use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::native_skill::NativeSkill;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug)]
pub struct SessionSearchTool {
    pub persistence: Arc<dyn PersistenceLayer>,
    pub max_results: usize,
}

#[async_trait]
impl NativeSkill for SessionSearchTool {
    fn name(&self) -> &str {
        "session_search"
    }

    fn description(&self) -> &str {
        "Search within a conversation session for messages matching a query. \
         Provide a session_id to search that session's history. \
         Use this to recall what was discussed in a previous conversation."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["query", "session_id"],
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keyword or phrase to search for in the conversation."
                },
                "session_id": {
                    "type": "string",
                    "description": "The session ID to search within."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 10).",
                    "minimum": 1,
                    "maximum": 50
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let query = match args.get("query").and_then(Value::as_str) {
            Some(q) => q.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: query"})),
        };
        let session_id = match args.get("session_id").and_then(Value::as_str) {
            Some(s) => s.to_string(),
            None => {
                return Ok(json!({
                    "ok": false,
                    "error": "Missing required parameter: session_id. \
                              Provide a session ID to search within a specific conversation."
                }));
            }
        };
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| n as usize)
            .unwrap_or(self.max_results)
            .min(self.max_results);

        let session = match self.persistence.load_session(&session_id).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                return Ok(
                    json!({"ok": false, "error": format!("Session '{}' not found", session_id)}),
                );
            }
            Err(e) => return Ok(json!({"ok": false, "error": format!("Load failed: {}", e)})),
        };

        let query_lower = query.to_lowercase();
        let hits: Vec<Value> = session
            .messages()
            .into_iter()
            .filter(|m| {
                m.content
                    .as_text()
                    .map(|t| t.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .take(limit)
            .map(|m| {
                json!({
                    "session_id": session_id,
                    "role": format!("{:?}", m.role),
                    "excerpt": m.content.as_text().unwrap_or(""),
                })
            })
            .collect();

        Ok(json!({
            "ok": true,
            "query": query,
            "session_id": session_id,
            "total": hits.len(),
            "results": hits
        }))
    }
}
