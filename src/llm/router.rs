//! Catalog-driven model routing.
//!
//! Selects the best model for a given set of requirements (tool calling,
//! reasoning, vision, context window, cost) using the compile-time catalog
//! and the runtime provider registry.

use std::sync::Arc;

use super::benchmarks::{self, BenchmarkDimension};
use super::catalog::{CapabilityFilter, ModelCatalog, ModelInfo, ProviderInfo};
use super::registry::ProviderRegistry;

/// Routes requests to the most appropriate model based on capability requirements.
#[derive(Debug)]
pub struct ModelRouter {
    catalog: &'static ModelCatalog,
    registry: Arc<ProviderRegistry>,
}

impl ModelRouter {
    /// Create a new router backed by the global catalog and the given registry.
    #[must_use]
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            catalog: ModelCatalog::global(),
            registry,
        }
    }

    /// Select the best model matching the given requirements.
    ///
    /// Returns the fully-qualified model string (`"provider/model"`) or `None`
    /// if no configured provider has a model matching the requirements.
    pub async fn route(&self, requirements: &RouteRequirements) -> Option<String> {
        let filter = CapabilityFilter {
            needs_tools: requirements.needs_tools,
            needs_reasoning: requirements.needs_reasoning,
            needs_vision: requirements.needs_vision,
            needs_structured_output: requirements.needs_structured_output,
            min_context: requirements.min_context,
            max_cost_per_1m_input: requirements.max_cost_per_1m_input,
        };

        let candidates: Vec<(&ProviderInfo, &ModelInfo)> =
            self.catalog.models_with_capabilities(&filter);

        // Filter to only configured, healthy providers — check each async.
        // CH-03: a provider in an active failover cooldown is excluded from
        // selection entirely (not just deprioritized), so a struggling
        // provider doesn't keep winning ties against healthy alternatives.
        let mut configured_candidates = Vec::new();
        for (provider, model) in candidates {
            if self.registry.is_configured(&provider.id).await
                && self.registry.health().is_available(&provider.id).await
            {
                configured_candidates.push((provider, model));
            }
        }
        let mut candidates = configured_candidates;

        // Apply cost filter
        if let Some(max_cost) = requirements.max_cost_per_1m_input {
            candidates
                .retain(|(_, model)| model.cost.as_ref().map_or(true, |c| c.input <= max_cost));
        }

        // Prefer the requested provider if specified
        if let Some(ref preferred) = requirements.preferred_provider {
            let preferred_match = candidates
                .iter()
                .find(|(p, _)| p.id == *preferred)
                .map(|(p, m)| format!("{}/{}", p.id, m.id));

            if preferred_match.is_some() {
                return preferred_match;
            }
        }

        // Sort by cost (cheapest first), then by context window (largest
        // first), then — CH-09 — by sourced coding-benchmark score (highest
        // first) as a final tiebreaker among otherwise-equal candidates.
        // Extracted to `compare_candidates` so the tiebreak chain is unit-
        // testable against synthetic candidates, independent of the global
        // catalog/benchmark singletons' actual contents.
        candidates.sort_by(|(pa, a), (pb, b)| compare_candidates(pa, a, pb, b));

        let selection = candidates
            .first()
            .map(|(provider, model)| format!("{}/{}", provider.id, model.id));

        // Routing-decision audit trail (Rule 34): what was chosen, from how many
        // configured candidates, against which requirements.
        tracing::info!(
            name: "llm.router.decision",
            selected = selection.as_deref().unwrap_or("<none>"),
            candidate_count = candidates.len(),
            needs_tools = requirements.needs_tools,
            needs_reasoning = requirements.needs_reasoning,
            needs_vision = requirements.needs_vision,
            needs_structured_output = requirements.needs_structured_output,
            min_context = requirements.min_context,
            max_cost_per_1m_input = requirements.max_cost_per_1m_input,
            "model router selected a model"
        );

        selection
    }
}

/// Ordering for candidate `(provider, model)` pairs: cheapest cost first,
/// then largest context window, then — CH-09 — highest sourced coding-
/// benchmark score, as a final tiebreaker among otherwise-equal candidates.
///
/// A model with NO cost data must NOT sort as free (0.0) — that made
/// uncatalogued/unknown-priced models win the "cheapest" selection, silently
/// preferring models we can't cost-account. Treat missing cost as
/// most-expensive (`f64::INFINITY`) so priced models are preferred and
/// unknown-cost models are only chosen when nothing else qualifies. Missing
/// benchmark data (the common case — coverage is a small verified subset,
/// see `src/llm/data/UPDATE.md`) must likewise NOT sort as a worst-possible
/// 0.0 score, for the same reason — only break the tie when BOTH sides have
/// a score.
fn compare_candidates(
    pa: &ProviderInfo,
    a: &ModelInfo,
    pb: &ProviderInfo,
    b: &ModelInfo,
) -> std::cmp::Ordering {
    let cost_a = a.cost.as_ref().map_or(f64::INFINITY, |c| c.input);
    let cost_b = b.cost.as_ref().map_or(f64::INFINITY, |c| c.input);
    cost_a
        .partial_cmp(&cost_b)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| b.limits.context_window.cmp(&a.limits.context_window))
        .then_with(|| {
            let bench_a =
                benchmarks::best_score(&format!("{}/{}", pa.id, a.id), BenchmarkDimension::Coding);
            let bench_b =
                benchmarks::best_score(&format!("{}/{}", pb.id, b.id), BenchmarkDimension::Coding);
            match (bench_a, bench_b) {
                (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            }
        })
}

#[cfg(test)]
mod compare_candidates_tests {
    use super::*;
    use crate::llm::catalog::{ModelCost, ModelLimits};

    fn provider(id: &str) -> ProviderInfo {
        ProviderInfo {
            id: id.to_string(),
            ..Default::default()
        }
    }

    fn model(id: &str, cost_input: Option<f64>, context_window: u64) -> ModelInfo {
        ModelInfo {
            id: id.to_string(),
            cost: cost_input.map(|input| ModelCost {
                input,
                ..Default::default()
            }),
            limits: ModelLimits {
                context_window,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn cheaper_model_sorts_first() {
        let pa = provider("openai");
        let a = model("cheap", Some(1.0), 100_000);
        let pb = provider("openai");
        let b = model("expensive", Some(5.0), 100_000);

        assert_eq!(
            compare_candidates(&pa, &a, &pb, &b),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn missing_cost_sorts_as_most_expensive_not_free() {
        let pa = provider("openai");
        let priced = model("priced", Some(1.0), 100_000);
        let pb = provider("openai");
        let unpriced = model("unpriced", None, 100_000);

        assert_eq!(
            compare_candidates(&pa, &priced, &pb, &unpriced),
            std::cmp::Ordering::Less,
            "priced model must sort before an unpriced one, not lose to it as if unpriced meant free"
        );
    }

    #[test]
    fn equal_cost_breaks_tie_on_larger_context_window() {
        let pa = provider("openai");
        let a = model("small-context", Some(1.0), 32_000);
        let pb = provider("openai");
        let b = model("large-context", Some(1.0), 200_000);

        assert_eq!(
            compare_candidates(&pa, &a, &pb, &b),
            std::cmp::Ordering::Greater,
            "the larger-context model should sort first (Less), so a-vs-b is Greater"
        );
    }

    #[test]
    fn equal_cost_and_context_breaks_tie_on_benchmark_score() {
        // Real ids from src/llm/data/model_benchmarks.json: claude-opus-4-8
        // (88.6) outscores claude-haiku-4-5 (73.3) on SWE-bench Verified.
        // Cost/context set identically so ONLY the benchmark tiebreak can
        // decide the ordering — proves CH-09's data actually reaches the
        // router's candidate sort, not just the standalone helper function.
        let anthropic_a = provider("anthropic");
        let higher_scoring = model("claude-opus-4-8", Some(3.0), 200_000);
        let anthropic_b = provider("anthropic");
        let lower_scoring = model("claude-haiku-4-5", Some(3.0), 200_000);

        assert_eq!(
            compare_candidates(&anthropic_a, &higher_scoring, &anthropic_b, &lower_scoring),
            std::cmp::Ordering::Less,
            "claude-opus-4-8 (88.6) should sort before claude-haiku-4-5 (73.3) once cost/context tie"
        );
    }

    #[test]
    fn tie_with_no_benchmark_data_on_either_side_stays_equal() {
        let pa = provider("some-uncatalogued-provider");
        let a = model("no-data-a", Some(1.0), 100_000);
        let pb = provider("some-uncatalogued-provider");
        let b = model("no-data-b", Some(1.0), 100_000);

        assert_eq!(
            compare_candidates(&pa, &a, &pb, &b),
            std::cmp::Ordering::Equal,
            "neither candidate has benchmark data -- must not be silently penalized to 0.0"
        );
    }
}

/// Requirements for model selection.
#[derive(Debug, Clone, Default)]
pub struct RouteRequirements {
    /// Model must support tool calling.
    pub needs_tools: bool,
    /// Model must support reasoning/chain-of-thought.
    pub needs_reasoning: bool,
    /// Model must support vision/image input.
    pub needs_vision: bool,
    /// Model must support structured output (JSON mode).
    pub needs_structured_output: bool,
    /// Minimum context window size in tokens.
    pub min_context: Option<u64>,
    /// Maximum input cost per 1M tokens (USD).
    pub max_cost_per_1m_input: Option<f64>,
    /// Preferred provider ID (used as tie-breaker).
    pub preferred_provider: Option<String>,
}

/// Approximate cost ceiling (USD per 1M input tokens) for each model-class
/// string the skill pack's `model_routing.phases` frontmatter uses.
/// **Provisional** — there is no established `ModelClass` concept elsewhere
/// in this codebase yet (CH-16 loader-upgrade scope is "don't drop this
/// data", not "build a full model-tier resolution engine"); this exists so
/// `model_routing` closes the loop into `RouteRequirements` at all rather
/// than being silently dropped, per docs/uar-next-fable.md §6.3 item 1.
/// Tighten/replace when a real tiering system lands.
fn cost_ceiling_for_model_class(class: &str) -> Option<f64> {
    match class.to_lowercase().as_str() {
        "small" => Some(1.0),
        "medium" => Some(5.0),
        "large" => Some(15.0),
        // "frontier" (and anything unrecognized) — no cost ceiling; let
        // cost/context/benchmark tiebreaks decide instead of guessing a cap.
        _ => None,
    }
}

/// Turn a skill's declared model-class hint for one phase (`model_routing.phases`
/// in its SKILL.md frontmatter — see [`crate::uar::domain::skills::SkillModelRouting`])
/// into a [`RouteRequirements`]. Returns `None` when the skill has no routing
/// hint for `phase`.
#[must_use]
pub fn route_requirements_for_model_class(
    routing: &crate::uar::domain::skills::SkillModelRouting,
    phase: &str,
) -> Option<RouteRequirements> {
    let class = routing.phases.get(phase)?;
    Some(RouteRequirements {
        max_cost_per_1m_input: cost_ceiling_for_model_class(class),
        ..RouteRequirements::default()
    })
}

#[cfg(test)]
mod model_class_tests {
    use super::*;
    use crate::uar::domain::skills::SkillModelRouting;

    #[test]
    fn known_class_maps_to_a_cost_ceiling() {
        let routing = SkillModelRouting {
            phases: std::collections::HashMap::from([(
                "my-phase".to_string(),
                "small".to_string(),
            )]),
            ..Default::default()
        };
        let req = route_requirements_for_model_class(&routing, "my-phase").unwrap();
        assert_eq!(req.max_cost_per_1m_input, Some(1.0));
    }

    #[test]
    fn frontier_class_has_no_cost_ceiling() {
        let routing = SkillModelRouting {
            phases: std::collections::HashMap::from([(
                "my-phase".to_string(),
                "frontier".to_string(),
            )]),
            ..Default::default()
        };
        let req = route_requirements_for_model_class(&routing, "my-phase").unwrap();
        assert_eq!(req.max_cost_per_1m_input, None);
    }

    #[test]
    fn unknown_phase_is_none() {
        let routing = SkillModelRouting::default();
        assert!(route_requirements_for_model_class(&routing, "no-such-phase").is_none());
    }
}
