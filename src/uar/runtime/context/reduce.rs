//! The one history-reduction path.
//!
//! A run previously reduced history twice: a message-count pass over
//! [`crate::uar::context::ContextStrategy`] followed by an independent
//! token-budget pass with its own strategy enum and its own tokenizer. The two
//! disagreed about what a token was, neither knew the system message was
//! pinned, and neither knew about tool-call pairs.
//!
//! [`reduce_history`] is now the only way a run reduces history. It runs the
//! structural stage and the token-budget stage in order, from the single
//! operator-declared strategy, with the system message pinned out of reach of
//! both, and normalizes tool-call pairs once at the end so the provider never
//! receives a dangling call or an orphaned result.
//!
//! The operator-facing [`crate::uar::context::ContextStrategy`] remains the
//! only declared strategy. The internal
//! [`crate::uar::domain::context::ContextStrategy`] is derived from it here and
//! is not part of any persisted policy.

use crate::llm::{LlmDriver, Message};
use crate::uar::context::{
    split_pinned_system, trim_history_with_summarization, ContextStrategy as DeclaredStrategy,
};
use crate::uar::domain::context::{
    ContextAction, ContextConfig, ContextStrategy as BudgetStrategy,
};

use super::manager::ContextManager;
use super::normalize::{normalize_history, NormalizeReport};

/// What [`reduce_history`] did to a run's history.
#[derive(Debug, Clone, Default)]
pub struct ReduceReport {
    /// The token-budget stage's report, when it changed anything.
    pub context_action: Option<ContextAction>,
    /// What normalization repaired, if anything.
    pub normalize: NormalizeReport,
}

/// Map the operator-declared strategy onto the token-budget stage's strategy.
///
/// The declared strategy is the source of truth; this is a derivation, not a
/// second configuration surface.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::uar::context::ContextStrategy as Declared;
/// use universal_agent_runtime::uar::domain::context::ContextStrategy as Budget;
/// use universal_agent_runtime::uar::runtime::context::reduce::budget_strategy_for;
///
/// assert_eq!(
///     budget_strategy_for(&Declared::TruncateMiddle { keep_first: 2, keep_last: 4 }),
///     Budget::KeepFirstLast
/// );
/// ```
#[must_use]
pub fn budget_strategy_for(declared: &DeclaredStrategy) -> BudgetStrategy {
    match declared {
        DeclaredStrategy::None => BudgetStrategy::None,
        // A message-count window still needs a token ceiling; the budget stage
        // enforces it by keeping the most recent turns that fit.
        DeclaredStrategy::SlidingWindow { .. } | DeclaredStrategy::Auto => {
            BudgetStrategy::SlidingWindow
        }
        DeclaredStrategy::TruncateMiddle { .. } => BudgetStrategy::KeepFirstLast,
        DeclaredStrategy::Summarize { .. } | DeclaredStrategy::Hierarchical { .. } => {
            BudgetStrategy::ProgressiveSummarization
        }
    }
}

/// Build the token-budget stage's configuration from the declared strategy and
/// the resolved model's context window.
#[must_use]
fn budget_config(declared: &DeclaredStrategy, context_limit: usize) -> ContextConfig {
    let mut config = ContextConfig {
        strategy: budget_strategy_for(declared),
        ..ContextConfig::default()
    };
    // Leave room for the response; the budget stage subtracts its own buffer
    // when `max_tokens` is unset, so set it explicitly for determinism.
    config.max_tokens = Some(context_limit.saturating_sub(1_000));
    if let DeclaredStrategy::Summarize {
        summary_max_tokens,
        model,
        ..
    } = declared
    {
        config.summary_budget = Some(*summary_max_tokens);
        config.summarization_model.clone_from(model);
    }
    config
}

/// Reduce a run's history once: structural stage, token-budget stage, then
/// tool-call normalization, with the system message pinned throughout.
///
/// `messages` is the full list including the system message at index 0 when
/// one is present. The returned list is what the provider receives.
pub async fn reduce_history(
    messages: Vec<Message>,
    declared: &DeclaredStrategy,
    context_limit: usize,
    driver: Option<&dyn LlmDriver>,
) -> (Vec<Message>, ReduceReport) {
    let (system, history) = split_pinned_system(messages);

    // Stage 1, structural: message-count trimming and LLM summarization.
    let after_structural =
        trim_history_with_summarization(system, history, declared, driver).await;

    // Stage 2, token budget: enforce the model's window. The manager preserves
    // system messages itself, so the pinned message can travel with the list.
    let manager = ContextManager::new(budget_config(declared, context_limit));
    let (after_budget, context_action) = manager
        .apply_with_driver(after_structural, context_limit, driver)
        .await;

    // Stage 3: repair any tool-call pair the reduction severed.
    let mut final_messages = after_budget;
    let normalize = normalize_history(&mut final_messages);

    (
        final_messages,
        ReduceReport {
            context_action,
            normalize,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{MessageContent, MessageRole, ToolCall, ToolCallFunction};

    fn msg(role: MessageRole, s: &str) -> Message {
        Message {
            role,
            content: MessageContent::text(s),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    #[test]
    fn declared_strategy_drives_the_budget_stage() {
        assert_eq!(
            budget_strategy_for(&DeclaredStrategy::SlidingWindow { max_messages: 20 }),
            BudgetStrategy::SlidingWindow
        );
        assert_eq!(
            budget_strategy_for(&DeclaredStrategy::Hierarchical {
                short_term_turns: 5,
                mid_term_summary_tokens: 2_000,
                long_term_facts_tokens: 500,
            }),
            BudgetStrategy::ProgressiveSummarization
        );
        assert_eq!(
            budget_strategy_for(&DeclaredStrategy::None),
            BudgetStrategy::None
        );
    }

    #[tokio::test]
    async fn system_message_survives_both_stages() {
        let mut messages = vec![msg(MessageRole::System, "identity and skills")];
        for i in 0..80 {
            messages.push(msg(MessageRole::User, &format!("turn-{i}")));
        }

        let (out, _report) = reduce_history(
            messages,
            &DeclaredStrategy::SlidingWindow { max_messages: 20 },
            4_000,
            None,
        )
        .await;

        assert_eq!(out[0].role, MessageRole::System);
        assert_eq!(out[0].content.as_text(), Some("identity and skills"));
        assert!(out.len() <= 21);
    }

    #[tokio::test]
    async fn severed_tool_pair_is_repaired_after_reduction() {
        let call = ToolCall {
            id: "c1".to_string(),
            call_type: "function".to_string(),
            function: ToolCallFunction {
                name: "t".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let mut messages = vec![msg(MessageRole::System, "sys")];
        for i in 0..40 {
            messages.push(msg(MessageRole::User, &format!("turn-{i}")));
        }
        // A call whose result the window will cut away.
        messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(vec![call]),
        });

        let (out, report) = reduce_history(
            messages,
            &DeclaredStrategy::SlidingWindow { max_messages: 5 },
            4_000,
            None,
        )
        .await;

        assert_eq!(report.normalize.synthesized, vec!["c1".to_string()]);
        let last = out.last().expect("history is non-empty");
        assert_eq!(last.role, MessageRole::Tool);
        assert_eq!(last.tool_call_id.as_deref(), Some("c1"));
    }
}
