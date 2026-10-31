//! Catalog-driven model routing.
//!
//! Selects the best model for a given set of requirements (tool calling,
//! reasoning, vision, context window, cost) using the compile-time catalog
//! and the runtime provider registry.

use std::{
    collections::BTreeSet,
    fmt,
    sync::Arc,
};

use serde::Deserialize;

use super::benchmarks::{self, BenchmarkDimension};
use super::catalog::{
    CapabilityFilter, Modalities, ModelCapabilities, ModelCatalog, ModelInfo, ModelLimits,
    ProviderInfo,
};
use super::registry::ProviderRegistry;

/// Immutable destination identities captured by the trusted host for one run.
///
/// This is authority plumbing, not a routing preference or a provider policy.
/// A destination never becomes permitted because it appears in catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationScope {
    /// An explicit or team-selected target with no permitted substitute.
    Exact { primary: String },
    /// The captured primary and already-authorized fallback identities.
    PrimaryWithFallbacks {
        primary: String,
        fallbacks: BTreeSet<String>,
    },
}

impl DestinationScope {
    #[must_use]
    pub(crate) fn permits(&self, destination: &str) -> bool {
        match self {
            Self::Exact { primary } => primary == destination,
            Self::PrimaryWithFallbacks { primary, fallbacks } => {
                primary == destination || fallbacks.contains(destination)
            }
        }
    }
}

/// Why a captured destination cannot receive the prepared request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationAdmissionError {
    MalformedDestination,
    OutsideCapturedScope,
    InvalidFinalRequest,
    RuntimeMetadataUnavailable,
    ProviderDisabled,
    ModelDisabled,
    ProviderInCooldown,
    UnprovenImageBoundForFiniteAccounting,
    MissingRequiredCapability { capability: &'static str },
}

impl fmt::Display for DestinationAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedDestination => formatter.write_str("destination admission rejected malformed destination"),
            Self::OutsideCapturedScope => formatter.write_str("destination admission rejected destination outside captured scope"),
            Self::InvalidFinalRequest => formatter.write_str("destination admission rejected invalid final request controls"),
            Self::RuntimeMetadataUnavailable => formatter.write_str("destination admission requires unavailable runtime metadata"),
            Self::ProviderDisabled => formatter.write_str("destination admission rejected disabled provider"),
            Self::ModelDisabled => formatter.write_str("destination admission rejected disabled model"),
            Self::ProviderInCooldown => formatter.write_str("destination admission rejected provider in cooldown"),
            Self::UnprovenImageBoundForFiniteAccounting => formatter
                .write_str("destination admission rejected unproven image bound for finite accounting"),
            Self::MissingRequiredCapability { capability } => {
                write!(formatter, "destination admission rejected missing required capability: {capability}")
            }
        }
    }
}

impl std::error::Error for DestinationAdmissionError {}

/// A fresh live admission result, carrying only a destination-specific
/// restriction that must be applied before the send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdmittedDestination {
    maximum_output_tokens: Option<u32>,
}

impl AdmittedDestination {
    /// Apply only the live registered destination's downward output limit.
    /// This never creates a budget contract or raises an existing request or
    /// contract limit, so destination preparations remain authoritative.
    pub(crate) fn restrict_output(&self, request: &mut super::LlmRequest) {
        let Some(maximum) = self.maximum_output_tokens else {
            return;
        };
        request.output_token_cap = Some(
            request
                .output_token_cap
                .map_or(maximum, |current| current.min(maximum)),
        );
        if let Some(contract) = &mut request.budget_contract {
            contract.limits.output_tokens = contract.limits.output_tokens.min(i64::from(maximum));
        }
    }
}

/// Send-boundary admission over immutable authority and live runtime metadata.
#[derive(Debug)]
pub(crate) struct DestinationAdmission {
    scope: DestinationScope,
    admitted_inline_image_destinations: BTreeSet<String>,
    admitted_run_local_plaintext_destinations: BTreeSet<String>,
    admitted_inline_image_adapter_required: bool,
    router: Option<ModelRouter>,
}

impl DestinationAdmission {
    #[must_use]
    pub(crate) fn new(
        scope: DestinationScope,
        admitted_inline_image_destinations: BTreeSet<String>,
        registry: Arc<ProviderRegistry>,
    ) -> Self {
        Self {
            scope,
            admitted_inline_image_destinations,
            admitted_run_local_plaintext_destinations: BTreeSet::new(),
            admitted_inline_image_adapter_required: false,
            router: Some(ModelRouter::new(registry)),
        }
    }

    /// Preserve scoped text-only external-driver behavior without inventing a
    /// capability claim when the host has no runtime registry.
    #[must_use]
    pub(crate) fn without_registry(
        scope: DestinationScope,
        admitted_inline_image_destinations: BTreeSet<String>,
    ) -> Self {
        Self {
            scope,
            admitted_inline_image_destinations,
            admitted_run_local_plaintext_destinations: BTreeSet::new(),
            admitted_inline_image_adapter_required: false,
            router: None,
        }
    }

    /// Carry trusted ordinary typed-image route provenance without exposing a
    /// routing input that callers could use to grant transport authority.
    #[must_use]
    pub(crate) fn with_admitted_inline_image_adapter_required(mut self, required: bool) -> Self {
        self.admitted_inline_image_adapter_required = required;
        self
    }

    /// Carry exact host-captured explicit-request destinations that may send
    /// plain text when global model metadata is absent. This records no
    /// capability or registry authority.
    #[must_use]
    pub(crate) fn with_admitted_run_local_plaintext_destinations(
        mut self,
        destinations: BTreeSet<String>,
    ) -> Self {
        self.admitted_run_local_plaintext_destinations = destinations;
        self
    }

    /// Re-evaluate an already-selected literal destination immediately before
    /// dispatch. This never routes, acquires credentials, or expands scope.
    pub(crate) async fn admit(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<AdmittedDestination, DestinationAdmissionError> {
        if !self.scope.permits(destination) {
            return Err(DestinationAdmissionError::OutsideCapturedScope);
        }
        if self.admitted_inline_image_adapter_required
            && requirements.needs_vision
            && !self.admitted_inline_image_destinations.contains(destination)
        {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "admitted_inline_image_adapter",
            });
        }
        let maximum_output_tokens = match &self.router {
            Some(router) => match router.evaluate_destination(destination, requirements).await {
                Ok(maximum) => maximum,
                Err(DestinationAdmissionError::RuntimeMetadataUnavailable)
                    if self
                        .admitted_run_local_plaintext_destinations
                        .contains(destination)
                        && requirements.is_plain_text_without_runtime_metadata() =>
                {
                    // The captured explicit-request binding supplies only this
                    // literal connection. A registry miss still observes its
                    // current cooldown state before dispatch.
                    router.ensure_provider_not_in_cooldown(destination).await?;
                    None
                }
                Err(error) => return Err(error),
            },
            None if requirements.has_hard_capability_requirement() => {
                return Err(DestinationAdmissionError::RuntimeMetadataUnavailable);
            }
            None => None,
        };
        Ok(AdmittedDestination {
            maximum_output_tokens,
        })
    }
}

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
        self.route_with_preferred_model(requirements, None).await
    }

    /// Select a healthy model while preferring one exact fully-qualified model.
    ///
    /// The exact preference is applied only after the normal capability and
    /// health filters, so a model in provider cooldown can never win merely
    /// because policy selected it before routing.
    pub async fn route_with_preferred_model(
        &self,
        requirements: &RouteRequirements,
        preferred_model: Option<&str>,
    ) -> Option<String> {
        let filter = CapabilityFilter {
            needs_tools: requirements.needs_tools,
            needs_reasoning: requirements.needs_reasoning,
            needs_vision: requirements.needs_vision,
            needs_structured_output: requirements.needs_structured_output,
            min_context: requirements.min_context,
            max_cost_per_1m_input: requirements.max_cost_per_1m_input,
        };

        // Runtime registrations are first-class catalog candidates. This is
        // essential for embedded/local models, which intentionally do not live
        // in Liter-LLM's compile-time remote-provider catalog.
        let mut candidates = Vec::<(ProviderInfo, ModelInfo)>::new();
        for provider in self.registry.list().await {
            if !provider.enabled || !self.registry.health().is_available(&provider.id).await {
                continue;
            }
            for model in provider.models.iter().filter(|model| model.enabled) {
                let catalog_model = self.catalog.model(&provider.id, &model.id);
                let model = ModelInfo {
                    id: model.id.clone(),
                    name: model
                        .display_name
                        .clone()
                        .unwrap_or_else(|| model.id.clone()),
                    capabilities: ModelCapabilities {
                        tool_call: model.supports_tools,
                        reasoning: model.supports_reasoning,
                        structured_output: model.supports_structured_output,
                        attachment: model.supports_vision,
                        temperature: true,
                        streaming: model.supports_streaming,
                    },
                    modalities: Modalities {
                        input: if model.supports_vision {
                            vec!["text".to_string(), "image".to_string()]
                        } else {
                            vec!["text".to_string()]
                        },
                        output: vec!["text".to_string()],
                    },
                    limits: ModelLimits {
                        context_window: model.context_window.map_or(0, u64::from),
                        max_output: model.max_output_tokens.map_or(0, u64::from),
                    },
                    cost: catalog_model.and_then(|model| model.cost.clone()),
                    open_weights: provider.api_key.is_none() && provider.base_url.is_empty(),
                    ..ModelInfo::default()
                };
                if filter.matches(&model.capabilities, &model.limits) {
                    candidates.push((
                        ProviderInfo {
                            id: provider.id.clone(),
                            display_name: provider.display_name.clone(),
                            base_url: (!provider.base_url.is_empty())
                                .then(|| provider.base_url.clone()),
                            models: vec![model.clone()],
                            source: Some("runtime-registry".to_string()),
                            ..ProviderInfo::default()
                        },
                        model,
                    ));
                }
            }
        }

        // Add compile-time catalog entries that are configured but were not
        // already represented by their runtime registration metadata.
        let catalog_candidates = self.catalog.models_with_capabilities(&filter);
        for (provider, model) in catalog_candidates {
            if candidates
                .iter()
                .any(|(candidate_provider, candidate_model)| {
                    candidate_provider.id == provider.id && candidate_model.id == model.id
                })
            {
                continue;
            }
            let Some(runtime_provider) = self.registry.get(&provider.id).await else {
                continue;
            };
            if !runtime_provider.enabled
                || !self.registry.health().is_available(&provider.id).await
            {
                continue;
            }
            if runtime_provider
                .models
                .iter()
                .any(|runtime_model| runtime_model.id == model.id)
            {
                // A registration is the live source of facts for its exact
                // identity. Catalog discovery cannot restore a disabled,
                // filtered, low-context, or context-unknown registration.
                continue;
            }
            if requirements.has_hard_capability_requirement() || requirements.min_context.is_some() {
                // Catalog values support only legacy text discovery for an
                // unregistered provider model. They cannot establish a hard
                // payload capability or destination capacity for final send.
                continue;
            }
            candidates.push((provider.clone(), model.clone()));
        }

        // Apply cost filter
        if let Some(max_cost) = requirements.max_cost_per_1m_input {
            candidates.retain(|(_, model)| {
                model
                    .cost
                    .as_ref()
                    .is_some_and(|cost| cost.input <= max_cost)
            });
        }

        if let Some(preferred) = preferred_model {
            let preferred_match = candidates
                .iter()
                .find(|(provider, model)| format!("{}/{}", provider.id, model.id) == preferred)
                .map(|(provider, model)| format!("{}/{}", provider.id, model.id));

            if preferred_match.is_some() {
                return preferred_match;
            }
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

    async fn evaluate_destination(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<Option<u32>, DestinationAdmissionError> {
        let Some((provider_id, model_id)) = destination.split_once('/') else {
            return Err(DestinationAdmissionError::MalformedDestination);
        };
        if provider_id.trim().is_empty() || model_id.trim().is_empty() {
            return Err(DestinationAdmissionError::MalformedDestination);
        }
        let provider = self
            .registry
            .get(provider_id)
            .await
            .ok_or(DestinationAdmissionError::RuntimeMetadataUnavailable)?;
        if !provider.enabled {
            return Err(DestinationAdmissionError::ProviderDisabled);
        }
        if !self.registry.health().is_available(provider_id).await {
            return Err(DestinationAdmissionError::ProviderInCooldown);
        }
        let model = provider
            .models
            .iter()
            .find(|candidate| candidate.id == model_id)
            .ok_or(DestinationAdmissionError::RuntimeMetadataUnavailable)?;
        if !model.enabled {
            return Err(DestinationAdmissionError::ModelDisabled);
        }
        if requirements.needs_tools && !model.supports_tools {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "tools",
            });
        }
        if requirements.needs_reasoning && !model.supports_reasoning {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "reasoning",
            });
        }
        if requirements.needs_vision && !model.supports_vision {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "vision",
            });
        }
        if requirements.needs_structured_output && !model.supports_structured_output {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "structured_output",
            });
        }
        if let Some(minimum) = requirements.min_context
            && model.context_window.map_or(true, |available| u64::from(available) < minimum)
        {
            return Err(DestinationAdmissionError::MissingRequiredCapability {
                capability: "context",
            });
        }
        if let Some(maximum) = requirements.max_cost_per_1m_input {
            let catalog_cost = self
                .catalog
                .model(provider_id, model_id)
                .and_then(|entry| entry.cost.as_ref())
                .map(|cost| cost.input);
            if catalog_cost.is_none_or(|cost| cost > maximum) {
                return Err(DestinationAdmissionError::MissingRequiredCapability {
                    capability: "cost",
                });
            }
        }
        Ok(model.max_output_tokens)
    }

    async fn ensure_provider_not_in_cooldown(
        &self,
        destination: &str,
    ) -> Result<(), DestinationAdmissionError> {
        let Some((provider_id, model_id)) = destination.split_once('/') else {
            return Err(DestinationAdmissionError::MalformedDestination);
        };
        if provider_id.trim().is_empty() || model_id.trim().is_empty() {
            return Err(DestinationAdmissionError::MalformedDestination);
        }
        if !self.registry.health().is_available(provider_id).await {
            return Err(DestinationAdmissionError::ProviderInCooldown);
        }
        Ok(())
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
    use crate::llm::catalog::ModelCost;
    use crate::llm::registry::{ModelConfig, ProtocolSetting, ProviderConfig};

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

    #[tokio::test]
    async fn disabled_runtime_catalog_model_is_not_reentered_or_preferred() {
        // Select an actual embedded catalog entry instead of constructing a
        // look-alike: the regression is the catalog-overlay path itself.
        let (provider_id, provider_name, provider_base_url, model_id) = {
            let (catalog_provider, catalog_model) = ModelCatalog::global()
                .models_with_capabilities(&CapabilityFilter::default())
                .into_iter()
                .next()
                .expect("embedded model catalog has a selectable entry");
            (
                catalog_provider.id.clone(),
                catalog_provider.display_name.clone(),
                catalog_provider.base_url.clone().unwrap_or_default(),
                catalog_model.id.clone(),
            )
        };
        let registry = Arc::new(ProviderRegistry::new());
        registry
            .register(ProviderConfig {
                id: provider_id.clone(),
                display_name: provider_name,
                base_url: provider_base_url,
                api_key: None,
                protocol: ProtocolSetting::Auto,
                default_model: Some(model_id.clone()),
                models: vec![ModelConfig {
                    id: model_id.clone(),
                    pricing_identity: None,
                    execution_profile: None,
                    display_name: None,
                    context_window: None,
                    supports_vision: false,
                    supports_tools: true,
                    supports_reasoning: false,
                    supports_structured_output: false,
                    supports_streaming: true,
                    max_output_tokens: None,
                    enabled: false,
                }],
                enabled: true,
            })
            .await
            .expect("disabled runtime catalog model registers");

        let router = ModelRouter::new(registry);
        let requirements = RouteRequirements::default();
        let destination = format!("{provider_id}/{model_id}");

        assert_eq!(router.route(&requirements).await, None);
        assert_eq!(
            router
                .route_with_preferred_model(&requirements, Some(&destination))
                .await,
            None,
            "a preferred catalog identity must not resurrect a disabled runtime model"
        );
    }

    #[tokio::test]
    async fn registered_context_limit_is_not_reentered_from_catalog_or_preference() {
        let required_context = 8_192_u64;
        let (provider_id, provider_name, provider_base_url, model_id) = {
            let (catalog_provider, catalog_model) = ModelCatalog::global()
                .models_with_capabilities(&CapabilityFilter {
                    min_context: Some(required_context),
                    ..CapabilityFilter::default()
                })
                .into_iter()
                .next()
                .expect("embedded model catalog has an 8192-token entry");
            (
                catalog_provider.id.clone(),
                catalog_provider.display_name.clone(),
                catalog_provider.base_url.clone().unwrap_or_default(),
                catalog_model.id.clone(),
            )
        };
        let registry = Arc::new(ProviderRegistry::new());
        registry
            .register(ProviderConfig {
                id: provider_id.clone(),
                display_name: provider_name,
                base_url: provider_base_url,
                api_key: None,
                protocol: ProtocolSetting::Auto,
                default_model: Some(model_id.clone()),
                models: vec![ModelConfig {
                    id: model_id.clone(),
                    pricing_identity: None,
                    execution_profile: None,
                    display_name: None,
                    context_window: Some(4_096),
                    supports_vision: false,
                    supports_tools: true,
                    supports_reasoning: false,
                    supports_structured_output: false,
                    supports_streaming: true,
                    max_output_tokens: None,
                    enabled: true,
                }],
                enabled: true,
            })
            .await
            .expect("low-context runtime catalog model registers");

        let router = ModelRouter::new(registry);
        let requirements = RouteRequirements {
            min_context: Some(required_context),
            ..RouteRequirements::default()
        };
        let destination = format!("{provider_id}/{model_id}");

        assert_eq!(router.route(&requirements).await, None);
        assert_eq!(
            router
                .route_with_preferred_model(&requirements, Some(&destination))
                .await,
            None,
            "catalog context metadata must not resurrect a filtered runtime identity"
        );
    }
}

/// Requirements for model selection.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
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

impl RouteRequirements {
    #[must_use]
    pub(crate) fn has_hard_capability_requirement(&self) -> bool {
        self.needs_tools
            || self.needs_reasoning
            || self.needs_vision
            || self.needs_structured_output
    }

    /// A metadata-absent run-local binding may carry only ordinary text. It
    /// cannot stand in for declared capability, context, or catalog-cost data.
    fn is_plain_text_without_runtime_metadata(&self) -> bool {
        !self.has_hard_capability_requirement()
            && self.min_context.is_none()
            && self.max_cost_per_1m_input.is_none()
    }
}

/// Derive hard capability demand from the fully prepared request that will be
/// sent, rather than from an advisory route query or a partial caller payload.
/// Provider dialect defaults and disabled fields are not capability requests.
pub(crate) fn requirements_for_final_request_with_explicit_reasoning(
    request: &super::LlmRequest,
    explicit_reasoning: bool,
) -> anyhow::Result<RouteRequirements> {
    let needs_vision = request.messages.iter().any(|message| {
        message
            .get("content")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|parts| {
                parts.iter().any(|part| {
                    part.get("type").and_then(serde_json::Value::as_str) == Some("image_url")
                        && part.get("image_url").is_some()
                })
            })
    });
    let needs_tools = !request.tools.is_empty()
        || matches!(
            request.tool_choice.as_ref(),
            Some(super::ToolChoice::Required | super::ToolChoice::Named(_))
        );
    let needs_structured_output = match request
        .extra_params
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .and_then(|params| params.get("response_format"))
    {
        Some(value) => {
            let response_format: super::response_format::ResponseFormat =
                serde_json::from_value(value.clone())?;
            response_format.validate()?;
            response_format.is_structured()
        }
        None => false,
    };
    Ok(RouteRequirements {
        needs_tools,
        needs_reasoning: explicit_reasoning,
        needs_vision,
        needs_structured_output,
        min_context: request
            .budget_contract
            .as_ref()
            .and_then(|contract| u64::try_from(contract.limits.context_tokens).ok()),
        ..RouteRequirements::default()
    })
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
