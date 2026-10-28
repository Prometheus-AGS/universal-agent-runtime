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
    ContextStrategy as DeclaredStrategy, split_pinned_system,
};
use crate::uar::context::strategy::trim_history_with_marked_prose_receipt_with_allowance;
use crate::uar::domain::context::{
    ContextAction, ContextConfig, ContextStrategy as BudgetStrategy,
};

use super::manager::ContextManager;
use super::normalize::{HistoryValidationError, NormalizeReport, normalize_history};
use super::summarizer::HostMarkedProseSpan;
use super::token_service::TokenService;

/// Mark only canonical plain dialogue as reducible prose. Input includes the
/// optional pinned system message; returned offsets exclude that message, as
/// required by the structural summarizer. No content heuristic grants authority.
#[must_use]
pub fn canonical_prose_spans(messages: &[Message]) -> Vec<HostMarkedProseSpan> {
    let offset = usize::from(messages.first().is_some_and(|message|
        message.role == crate::llm::MessageRole::System));
    let history = &messages[offset..];
    let eligible = |message: &Message| matches!(message.role,
        crate::llm::MessageRole::User | crate::llm::MessageRole::Assistant)
        && message.tool_call_id.is_none()
        && message.tool_calls.as_ref().is_none_or(Vec::is_empty)
        && matches!(&message.content, crate::llm::MessageContent::Text { content }
            if !content.trim().is_empty());
    let mut spans = Vec::new();
    let mut index = 0;
    while index < history.len() {
        if !eligible(&history[index]) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < history.len() && eligible(&history[index])
            && history[index].role == history[start].role
        {
            index += 1;
        }
        spans.push(HostMarkedProseSpan::new(format!("canonical-prose-{start}-{index}"), start, index));
    }
    spans
}

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
    /// Request content the history cannot displace (tool definitions, world
    /// state and the reserved response) already fills the model context, so
    /// no history can be sent. Refused before any provider call.
    #[error(
        "fixed request content needs {required_tokens} tokens ({tool_tokens} tool definitions, {world_tokens} world state, {output_tokens} reserved output) but the model context is {context_limit}"
    )]
    FixedRequestOverflow {
        required_tokens: usize,
        tool_tokens: usize,
        world_tokens: usize,
        output_tokens: usize,
        context_limit: usize,
    },
}

impl ReduceHistoryError {
    /// Stable event code for the explicit preparation failure.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidHistory(_) => "invalid_history",
            Self::ProtectedHistoryOverflow { .. } => "protected_history_overflow",
            Self::ContextOverflow { .. } | Self::FixedRequestOverflow { .. } => "context_overflow",
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
    /// Body-free observed structural reduction receipt. Digests identify exact
    /// input/output messages; labels alone never establish a summary occurred.
    pub structural_receipt: Option<serde_json::Value>,
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
    match declared {
        DeclaredStrategy::Summarize { summary_max_tokens, model, .. } => {
            config.summary_budget = Some(*summary_max_tokens);
            config.summarization_model.clone_from(model);
        }
        DeclaredStrategy::Hierarchical { mid_term_summary_tokens, long_term_facts_tokens, .. } => {
            config.summary_budget = Some(mid_term_summary_tokens.saturating_add(*long_term_facts_tokens));
        }
        _ => {}
    }
    config
}

/// Output tokens withheld from the model context for the response. This is the
/// reserve [`reduce_history`] subtracts when it derives the input allowance.
pub const RESPONSE_RESERVE_TOKENS: usize = 1_000;

/// Tokens the tool definitions occupy in a request, counted over the same
/// OpenAI function-schema JSON the run attaches to every model call.
#[must_use]
pub fn tool_definition_tokens(model: &str, tools: &[serde_json::Value]) -> usize {
    if tools.is_empty() {
        return 0;
    }
    // `Value`'s `Display` is infallible; the array form matches the wire field.
    let serialized = serde_json::Value::Array(tools.to_vec()).to_string();
    TokenService::count(model, &serialized)
}

/// OpenAI tool JSON of the `search_tools` control the orchestrator adds to the
/// request whenever deferred MCP tools exist. Built from a scratch registry so
/// admission counts exactly the descriptor the first request carries. `None`
/// when the control cannot be built; the orchestrator then counts the actual
/// request tools per step.
pub async fn search_tools_definition() -> Option<serde_json::Value> {
    use crate::mcp::exposure::McpToolExposure;
    use crate::uar::runtime::native_skills::search_tools::{SEARCH_TOOLS_NAME, SearchToolsTool};
    let scratch = crate::uar::runtime::native_skill::NativeSkillRegistry::new();
    scratch
        .register(SearchToolsTool::new(McpToolExposure::default()))
        .await
        .ok()?;
    scratch
        .descriptor(SEARCH_TOOLS_NAME)
        .await
        .map(|descriptor| descriptor.openai_tool_json())
}

/// Event text for a reduction failure. An overflow names the liabilities the
/// allowance already excludes: tool definitions plus world state
/// (`fixed_tokens`, combined) and the response reserve.
#[must_use]
pub fn liability_message(error: &ReduceHistoryError, fixed_tokens: usize) -> String {
    match error {
        ReduceHistoryError::ContextOverflow { .. } => format!(
            "{error}; the allowance already excludes {fixed_tokens} tokens of tool definitions and world state and the applicable output/reasoning reserve"
        ),
        _ => error.to_string(),
    }
}

/// Carry a reduction failure's stable code through `AssemblyError`, which has
/// no code field: `"<code>: <liability message>"`.
#[must_use]
pub fn tag_assembly_failure(error: &ReduceHistoryError, fixed_tokens: usize) -> String {
    format!("{}: {}", error.code(), liability_message(error, fixed_tokens))
}

/// Inverse of [`tag_assembly_failure`]: the stable code and message, or `None`
/// for an untagged contributor failure.
#[must_use]
pub fn split_assembly_failure(message: &str) -> Option<(&'static str, &str)> {
    ["context_overflow", "protected_history_overflow", "invalid_history"]
        .into_iter()
        .find_map(|code| {
            message
                .strip_prefix(code)?
                .strip_prefix(": ")
                .map(|rest| (code, rest))
        })
}

/// The context limit to hand [`reduce_history`] once request content that
/// history cannot displace (world state and tool definitions) is withheld.
/// [`reduce_history`] then subtracts the response reserve itself.
///
/// # Errors
/// [`ReduceHistoryError::FixedRequestOverflow`] when that content plus the
/// response reserve leaves no room for any history.
pub fn reserve_fixed_request_tokens(
    context_limit: usize,
    world_tokens: usize,
    tool_tokens: usize,
) -> Result<usize, ReduceHistoryError> {
    let fixed = world_tokens.saturating_add(tool_tokens);
    let required_tokens = fixed.saturating_add(RESPONSE_RESERVE_TOKENS);
    if required_tokens >= context_limit {
        return Err(ReduceHistoryError::FixedRequestOverflow {
            required_tokens,
            tool_tokens,
            world_tokens,
            output_tokens: RESPONSE_RESERVE_TOKENS,
            context_limit,
        });
    }
    Ok(context_limit - fixed)
}

/// Withhold tools and world state from an already reserved input allowance.
/// No capacity is inferred here; the caller owns the destination contract.
pub fn reserve_fixed_input_tokens(
    input_allowance: usize,
    world_tokens: usize,
    tool_tokens: usize,
) -> Result<usize, ReduceHistoryError> {
    let fixed = world_tokens.saturating_add(tool_tokens);
    if fixed >= input_allowance {
        return Err(ReduceHistoryError::ContextOverflow {
            required_tokens: fixed,
            input_allowance,
        });
    }
    Ok(input_allowance - fixed)
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
    reduce_history_with_input_allowance(
        messages, declared, model, context_limit.saturating_sub(RESPONSE_RESERVE_TOKENS),
        driver, request_budget_contract, eligible_prose,
    ).await
}

/// Reduce against a host-reviewed input allowance that already excludes output,
/// reasoning, uncertainty and fixed request content. Counts used for trimming
/// remain estimates; the destination driver recounts the complete final wire.
pub async fn reduce_history_with_input_allowance(
    messages: Vec<Message>,
    declared: &DeclaredStrategy,
    model: &str,
    input_allowance: usize,
    driver: Option<&dyn LlmDriver>,
    request_budget_contract: Option<&super::budget::RequestBudgetContract>,
    eligible_prose: &[HostMarkedProseSpan],
) -> Result<(Vec<Message>, ReduceReport), ReduceHistoryError> {
    let allowance = request_budget_contract.cloned().map(super::budget::AuxiliaryRequestAllowance::Exact);
    reduce_history_with_input_allowance_and_auxiliary(messages, declared, model, input_allowance,
        driver, allowance.as_ref(), eligible_prose).await
}

/// Reduce with a distinct captured allowance for auxiliary model requests.
pub async fn reduce_history_with_input_allowance_and_auxiliary(
    messages: Vec<Message>,
    declared: &DeclaredStrategy,
    model: &str,
    input_allowance: usize,
    driver: Option<&dyn LlmDriver>,
    auxiliary_allowance: Option<&super::budget::AuxiliaryRequestAllowance>,
    eligible_prose: &[HostMarkedProseSpan],
) -> Result<(Vec<Message>, ReduceReport), ReduceHistoryError> {
    let original_history = serde_json::json!(&messages);
    let original_tokens = TokenService::count_messages(model, &messages);
    let original_message_count = messages.len();
    let normalize = normalize_history(&messages)?;
    let protected = protected_groups(&messages);
    let (system, history) = split_pinned_system(messages);

    // Stage 1, structural: message-count trimming and LLM summarization.
    let structural = trim_history_with_marked_prose_receipt_with_allowance(
        system,
        history,
        declared,
        driver,
        model,
        auxiliary_allowance,
        eligible_prose,
    )
    .await;
    let after_structural = structural.messages;
    let structural_action = (original_history != serde_json::json!(&after_structural)).then(|| ContextAction {
        strategy: budget_strategy_for(declared),
        messages_removed: original_message_count.saturating_sub(after_structural.len()),
        tokens_saved: original_tokens.saturating_sub(TokenService::count_messages(model, &after_structural)),
        was_applied: true,
        summary_generated: matches!(declared, DeclaredStrategy::Summarize { .. } | DeclaredStrategy::Hierarchical { .. }),
    });
    let mut structural_receipt = serde_json::json!({
        "schema_version": 1,
        "requested_strategy": declared,
        "source_sha256": format!("sha256:{}", crate::uar::runtime::checkpoint::raw_digest(&original_history)),
        "prepared_sha256": format!("sha256:{}", crate::uar::runtime::checkpoint::raw_digest(&after_structural)),
        "source_messages": original_message_count,
        "prepared_messages": after_structural.len(),
        "source_tokens": original_tokens,
        "prepared_tokens": TokenService::count_messages(model, &after_structural),
        "eligible_spans": eligible_prose.len(),
        "changed": structural_action.is_some(),
        "summary_generated": structural_action.as_ref().is_some_and(|action| action.summary_generated),
        "tier_calls": structural.tier_receipts,
    });

    // Stage 2, token budget: enforce the model's window. The manager preserves
    // system messages itself, so the pinned message can travel with the list.
    let mut config = budget_config(declared, input_allowance);
    config.max_tokens = Some(input_allowance);
    let manager = ContextManager::for_model(config, model);
    let (after_budget, mut context_action) = manager
        // Structural summarization has already consumed the marks. Generated
        // summaries do not inherit eligibility implicitly and cannot recurse.
        .apply_with_marked_prose(after_structural, input_allowance, driver, None, &[])
        .await;
    if let Some(structural) = structural_action {
        match &mut context_action {
            Some(action) => {
                action.messages_removed = original_message_count.saturating_sub(after_budget.len());
                action.tokens_saved = original_tokens.saturating_sub(TokenService::count_messages(model, &after_budget));
                action.summary_generated |= structural.summary_generated;
            }
            None => context_action = Some(structural),
        }
    }

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
    structural_receipt["final_sha256"] = format!("sha256:{}",
        crate::uar::runtime::checkpoint::raw_digest(&final_messages)).into();
    structural_receipt["final_messages"] = final_messages.len().into();
    structural_receipt["final_tokens"] = required_tokens.into();
    structural_receipt["input_allowance"] = input_allowance.into();
    structural_receipt["protected_groups"] = protected.len().into();
    structural_receipt["protected_groups_preserved"] = true.into();

    let history_rewritten = original_history != serde_json::json!(&final_messages);
    Ok((
        final_messages,
        ReduceReport {
            history_rewritten,
            context_action,
            normalize,
            structural_receipt: Some(structural_receipt),
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

    const COUNT_MODEL: &str = "openai/gpt-4o";

    #[tokio::test]
    async fn tool_definitions_shrink_the_history_allowance() {
        let user = msg(
            MessageRole::User,
            "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu",
        );
        let system = msg(MessageRole::System, "identity and skills");
        let per_message =
            TokenService::count_messages(COUNT_MODEL, std::slice::from_ref(&user)) - 3;
        let system_cost =
            TokenService::count_messages(COUNT_MODEL, std::slice::from_ref(&system)) - 3;
        assert!(per_message >= 8, "slack arithmetic below needs >= 8");

        // Pad one tool definition until it costs a whole number of messages, so
        // both reductions below leave the same half-message of slack.
        let (tools, tool_tokens) = (2 * per_message..6 * per_message)
            .map(|pad| {
                let tools = vec![serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "lookup",
                        "description": "word ".repeat(pad),
                        "parameters": {"type": "object", "properties": {}},
                    },
                })];
                let tokens = tool_definition_tokens(COUNT_MODEL, &tools);
                (tools, tokens)
            })
            .find(|(_, tokens)| tokens % per_message == 0)
            .expect("padding reaches a whole number of messages");
        assert_eq!(tool_definition_tokens(COUNT_MODEL, &[]), 0);
        assert!(!tools.is_empty());

        let kept_without_tools = 40_usize;
        let allowance = system_cost + kept_without_tools * per_message + per_message / 2;
        let context_limit = allowance + RESPONSE_RESERVE_TOKENS;
        let mut history = vec![system];
        history.extend(std::iter::repeat_n(user, 120));
        let strategy = DeclaredStrategy::SlidingWindow { max_messages: 200 };

        let (unaware, _) =
            reduce_history(history.clone(), &strategy, COUNT_MODEL, context_limit, None)
                .await
                .expect("history reduces without tool accounting");
        let reduction_limit = reserve_fixed_request_tokens(context_limit, 0, tool_tokens)
            .expect("tools leave room for history");
        let (aware, _) = reduce_history(history, &strategy, COUNT_MODEL, reduction_limit, None)
            .await
            .expect("history reduces with tool accounting");

        assert_eq!(unaware.len(), 1 + kept_without_tools);
        assert_eq!(
            aware.len(),
            1 + kept_without_tools - tool_tokens / per_message
        );
        assert!(
            TokenService::count_messages(COUNT_MODEL, &aware)
                + tool_tokens
                + RESPONSE_RESERVE_TOKENS
                <= context_limit,
            "messages plus tools plus reserved output fit the window"
        );
    }

    #[test]
    fn unsatisfiable_fixed_request_content_is_refused_before_reduction() {
        // Measure a large tool definition instead of assuming its cost.
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "lookup",
                "description": "word ".repeat(2_000),
                "parameters": {"type": "object", "properties": {}},
            },
        })];
        let tool_tokens = tool_definition_tokens(COUNT_MODEL, &tools);
        assert!(tool_tokens > 1_500, "a large schema is measured");
        let tight = tool_tokens + RESPONSE_RESERVE_TOKENS;

        // A window holding only the measured tools and the response reserve
        // leaves no room for history: refused, naming each liability.
        let error = reserve_fixed_request_tokens(tight, 0, tool_tokens)
            .expect_err("no room is left for any history");
        assert_eq!(
            error,
            ReduceHistoryError::FixedRequestOverflow {
                required_tokens: tight,
                tool_tokens,
                world_tokens: 0,
                output_tokens: RESPONSE_RESERVE_TOKENS,
                context_limit: tight,
            }
        );
        assert_eq!(error.code(), "context_overflow");
        let text = error.to_string();
        assert!(text.contains(&format!("{tool_tokens} tool definitions")));
        assert!(text.contains("reserved output"));

        // One more token of window admits history; world state is a liability.
        assert_eq!(
            reserve_fixed_request_tokens(tight + 1, 0, tool_tokens),
            Ok(RESPONSE_RESERVE_TOKENS + 1)
        );
        assert!(reserve_fixed_request_tokens(tight + 1, 1, tool_tokens).is_err());
    }

    #[tokio::test]
    async fn deferred_search_tools_control_is_counted() {
        let search = search_tools_definition()
            .await
            .expect("search_tools control descriptor builds");
        let visible = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "lookup",
                "description": "find a record",
                "parameters": {"type": "object", "properties": {}},
            },
        })];
        let mut with_control = visible.clone();
        with_control.push(search);
        assert!(
            tool_definition_tokens(COUNT_MODEL, &with_control)
                > tool_definition_tokens(COUNT_MODEL, &visible)
        );
    }

    #[test]
    fn assembly_failures_keep_their_stable_code() {
        let error = ReduceHistoryError::ContextOverflow {
            required_tokens: 419,
            input_allowance: 117,
        };
        let tagged = tag_assembly_failure(&error, 2_391);
        let (code, message) = split_assembly_failure(&tagged).expect("tagged failure");
        assert_eq!(code, "context_overflow");
        assert!(message.contains("2391 tokens of tool definitions and world state"));
        assert_eq!(split_assembly_failure("something else failed"), None);
    }

    #[tokio::test]
    async fn protected_system_prompt_beyond_the_reduced_allowance_is_refused() {
        let messages = vec![
            msg(MessageRole::System, &"word ".repeat(3_000)),
            msg(MessageRole::User, "hello"),
        ];
        let error = reduce_history(
            messages,
            &DeclaredStrategy::SlidingWindow { max_messages: 20 },
            COUNT_MODEL,
            2_000,
            None,
        )
        .await
        .expect_err("a pinned system message never shrinks to fit");
        assert!(matches!(
            error,
            ReduceHistoryError::ContextOverflow {
                input_allowance: 1_000,
                ..
            }
        ));
    }
}
