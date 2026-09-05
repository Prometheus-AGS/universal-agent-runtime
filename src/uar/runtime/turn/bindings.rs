//! Model clients resolved once by the trusted run host. Reusing this snapshot
//! never re-reads provider credentials, environment variables, or routing state.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{FailoverConfig, LlmConfig};
use crate::llm::{LlmDriver, Orchestrator, ProviderHealthMonitor};
use crate::mcp::registry::McpRegistry;
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::runtime::skills::{
    SkillRegistry,
    service::{SkillMatchingSnapshot, SkillService},
};
use crate::uar::runtime::thread::policy_intersection::{
    CredentialGrant, CredentialTarget, ThreadPolicy,
};

/// Host-only child inputs. The manager must not resolve replacements from its
/// global registries when any inherited binding is unavailable.
pub(crate) struct InheritedRunBindings {
    pub(crate) policy: Arc<ThreadPolicy>,
    pub(crate) presentations: Arc<super::super::presentations::RunPresentationSnapshot>,
    pub(crate) thread: crate::uar::runtime::thread::AgentThread,
    pub(crate) controls: Arc<crate::uar::runtime::thread::control::AgentToolContext>,
    pub(crate) models: RunModelBindings,
    pub(crate) skills: Arc<RunSkillBindings>,
    pub(crate) mcp: Arc<McpRegistry>,
    pub(crate) native: Arc<NativeSkillRegistry>,
    pub(crate) sandbox: Option<Arc<crate::sandbox::bindings::SandboxBinding>>,
    pub(crate) harness: crate::config::HarnessConfig,
    pub(crate) working_directory: std::path::PathBuf,
    pub(crate) approvals: crate::uar::runtime::thread::approvals::RootApprovalChannel,
}

/// Executable resources retained by one live root, not recipes for rebuilding
/// clients. The run index holds only a weak reference to this capture.
pub(crate) struct RunDelegationBindings {
    pub(crate) owner: crate::uar::runtime::actor::messages::ActorOwner,
    pub(crate) run_id: String,
    pub(crate) artifact: crate::uar::domain::artifact::AgentArtifact,
    /// Shared across all captures of this root so tree limits cannot be reset
    /// by attaching a second scheduler before the first child is persisted.
    pub(crate) thread_attachment_claimed: std::sync::atomic::AtomicBool,
    /// The root host commits to installing the built-in turn-local factories.
    /// This is not a user/artifact authorization to spawn.
    pub(crate) thread_controls: bool,
    pub(crate) policy: crate::uar::domain::policy::EffectiveRunPolicy,
    pub(crate) presentations: Arc<super::super::presentations::RunPresentationSnapshot>,
    pub(crate) models: RunModelBindings,
    pub(crate) skills: Arc<RunSkillBindings>,
    pub(crate) native: Arc<NativeSkillRegistry>,
    pub(crate) sandbox: Option<Arc<crate::sandbox::bindings::SandboxBinding>>,
    pub(crate) activation:
        Arc<tokio::sync::Mutex<crate::uar::runtime::skills::activation::ActivationContext>>,
    pub(crate) harness: crate::config::HarnessConfig,
    pub(crate) working_directory: std::path::PathBuf,
    pub(crate) approvals: crate::uar::runtime::thread::approvals::RootApprovalChannel,
    pub(crate) cancellation: tokio_util::sync::CancellationToken,
}

impl std::fmt::Debug for RunDelegationBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunDelegationBindings")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

/// Only the root kernel owns this lease. Borrowers retaining resource Arcs
/// cannot keep the delegation lifetime open after root completion or unwind.
pub(crate) struct RunDelegationLifetime(pub(crate) Option<Arc<RunDelegationBindings>>);

impl Drop for RunDelegationLifetime {
    fn drop(&mut self) {
        if let Some(bindings) = &self.0 {
            bindings.cancellation.cancel();
        }
    }
}

/// Matching, catalog rendering, and activation share this one captured index.
pub(crate) struct RunSkillBindings {
    pub(crate) registry: Arc<RwLock<SkillRegistry>>,
    pub(crate) matching: Option<SkillMatchingSnapshot>,
}

impl RunSkillBindings {
    pub(crate) async fn capture(
        legacy_registry: &Arc<RwLock<SkillRegistry>>,
        service: Option<&SkillService>,
    ) -> Self {
        match service {
            Some(service) => {
                let matching = service.matching_snapshot().await;
                Self {
                    registry: Arc::clone(&matching.registry),
                    matching: Some(matching),
                }
            }
            None => Self {
                registry: Arc::new(RwLock::new(legacy_registry.read().await.clone())),
                matching: None,
            },
        }
    }
}

#[derive(Clone)]
struct BoundModel {
    model: String,
    grant: CredentialGrant,
    driver: Arc<dyn LlmDriver>,
}

impl BoundModel {
    fn capture(model: String, driver: Arc<dyn LlmDriver>) -> Self {
        let (provider, _) = crate::llm::registry::split_model_string_pub(&model);
        Self {
            model,
            driver,
            grant: CredentialGrant {
                target: CredentialTarget::Provider(provider),
                binding_id: uuid::Uuid::new_v4().to_string(),
            },
        }
    }
}

#[derive(Clone)]
pub(crate) struct RunModelBindings {
    config: LlmConfig,
    primary: BoundModel,
    fallbacks: Vec<BoundModel>,
    /// All still-granted clients, including providers not selected for this
    /// turn. Descendants may select them only while the policy retains the grant.
    catalog: Vec<BoundModel>,
    failover: FailoverConfig,
    health: Option<Arc<ProviderHealthMonitor>>,
    budget: crate::uar::runtime::cost_budget::ModelCallBudget,
}

impl RunModelBindings {
    /// Capture at the root host after policy, routing, and credentials resolve.
    /// Child execution must reuse an authorized subset, never call this method.
    ///
    /// # Errors
    /// Returns the primary client's construction failure. Unavailable fallback
    /// clients retain the existing skip-and-report behavior.
    pub(crate) async fn capture(
        config: LlmConfig,
        supplied_primary: Option<Arc<dyn LlmDriver>>,
        failover: FailoverConfig,
        health: Option<Arc<ProviderHealthMonitor>>,
        budget: crate::uar::runtime::cost_budget::ModelCallBudget,
    ) -> anyhow::Result<Self> {
        budget.admit()?;
        let primary = match supplied_primary {
            Some(driver) => driver,
            None => crate::llm::orchestrator::build_driver(&config)?,
        };
        let (model_provider, model) = crate::llm::registry::split_model_string_pub(&config.model);
        let provider = config
            .resolved_provider_id
            .as_deref()
            .unwrap_or(&model_provider);
        let primary_model = format!("{provider}/{model}");
        let primary = BoundModel::capture(primary_model.clone(), primary);
        let mut fallbacks = Vec::new();
        if failover.enabled {
            for fallback in &failover.fallback_models {
                if fallback.model == primary_model {
                    continue;
                }
                let (provider, _) = crate::llm::registry::split_model_string_pub(&fallback.model);
                if let Some(health) = &health
                    && !health.is_available(&provider).await
                {
                    tracing::info!(model = %fallback.model, %provider,
                        "Skipping fallback provider in cooldown while capturing run bindings");
                    continue;
                }
                match Orchestrator::build_fallback_driver(&config, fallback) {
                    Ok(driver) => {
                        fallbacks.push(BoundModel::capture(fallback.model.clone(), driver))
                    }
                    Err(error) => tracing::warn!(model = %fallback.model, %error,
                        "Failed to capture fallback driver; continuing with remaining candidates"),
                }
            }
        }
        let catalog = std::iter::once(&primary)
            .chain(fallbacks.iter())
            .cloned()
            .collect();
        Ok(Self {
            config,
            primary,
            fallbacks,
            catalog,
            failover,
            health,
            budget,
        })
    }

    pub(crate) fn primary(&self) -> Arc<dyn LlmDriver> {
        self.budget
            .bind(self.primary.model.clone(), Arc::clone(&self.primary.driver))
    }

    pub(crate) fn config(&self) -> &LlmConfig {
        &self.config
    }

    pub(crate) fn budget(&self) -> &crate::uar::runtime::cost_budget::ModelCallBudget {
        &self.budget
    }

    /// Opaque grants for root policy capture; no keys or connection recipes.
    pub(crate) fn credential_grants(&self) -> std::collections::BTreeSet<CredentialGrant> {
        self.catalog
            .iter()
            .map(|binding| binding.grant.clone())
            .collect()
    }

    /// Select a child's route only from exact inherited credential bindings.
    /// Rebinding preserves the client and narrows its shared-root budget. Raw
    /// captured clients are wrapped only when used, never doubly charged.
    /// It never invokes a provider constructor or a credential resolver.
    pub(crate) fn for_policy(&self, policy: &ThreadPolicy) -> anyhow::Result<Self> {
        let budget = self.budget.narrowed(&policy.permissions().budgets)?;
        let grants = self.credential_grants();
        if policy.permissions().credentials.iter().any(|grant| {
            matches!(grant.target, CredentialTarget::Provider(_)) && !grants.contains(grant)
        }) {
            anyhow::bail!("Child provider grant is not present in the inherited model catalog");
        }
        let route = policy
            .effective()
            .model
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Child model route is missing"))?;
        let bind = |provider: &str, model: &str| -> anyhow::Result<BoundModel> {
            let qualified = format!("{provider}/{model}");
            let eligible = |binding: &&BoundModel| {
                binding.grant.target == CredentialTarget::Provider(provider.to_string())
                    && policy.permissions().credentials.contains(&binding.grant)
            };
            let bindings = || self.catalog.iter();
            let binding = bindings()
                .filter(eligible)
                .find(|binding| binding.model == qualified)
                .or_else(|| bindings().find(eligible))
                .ok_or_else(|| {
                    anyhow::anyhow!("Child model has no inherited credential binding")
                })?;
            Ok(BoundModel {
                driver: if binding.model == qualified {
                    Arc::clone(&binding.driver)
                } else {
                    binding.driver.with_bound_model(&qualified)?
                },
                model: qualified,
                grant: binding.grant.clone(),
            })
        };
        let primary = bind(&route.provider_id, &route.model_id)?;
        let mut fallbacks = Vec::new();
        if self.failover.enabled {
            for fallback in &policy.artifact().policy.provider.fallbacks {
                let binding = bind(&fallback.provider, &fallback.model)?;
                if binding.model != primary.model
                    && !fallbacks
                        .iter()
                        .any(|existing: &BoundModel| existing.model == binding.model)
                {
                    fallbacks.push(binding);
                }
            }
        }
        let mut config = self.config.clone();
        config.model.clone_from(&primary.model);
        config.resolved_provider_id = Some(route.provider_id.clone());
        // The selected driver already owns its exact connection. These values
        // must not suggest that reconstructing it from a config is permitted.
        config.api_key = None;
        config.api_key_env = None;
        config.base_url = None;
        config.provider_keys.clear();
        let mut failover = self.failover.clone();
        failover.fallback_models.clear();
        let catalog = self
            .catalog
            .iter()
            .filter(|binding| policy.permissions().credentials.contains(&binding.grant))
            .cloned()
            .collect();
        Ok(Self {
            config,
            primary,
            fallbacks,
            catalog,
            failover,
            health: self.health.clone(),
            budget,
        })
    }

    /// Assemble a tool loop over the exact captured clients, with no provider
    /// construction or credential lookup. Live health may revoke availability;
    /// it cannot introduce an uncaptured driver.
    pub(crate) fn orchestrator(
        &self,
        mcp: Arc<McpRegistry>,
        native: Arc<NativeSkillRegistry>,
    ) -> Orchestrator {
        let orchestrator =
            Orchestrator::from_driver(self.config.clone(), mcp, native, self.primary())
                .with_failovers(
                    self.fallbacks
                        .iter()
                        .map(|binding| {
                            (
                                binding.model.clone(),
                                self.budget
                                    .bind(binding.model.clone(), Arc::clone(&binding.driver)),
                            )
                        })
                        .collect(),
                    self.failover.clone(),
                );
        match &self.health {
            Some(health) => orchestrator.with_health_monitor(Arc::clone(health)),
            None => orchestrator,
        }
    }
}
