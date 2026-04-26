//! Prompt caching strategy for the Anthropic Messages API.
//!
//! Anthropic's prompt caching allows reuse of previously processed prompt
//! prefixes, reducing latency and cost. This module provides a configurable
//! strategy that annotates request components with `cache_control` markers.
//!
//! Cache breakpoints are placed on:
//! - The last system prompt block (stable across turns)
//! - The last tool definition (stable across turns)
//! - Recent conversation turns (sliding window)

use super::anthropic_types::{CacheControl, MessagesRequest};

/// Configuration for Anthropic prompt caching behavior.
///
/// The strategy determines which parts of the request are annotated
/// with `cache_control: { type: "ephemeral" }` markers.
#[derive(Debug, Clone)]
pub struct CacheStrategy {
    /// Whether to cache the system prompt.
    pub cache_system_prompt: bool,
    /// Whether to cache tool definitions.
    pub cache_tools: bool,
    /// Number of recent conversation turns (user+assistant pairs) to cache.
    pub cache_conversation_turns: usize,
    /// Minimum estimated token count before caching a block.
    /// Uses a rough heuristic of `text.len() / 4` to estimate tokens.
    pub min_tokens_to_cache: u32,
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self {
            cache_system_prompt: true,
            cache_tools: true,
            cache_conversation_turns: 4,
            min_tokens_to_cache: 1024,
        }
    }
}

impl CacheStrategy {
    /// Apply cache control annotations to a mutable [`MessagesRequest`].
    ///
    /// This modifies the request in-place, adding `cache_control` markers
    /// to the appropriate blocks based on the strategy configuration.
    pub fn apply(&self, req: &mut MessagesRequest) {
        self.apply_system_cache(req);
        self.apply_tools_cache(req);
        self.apply_conversation_cache(req);
    }

    /// Annotate the last system block with cache control if it meets
    /// the minimum token threshold.
    fn apply_system_cache(&self, req: &mut MessagesRequest) {
        if !self.cache_system_prompt {
            return;
        }

        if let Some(ref mut system) = req.system {
            if let Some(last) = system.last_mut() {
                // Simple heuristic: text length / 4 approximates token count.
                let estimated_tokens = last.text.len() / 4;
                if estimated_tokens >= self.min_tokens_to_cache as usize {
                    last.cache_control = Some(CacheControl::ephemeral());
                }
            }
        }
    }

    /// Annotate the last tool definition with cache control.
    ///
    /// Tool definitions are typically stable across conversation turns,
    /// making them excellent candidates for caching.
    fn apply_tools_cache(&self, req: &mut MessagesRequest) {
        if !self.cache_tools {
            return;
        }

        if let Some(ref mut tools) = req.tools {
            if let Some(last) = tools.last_mut() {
                last.cache_control = Some(CacheControl::ephemeral());
            }
        }
    }

    /// Annotate recent conversation turns with cache control.
    ///
    /// Walks backwards through the messages and marks up to
    /// `cache_conversation_turns` user+assistant boundaries. Each
    /// boundary is the last message before a role transition from
    /// assistant to user (i.e., the assistant message just before
    /// the user speaks again).
    fn apply_conversation_cache(&self, req: &mut MessagesRequest) {
        if self.cache_conversation_turns == 0 {
            return;
        }

        let len = req.messages.len();
        if len < 2 {
            return;
        }

        // Walk backwards, finding turn boundaries where the role changes
        // from assistant to user. We annotate the last message in each
        // turn pair by injecting cache_control into the content.
        let mut turns_marked = 0;
        let mut i = len;

        while i >= 2 && turns_marked < self.cache_conversation_turns {
            i -= 1;

            let current_role = req.messages[i].role.as_str();
            let prev_role = req.messages[i - 1].role.as_str();

            // Mark turn boundaries: where an assistant message is followed by a user message.
            if prev_role == "assistant" && current_role == "user" {
                // Annotate the assistant message (the end of the turn).
                Self::annotate_message_cache(&mut req.messages[i - 1]);
                turns_marked += 1;
            }
        }
    }

    /// Inject `cache_control` into a message's content.
    ///
    /// If the content is a JSON array (multipart), annotates the last block.
    /// If it is a string, converts it to an array with a single annotated text block.
    fn annotate_message_cache(message: &mut super::anthropic_types::AnthropicMessage) {
        let content = &mut message.content;

        if let Some(arr) = content.as_array_mut() {
            // Annotate the last content block in the array.
            if let Some(last) = arr.last_mut() {
                if let Some(obj) = last.as_object_mut() {
                    obj.insert(
                        "cache_control".to_string(),
                        serde_json::json!({"type": "ephemeral"}),
                    );
                }
            }
        } else if let Some(text) = content.as_str() {
            // Convert string content to an array with a cache-annotated text block.
            let text = text.to_owned();
            *content = serde_json::json!([{
                "type": "text",
                "text": text,
                "cache_control": {"type": "ephemeral"}
            }]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::anthropic_types::{AnthropicMessage, SystemBlock};

    fn make_request() -> MessagesRequest {
        MessagesRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            system: Some(vec![SystemBlock {
                block_type: "text".to_string(),
                // ~2000 chars => ~500 estimated tokens — above default threshold of 1024?
                // Actually 2000/4 = 500, which is below 1024. Let's use a bigger string.
                text: "x".repeat(5000), // 5000/4 = 1250 estimated tokens
                cache_control: None,
            }]),
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: serde_json::json!("Hello"),
                },
                AnthropicMessage {
                    role: "assistant".to_string(),
                    content: serde_json::json!("Hi there!"),
                },
                AnthropicMessage {
                    role: "user".to_string(),
                    content: serde_json::json!("How are you?"),
                },
            ],
            tools: Some(vec![crate::llm::anthropic_types::ToolDef {
                name: "get_weather".to_string(),
                description: "Get the weather".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                cache_control: None,
            }]),
            tool_choice: None,
            stream: true,
            thinking: None,
        }
    }

    #[test]
    fn test_default_strategy_caches_system() {
        let strategy = CacheStrategy::default();
        let mut req = make_request();
        strategy.apply(&mut req);

        let system = req.system.as_ref().unwrap();
        assert!(system.last().unwrap().cache_control.is_some());
    }

    #[test]
    fn test_default_strategy_caches_tools() {
        let strategy = CacheStrategy::default();
        let mut req = make_request();
        strategy.apply(&mut req);

        let tools = req.tools.as_ref().unwrap();
        assert!(tools.last().unwrap().cache_control.is_some());
    }

    #[test]
    fn test_strategy_skips_short_system_prompt() {
        let strategy = CacheStrategy::default();
        let mut req = make_request();
        // Set a short system prompt that won't meet the threshold.
        req.system = Some(vec![SystemBlock {
            block_type: "text".to_string(),
            text: "Be helpful.".to_string(),
            cache_control: None,
        }]);
        strategy.apply(&mut req);

        let system = req.system.as_ref().unwrap();
        assert!(system.last().unwrap().cache_control.is_none());
    }

    #[test]
    fn test_disabled_caching() {
        let strategy = CacheStrategy {
            cache_system_prompt: false,
            cache_tools: false,
            cache_conversation_turns: 0,
            min_tokens_to_cache: 1024,
        };
        let mut req = make_request();
        strategy.apply(&mut req);

        let system = req.system.as_ref().unwrap();
        assert!(system.last().unwrap().cache_control.is_none());

        let tools = req.tools.as_ref().unwrap();
        assert!(tools.last().unwrap().cache_control.is_none());
    }

    #[test]
    fn test_conversation_turn_caching() {
        let strategy = CacheStrategy {
            cache_conversation_turns: 2,
            ..CacheStrategy::default()
        };
        let mut req = make_request();
        strategy.apply(&mut req);

        // The assistant message (index 1) should have cache_control injected.
        let assistant_content = &req.messages[1].content;
        // It was a string, so it should now be an array with cache_control.
        assert!(assistant_content.is_array());
        let blocks = assistant_content.as_array().unwrap();
        assert!(blocks[0].get("cache_control").is_some());
    }
}
