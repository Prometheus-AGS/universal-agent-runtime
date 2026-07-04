//! Targeted eval providers (CH-17): skill-activation accuracy, routing
//! accuracy, and context-strategy efficiency.
//!
//! Unlike the general `starter` suite (which exercises real LLM completions
//! through `OrchestratorCompletionProvider`), these three suites test
//! deterministic *runtime decision* code paths directly — no model call, no
//! API key required. Each provider seeds a small, fixed fixture once, then
//! `complete()` interprets one JSON-encoded case input per call and returns
//! the actual decision the real runtime code made, as a string the existing
//! `Contains`/`ExactMatch` scorers can grade unchanged.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::llm::health::ProviderHealthMonitor;
use crate::llm::registry::ProviderRegistry;
use crate::llm::router::{ModelRouter, RouteRequirements};
use crate::uar::context::strategy::{ContextStrategy, strategy_for_model};
use crate::uar::domain::skills::{Skill, SkillTriggers};
use crate::uar::eval::CompletionProvider;
use crate::uar::runtime::skills::service::SkillService;

// ─────────────────────────────────────────────────────────────────────────
// Skill activation
// ─────────────────────────────────────────────────────────────────────────

/// Exercises `SkillService::match_skills` (the real keyword matcher CH-08
/// instruments) against a small fixed set of fixture skills. Case input is
/// the raw query text; the returned completion is the comma-joined matched
/// `skill_id`s (or `"none"` when nothing matched).
#[derive(Debug)]
pub struct SkillActivationProvider {
    service: SkillService,
}

impl SkillActivationProvider {
    /// Build the provider and register the fixture skill set. Never fails —
    /// `create_skill` on a fresh in-memory `SkillService` (no persistence,
    /// no governance) cannot error on well-formed input.
    pub async fn new() -> Self {
        let service = SkillService::new(None, None);
        for (id, title, keywords) in Self::fixture_skills() {
            let skill = Skill {
                skill_id: id.into(),
                version: "1.0.0".into(),
                title: title.into(),
                description: title.into(),
                triggers: SkillTriggers {
                    keywords: keywords.into_iter().map(str::to_string).collect(),
                    semantic: None,
                },
                prompt_overlay: format!("# {title}"),
                // `#[serde(default = "default_enabled")]` on `Skill::enabled`
                // only affects Deserialize, not `#[derive(Default)]` (bool's
                // Default is `false`) — must set this explicitly or every
                // fixture skill is silently filtered out by `list_enabled()`.
                enabled: true,
                ..Default::default()
            };
            let _ = service.create_skill(skill).await;
        }
        Self { service }
    }

    fn fixture_skills() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
        vec![
            (
                "web-search",
                "Web Search",
                vec!["search", "web", "internet", "google", "look up"],
            ),
            (
                "code-review",
                "Code Review",
                vec!["review", "pull request", "diff", "code review"],
            ),
            (
                "data-analysis",
                "Data Analysis",
                vec!["csv", "dataframe", "statistics", "analyze data"],
            ),
            (
                "image-generation",
                "Image Generation",
                vec!["image", "picture", "draw", "generate art"],
            ),
            (
                "translation",
                "Translation",
                vec!["translate", "translation", "language"],
            ),
        ]
    }
}

#[async_trait]
impl CompletionProvider for SkillActivationProvider {
    async fn complete(&self, input: &str) -> anyhow::Result<String> {
        let matched = self.service.match_skills(input, None).await;
        if matched.is_empty() {
            Ok("none".to_string())
        } else {
            Ok(matched
                .iter()
                .map(|s| s.skill_id.as_str())
                .collect::<Vec<_>>()
                .join(","))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Routing accuracy
// ─────────────────────────────────────────────────────────────────────────

/// Exercises `ModelRouter::route` (CH-03/CH-09's real capability-filter +
/// health-aware selection) against a small fixed fixture registry seeded
/// with real catalog providers. Case input is a JSON object with a
/// `requirements` field (deserialized as `RouteRequirements`) and an
/// optional `trip_health_for` provider id (simulates a CH-03 cooldown
/// before routing). The returned completion is the selected
/// `"provider/model"` string, or `"none"`.
#[derive(Debug)]
pub struct RoutingProvider {
    router: ModelRouter,
    health: Arc<ProviderHealthMonitor>,
}

#[derive(Debug, Deserialize)]
struct RoutingCaseInput {
    #[serde(default)]
    requirements: RouteRequirements,
    /// Provider id to push into cooldown before routing (CH-03 exclusion path).
    #[serde(default)]
    trip_health_for: Option<String>,
}

impl RoutingProvider {
    /// Build the provider and seed two real, stable catalog entries: a
    /// vision+tools-capable model (`openai/gpt-4o`) and a tools-capable
    /// non-vision model (`anthropic/claude-3-5-haiku`). Both are widely
    /// used elsewhere in this codebase's own tests as stable catalog
    /// references (see `src/llm/registry.rs` tests).
    pub async fn new() -> Self {
        let registry = ProviderRegistry::new();
        for (model, key) in [
            ("openai/gpt-4o", "sk-fixture-openai"),
            ("anthropic/claude-3-5-haiku", "sk-fixture-anthropic"),
        ] {
            let cfg = crate::config::LlmConfig {
                model: model.to_string(),
                api_key: Some(key.to_string()),
                ..crate::config::LlmConfig::default()
            };
            registry.seed_from_llm_config(&cfg).await;
        }
        let health = Arc::clone(registry.health());
        let router = ModelRouter::new(Arc::new(registry));
        Self { router, health }
    }
}

#[async_trait]
impl CompletionProvider for RoutingProvider {
    async fn complete(&self, input: &str) -> anyhow::Result<String> {
        let parsed: RoutingCaseInput = serde_json::from_str(input)
            .map_err(|e| anyhow::anyhow!("routing case input must be JSON: {e}"))?;

        if let Some(provider_id) = &parsed.trip_health_for {
            // error_threshold=1 trips the cooldown on this single call —
            // deterministic, no retry/timing dependence.
            self.health.record_failure(provider_id, 1, 999).await;
        }

        Ok(self
            .router
            .route(&parsed.requirements)
            .await
            .unwrap_or_else(|| "none".to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Context-strategy efficiency
// ─────────────────────────────────────────────────────────────────────────

/// Exercises `strategy_for_model` (CH-05's model-aware `Auto` resolution) —
/// a pure function, no fixture needed. Case input is a JSON object
/// `{"effective_context_tokens": <u32>}`; the returned completion describes
/// the resolved strategy and its key parameter.
#[derive(Debug)]
pub struct ContextEfficiencyProvider;

#[derive(Debug, Deserialize)]
struct ContextCaseInput {
    effective_context_tokens: u32,
}

#[async_trait]
impl CompletionProvider for ContextEfficiencyProvider {
    async fn complete(&self, input: &str) -> anyhow::Result<String> {
        let parsed: ContextCaseInput = serde_json::from_str(input)
            .map_err(|e| anyhow::anyhow!("context-efficiency case input must be JSON: {e}"))?;
        let strategy = strategy_for_model(parsed.effective_context_tokens);
        Ok(match strategy {
            ContextStrategy::SlidingWindow { max_messages } => {
                format!("sliding_window max_messages={max_messages}")
            }
            ContextStrategy::Summarize {
                threshold,
                summary_max_tokens,
                ..
            } => format!("summarize threshold={threshold} summary_max_tokens={summary_max_tokens}"),
            other => format!("{other:?}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn skill_activation_matches_expected_skill() {
        let provider = SkillActivationProvider::new().await;
        let out = provider
            .complete("Can you search the web for the latest news?")
            .await
            .unwrap();
        assert!(out.contains("web-search"), "got: {out}");
    }

    #[tokio::test]
    async fn skill_activation_no_match_returns_none() {
        let provider = SkillActivationProvider::new().await;
        let out = provider
            .complete("asdkjqwoiej unrelated gibberish")
            .await
            .unwrap();
        assert_eq!(out, "none");
    }

    #[tokio::test]
    async fn routing_preferred_provider_wins_regardless_of_cost() {
        let provider = RoutingProvider::new().await;
        let out = provider
            .complete(
                r#"{"requirements": {"needs_tools": true, "preferred_provider": "anthropic"}}"#,
            )
            .await
            .unwrap();
        assert!(out.starts_with("anthropic/"), "got: {out}");
    }

    #[tokio::test]
    async fn routing_excludes_provider_in_cooldown() {
        let provider = RoutingProvider::new().await;
        let out = provider
            .complete(
                r#"{"requirements": {"needs_tools": true, "preferred_provider": "anthropic"}, "trip_health_for": "anthropic"}"#,
            )
            .await
            .unwrap();
        // anthropic is tripped into cooldown, so the preferred-provider match
        // it would otherwise win never appears as a configured candidate --
        // openai remains the only healthy option.
        assert!(out.starts_with("openai/"), "got: {out}");
    }

    #[tokio::test]
    async fn routing_impossible_cost_ceiling_yields_none() {
        let provider = RoutingProvider::new().await;
        let out = provider
            .complete(
                r#"{"requirements": {"needs_tools": true, "max_cost_per_1m_input": 0.0000001}}"#,
            )
            .await
            .unwrap();
        assert_eq!(out, "none");
    }

    #[tokio::test]
    async fn context_efficiency_small_window_uses_sliding_window() {
        let provider = ContextEfficiencyProvider;
        let out = provider
            .complete(r#"{"effective_context_tokens": 32000}"#)
            .await
            .unwrap();
        assert!(out.starts_with("sliding_window"), "got: {out}");
    }

    #[tokio::test]
    async fn context_efficiency_large_window_uses_summarize() {
        let provider = ContextEfficiencyProvider;
        let out = provider
            .complete(r#"{"effective_context_tokens": 250000}"#)
            .await
            .unwrap();
        assert!(out.starts_with("summarize"), "got: {out}");
    }
}
