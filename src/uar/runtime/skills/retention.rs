//! Bounded reattachment from the host's latest activation state.

use crate::config::SkillReattachmentBudget;
use serde::Serialize;
use sha2::{Digest, Sha256};
use crate::llm::{Message, MessageContent, MessageRole};
use crate::uar::runtime::context::token_service::TokenService;
use crate::uar::runtime::context::truncate::{TruncationPolicy, formatted_truncate_for_model};
use crate::uar::runtime::prompt::{PromptFragment, RenderOptions, render_with_options};

use super::activation::ActivatedSkill;

/// Content-free evidence about the actual initial request attachment. These
/// tokenizer estimates do not certify an exact destination framing contract.
#[derive(Debug, Clone, Serialize)]
pub struct SkillReattachmentReceipt {
    pub schema_version: u32,
    pub scope: &'static str,
    pub model: String,
    pub count_quality: &'static str,
    pub counting_revision: &'static str,
    pub per_skill_tokens: usize,
    pub total_tokens: usize,
    pub context_limit: usize,
    pub reserve_tokens: usize,
    pub request_input_limit: usize,
    pub history_tokens: usize,
    pub request_tokens: usize,
    pub rendered_tokens: usize,
    pub rendered_sha256: String,
    pub fragments: Vec<SkillFragmentReceipt>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillFragmentReceipt {
    pub fragment_id: String,
    pub skill_id: Option<String>,
    pub original_body_tokens: usize,
    pub retained_body_tokens: usize,
    pub original_body_sha256: String,
    pub retained_body_sha256: String,
    pub original_rendered_tokens: usize,
    pub retained_rendered_tokens: usize,
    pub original_rendered_sha256: String,
    pub retained_rendered_sha256: String,
    pub retained: bool,
    pub truncated: bool,
}

fn digest(content: &str) -> String {
    let mut result = String::from("sha256:");
    for byte in Sha256::digest(content.as_bytes()) {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

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

/// Initial attachment with content-free evidence from the same bounded result.
pub fn reattach_skills_with_receipt(
    history: &[Message], active: &[ActivatedSkill], model: &str, context_limit: usize,
    budget: SkillReattachmentBudget, options: RenderOptions,
) -> (Vec<Message>, Vec<PromptFragment>, SkillReattachmentReceipt) {
    let mut latest = active.iter().collect::<Vec<_>>();
    latest.sort_by_key(|activation| activation.sequence);
    let fragments = latest.into_iter().map(|activation| activation.fragment()).collect::<Vec<_>>();
    reattach_fragments_with_receipt(history, &fragments, model, context_limit, budget, options)
}

/// Runs the existing truncation once, then describes only its actual outputs.
pub fn reattach_fragments_with_receipt(
    history: &[Message], bodies: &[PromptFragment], model: &str, context_limit: usize,
    budget: SkillReattachmentBudget, options: RenderOptions,
) -> (Vec<Message>, Vec<PromptFragment>, SkillReattachmentReceipt) {
    let (request, fragments) = reattach_fragments(history, bodies, model, context_limit, budget, options);
    let rendered = render_with_options(&fragments, options);
    let entries = bodies.iter().map(|original| {
        let retained = fragments.iter().find(|fragment| fragment.id == original.id);
        let original_rendered = render_with_options(std::slice::from_ref(original), options);
        let retained_rendered = retained.map(|fragment| render_with_options(std::slice::from_ref(fragment), options)).unwrap_or_default();
        let retained_content = retained.map_or("", |fragment| fragment.content.as_str());
        SkillFragmentReceipt {
            fragment_id: original.id.clone(),
            skill_id: original.id.strip_prefix("skill.").map(str::to_owned),
            original_body_tokens: TokenService::count(model, &original.content),
            retained_body_tokens: TokenService::count(model, retained_content),
            original_body_sha256: digest(&original.content), retained_body_sha256: digest(retained_content),
            original_rendered_tokens: TokenService::count(model, &original_rendered),
            retained_rendered_tokens: TokenService::count(model, &retained_rendered),
            original_rendered_sha256: digest(&original_rendered), retained_rendered_sha256: digest(&retained_rendered),
            retained: retained.is_some(), truncated: retained_content != original.content,
        }
    }).collect();
    let receipt = SkillReattachmentReceipt {
        schema_version: 1, scope: "initial_assembly", model: model.to_owned(),
        count_quality: "approximate", counting_revision: "token-service-model-estimate-v1",
        per_skill_tokens: budget.per_skill_tokens, total_tokens: budget.total_tokens, context_limit,
        reserve_tokens: 1_000, request_input_limit: context_limit.saturating_sub(1_000),
        history_tokens: TokenService::count_messages(model, history),
        request_tokens: TokenService::count_messages(model, &request),
        rendered_tokens: TokenService::count(model, &rendered), rendered_sha256: digest(&rendered), fragments: entries,
    };
    (request, fragments, receipt)
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
