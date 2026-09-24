//! Per-model prompt dialect engine (uar-next-fable §2.1).
//!
//! Different model families have documented, web-verified preferences for how
//! prompts and reasoning are expressed at the API layer. Treating every model
//! as an interchangeable black box leaves capability on the table. This module
//! detects a model's dialect from its id and produces the extra request
//! parameters that dialect wants — reasoning-persistence toggles, structured
//! output hints, thinking-effort levels — as a JSON object the driver merges
//! into the outbound request body.
//!
//! Only web-verified parameters are encoded here (see the fable doc's §2.3:
//! encode NO numbers from the model-comparison document). Values that vary by
//! provider deployment (e.g. Qwen's DashScope `extra_body` vs vLLM
//! `chat_template_kwargs` split) are surfaced as flags for the driver, not
//! hardcoded, since the correct wrapper depends on the endpoint.

use std::fmt;

use serde_json::{Value, json};

use crate::uar::runtime::prompt::{PromptTemplateProfile, PromptTemplateSelector};

/// The prompt dialect a model family prefers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDialect {
    /// Anthropic Claude: XML structure, extended-thinking budgets.
    AnthropicXml,
    /// OpenAI GPT: Responses-API `text.format` structured output, avoid CoT prose.
    OpenAiJson,
    /// Moonshot Kimi: preserved thinking (`thinking.keep`), Markdown headings.
    KimiMarkdown,
    /// Zhipu/Z.ai GLM: `thinking_mode` high/max.
    GlmThinking,
    /// Alibaba Qwen: `enable_thinking` / `preserve_thinking` (endpoint-dependent wrapper).
    QwenHybrid,
    /// MiniMax M-series: Markdown-averse — prefer XML/JSON structure.
    MiniMaxStructured,
    /// Fallback for unknown families.
    Generic,
}

impl PromptDialect {
    /// Detect a legacy rendering hint from a `provider/model` or bare model id.
    ///
    /// This function does not establish template, setting, counting, or wire
    /// compatibility. Dispatch preparation uses [`PromptTemplateResolver`],
    /// whose family matches require an explicit reviewed family revision.
    #[must_use]
    pub fn detect(model_id: &str) -> Self {
        let m = model_id.to_ascii_lowercase();
        if m.contains("claude") || m.contains("anthropic") {
            Self::AnthropicXml
        } else if m.contains("gpt") || m.contains("openai") || m.contains("o1") || m.contains("o3")
        {
            Self::OpenAiJson
        } else if m.contains("kimi") || m.contains("moonshot") {
            Self::KimiMarkdown
        } else if m.contains("glm") || m.contains("zhipu") || m.contains("z.ai") {
            Self::GlmThinking
        } else if m.contains("qwen") || m.contains("alibaba") || m.contains("dashscope") {
            Self::QwenHybrid
        } else if m.contains("minimax") {
            Self::MiniMaxStructured
        } else {
            Self::Generic
        }
    }

    /// Whether this dialect prefers XML-structured prompt envelopes
    /// (`<context>`, `<instructions>`) over plain text.
    #[must_use]
    pub fn prefers_xml_envelope(self) -> bool {
        matches!(self, Self::AnthropicXml | Self::MiniMaxStructured)
    }

    /// Whether Markdown structure degrades this model's output
    /// (MiniMax's documented Markdown-aversion).
    #[must_use]
    pub fn markdown_averse(self) -> bool {
        matches!(self, Self::MiniMaxStructured)
    }

    /// The string name a compiled agent descriptor's `prompt_dialect.dialect`
    /// override uses to name this variant (CH-12/CH-14) — the inverse of
    /// [`Self::detect`]'s model-id sniffing. Kept in the same module as
    /// `detect` so the two can't drift apart.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::AnthropicXml => "anthropic_xml",
            Self::OpenAiJson => "openai_json",
            Self::KimiMarkdown => "kimi_markdown",
            Self::GlmThinking => "glm_thinking",
            Self::QwenHybrid => "qwen_hybrid",
            Self::MiniMaxStructured => "minimax_structured",
            Self::Generic => "generic",
        }
    }
}

/// Exact trusted-host destination identity used for template resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateDestination {
    pub provider_id: String,
    pub endpoint_kind: String,
    pub model_id: String,
    pub model_revision: String,
    /// Reviewed compatibility identity. Never inferred from `model_id`.
    pub verified_family_revision: Option<String>,
    /// True only after the host has established the generic contract's
    /// required endpoint capabilities and limits.
    pub generic_contract_eligible: bool,
}

/// Origin of a host-constrained template override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateOverrideProvenance {
    Descriptor,
    Operator,
}

/// One requested template override carried from a trusted compiler/operator
/// boundary. The model cannot create this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateOverrideRequest {
    pub template_id: String,
    pub provenance: TemplateOverrideProvenance,
}

/// Host policy allow-list for descriptor and operator template overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateOverridePolicy {
    pub allowed_template_ids: Vec<String>,
}

impl TemplateOverridePolicy {
    #[must_use]
    pub fn allows(&self, template_id: &str) -> bool {
        self.allowed_template_ids
            .iter()
            .any(|allowed| allowed == template_id)
    }
}

/// Why one profile won resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateResolutionSource {
    DescriptorOverride,
    OperatorOverride,
    ExactProfile,
    VerifiedFamilyProfile,
    GenericProfile,
}

/// Profile and provenance selected for one destination.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedPromptTemplate<'a> {
    pub profile: &'a PromptTemplateProfile,
    pub source: TemplateResolutionSource,
}

/// Explicit profile resolution failure. No variant authorizes dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateResolutionError {
    InvalidDestinationIdentity {
        field: &'static str,
    },
    InvalidProfileIdentity {
        profile_id: String,
        field: &'static str,
    },
    OverrideForbidden {
        template_id: String,
    },
    UnknownOverride {
        template_id: String,
    },
    IncompatibleOverride {
        template_id: String,
    },
    AmbiguousMatch {
        scope: &'static str,
        profile_ids: Vec<String>,
    },
    UnsupportedProfile {
        provider_id: String,
        endpoint_kind: String,
        model_id: String,
        model_revision: String,
    },
}

impl fmt::Display for TemplateResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDestinationIdentity { field } => {
                write!(formatter, "template destination has an empty `{field}`")
            }
            Self::InvalidProfileIdentity { profile_id, field } => write!(
                formatter,
                "prompt template `{profile_id}` has an empty `{field}`"
            ),
            Self::OverrideForbidden { template_id } => {
                write!(
                    formatter,
                    "template override `{template_id}` is forbidden by host policy"
                )
            }
            Self::UnknownOverride { template_id } => {
                write!(
                    formatter,
                    "template override `{template_id}` is not registered"
                )
            }
            Self::IncompatibleOverride { template_id } => write!(
                formatter,
                "template override `{template_id}` is incompatible with the exact destination"
            ),
            Self::AmbiguousMatch { scope, profile_ids } => write!(
                formatter,
                "multiple {scope} prompt templates match: {}",
                profile_ids.join(", ")
            ),
            Self::UnsupportedProfile {
                provider_id,
                endpoint_kind,
                model_id,
                model_revision,
            } => write!(
                formatter,
                "no supported prompt template for provider `{provider_id}`, endpoint `{endpoint_kind}`, model `{model_id}` revision `{model_revision}`"
            ),
        }
    }
}

impl std::error::Error for TemplateResolutionError {}

/// Deterministic exact/family/generic template registry.
#[derive(Debug, Clone, Default)]
pub struct PromptTemplateResolver {
    profiles: Vec<PromptTemplateProfile>,
}

impl PromptTemplateResolver {
    #[must_use]
    pub fn new(profiles: Vec<PromptTemplateProfile>) -> Self {
        Self { profiles }
    }

    pub fn resolve<'a>(
        &'a self,
        destination: &TemplateDestination,
        requested_override: Option<&TemplateOverrideRequest>,
        override_policy: &TemplateOverridePolicy,
    ) -> Result<ResolvedPromptTemplate<'a>, TemplateResolutionError> {
        validate_destination(destination)?;
        for profile in &self.profiles {
            validate_profile(profile)?;
        }

        if let Some(requested) = requested_override {
            if !override_policy.allows(&requested.template_id) {
                return Err(TemplateResolutionError::OverrideForbidden {
                    template_id: requested.template_id.clone(),
                });
            }
            let matches = self
                .profiles
                .iter()
                .filter(|profile| profile.id == requested.template_id)
                .collect::<Vec<_>>();
            let profile = unique_profile("override", matches)?.ok_or_else(|| {
                TemplateResolutionError::UnknownOverride {
                    template_id: requested.template_id.clone(),
                }
            })?;
            if !selector_matches(&profile.selector, destination) {
                return Err(TemplateResolutionError::IncompatibleOverride {
                    template_id: requested.template_id.clone(),
                });
            }
            let source = match requested.provenance {
                TemplateOverrideProvenance::Descriptor => {
                    TemplateResolutionSource::DescriptorOverride
                }
                TemplateOverrideProvenance::Operator => TemplateResolutionSource::OperatorOverride,
            };
            return Ok(ResolvedPromptTemplate { profile, source });
        }

        for (scope, source, predicate) in [
            (
                "exact",
                TemplateResolutionSource::ExactProfile,
                is_exact as fn(&PromptTemplateSelector) -> bool,
            ),
            (
                "verified-family",
                TemplateResolutionSource::VerifiedFamilyProfile,
                is_verified_family as fn(&PromptTemplateSelector) -> bool,
            ),
            (
                "generic",
                TemplateResolutionSource::GenericProfile,
                is_generic as fn(&PromptTemplateSelector) -> bool,
            ),
        ] {
            let matches = self
                .profiles
                .iter()
                .filter(|profile| predicate(&profile.selector))
                .filter(|profile| selector_matches(&profile.selector, destination))
                .collect::<Vec<_>>();
            if let Some(profile) = unique_profile(scope, matches)? {
                return Ok(ResolvedPromptTemplate { profile, source });
            }
        }

        Err(TemplateResolutionError::UnsupportedProfile {
            provider_id: destination.provider_id.clone(),
            endpoint_kind: destination.endpoint_kind.clone(),
            model_id: destination.model_id.clone(),
            model_revision: destination.model_revision.clone(),
        })
    }
}

fn validate_destination(destination: &TemplateDestination) -> Result<(), TemplateResolutionError> {
    for (field, value) in [
        ("provider_id", destination.provider_id.as_str()),
        ("endpoint_kind", destination.endpoint_kind.as_str()),
        ("model_id", destination.model_id.as_str()),
        ("model_revision", destination.model_revision.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(TemplateResolutionError::InvalidDestinationIdentity { field });
        }
    }
    if destination
        .verified_family_revision
        .as_deref()
        .is_some_and(|revision| revision.trim().is_empty())
    {
        return Err(TemplateResolutionError::InvalidDestinationIdentity {
            field: "verified_family_revision",
        });
    }
    Ok(())
}

fn validate_profile(profile: &PromptTemplateProfile) -> Result<(), TemplateResolutionError> {
    for (field, value) in [
        ("id", profile.id.as_str()),
        ("revision", profile.revision.as_str()),
        ("wire_contract_id", profile.wire_contract_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(TemplateResolutionError::InvalidProfileIdentity {
                profile_id: profile.id.clone(),
                field,
            });
        }
    }

    let identities = match &profile.selector {
        PromptTemplateSelector::Exact {
            provider_id,
            endpoint_kind,
            model_id,
            model_revision,
        } => vec![
            ("provider_id", provider_id.as_str()),
            ("endpoint_kind", endpoint_kind.as_str()),
            ("model_id", model_id.as_str()),
            ("model_revision", model_revision.as_str()),
        ],
        PromptTemplateSelector::VerifiedFamily {
            provider_id,
            endpoint_kind,
            family_revision,
        } => vec![
            ("provider_id", provider_id.as_str()),
            ("endpoint_kind", endpoint_kind.as_str()),
            ("family_revision", family_revision.as_str()),
        ],
        PromptTemplateSelector::Generic { endpoint_kind } => {
            vec![("endpoint_kind", endpoint_kind.as_str())]
        }
    };
    for (field, value) in identities {
        if value.trim().is_empty() {
            return Err(TemplateResolutionError::InvalidProfileIdentity {
                profile_id: profile.id.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn unique_profile<'a>(
    scope: &'static str,
    matches: Vec<&'a PromptTemplateProfile>,
) -> Result<Option<&'a PromptTemplateProfile>, TemplateResolutionError> {
    match matches.as_slice() {
        [] => Ok(None),
        [profile] => Ok(Some(*profile)),
        profiles => Err(TemplateResolutionError::AmbiguousMatch {
            scope,
            profile_ids: profiles.iter().map(|profile| profile.id.clone()).collect(),
        }),
    }
}

fn selector_matches(selector: &PromptTemplateSelector, destination: &TemplateDestination) -> bool {
    match selector {
        PromptTemplateSelector::Exact {
            provider_id,
            endpoint_kind,
            model_id,
            model_revision,
        } => {
            provider_id == &destination.provider_id
                && endpoint_kind == &destination.endpoint_kind
                && model_id == &destination.model_id
                && model_revision == &destination.model_revision
        }
        PromptTemplateSelector::VerifiedFamily {
            provider_id,
            endpoint_kind,
            family_revision,
        } => {
            provider_id == &destination.provider_id
                && endpoint_kind == &destination.endpoint_kind
                && destination.verified_family_revision.as_ref() == Some(family_revision)
        }
        PromptTemplateSelector::Generic { endpoint_kind } => {
            destination.generic_contract_eligible && endpoint_kind == &destination.endpoint_kind
        }
    }
}

const fn is_exact(selector: &PromptTemplateSelector) -> bool {
    matches!(selector, PromptTemplateSelector::Exact { .. })
}

const fn is_verified_family(selector: &PromptTemplateSelector) -> bool {
    matches!(selector, PromptTemplateSelector::VerifiedFamily { .. })
}

const fn is_generic(selector: &PromptTemplateSelector) -> bool {
    matches!(selector, PromptTemplateSelector::Generic { .. })
}

/// Options controlling dialect parameter generation for one request.
#[derive(Debug, Clone, Copy, Default)]
pub struct DialectRequest {
    /// The task benefits from reasoning/extended thinking.
    pub wants_reasoning: bool,
    /// This is a multi-turn conversation (reasoning persistence matters).
    pub multi_turn: bool,
    /// Hard problem — request the highest thinking effort where supported.
    pub hard: bool,
    /// Explicit host override. Absence preserves the configured behavior;
    /// `ReasoningEffort::None` deliberately removes reasoning parameters.
    pub effort: Option<crate::config::ReasoningEffort>,
}

/// The prompt dialect engine: detect dialect, emit per-model request params.
#[derive(Debug, Clone, Copy, Default)]
pub struct PromptDialectEngine;

impl PromptDialectEngine {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Detect the dialect for a model id.
    #[must_use]
    pub fn dialect_for(&self, model_id: &str) -> PromptDialect {
        PromptDialect::detect(model_id)
    }

    /// Produce the extra request-body parameters this model's dialect wants,
    /// as a JSON object to merge into the outbound completion request. Returns
    /// an empty object when the dialect needs no extra params for this request.
    ///
    /// All parameter names/shapes are web-verified (fable §2.1):
    /// - Anthropic: `thinking: {type: "enabled", budget_tokens}` for reasoning.
    /// - OpenAI: no reasoning body param here (Responses `text.format` is a
    ///   structured-output concern set by the caller's schema, not reasoning).
    /// - Kimi: `thinking: {type: "enabled", keep: "all"}` to preserve reasoning
    ///   across turns; on k2.7-code this is always-on and harmless to send.
    /// - GLM: `thinking: {type: "enabled"}` + `reasoning_effort: "max"|"high"`.
    /// - Qwen: `enable_thinking: true` (+ `preserve_thinking` for multi-turn);
    ///   the DashScope-vs-vLLM wrapper is the driver's concern.
    #[must_use]
    pub fn request_params(&self, model_id: &str, req: DialectRequest) -> Value {
        let dialect = PromptDialect::detect(model_id);
        let explicit = req.effort;
        let wants_reasoning = explicit.map_or(req.wants_reasoning, |effort| {
            effort != crate::config::ReasoningEffort::None
        });
        let effort = explicit.map(crate::config::ReasoningEffort::as_str);
        let budget = explicit
            .and_then(crate::config::ReasoningEffort::thinking_budget)
            .unwrap_or(if req.hard { 8_192 } else { 2_048 });
        match dialect {
            PromptDialect::AnthropicXml if wants_reasoning => json!({
                "thinking": { "type": "enabled", "budget_tokens": budget }
            }),
            PromptDialect::OpenAiJson if wants_reasoning && effort.is_some() => json!({
                "reasoning_effort": effort
            }),
            PromptDialect::KimiMarkdown
                if wants_reasoning || (explicit.is_none() && req.multi_turn) =>
            {
                json!({
                    "thinking": { "type": "enabled", "keep": "all" }
                })
            }
            PromptDialect::GlmThinking if wants_reasoning => json!({
                "thinking": { "type": "enabled" },
                "reasoning_effort": effort.unwrap_or(if req.hard { "max" } else { "high" })
            }),
            PromptDialect::QwenHybrid if wants_reasoning => {
                if req.multi_turn {
                    json!({ "enable_thinking": true, "preserve_thinking": true })
                } else {
                    json!({ "enable_thinking": true })
                }
            }
            _ => json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_families() {
        assert_eq!(
            PromptDialect::detect("anthropic/claude-sonnet-5"),
            PromptDialect::AnthropicXml
        );
        assert_eq!(
            PromptDialect::detect("openai/gpt-5.5"),
            PromptDialect::OpenAiJson
        );
        assert_eq!(
            PromptDialect::detect("moonshot/kimi-k2.7-code"),
            PromptDialect::KimiMarkdown
        );
        assert_eq!(
            PromptDialect::detect("z.ai/glm-5.2"),
            PromptDialect::GlmThinking
        );
        assert_eq!(
            PromptDialect::detect("alibaba/qwen3.7-max"),
            PromptDialect::QwenHybrid
        );
        assert_eq!(
            PromptDialect::detect("minimax/m3"),
            PromptDialect::MiniMaxStructured
        );
        assert_eq!(
            PromptDialect::detect("meta/llama-4"),
            PromptDialect::Generic
        );
    }

    #[test]
    fn anthropic_reasoning_emits_thinking_budget() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "anthropic/claude-opus-4-8",
            DialectRequest {
                wants_reasoning: true,
                hard: true,
                ..Default::default()
            },
        );
        assert_eq!(p["thinking"]["type"], "enabled");
        assert_eq!(p["thinking"]["budget_tokens"], 8192);
    }

    #[test]
    fn kimi_multi_turn_preserves_thinking() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "moonshot/kimi-k2.6",
            DialectRequest {
                multi_turn: true,
                ..Default::default()
            },
        );
        assert_eq!(p["thinking"]["keep"], "all");
    }

    #[test]
    fn qwen_multi_turn_adds_preserve_thinking() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "alibaba/qwen3.7-max",
            DialectRequest {
                wants_reasoning: true,
                multi_turn: true,
                ..Default::default()
            },
        );
        assert_eq!(p["enable_thinking"], true);
        assert_eq!(p["preserve_thinking"], true);
    }

    #[test]
    fn no_params_when_reasoning_not_wanted() {
        let e = PromptDialectEngine::new();
        let p = e.request_params("openai/gpt-5.5", DialectRequest::default());
        assert_eq!(p, serde_json::json!({}));
    }

    #[test]
    fn minimax_is_markdown_averse() {
        assert!(PromptDialect::MiniMaxStructured.markdown_averse());
        assert!(!PromptDialect::AnthropicXml.markdown_averse());
        assert!(PromptDialect::AnthropicXml.prefers_xml_envelope());
    }
}
