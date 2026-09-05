//! Bounded reattachment from the host's latest activation state.

use crate::config::SkillReattachmentBudget;
use crate::llm::{Message, MessageContent, MessageRole};
use crate::uar::runtime::context::token_service::TokenService;
use crate::uar::runtime::context::truncate::{TruncationPolicy, formatted_truncate_for_model};
use crate::uar::runtime::prompt::{PromptFragment, RenderOptions, render_with_options};

use super::activation::ActivatedSkill;

/// Attach to a request copy only. Persistent history remains body-free, so
/// subsequent compaction cannot summarize obsolete copies of a skill body.
/// Reattachment follows the leading system prompt and any summary messages.
pub fn reattach_skills(
    history: &[Message],
    active: &[ActivatedSkill],
    model: &str,
    context_limit: usize,
    budget: SkillReattachmentBudget,
    options: RenderOptions,
) -> (Vec<Message>, Vec<PromptFragment>) {
    let mut latest = active.iter().collect::<Vec<_>>();
    latest.sort_by_key(|activation| activation.sequence);
    let fragments = latest
        .into_iter()
        .map(|activation| activation.fragment())
        .collect::<Vec<_>>();
    reattach_fragments(history, &fragments, model, context_limit, budget, options)
}

/// Equivalent pure-data seam for contributors: body fragments are oldest-first.
pub fn reattach_fragments(
    history: &[Message],
    bodies: &[PromptFragment],
    model: &str,
    context_limit: usize,
    budget: SkillReattachmentBudget,
    options: RenderOptions,
) -> (Vec<Message>, Vec<PromptFragment>) {
    let insert_at = history
        .iter()
        .take_while(|message| message.role == MessageRole::System)
        .count();
    let mut fragments = Vec::new();
    let mut request = history.to_vec();
    let request_limit = context_limit.saturating_sub(1_000);

    for original in bodies.iter().rev() {
        let mut body_limit = budget
            .per_skill_tokens
            .min(budget.total_tokens)
            .min(TokenService::count(model, &original.content));
        while body_limit > 0 {
            let content = formatted_truncate_for_model(
                &original.content,
                TruncationPolicy::Tokens(body_limit),
                model,
            );
            if content.is_empty() {
                break;
            }
            let fragment = PromptFragment::new(
                &original.id,
                original.section,
                &original.source,
                original.authority,
                original.role,
                original.retention,
                content,
            );
            let individual = TokenService::count(
                model,
                &render_with_options(std::slice::from_ref(&fragment), options),
            );
            let mut candidate = fragments.clone();
            candidate.push(fragment);
            let rendered = render_with_options(&candidate, options);
            let total = TokenService::count(model, &rendered);
            let mut candidate_request = history.to_vec();
            candidate_request.insert(
                insert_at,
                Message {
                    role: MessageRole::System,
                    content: MessageContent::text(rendered),
                    tool_call_id: None,
                    tool_calls: None,
                },
            );
            let request_tokens = TokenService::count_messages(model, &candidate_request);
            let overflow = individual
                .saturating_sub(budget.per_skill_tokens)
                .max(total.saturating_sub(budget.total_tokens))
                .max(request_tokens.saturating_sub(request_limit));
            if overflow == 0 {
                fragments = candidate;
                request = candidate_request;
                break;
            }
            body_limit = body_limit.saturating_sub(overflow);
        }
    }
    (request, fragments)
}
