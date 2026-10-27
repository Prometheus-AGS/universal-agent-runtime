//! Host-path regressions: real manager capture, actor root, persistence and jobs.

use super::*;
use crate::config::{A2aConfig, A2aPeerConfig, LlmConfig, SecurityConfig};
use crate::llm::{
    DestinationRequestPreparation, DestinationRequestPreparations, EndpointRequestProfile,
    EndpointRequestTransform, ExternalDriverStream, LlmDriver, LlmRequest,
};
use crate::uar::persistence::accounting::{
    AccountingCall, AccountingScope, AccountingState, ScopeIdentity,
};
use crate::uar::persistence::providers::surreal::SurrealDbProvider;
use crate::uar::runtime::context::budget::{
    DestinationLimits, OutputCeilingField, RequestBudgetContract, WireContract,
    endpoint_fingerprint,
};
use crate::uar::runtime::context::token_service::SerializedCountingContract;
use crate::uar::runtime::prompt::{
    PromptRole, PromptTemplateLayout, PromptTemplateProfile, PromptTemplateSelector,
};
use crate::uar::runtime::thread::actor_host::{ActorRootBinding, ActorThreadSession};
use crate::uar::runtime::{
    actor::messages::ActorOwner, manager::RunManager, matching::VectorMatcher,
    skills::SkillRegistry,
};
use axum::response::IntoResponse;
use futures::FutureExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const MODEL: &str = "openai/gpt-4o-mini";
const WAIT: Duration = Duration::from_secs(10);

async fn bounded<T>(operation: &str, future: impl std::future::Future<Output = T>) -> T {
    tokio::time::timeout(WAIT, future)
        .await
        .unwrap_or_else(|_| panic!("{operation} exceeded the fixture deadline"))
}

#[derive(Default)]
struct HoldingDriver {
    ready: tokio::sync::Notify,
    request: std::sync::Mutex<Option<LlmRequest>>,
}

#[async_trait::async_trait]
impl LlmDriver for HoldingDriver {
    async fn stream(&self, request: LlmRequest) -> anyhow::Result<ExternalDriverStream> {
        assert!(
            self.request.lock().unwrap().replace(request).is_none(),
            "fixture admits exactly one root model request"
        );
        self.ready.notify_one();
        Ok(Box::pin(futures::stream::pending()))
    }
}

struct DiscoveryPeer {
    endpoint: String,
    discoveries: Arc<AtomicUsize>,
    mutations: Arc<AtomicUsize>,
    invalid_requests: Arc<AtomicUsize>,
    cancellation: CancellationToken,
    job: JoinHandle<()>,
}

impl DiscoveryPeer {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!(
            "http://{}/a2a/agents/peer-agent",
            listener.local_addr().unwrap()
        );
        let discoveries = Arc::new(AtomicUsize::new(0));
        let mutations = Arc::new(AtomicUsize::new(0));
        let invalid_requests = Arc::new(AtomicUsize::new(0));
        let card = serde_json::json!({
            "capabilities": {"streaming": true},
            "supportedInterfaces": [{"protocolBinding": "JSONRPC", "protocolVersion": "1.0", "url": endpoint}],
        });
        let reads = discoveries.clone();
        let writes = mutations.clone();
        let invalid = invalid_requests.clone();
        let app = axum::Router::new().fallback(move |request: axum::extract::Request| {
            let reads = reads.clone();
            let writes = writes.clone();
            let invalid = invalid.clone();
            let card = card.clone();
            async move {
                if request.method() != axum::http::Method::GET {
                    writes.fetch_add(1, Ordering::SeqCst);
                    return axum::http::StatusCode::METHOD_NOT_ALLOWED.into_response();
                }
                if request.uri().path() != "/.well-known/agent-card.json"
                    || request
                        .headers()
                        .get("authorization")
                        .and_then(|value| value.to_str().ok())
                        != Some("Bearer test-peer-token")
                    || request
                        .headers()
                        .get("a2a-version")
                        .and_then(|value| value.to_str().ok())
                        != Some("1.0")
                {
                    invalid.fetch_add(1, Ordering::SeqCst);
                    return axum::http::StatusCode::BAD_REQUEST.into_response();
                }
                reads.fetch_add(1, Ordering::SeqCst);
                axum::Json(card).into_response()
            }
        });
        let cancellation = CancellationToken::new();
        let stop = cancellation.clone();
        let job = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(stop.cancelled_owned())
                .await
                .unwrap();
        });
        Self {
            endpoint,
            discoveries,
            mutations,
            invalid_requests,
            cancellation,
            job,
        }
    }

    fn assert_observations(&self) {
        assert_eq!(self.discoveries.load(Ordering::SeqCst), 1);
        assert_eq!(
            self.mutations.load(Ordering::SeqCst),
            0,
            "undispatched child must not execute a peer mutation"
        );
        assert_eq!(self.invalid_requests.load(Ordering::SeqCst), 0);
    }
}

impl Drop for DiscoveryPeer {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.job.abort();
    }
}

async fn fixture_owner() -> ActorOwner {
    let config: SecurityConfig = serde_json::from_value(serde_json::json!({
        "jwt_required": true, "jwt_secret": "remote-host-fixture-signing-key",
        "jwt_issuer": "remote-host-fixture", "jwt_audience": "remote-host-fixture",
        "jwt_algorithm": "HS256", "settings_mutation_auth_required": false,
    }))
    .unwrap();
    let token = crate::uar::security::jwt::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({"sub": "remote-budget-owner", "iss": "remote-host-fixture",
            "aud": "remote-host-fixture", "exp": chrono::Utc::now().timestamp() + 300,
            "uar_credential_kind": "issuer"}),
        &jsonwebtoken::EncodingKey::from_secret(b"remote-host-fixture-signing-key"),
    )
    .unwrap();
    let user = crate::uar::security::middleware::authenticate_bearer_context(&config, &token)
        .await
        .unwrap();
    let owner = ActorOwner::from_verified_context(&user).unwrap();
    assert!(owner.credential_principal().is_some());
    owner
}

fn preparation(endpoint: &str) -> Arc<DestinationRequestPreparations> {
    let fingerprint = endpoint_fingerprint(endpoint);
    let profile = DestinationRequestPreparation {
        destination: crate::llm::prompt_dialect::TemplateDestination {
            provider_id: "openai".into(),
            endpoint_kind: "fixture-chat".into(),
            model_id: "gpt-4o-mini".into(),
            model_revision: "host-lifecycle-v1".into(),
            verified_family_revision: None,
            generic_contract_eligible: false,
        },
        template: PromptTemplateProfile {
            id: "host-lifecycle".into(),
            revision: "1".into(),
            selector: PromptTemplateSelector::Exact {
                provider_id: "openai".into(),
                endpoint_kind: "fixture-chat".into(),
                model_id: "gpt-4o-mini".into(),
                model_revision: "host-lifecycle-v1".into(),
            },
            layout: PromptTemplateLayout::Plain,
            required_slots: Vec::new(),
            supported_roles: vec![
                PromptRole::System,
                PromptRole::User,
                PromptRole::Assistant,
                PromptRole::Tool,
            ],
            wire_contract_id: "host-lifecycle-json-v1".into(),
        },
        request_profile: EndpointRequestProfile {
            profile_id: "host-lifecycle".into(),
            profile_revision: "1".into(),
            provider_id: "openai".into(),
            endpoint_kind: "fixture-chat".into(),
            endpoint_fingerprint: fingerprint.clone(),
            qualified_model: MODEL.into(),
            wire_model: "gpt-4o-mini".into(),
            model_revision: "host-lifecycle-v1".into(),
            settings_revision: "1".into(),
            transform: EndpointRequestTransform::OpenAiCompatibleChatV1,
            allowed_request_fields: [
                "model",
                "messages",
                "tools",
                "tool_choice",
                "parallel_tool_calls",
                "stream_options",
                "max_tokens",
                "stream",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            output_ceiling: Some(OutputCeilingField::MaxTokens),
        },
        budget_contract: RequestBudgetContract {
            destination_model: MODEL.into(),
            destination_endpoint_fingerprint: fingerprint,
            limits: DestinationLimits {
                context_tokens: 960,
                independent_input_tokens: Some(928),
                host_input_tokens: None,
                output_tokens: 32,
                additional_reasoning_tokens: 0,
                count_uncertainty_tokens: 0,
            },
            counting: SerializedCountingContract::ExactCl100kJsonV1,
            wire: WireContract::SyntheticOpenAiCompatibleChatV1,
            output_ceiling: OutputCeilingField::MaxTokens,
        },
        exact_settings: None,
        compatible_continuity: Vec::new(),
    };
    Arc::new(DestinationRequestPreparations::new(vec![profile]).unwrap())
}

/// Remains outside setup and scenario futures so unwinding cannot discard
/// acquired resources before their cleanup has been attempted.
#[derive(Default)]
struct FixtureResources {
    database: Option<tempfile::TempDir>,
    bootstrap_started: bool,
    store_closed: bool,
    peer: Option<DiscoveryPeer>,
    provider: Option<Arc<SurrealDbProvider>>,
    manager: Option<Arc<RunManager>>,
    cancellation: CancellationToken,
    root_job: Option<JoinHandle<anyhow::Result<PersistedAgentThread>>>,
    owned_root: Arc<Mutex<Option<ActorRootBinding>>>,
}

async fn cleanup_step<T>(
    errors: &mut Vec<String>,
    operation: &str,
    deadline: Duration,
    future: impl std::future::Future<Output = anyhow::Result<T>>,
) -> Option<T> {
    match std::panic::AssertUnwindSafe(tokio::time::timeout(deadline, future))
        .catch_unwind()
        .await
    {
        Ok(Ok(Ok(value))) => Some(value),
        Ok(Ok(Err(error))) => {
            errors.push(format!("{operation}: {error:#}"));
            None
        }
        Ok(Err(_)) => {
            errors.push(format!(
                "{operation}: deadline elapsed; release unconfirmed"
            ));
            None
        }
        Err(_) => {
            errors.push(format!(
                "{operation}: cleanup panicked; release unconfirmed"
            ));
            None
        }
    }
}

impl FixtureResources {
    async fn cleanup(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        self.cancellation.cancel();
        let mut root = cleanup_step(&mut errors, "captured root lookup", WAIT, async {
            Ok(self.owned_root.lock().await.clone())
        })
        .await
        .flatten();
        if let Some(root) = &root {
            let _ = cleanup_step(
                &mut errors,
                "captured host shutdown",
                Duration::from_secs(5),
                root.shutdown(),
            )
            .await;
        }
        if let Some(job) = self.root_job.as_mut() {
            let joined = cleanup_step(
                &mut errors,
                "root task join",
                Duration::from_secs(5),
                async { Ok(job.await) },
            )
            .await;
            if let Some(result) = joined {
                let _ = self.root_job.take();
                match result {
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) => errors.push(format!("root task: {error:#}")),
                    Err(error) => errors.push(format!("root task join: {error}")),
                }
            }
        }
        if root.is_none() {
            // Registration may have completed while the canceled actor unwound.
            root = cleanup_step(&mut errors, "final captured root lookup", WAIT, async {
                Ok(self.owned_root.lock().await.clone())
            })
            .await
            .flatten();
        }
        if let Some(root) = &root {
            let _ = cleanup_step(
                &mut errors,
                "captured producer join",
                Duration::from_secs(5),
                root.finish(),
            )
            .await;
        }
        if let Some(manager) = &self.manager {
            let _ = cleanup_step(&mut errors, "manager lifecycle shutdown", WAIT, async {
                let report = manager.shutdown_lifecycle().await;
                anyhow::ensure!(
                    report.released
                        && report.failures.is_empty()
                        && report.unresolved_effects.is_empty()
                        && report.dependency_failures.is_empty(),
                    "{report:?}"
                );
                Ok(())
            })
            .await;
            let _ = cleanup_step(
                &mut errors,
                "run history shutdown",
                WAIT,
                manager.shutdown_run_history(),
            )
            .await;
        }
        if let Some(peer) = &mut self.peer {
            peer.cancellation.cancel();
            let _ = cleanup_step(&mut errors, "discovery peer shutdown", WAIT, async {
                (&mut peer.job).await?;
                Ok(())
            })
            .await;
        }
        if let Some(provider) = &self.provider {
            self.store_closed = cleanup_step(&mut errors, "embedded store shutdown", WAIT, async {
                let report = provider.close_scope().await;
                anyhow::ensure!(
                    report.scope_released
                        && report.local_connection_released
                        && report.engine_close_acknowledged
                        && report.failure.is_none(),
                    "{report:?}"
                );
                Ok(())
            })
            .await
            .is_some();
        }
        if self.bootstrap_started && !self.store_closed {
            if let Some(database) = self.database.take() {
                let path = database.keep();
                errors.push(format!(
                    "store closure unacknowledged; retained {}",
                    path.display()
                ));
            }
        }
        errors
    }
}

impl Drop for FixtureResources {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(job) = &self.root_job {
            job.abort();
        }
        // A fallback drop is not an engine-close acknowledgement.
        if self.bootstrap_started && !self.store_closed {
            if let Some(database) = self.database.take() {
                eprintln!(
                    "unclosed fixture store retained at {}",
                    database.keep().display()
                );
            }
        }
    }
}

struct Harness {
    service: Arc<ThreadService>,
    controls: Arc<AgentToolContext>,
    endpoint: String,
    provider: Arc<SurrealDbProvider>,
    root_call: AccountingCall,
    root_scope: AccountingScope,
    available_tokens: u64,
}

impl Harness {
    async fn start(resources: &mut FixtureResources) -> anyhow::Result<Self> {
        resources.peer = Some(bounded("discovery peer setup", DiscoveryPeer::new()).await);
        let endpoint = resources.peer.as_ref().unwrap().endpoint.clone();
        resources.database = Some(tempfile::tempdir()?);
        let database_endpoint = format!(
            "surrealkv://{}",
            resources
                .database
                .as_ref()
                .unwrap()
                .path()
                .join("threads.db")
                .display()
        );
        // Cold migration has no cancellation/join API. Keep its waiter, runtime
        // and directory alive until the actual constructor returns.
        resources.bootstrap_started = true;
        let provider = Arc::new(
            SurrealDbProvider::new(
                &database_endpoint,
                None,
                None,
                Some("remote-host"),
                Some("remote-host"),
            )
            .await?,
        );
        resources.provider = Some(provider.clone());
        let persistence: Arc<dyn PersistenceLayer> = provider.clone();
        let driver = Arc::new(HoldingDriver::default());
        let preparations = preparation(&endpoint);
        let manager = Arc::new(
            bounded(
                "manager setup",
                RunManager::new(
                    LlmConfig {
                        model: MODEL.into(),
                        resolved_provider_id: Some("openai".into()),
                        base_url: Some(endpoint.clone()),
                        api_key: Some(String::new()),
                        ..LlmConfig::default()
                    },
                    Arc::new(crate::mcp::registry::McpRegistry::new_empty()),
                    crate::session::SessionStore::new(),
                    Arc::new(tokio::sync::RwLock::new(SkillRegistry::default())),
                    Arc::new(VectorMatcher::new(
                        Arc::new(
                            crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend::new(
                                Arc::new(
                                    crate::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
                                        384,
                                        "host lifecycle fixture",
                                    ),
                                ),
                            ),
                        ),
                        0.75,
                    )),
                    Some(persistence.clone()),
                ),
            )
            .await
            .with_llm_driver(driver.clone())
            .with_destination_preparations(preparations)
            .with_a2a_config(&A2aConfig {
                instance_id: "source-instance".into(),
                trusted_peers: vec![A2aPeerConfig {
                    instance_id: "peer-instance".into(),
                    agent_id: "peer-agent".into(),
                    endpoint: endpoint.clone(),
                    bearer_token: "test-peer-token".to_owned().into(),
                }],
            }),
        );
        resources.manager = Some(manager.clone());
        let mut artifact = crate::uar::defaults::default_agent();
        artifact.policy.tools.allow = vec!["spawn_agent".into()];
        artifact.extensions.insert(
            "budgets".into(),
            serde_json::json!({"max_tokens_per_turn":1000}),
        );
        artifact.extensions.insert(
            "a2a".into(),
            serde_json::json!({"dependencies":[{"agent_id":"peer-agent","endpoint":endpoint}]}),
        );
        let owner = bounded("fixture authentication", fixture_owner()).await;
        let owner_key = owner.session_owner_key().unwrap();
        let (state, _receiver) = watch::channel(None);
        let owned = resources.owned_root.clone();
        let mut actor = ActorThreadSession::new(
            owner,
            artifact,
            uuid::Uuid::new_v4().to_string(),
            manager.clone(),
            persistence,
            resources.cancellation.clone(),
            state,
            owned.clone(),
        );
        resources.root_job = Some(tokio::spawn(async move {
            actor
                .execute("hold root while testing remote admission".into())
                .await
        }));
        let early_root = bounded("real root reaches model after capture", async {
            tokio::select! {
                biased;
                result = resources.root_job.as_mut().unwrap() => Some(result),
                () = driver.ready.notified() => None,
            }
        })
        .await;
        if let Some(result) = early_root {
            let _ = resources.root_job.take();
            let record = result??;
            if let Some(AgentThreadResult::Failed { code, message }) = &record.thread.result {
                anyhow::bail!("root failed before model readiness: {code}: {message}");
            }
            anyhow::bail!(
                "root terminated before model readiness: {:?}; result: {:?}",
                record.thread.status,
                record.thread.result
            );
        }
        let service = bounded("captured host lookup", owned.lock())
            .await
            .as_ref()
            .unwrap()
            .service
            .get()
            .unwrap()
            .clone();
        let controls = bounded("root controls", service.root_controls())
            .await
            .unwrap();
        assert!(controls.permits("spawn_agent"));
        let request = driver
            .request
            .lock()
            .unwrap()
            .clone()
            .expect("actual admitted request");
        let (input, output) =
            crate::llm::liter_driver::request_budget_bound(MODEL, &request).unwrap();
        assert_eq!(output, 32);
        let reserved_tokens = input.checked_add(output).unwrap();
        assert!(input > 0 && reserved_tokens <= 960 && reserved_tokens < 1000);
        let price = |reads, writes| {
            crate::llm::catalog::estimate_cost_with_cache_write(MODEL, input, output, reads, writes)
                .expect("fixture model has catalog pricing")
        };
        let reserved_cost = price(0, 0).max(price(0, input)).max(price(input, 0));
        let run_id = &service.inner.root.root_run_id;
        let calls = bounded(
            "initial accounting readback",
            provider.list_accounting_calls(&owner_key, run_id),
        )
        .await
        .unwrap();
        assert_eq!(calls.len(), 1);
        let root_call = calls.into_iter().next().unwrap();
        assert_eq!(root_call.admission.identity.owner, owner_key);
        assert_eq!(&root_call.admission.identity.run_id, run_id);
        assert_eq!(root_call.admission.model, MODEL);
        assert_eq!(root_call.admission.reserved_tokens, Some(reserved_tokens));
        assert_eq!(root_call.admission.reserved_cost_usd, Some(reserved_cost));
        assert_eq!(root_call.state, AccountingState::Pending);
        assert_eq!(root_call.observed_tokens, 0);
        assert_eq!(root_call.observed_cost_usd, None);
        assert!(!root_call.unknown());
        let root_scope = bounded(
            "initial scope readback",
            provider.get_accounting_scope(&ScopeIdentity {
                owner: owner_key,
                scope: "run".into(),
                scope_id: run_id.clone(),
            }),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(root_scope.limits.tokens, Some(1000));
        assert_eq!(root_scope.settled_tokens, 0);
        assert_eq!(root_scope.settled_cost_usd, 0.0);
        assert_eq!(root_scope.reserved_tokens, reserved_tokens);
        assert_eq!(root_scope.reserved_cost_usd, reserved_cost);
        assert_eq!(root_scope.pending_calls, 1);
        assert_eq!(root_scope.unknown_calls, 0);
        let available_tokens = 1000 - reserved_tokens;
        Ok(Self {
            service,
            controls,
            endpoint,
            provider,
            root_call,
            root_scope,
            available_tokens,
        })
    }

    fn request(&self) -> RemoteAgentSpawnRequest {
        RemoteAgentSpawnRequest {
            endpoint: self.endpoint.clone(),
            delegated_prompt: "never send this task".into(),
            task_name: Some("remote-probe".into()),
        }
    }

    async fn assert_liability_restored(&self, expected_delegations: usize) {
        let calls = bounded(
            "accounting readback",
            self.provider.list_accounting_calls(
                &self.root_call.admission.identity.owner,
                &self.root_call.admission.identity.run_id,
            ),
        )
        .await
        .unwrap();
        assert_eq!(calls.len(), expected_delegations + 1);
        assert_eq!(
            calls
                .iter()
                .find(|call| call.admission.identity == self.root_call.admission.identity),
            Some(&self.root_call)
        );
        for call in calls
            .iter()
            .filter(|call| call.admission.identity != self.root_call.admission.identity)
        {
            assert_eq!(call.admission.model, "remote-delegation");
            assert_eq!(call.admission.scopes, self.root_call.admission.scopes);
            assert_eq!(call.admission.reserved_tokens, Some(self.available_tokens));
            assert_eq!(call.state, AccountingState::Settled);
            assert_eq!(call.observed_tokens, 0);
            assert_eq!(call.observed_cost_usd, Some(0.0));
            let evidence = format!("host-confirmed-terminal:{}", call.admission.identity.key());
            assert_eq!(
                call.reconciliation_evidence.as_deref(),
                Some(evidence.as_str())
            );
        }
        let scope = bounded(
            "scope readback",
            self.provider
                .get_accounting_scope(&self.root_scope.identity),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(scope.identity, self.root_scope.identity);
        assert_eq!(scope.generation, self.root_scope.generation);
        assert_eq!(scope.limits, self.root_scope.limits);
        assert_eq!(scope.settled_tokens, self.root_scope.settled_tokens);
        assert_eq!(scope.settled_cost_usd, self.root_scope.settled_cost_usd);
        assert_eq!(scope.reserved_tokens, self.root_scope.reserved_tokens);
        assert_eq!(scope.reserved_cost_usd, self.root_scope.reserved_cost_usd);
        assert_eq!(scope.pending_calls, self.root_scope.pending_calls);
        assert_eq!(scope.unknown_calls, self.root_scope.unknown_calls);
        assert_eq!(scope.reconciled, self.root_scope.reconciled);
        assert_eq!(
            scope.reconciliation_evidence,
            self.root_scope.reconciliation_evidence
        );
        assert!(scope.revision >= self.root_scope.revision);
    }

    async fn assert_capacity_restored(&self, expected_delegations: usize) {
        self.assert_liability_restored(expected_delegations).await;
        let budget = self.service.inner.kernel.remote_budget();
        let lease = bounded(
            "capacity reservation",
            budget.reserve_remote(&ThreadBudgets::default()),
        )
        .await
        .expect("root can reserve its full unused allowance again");
        assert_eq!(lease.grant().max_total_tokens, Some(self.available_tokens));
        bounded("capacity release", lease.release_confirmed())
            .await
            .unwrap();
        assert!(lease.is_released());
        self.assert_liability_restored(expected_delegations + 1)
            .await;
    }

    async fn assert_shutdown(&self, resources: &mut FixtureResources) {
        let expected_children = bounded("child snapshot", self.service.inner.entries.lock())
            .await
            .values()
            .filter(|entry| entry.record.thread.parent_thread_id.is_some())
            .map(|entry| {
                let ChildTarget::Remote(remote) = &entry.target else {
                    panic!("fixture child must be remote")
                };
                assert!(!remote.execution_admitted);
                assert_eq!(remote.contract.owner_id, self.service.inner.root.owner_id);
                assert_eq!(
                    remote.contract.root_run_id,
                    self.root_call.admission.identity.run_id
                );
                assert_eq!(
                    remote.contract.child_thread_id,
                    entry.record.thread.thread_id
                );
                assert_eq!(
                    remote.contract.usage_grant.max_total_tokens,
                    Some(self.available_tokens)
                );
                (
                    entry.record.thread.thread_id.clone(),
                    remote.contract.digest().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        tokio::time::timeout(Duration::from_secs(5), self.service.shutdown())
            .await
            .expect("shutdown is bounded")
            .expect("all host jobs join and pending records settle");
        tokio::time::timeout(Duration::from_secs(5), self.service.shutdown())
            .await
            .expect("repeated shutdown is bounded")
            .expect("joined shutdown remains successful");
        let records = bounded(
            "durable child readback",
            self.service.inner.persistence.list_agent_threads(
                &self.service.inner.root.owner_id,
                &self.service.inner.root.root_run_id,
            ),
        )
        .await
        .unwrap();
        for (child_id, contract_digest) in &expected_children {
            let child = records
                .iter()
                .find(|record| &record.thread.thread_id == child_id)
                .expect("accepted child remains durably present after shutdown");
            assert_eq!(
                child.thread.status,
                AgentThreadStatus::Cancelled,
                "shutdown must settle the durable child, not only its watcher"
            );
            let delegation = bounded(
                "bound delegation readback",
                self.provider.get_accounting_delegation(contract_digest),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(
                delegation.admission.identity.owner,
                self.root_call.admission.identity.owner
            );
            assert_eq!(
                delegation.admission.identity.run_id,
                self.root_call.admission.identity.run_id
            );
            assert_ne!(
                delegation.admission.identity,
                self.root_call.admission.identity
            );
            assert_eq!(delegation.admission.model, "remote-delegation");
            assert_eq!(
                delegation.admission.reserved_tokens,
                Some(self.available_tokens)
            );
            assert_eq!(delegation.state, AccountingState::Settled);
            assert_eq!(delegation.observed_tokens, 0);
            assert_eq!(delegation.observed_cost_usd, Some(0.0));
        }
        resources.cancellation.cancel();
        let joined = tokio::time::timeout(
            Duration::from_secs(5),
            resources
                .root_job
                .as_mut()
                .expect("root job remains owned until joined"),
        )
        .await
        .unwrap();
        let _ = resources.root_job.take();
        let record = joined.unwrap().unwrap();
        assert_eq!(record.thread.status, AgentThreadStatus::Cancelled);
        assert!(
            self.service
                .inner
                .jobs
                .lock()
                .unwrap()
                .entries
                .iter()
                .all(|job| job.is_joined())
        );
        // Cancellation is not a terminal usage receipt for the held model.
        self.assert_liability_restored(expected_children.len() + 1)
            .await;
    }
}

use super::super::policy_intersection::ThreadBudgets;

enum LifecycleScenario {
    AdmissionRefusal,
    PersistedLaunchRefusal,
    PendingCancellation,
}

async fn run_scenario(scenario: LifecycleScenario) {
    let mut resources = FixtureResources::default();
    let outcome = std::panic::AssertUnwindSafe(async {
        let harness = Harness::start(&mut resources).await?;
        match scenario {
            LifecycleScenario::AdmissionRefusal => admission_refusal(&harness).await,
            LifecycleScenario::PersistedLaunchRefusal => persisted_launch_refusal(&harness).await,
            LifecycleScenario::PendingCancellation => pending_cancellation(&harness).await,
        }
        harness.assert_shutdown(&mut resources).await;
        Ok::<(), anyhow::Error>(())
    })
    .catch_unwind()
    .await;
    let cleanup = resources.cleanup().await;
    match outcome {
        Err(panic) => {
            if !cleanup.is_empty() {
                eprintln!(
                    "fixture cleanup after original panic: {}",
                    cleanup.join("; ")
                );
            }
            std::panic::resume_unwind(panic);
        }
        Ok(Err(error)) => panic!("fixture failed: {error:#}; cleanup: {cleanup:?}"),
        Ok(Ok(())) => {
            assert!(
                cleanup.is_empty(),
                "fixture cleanup incomplete: {cleanup:?}"
            );
            resources.peer.as_ref().unwrap().assert_observations();
        }
    }
}

async fn admission_refusal(harness: &Harness) {
    let mut occupied = Vec::new();
    for index in 0..4 {
        let child = AgentThread::child(
            &harness.service.inner.root,
            "local-slot".into(),
            Some(&format!("slot-{index}")),
        )
        .unwrap();
        occupied.push(
            harness
                .service
                .inner
                .admission
                .reserve_child(&child)
                .unwrap(),
        );
    }
    let error = bounded(
        "remote admission refusal",
        harness
            .service
            .spawn_remote(harness.controls.scope(), harness.request()),
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("maximum concurrent children"),
        "{error}"
    );
    harness.assert_capacity_restored(0).await;
    drop(occupied);
}

async fn persisted_launch_refusal(harness: &Harness) {
    // Exercise the exact post-persistence launch refusal without cancelling the
    // root before authorize() has a chance to admit the operation.
    harness.service.inner.jobs.lock().unwrap().closed = true;
    let error = bounded(
        "persisted launch refusal",
        harness
            .service
            .spawn_remote_inner(harness.controls.scope().clone(), harness.request()),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("shutting down"), "{error}");
    let entries = bounded(
        "persisted child snapshot",
        harness.service.inner.entries.lock(),
    )
    .await;
    let child = entries
        .values()
        .find(|entry| entry.record.thread.parent_thread_id.is_some())
        .unwrap();
    assert!(child.confirmed);
    assert_eq!(child.record.thread.status, AgentThreadStatus::Pending);
    let ChildTarget::Remote(remote) = &child.target else {
        panic!("remote child")
    };
    assert!(!remote.execution_admitted);
    assert!(remote.reservation.is_released());
    drop(entries);
    harness.assert_capacity_restored(1).await;
}

async fn pending_cancellation(harness: &Harness) {
    // launch() is the last synchronous operation after persistence. On this
    // current-thread runtime, cancel before yielding to its spawned worker.
    let child = bounded(
        "remote pending spawn",
        harness
            .service
            .spawn_remote_inner(harness.controls.scope().clone(), harness.request()),
    )
    .await
    .unwrap();
    let entries = bounded(
        "pending child snapshot",
        harness.service.inner.entries.lock(),
    )
    .await;
    let entry = &entries[&child.thread_id];
    let ChildTarget::Remote(remote) = &entry.target else {
        panic!("remote child")
    };
    assert!(
        !remote.execution_admitted,
        "cancellation point must precede dispatch"
    );
    assert!(!remote.reservation.is_released());
    entry.cancellation.cancel();
    let handle = entry.handle.clone();
    drop(entries);
    let terminal = tokio::time::timeout(Duration::from_secs(5), handle.wait_until_terminal())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(terminal.status, AgentThreadStatus::Cancelled);
    harness.assert_capacity_restored(1).await;
}

#[tokio::test]
async fn remote_admission_refusal_does_not_lease_root_budget() {
    run_scenario(LifecycleScenario::AdmissionRefusal).await;
}

#[tokio::test]
async fn remote_persisted_launch_refusal_releases_capacity_and_shutdown_settles_record() {
    run_scenario(LifecycleScenario::PersistedLaunchRefusal).await;
}

#[tokio::test]
async fn remote_pending_cancellation_releases_capacity_before_any_dispatch() {
    run_scenario(LifecycleScenario::PendingCancellation).await;
}
