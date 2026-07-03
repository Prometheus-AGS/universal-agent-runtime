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

        // Filter to only configured (available) providers — check each async
        let mut configured_candidates = Vec::new();
        for (provider, model) in candidates {
            if self.registry.is_configured(&provider.id).await {
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
        //
        // A model with NO cost data must NOT sort as free (0.0) — that made
        // uncatalogued/unknown-priced models win the "cheapest" selection,
        // silently preferring models we can't cost-account. Treat missing cost
        // as most-expensive (`f64::INFINITY`) so priced models are preferred
        // and unknown-cost models are only chosen when nothing else qualifies.
        candidates.sort_by(|(pa, a), (pb, b)| {
            let cost_a = a.cost.as_ref().map_or(f64::INFINITY, |c| c.input);
            let cost_b = b.cost.as_ref().map_or(f64::INFINITY, |c| c.input);
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.limits.context_window.cmp(&a.limits.context_window))
                .then_with(|| {
                    // Missing benchmark data (the common case — coverage is a
                    // small verified subset, see src/llm/data/UPDATE.md) must
                    // NOT sort as a worst-possible 0.0 score for the same
                    // reason missing cost must not sort as free: it would
                    // silently penalize uncatalogued-but-otherwise-tied
                    // models against ones that happen to have data. Only
                    // break the tie when BOTH sides have a score.
                    let bench_a = benchmarks::best_score(
                        &format!("{}/{}", pa.id, a.id),
                        BenchmarkDimension::Coding,
                    );
                    let bench_b = benchmarks::best_score(
                        &format!("{}/{}", pb.id, b.id),
                        BenchmarkDimension::Coding,
                    );
                    match (bench_a, bench_b) {
                        (Some(x), Some(y)) => y
                            .partial_cmp(&x)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        _ => std::cmp::Ordering::Equal,
                    }
                })
        });

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
