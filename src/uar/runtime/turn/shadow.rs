//! Redacted shadow comparison. Comparison never chooses the dispatched request.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::llm::{LlmRequest, Message};
use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::runtime::context::token_service::TokenService;
use crate::uar::runtime::prompt::{ManifestFragment, PromptFragment};

/// Only fingerprints and counts can enter a parity artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowView {
    pub fields: BTreeMap<String, String>,
}

impl ShadowView {
    pub fn capture(
        fragments: &[PromptFragment],
        policy: &EffectiveRunPolicy,
        request: &LlmRequest,
        active_skills: &[String],
        mcp_catalog: &[String],
        model: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut fragments = fragments
            .iter()
            .map(ManifestFragment::from)
            .collect::<Vec<_>>();
        fragments.sort_by(|left, right| {
            left.section
                .cmp(&right.section)
                .then_with(|| left.id.cmp(&right.id))
        });
        let mut skills = active_skills.to_vec();
        skills.sort();
        skills.dedup();
        let mut servers = mcp_catalog.to_vec();
        servers.sort();
        let messages: Vec<Message> =
            serde_json::from_value(Value::Array(request.messages.clone()))?;
        let mut fields = BTreeMap::new();
        fields.insert(
            "fragment_order".into(),
            fingerprint(&json!(
                fragments
                    .iter()
                    .map(|fragment| &fragment.id)
                    .collect::<Vec<_>>()
            )),
        );
        fields.insert("fragments".into(), fingerprint(&json!(fragments)));
        fields.insert("policy".into(), fingerprint(&json!(policy)));
        fields.insert("tools".into(), fingerprint(&json!(request.tools)));
        fields.insert("active_skills".into(), fingerprint(&json!(skills)));
        fields.insert("mcp_catalog".into(), fingerprint(&json!(servers)));
        fields.insert("messages".into(), fingerprint(&json!(request.messages)));
        fields.insert(
            "context_tokens".into(),
            TokenService::count_messages(model, &messages).to_string(),
        );
        fields.insert("model".into(), fingerprint(&json!(model)));
        fields.insert(
            "model_params".into(),
            fingerprint(&json!(request.extra_params)),
        );
        Ok(Self { fields })
    }
}

/// Exemptions match exact fingerprints, not a broad class of differences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedDelta {
    pub id: String,
    pub field: String,
    pub legacy: Option<String>,
    pub typed: Option<String>,
    pub introduced_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowAllowlist {
    pub entries: Vec<AllowedDelta>,
}

impl ShadowAllowlist {
    pub fn checked_in() -> anyhow::Result<Self> {
        let allowlist: Self = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/turn_parity/intentional_deltas.json"
        )))?;
        for entry in &allowlist.entries {
            if entry.id.trim().is_empty() || entry.introduced_by.trim().is_empty() {
                anyhow::bail!("shadow allowlist entries require an id and introducing change");
            }
        }
        Ok(allowlist)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowDifference {
    pub field: String,
    pub legacy: Option<String>,
    pub typed: Option<String>,
    pub allowlist_entry: Option<String>,
    pub introduced_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowReport {
    pub differences: Vec<ShadowDifference>,
    pub unexpected_difference_count: usize,
    pub allowlisted_difference_count: usize,
    pub dispatched_path: String,
}

pub fn compare(
    legacy: &ShadowView,
    typed: &ShadowView,
    allowlist: &ShadowAllowlist,
) -> ShadowReport {
    let keys = legacy
        .fields
        .keys()
        .chain(typed.fields.keys())
        .collect::<BTreeSet<_>>();
    let mut differences = Vec::new();
    for key in keys {
        let left = legacy.fields.get(key);
        let right = typed.fields.get(key);
        if left == right {
            continue;
        }
        let exemption = allowlist.entries.iter().find(|entry| {
            entry.field == *key
                && entry.legacy.as_ref() == left
                && entry.typed.as_ref() == right
                && !entry.introduced_by.trim().is_empty()
        });
        differences.push(ShadowDifference {
            field: key.clone(),
            legacy: left.cloned(),
            typed: right.cloned(),
            allowlist_entry: exemption.map(|entry| entry.id.clone()),
            introduced_by: exemption.map(|entry| entry.introduced_by.clone()),
        });
    }
    let allowlisted_difference_count = differences
        .iter()
        .filter(|difference| difference.allowlist_entry.is_some())
        .count();
    ShadowReport {
        unexpected_difference_count: differences.len() - allowlisted_difference_count,
        allowlisted_difference_count,
        differences,
        dispatched_path: "legacy".into(),
    }
}

/// Independently render the typed request but return comparison metadata only.
/// The caller retains the legacy snapshot as the sole dispatch input.
pub fn compare_step(
    legacy: &super::resolved::ResolvedStep,
    typed_turn: &super::resolved::ResolvedTurn,
    history: &[Message],
    active_bodies: &[PromptFragment],
    budget: crate::config::SkillReattachmentBudget,
    options: crate::uar::runtime::prompt::RenderOptions,
) -> anyhow::Result<ShadowReport> {
    use crate::llm::{MessageContent, MessageRole};
    use crate::uar::runtime::prompt::{PromptSection, Retention, render_with_options};
    let mut fragments = typed_turn
        .fragments()
        .iter()
        .filter(|fragment| {
            fragment.retention != Retention::Reclaimable
                && (legacy.index() == 1 || fragment.section != PromptSection::WorldState)
        })
        .cloned()
        .collect::<Vec<_>>();
    if legacy.index() > 1 {
        // The current host snapshot is shared input, just like prepared history;
        // stale turn-start deltas must not appear in later step manifests.
        fragments.extend(
            legacy
                .fragments()
                .iter()
                .filter(|fragment| fragment.section == PromptSection::WorldState)
                .cloned(),
        );
    }
    let system = Message {
        role: MessageRole::System,
        // World-state updates are history messages after reduction, not part of
        // the pinned base prompt. Re-rendering them here would duplicate them.
        content: MessageContent::text(render_with_options(
            &fragments
                .iter()
                .filter(|fragment| fragment.section != PromptSection::WorldState)
                .cloned()
                .collect::<Vec<_>>(),
            options,
        )),
        tool_call_id: None,
        tool_calls: None,
    };
    let mut history = history.to_vec();
    if history
        .first()
        .is_some_and(|message| message.role == MessageRole::System)
    {
        history[0] = system;
    } else {
        history.insert(0, system);
    }
    let typed_skills = legacy
        .active_skills()
        .iter()
        .filter(|id| typed_turn.policy().skills.ids.contains(id))
        .cloned()
        .collect::<Vec<_>>();
    let typed_bodies = active_bodies
        .iter()
        .filter(|body| {
            typed_skills
                .iter()
                .any(|id| body.id == format!("skill.{id}"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let (messages, attached) = crate::uar::runtime::skills::retention::reattach_fragments(
        &history,
        &typed_bodies,
        &typed_turn.credentials().model,
        legacy.budgets().context_window_tokens.unwrap_or(8_192),
        budget,
        options,
    );
    fragments.extend(attached);
    let mut typed_request = legacy.request().clone();
    typed_request.messages = messages.iter().map(|message| json!(message)).collect();
    let policy = typed_turn.policy();
    typed_request.tools = legacy
        .tools()
        .values()
        .filter(|tool| {
            use crate::uar::domain::policy::SelectionMode;
            use crate::uar::tools::descriptor::{Exposure, ToolSource};
            let eligible_tool = (tool.source == ToolSource::BuiltIn
                && tool.exposure == Exposure::ModelOnly)
                || !matches!(
                    policy.tools.mode,
                    SelectionMode::None | SelectionMode::Selected
                )
                || policy.tools.ids.contains(&tool.provider_name)
                || policy.tools.ids.contains(&tool.id);
            let eligible_server = tool.server.as_ref().is_none_or(|server| {
                !matches!(
                    policy.mcp_servers.mode,
                    SelectionMode::None | SelectionMode::Selected
                ) || policy.mcp_servers.ids.contains(server)
            });
            eligible_tool && eligible_server
        })
        .map(|tool| tool.openai_tool_json())
        .collect();
    let legacy_view = ShadowView::capture(
        legacy.fragments(),
        legacy.turn().policy(),
        legacy.request(),
        legacy.active_skills(),
        legacy.mcp_catalog(),
        &legacy.turn().credentials().model,
    )?;
    let typed_view = ShadowView::capture(
        &fragments,
        typed_turn.policy(),
        &typed_request,
        &typed_skills,
        legacy.mcp_catalog(),
        &typed_turn.credentials().model,
    )?;
    Ok(compare(
        &legacy_view,
        &typed_view,
        &ShadowAllowlist::checked_in()?,
    ))
}

/// Object-key order never creates a spurious difference; array order does.
pub fn fingerprint(value: &Value) -> String {
    fn hash(value: &Value, digest: &mut Sha256) {
        match value {
            Value::Object(object) => {
                digest.update(b"object\0");
                let ordered = object.iter().collect::<BTreeMap<_, _>>();
                for (key, value) in ordered {
                    digest.update(key.len().to_le_bytes());
                    digest.update(key.as_bytes());
                    hash(value, digest);
                }
                digest.update(b"end\0");
            }
            Value::Array(array) => {
                digest.update(b"array\0");
                for value in array {
                    hash(value, digest);
                }
                digest.update(b"end\0");
            }
            value => {
                let scalar = value.to_string();
                digest.update(scalar.len().to_le_bytes());
                digest.update(scalar.as_bytes());
            }
        }
    }
    let mut digest = Sha256::new();
    hash(value, &mut digest);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
