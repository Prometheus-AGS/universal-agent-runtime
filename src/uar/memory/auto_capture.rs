//! Post-stream auto-capture of memories from conversation turns.
//!
//! After each assistant turn, this module inspects the latest messages and
//! calls `add_memories_from_conversation` to let the LLM-backed memory
//! extractor identify and persist notable information.
//!
//! This is intentionally fire-and-forget: errors are logged but not propagated
//! so that the main streaming path is never blocked.

use std::sync::Arc;
use surreal_memory::Memory;

use crate::uar::security::claims::UserContext;

use super::service::MemoryService;

/// Represents a single message in a conversation, compatible with the
/// `add_memories_from_conversation` wire format.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

/// Extract and persist memories from recent conversation messages.
///
/// Called after each assistant turn completes. Errors are intentionally
/// swallowed (logged as warnings) so the caller is never blocked.
///
/// Returns the extracted memories for observability (empty if disabled or on error).
pub async fn capture_from_stream_end(
    service: &Arc<MemoryService>,
    messages: &[ConversationMessage],
    user_ctx: &UserContext,
    agent_id: &str,
    session_id: &str,
) -> Vec<Memory> {
    if !service.config().auto_capture {
        return vec![];
    }

    if messages.is_empty() {
        return vec![];
    }

    // Convert to the JSON format expected by surreal-memory.
    let json_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        })
        .collect();

    match service
        .auto_capture(json_messages, user_ctx, Some(agent_id), Some(session_id))
        .await
    {
        Ok(extracted) => {
            if !extracted.is_empty() {
                tracing::debug!(
                    count = extracted.len(),
                    agent_id = %agent_id,
                    session_id = %session_id,
                    user_id = %user_ctx.user_id,
                    "Auto-captured memories from conversation turn"
                );
            }
            extracted
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                agent_id = %agent_id,
                session_id = %session_id,
                "auto_capture failed (non-blocking)"
            );
            vec![]
        }
    }
}
