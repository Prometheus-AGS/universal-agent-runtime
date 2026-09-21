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
//! both. Canonical tool groups are validated before reduction and must survive
//! byte-for-byte or the reduction returns an explicit overflow.
//!
//! The operator-facing [`crate::uar::context::ContextStrategy`] remains the
//! only declared strategy. The internal
//! [`crate::uar::domain::context::ContextStrategy`] is derived from it here and
//! is not part of any persisted policy.

use crate::llm::{LlmDriver, Message};
use crate::uar::context::{
    ContextStrategy as DeclaredStrategy, split_pinned_system, trim_history_with_marked_prose,
};
use crate::uar::domain::context::{
    ContextAction, ContextConfig, ContextStrategy as BudgetStrategy,
};

use super::manager::ContextManager;
use super::normalize::{HistoryValidationError, NormalizeReport, normalize_history};
use super::summarizer::HostMarkedProseSpan;
use super::token_service::TokenService;

#[derive(Debug)]
struct ProtectedGroup {
    identities: Vec<String>,
    records: Vec<serde_json::Value>,
}

fn protected_groups(messages: &[Message]) -> Vec<ProtectedGroup> {
    let mut groups = Vec::new();
    for (index, message) in messages.iter().enumerate() {
        let call_ids: Vec<String> = message
            .tool_calls
            .iter()
            .flatten()
            .map(|call| call.id.clone())
            .collect();
        let protects_multimodal = matches!(
            &message.content,
            crate::llm::MessageContent::Parts { content } if !content.is_empty()
        );
        if call_ids.is_empty() && !protects_multimodal {
            continue;
        }
        let mut end = index + 1;
        if !call_ids.is_empty() {
            while end < messages.len() && messages[end].role == crate::llm::MessageRole::Tool {
                end += 1;
            }
        }
        groups.push(ProtectedGroup {
            identities: if call_ids.is_empty() {
                vec![format!("assistant_history_index_{index}")]
            } else {
                call_ids
            },
            records: messages[index..end]
                .iter()
                .map(|record| {
                    serde_json::to_value(record).expect("Message serialization is infallible")
                })
                .collect(),
        });
    }
    groups
}

fn verify_protected_groups(
    groups: &[ProtectedGroup],
    reduced: &[Message],
) -> Result<(), ReduceHistoryError> {
    let reduced: Vec<serde_json::Value> = reduced
        .iter()
        .map(|message| serde_json::to_value(message).expect("Message serialization is infallible"))
        .collect();
    let mut search_from = 0usize;
    for group in groups {
        let Some(relative_index) = reduced[search_from..]
            .windows(group.records.len())
            .position(|window| window == group.records)
        else {
            return Err(ReduceHistoryError::ProtectedHistoryOverflow {
                protected_records: group.identities.clone(),
            });
        };
        search_from += relative_index + group.records.len();
    }
    Ok(())
}

/// Why a history cannot be prepared within the requested reduction policy.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReduceHistoryError {
    /// Canonical history is malformed and must be recovered before dispatch.
    #[error("invalid history: {0}")]
    InvalidHistory(#[from] HistoryValidationError),
    /// A reducer would remove, alter, or reorder protected history.
    #[error("protected history does not fit without loss: {protected_records:?}")]
    ProtectedHistoryOverflow { protected_records: Vec<String> },
    /// The intact prepared history exceeds the applicable destination input
    /// allowance. No truncation fallback is permitted for protected originals.
    #[error(
        "context requires {required_tokens} tokens but the input allowance is {input_allowance}"
    )]
    ContextOverflow {
        required_tokens: usize,
        input_allowance: usize,
    },
}

impl ReduceHistoryError {
    /// Stable event code for the explicit preparation failure.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidHistory(_) => "invalid_history",
            Self::ProtectedHistoryOverflow { .. } => "protected_history_overflow",
            Self::ContextOverflow { .. } => "context_overflow",
        }
    }
}

/// What [`reduce_history`] did to a run's history.
#[derive(Debug, Clone, Default)]
pub struct ReduceReport {
    /// Any structural, token-budget, or summarization rewrite.
    /// World-state baselines must be rendered in full after this signal.
    pub history_rewritten: bool,
    /// The token-budget stage's report, when it changed anything.
    pub context_action: Option<ContextAction>,
    /// Counts from lossless history validation.
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

/// Reduce a run's history once: lossless validation, structural stage, token
/// budget, then protected-group verification, with the system message pinned.
///
/// `messages` is the full list including the system message at index 0 when
/// one is present. The returned list is what the provider receives.
pub async fn reduce_history(
    messages: Vec<Message>,
    declared: &DeclaredStrategy,
    model: &str,
    context_limit: usize,
    driver: Option<&dyn LlmDriver>,
) -> Result<(Vec<Message>, ReduceReport), ReduceHistoryError> {
    reduce_history_with_marked_prose(messages, declared, model, context_limit, driver, None, &[])
        .await
}

/// Reduce history while permitting only explicitly host-marked prose ranges
/// to enter summarization. Span indices address conversation history after the
/// optional leading system message is removed.
pub async fn reduce_history_with_marked_prose(
    messages: Vec<Message>,
    declared: &DeclaredStrategy,
    model: &str,
    context_limit: usize,
    driver: Option<&dyn LlmDriver>,
    request_budget_contract: Option<&super::budget::RequestBudgetContract>,
    eligible_prose: &[HostMarkedProseSpan],
) -> Result<(Vec<Message>, ReduceReport), ReduceHistoryError> {
    let original_history = serde_json::json!(&messages);
    let normalize = normalize_history(&messages)?;
    let protected = protected_groups(&messages);
    let (system, history) = split_pinned_system(messages);

    // Stage 1, structural: message-count trimming and LLM summarization.
    let after_structural = trim_history_with_marked_prose(
        system,
        history,
        declared,
        driver,
        model,
        request_budget_contract,
        eligible_prose,
    )
    .await;

    // Stage 2, token budget: enforce the model's window. The manager preserves
    // system messages itself, so the pinned message can travel with the list.
    let config = budget_config(declared, context_limit);
    let input_allowance = config
        .max_tokens
        .unwrap_or(context_limit.saturating_sub(1_000));
    let manager = ContextManager::for_model(config, model);
    let (after_budget, context_action) = manager
        // Structural summarization has already consumed the marks. Generated
        // summaries do not inherit eligibility implicitly and cannot recurse.
        .apply_with_marked_prose(after_structural, context_limit, driver, None, &[])
        .await;

    // Stage 3: reducers may not remove, alter, or reorder protected records.
    // Returning the error leaves the caller's canonical source untouched.
    let final_messages = after_budget;
    verify_protected_groups(&protected, &final_messages)?;
    normalize_history(&final_messages)?;
    let required_tokens = TokenService::count_messages(model, &final_messages);
    if required_tokens > input_allowance {
        return Err(ReduceHistoryError::ContextOverflow {
            required_tokens,
            input_allowance,
        });
    }

    let history_rewritten = original_history != serde_json::json!(&final_messages);
    Ok((
        final_messages,
        ReduceReport {
            history_rewritten,
            context_action,
            normalize,
        },
    ))
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
            "openai/gpt-4o",
            4_000,
            None,
        )
        .await
        .expect("plain history reduces");

        assert_eq!(out[0].role, MessageRole::System);
        assert_eq!(out[0].content.as_text(), Some("identity and skills"));
        assert!(out.len() <= 21);
    }

    #[tokio::test]
    async fn severed_tool_pair_returns_protected_overflow() {
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
        // A call with no result is invalid before any reducer runs.
        messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(vec![call]),
        });

        let error = reduce_history(
            messages,
            &DeclaredStrategy::SlidingWindow { max_messages: 5 },
            "openai/gpt-4o",
            4_000,
            None,
        )
        .await
        .expect_err("missing result is never synthesized");

        assert_eq!(
            error,
            ReduceHistoryError::InvalidHistory(HistoryValidationError::MissingResult {
                call_id: "c1".to_string(),
            })
        );
    }
}
