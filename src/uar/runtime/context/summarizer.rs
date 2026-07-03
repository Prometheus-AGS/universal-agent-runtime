use crate::llm::{LlmDriver, LlmRequest, Message, MessageRole};
use crate::normalized::NormalizedEvent;
use anyhow::Result;
use futures::StreamExt;
use tracing::{info, warn};

const SUMMARIZATION_PROMPT: &str = r#"You are a conversation summarizer. Summarize the following conversation messages into a concise summary that preserves:
1. Key decisions and conclusions
2. Important tool call results and data
3. User preferences and requirements stated
4. Any action items or next steps discussed

Be concise but comprehensive. Output only the summary, no preamble."#;

/// Summarize a slice of conversation messages into a single summary string
/// using an LLM call.
pub async fn summarize_messages(messages: &[Message], driver: &dyn LlmDriver) -> Result<String> {
    if messages.is_empty() {
        return Ok(String::new());
    }

    // Build conversation text from messages
    let mut conversation_text = String::new();
    for msg in messages {
        let role = match msg.role {
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
            MessageRole::System => "System",
            MessageRole::Tool => "Tool",
        };
        let content = msg.content.as_text().unwrap_or("[non-text content]");
        conversation_text.push_str(&format!("{role}: {content}\n\n"));
    }

    // Build request as JSON values matching LlmRequest.messages format
    let request = LlmRequest {
        messages: vec![
            serde_json::json!({
                "role": "system",
                "content": SUMMARIZATION_PROMPT
            }),
            serde_json::json!({
                "role": "user",
                "content": format!("Summarize this conversation:\n\n{conversation_text}")
            }),
        ],
        tools: vec![],
        cache_strategy: None,
        thinking_config: None,
        anthropic_system: None,
        extra_params: None,
    };

    let mut stream = driver.stream(request).await?;
    let mut summary = String::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(NormalizedEvent::MessageDelta { text }) => {
                summary.push_str(&text);
            }
            Ok(NormalizedEvent::Done) => break,
            Err(e) => {
                warn!("Summarization stream error: {e}");
                break;
            }
            _ => {}
        }
    }

    if summary.is_empty() {
        anyhow::bail!("Summarization produced empty result");
    }

    info!(
        summary_len = summary.len(),
        input_messages = messages.len(),
        "Conversation summarized"
    );

    Ok(summary)
}
