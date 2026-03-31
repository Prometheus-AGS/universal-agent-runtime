//! Catalog-driven model routing.
//!
//! Selects the best model for a given set of requirements (tool calling,
//! reasoning, vision, context window, cost) using the compile-time catalog
//! and the runtime provider registry.

use std::sync::Arc;

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
            candidates.retain(|(_, model)| {
                model
                    .cost
                    .as_ref()
                    .map_or(true, |c| c.input <= max_cost)
            });
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

        // Sort by cost (cheapest first), then by context window (largest first)
        candidates.sort_by(|(_, a), (_, b)| {
            let cost_a = a.cost.as_ref().map_or(0.0, |c| c.input);
            let cost_b = b.cost.as_ref().map_or(0.0, |c| c.input);
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.limits
                        .context_window
                        .cmp(&a.limits.context_window)
                })
        });

        candidates
            .first()
            .map(|(provider, model)| format!("{}/{}", provider.id, model.id))
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
