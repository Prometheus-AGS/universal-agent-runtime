//! Test 4.5: Verify summarization is triggered when token count exceeds threshold.

#[cfg(test)]
mod summarization {
    use universal_agent_runtime::llm::{Message, MessageContent, MessageRole};
    use universal_agent_runtime::uar::domain::context::{ContextConfig, ContextStrategy};
    use universal_agent_runtime::uar::runtime::context::manager::ContextManager;

    fn make_msg(content: &str, role: MessageRole) -> Message {
        Message {
            role,
            content: MessageContent::text(content),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    #[tokio::test]
    async fn progressive_summarization_falls_back_without_driver() {
        // When no LLM driver is provided, ProgressiveSummarization
        // should fall back to KeepFirstLast
        let config = ContextConfig {
            strategy: ContextStrategy::ProgressiveSummarization,
            max_tokens: Some(50),
            trigger_threshold: 0.1, // very low to trigger
            ..Default::default()
        };
        let manager = ContextManager::new(config);

        let mut messages = vec![make_msg("System prompt", MessageRole::System)];
        for i in 0..30 {
            messages.push(make_msg(
                &format!("This is a longer message number {i} with enough content to fill tokens"),
                MessageRole::User,
            ));
        }

        // Without a driver, should fall back to KeepFirstLast
        let (result, action) = manager.apply(messages.clone(), 1000).await;

        assert!(action.is_some());
        let act = action.unwrap();
        // Falls back to KeepFirstLast since no driver provided
        assert_eq!(act.strategy, ContextStrategy::KeepFirstLast);
        assert!(act.messages_removed > 0);
        assert!(result.len() < messages.len());
    }

    #[tokio::test]
    async fn context_not_triggered_below_threshold() {
        let config = ContextConfig {
            strategy: ContextStrategy::SlidingWindow,
            max_tokens: Some(100_000),
            trigger_threshold: 0.85,
            ..Default::default()
        };
        let manager = ContextManager::new(config);

        // Small conversation - should NOT trigger
        let messages = vec![
            make_msg("System", MessageRole::System),
            make_msg("Hello", MessageRole::User),
            make_msg("Hi there!", MessageRole::Assistant),
        ];

        let (result, action) = manager.apply(messages.clone(), 128_000).await;

        assert!(action.is_none(), "Should not trigger with low token usage");
        assert_eq!(result.len(), messages.len());
    }
}
