//! Integration coverage for model selection, retry, and interrupted-stream behavior.

use std::{
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    Extension,
    body::{Body, to_bytes},
    http::Request,
};
use backon::BackoffBuilder;
use futures::{Stream, StreamExt};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::config::{FailoverConfig, FallbackModel, LlmConfig};
use universal_agent_runtime::llm::{
    DestinationRequestPreparation, DestinationRequestPreparations, EndpointRequestProfile,
    EndpointRequestTransform, LiterLlmDriver, LlmDriver, LlmRequest, Message, MessageContent,
    MessageRole, Orchestrator, ProtectedContinuity, ProviderError, ProviderErrorKind,
    health::ProviderHealthMonitor, mock_driver::MockLlmDriver,
};
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::normalized::NormalizedEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::events::NormalizedEvent as RunEvent;
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::context::{
    budget::{
        DestinationLimits, OutputCeilingField, RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
        WireContract, endpoint_fingerprint,
    },
    token_service::{SerializedCountingContract, TokenService},
    truncate::TruncationPolicy,
};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::native_skill::{NativeSkill, NativeSkillRegistry};
use universal_agent_runtime::uar::runtime::prompt::{
    Authority, PromptFragment, PromptRole, PromptSection, PromptTemplateLayout,
    PromptTemplateProfile, PromptTemplateSelector, Retention,
};
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};
use universal_agent_runtime::uar::settings::resilience_policy::ResiliencePolicy;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[derive(Debug, Default)]
struct FailingDriver {
    calls: AtomicUsize,
}

#[derive(Debug, Default)]
struct RetryCaptureDriver {
    calls: AtomicUsize,
    requests: Mutex<Vec<LlmRequest>>,
}

#[async_trait]
impl LlmDriver for RetryCaptureDriver {
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        self.requests.lock().expect("requests lock").push(request);
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            return Err(ProviderError::new(
                Some(503),
                ProviderErrorKind::Overloaded,
                None,
                "retry fixture unavailable",
            )
            .into());
        }
        Ok(Box::pin(futures::stream::iter([
            Ok(NormalizedEvent::MessageDelta {
                text: "prepared retry succeeded".to_string(),
            }),
            Ok(NormalizedEvent::Done),
        ])))
    }
}

#[derive(Debug, Default)]
struct CountingSuccessDriver {
    calls: AtomicUsize,
}

struct CanonicalLoopSkill;

#[async_trait]
impl NativeSkill for CanonicalLoopSkill {
    fn name(&self) -> &str {
        "canonical_loop_result"
    }

    fn description(&self) -> &str {
        "Returns a large result for canonical iteration coverage"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "additionalProperties": false})
    }

    fn output_limit(&self) -> Option<TruncationPolicy> {
        Some(TruncationPolicy::Bytes(128))
    }

    async fn execute(&self, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({"payload": "x".repeat(2_000)}))
    }
}

#[async_trait]
impl LlmDriver for CountingSuccessDriver {
    async fn stream(
        &self,
        _: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Box::pin(futures::stream::iter([
            Ok(NormalizedEvent::MessageDelta {
                text: "fallback should not dispatch".to_string(),
            }),
            Ok(NormalizedEvent::Done),
        ])))
    }
}

#[async_trait]
impl LlmDriver for FailingDriver {
    async fn stream(
        &self,
        _: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(anyhow::anyhow!("primary provider unavailable"))
    }
}

#[derive(Debug, Default)]
struct IdleThenSuccessDriver {
    calls: AtomicUsize,
    first_events: Vec<NormalizedEvent>,
    repeat_metadata: bool,
}

#[async_trait]
impl LlmDriver for IdleThenSuccessDriver {
    async fn stream(
        &self,
        _: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        let attempt = self.calls.fetch_add(1, Ordering::SeqCst);
        if attempt == 0 {
            let tail: Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>> =
                if self.repeat_metadata {
                    Box::pin(futures::stream::unfold(0, |index| async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Some((
                            Ok(NormalizedEvent::StreamStart {
                                request_id: format!("failed-prelude-{index}"),
                            }),
                            index + 1,
                        ))
                    }))
                } else {
                    Box::pin(futures::stream::pending())
                };
            return Ok(Box::pin(
                futures::stream::iter(self.first_events.clone().into_iter().map(Ok)).chain(tail),
            ));
        }
        Ok(Box::pin(futures::stream::iter([
            Ok(NormalizedEvent::MessageDelta {
                text: "retry succeeded".to_string(),
            }),
            Ok(NormalizedEvent::Done),
        ])))
    }
}

#[derive(Debug, Default)]
struct InterruptedThenSuccessDriver {
    calls: AtomicUsize,
    requests: Mutex<Vec<LlmRequest>>,
}

#[derive(Debug)]
struct TypedFailureThenSuccessDriver {
    calls: AtomicUsize,
    kind: ProviderErrorKind,
    status: u16,
    retry_after: Option<Duration>,
    message: &'static str,
}

#[async_trait]
impl LlmDriver for TypedFailureThenSuccessDriver {
    async fn stream(
        &self,
        _: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            return Err(ProviderError::new(
                Some(self.status),
                self.kind,
                self.retry_after,
                self.message,
            )
            .into());
        }
        Ok(Box::pin(futures::stream::iter([
            Ok(NormalizedEvent::MessageDelta {
                text: "typed retry succeeded".to_string(),
            }),
            Ok(NormalizedEvent::Done),
        ])))
    }
}

impl InterruptedThenSuccessDriver {
    fn requests(&self) -> Vec<LlmRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

#[async_trait]
impl LlmDriver for InterruptedThenSuccessDriver {
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        self.requests.lock().expect("requests lock").push(request);
        let attempt = self.calls.fetch_add(1, Ordering::SeqCst);
        let events: Vec<anyhow::Result<NormalizedEvent>> = if attempt == 0 {
            vec![
                Ok(NormalizedEvent::MessageDelta {
                    text: "partial assistant answer".to_string(),
                }),
                Err(ProviderError::new(
                    None,
                    ProviderErrorKind::Stream,
                    None,
                    "provider stream disconnected",
                )
                .into()),
            ]
        } else {
            vec![
                Ok(NormalizedEvent::MessageDelta {
                    text: "recovered next turn".to_string(),
                }),
                Ok(NormalizedEvent::Done),
            ]
        };
        Ok(Box::pin(futures::stream::iter(events)))
    }
}

async fn run_manager(driver: Arc<dyn LlmDriver>, sessions: SessionStore) -> Arc<RunManager> {
    run_manager_with_policy(
        driver,
        sessions,
        ResiliencePolicy {
            retries_enabled: false,
            ..ResiliencePolicy::default()
        },
    )
    .await
}

async fn run_manager_with_policy(
    driver: Arc<dyn LlmDriver>,
    sessions: SessionStore,
    policy: ResiliencePolicy,
) -> Arc<RunManager> {
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are outside this integration boundary",
    ));
    Arc::new(
        RunManager::new(
            LlmConfig {
                model: "provider/model".to_string(),
                api_key: Some("model-path-fixture-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            sessions,
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver)
        .with_resilience_policy(policy),
    )
}

async fn wait_for_run(manager: &RunManager, run_id: &str) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let complete = manager
                .history_since(run_id, None)
                .await
                .expect("started run keeps event history")
                .iter()
                .any(|event| matches!(event.event, RunEvent::RunDone { .. }));
            if complete {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("run reaches a terminal event");
}

fn ten_retry_policy(jitter: &str) -> ResiliencePolicy {
    ResiliencePolicy {
        // The schedule iterator counts retry sleeps, while the validated public
        // setting counts total attempts. Use eleven here to inspect ten seeded
        // retry values independently from configuration admission.
        retry_max_attempts: 11,
        retry_base_delay_ms: 1_000,
        retry_backoff_multiplier: 2.0,
        retry_max_delay_ms: 1_024_000,
        retry_budget_ms: 2_000_000,
        retry_jitter_mode: jitter.to_string(),
        ..ResiliencePolicy::default()
    }
}

fn liter_driver(base_url: &str) -> LiterLlmDriver {
    let llm = LlmConfig {
        model: "openai/retry-fixture".to_string(),
        resolved_provider_id: Some("openai".to_string()),
        api_key: Some("wiremock-fixture-key".to_string()),
        base_url: Some(format!("{base_url}/v1")),
        max_retries: 0,
        ..LlmConfig::default()
    };
    LiterLlmDriver::new(
        universal_agent_runtime::config::build_client_config(&llm),
        llm.model,
        Some(false),
    )
    .expect("wiremock base URL builds a liter driver")
}

fn direct_liter_request() -> LlmRequest {
    LlmRequest {
        messages: vec![serde_json::json!({
            "role": "user",
            "content": "exercise the provider HTTP boundary"
        })],
        tools: Vec::new(),
        cache_strategy: None,
        thinking_config: None,
        anthropic_system: None,
        extra_params: None,
        budget_contract: None,
    }
}

fn exact_endpoint_profile(
    base_url: &str,
    model: &str,
    transform: EndpointRequestTransform,
    allowed_request_fields: &[&str],
    output_ceiling: Option<OutputCeilingField>,
) -> EndpointRequestProfile {
    EndpointRequestProfile {
        profile_id: format!("fixture-{model}"),
        profile_revision: "fixture-profile-v1".to_string(),
        provider_id: model
            .split_once('/')
            .expect("fixture model has a provider identity")
            .0
            .to_string(),
        endpoint_kind: "fixture-chat".to_string(),
        endpoint_fingerprint: endpoint_fingerprint(base_url),
        qualified_model: model.to_string(),
        wire_model: model.to_string(),
        model_revision: "fixture-model-v1".to_string(),
        settings_revision: "fixture-settings-v1".to_string(),
        transform,
        allowed_request_fields: allowed_request_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        output_ceiling,
    }
}

fn exact_budget_contract(
    base_url: &str,
    model: &str,
    output_ceiling: OutputCeilingField,
    output_tokens: i64,
) -> RequestBudgetContract {
    RequestBudgetContract {
        destination_model: model.to_string(),
        destination_endpoint_fingerprint: endpoint_fingerprint(base_url),
        limits: DestinationLimits {
            context_tokens: 8_192,
            independent_input_tokens: Some(7_680),
            host_input_tokens: None,
            output_tokens,
            additional_reasoning_tokens: 0,
            count_uncertainty_tokens: 0,
        },
        counting: SerializedCountingContract::ExactCl100kJsonV1,
        wire: WireContract::SyntheticOpenAiCompatibleChatV1,
        output_ceiling,
    }
}

fn canonical_host_fragment(content: &str) -> PromptFragment {
    PromptFragment::new(
        "host-contract",
        PromptSection::HostInstructions,
        "fixture-host",
        Authority::Host,
        PromptRole::System,
        Retention::Session,
        content,
    )
}

fn destination_preparation(
    base_url: &str,
    model: &str,
    budget_contract: RequestBudgetContract,
    layout: PromptTemplateLayout,
    compatible_continuity: Vec<ProtectedContinuity>,
) -> DestinationRequestPreparation {
    let (provider_id, model_id) = model
        .split_once('/')
        .expect("fixture model has provider and model identities");
    DestinationRequestPreparation {
        destination: universal_agent_runtime::llm::prompt_dialect::TemplateDestination {
            provider_id: provider_id.to_string(),
            endpoint_kind: "fixture-chat".to_string(),
            model_id: model_id.to_string(),
            model_revision: "fixture-model-v1".to_string(),
            verified_family_revision: None,
            generic_contract_eligible: false,
        },
        template: PromptTemplateProfile {
            id: format!("template-{model}"),
            revision: "fixture-template-v1".to_string(),
            selector: PromptTemplateSelector::Exact {
                provider_id: provider_id.to_string(),
                endpoint_kind: "fixture-chat".to_string(),
                model_id: model_id.to_string(),
                model_revision: "fixture-model-v1".to_string(),
            },
            layout,
            required_slots: vec![PromptSection::HostInstructions],
            supported_roles: vec![PromptRole::System],
            wire_contract_id: "fixture-wire-v1".to_string(),
        },
        request_profile: exact_endpoint_profile(
            base_url,
            model,
            EndpointRequestTransform::OpenAiCompatibleChatV1,
            &[
                "model",
                "messages",
                "parallel_tool_calls",
                "stream_options",
                "max_completion_tokens",
                "stream",
            ],
            Some(OutputCeilingField::MaxCompletionTokens),
        ),
        budget_contract,
        exact_settings: None,
        compatible_continuity,
    }
}

#[tokio::test(start_paused = true)]
async fn retry_reprepares_exact_template_from_canonical_history() {
    let model = "provider/model";
    let base_url = "http://retry-preparation.invalid/v1";
    let preparation = destination_preparation(
        base_url,
        model,
        exact_budget_contract(
            base_url,
            model,
            OutputCeilingField::MaxCompletionTokens,
            512,
        ),
        PromptTemplateLayout::StructuredXml,
        Vec::new(),
    );
    let preparations = Arc::new(
        DestinationRequestPreparations::new(vec![preparation])
            .expect("exact retry preparation is valid"),
    );
    let driver = Arc::new(RetryCaptureDriver::default());
    let canonical_history = vec![Message {
        role: MessageRole::User,
        content: MessageContent::text("canonical retry input"),
        tool_call_id: None,
        tool_calls: None,
    }];
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: model.to_string(),
            resolved_provider_id: Some("provider".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    )
    .with_destination_preparations(
        preparations,
        canonical_history,
        vec![canonical_host_fragment("canonical host policy")],
    )
    .with_resilience_policy(ResiliencePolicy {
        retries_enabled: true,
        retry_max_attempts: 2,
        retry_base_delay_ms: 1,
        retry_max_delay_ms: 1,
        retry_jitter_mode: "none".to_string(),
        retry_budget_ms: 10,
        ..ResiliencePolicy::default()
    });

    let events = orchestrator
        .chat_with_history(vec![
            Message {
                role: MessageRole::System,
                content: MessageContent::text("stale rendered system"),
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: MessageRole::User,
                content: MessageContent::text("stale reduced input"),
                tool_call_id: None,
                tool_calls: None,
            },
        ])
        .await
        .expect("prepared retry starts")
        .collect::<Vec<_>>()
        .await;
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "prepared retry succeeded"
    )));

    let requests = driver.requests.lock().expect("requests lock");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].messages, requests[1].messages);
    assert!(
        requests[0].messages[0]["content"]
            .as_str()
            .expect("rendered system content")
            .contains("canonical host policy")
    );
    assert_eq!(requests[0].messages[1]["content"], "canonical retry input");
    assert!(
        requests[0]
            .messages
            .iter()
            .all(|message| message["content"] != "stale reduced input")
    );
    assert_eq!(
        requests[0]
            .budget_contract
            .as_ref()
            .expect("retry request has exact budget")
            .destination_model,
        model
    );
}

#[tokio::test]
async fn tool_iteration_reprepares_from_untruncated_canonical_result() {
    let model = "provider/model";
    let base_url = "http://iteration-preparation.invalid/v1";
    let preparations = Arc::new(
        DestinationRequestPreparations::new(vec![destination_preparation(
            base_url,
            model,
            exact_budget_contract(
                base_url,
                model,
                OutputCeilingField::MaxCompletionTokens,
                512,
            ),
            PromptTemplateLayout::Plain,
            Vec::new(),
        )])
        .expect("exact iteration preparation is valid"),
    );
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::ToolCallDelta {
                call_index: 0,
                id: Some("canonical-loop-call".to_string()),
                name: Some("canonical_loop_result".to_string()),
                arguments_delta: Some("{}".to_string()),
            },
            NormalizedEvent::ToolCallComplete {
                call_index: 0,
                id: "canonical-loop-call".to_string(),
                name: "canonical_loop_result".to_string(),
                arguments_json: "{}".to_string(),
            },
            NormalizedEvent::Done,
        ],
        vec![
            NormalizedEvent::MessageDelta {
                text: "iteration complete".to_string(),
            },
            NormalizedEvent::Done,
        ],
    ]));
    let native_skills = Arc::new(NativeSkillRegistry::new());
    native_skills
        .register(CanonicalLoopSkill)
        .await
        .expect("canonical loop skill registers");
    let canonical_history = vec![Message {
        role: MessageRole::User,
        content: MessageContent::text("run the canonical loop tool"),
        tool_call_id: None,
        tool_calls: None,
    }];
    let events = Orchestrator::from_driver(
        LlmConfig {
            model: model.to_string(),
            resolved_provider_id: Some("provider".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        native_skills,
        driver.clone(),
    )
    .with_destination_preparations(
        preparations,
        canonical_history.clone(),
        vec![canonical_host_fragment("canonical iteration policy")],
    )
    .chat_with_history(canonical_history)
    .await
    .expect("canonical iteration starts")
    .collect::<Vec<_>>()
    .await;

    let displayed = events
        .iter()
        .find_map(|event| match event {
            NormalizedEvent::ToolResult {
                name,
                content,
                success: true,
                ..
            } if name == "canonical_loop_result" => Some(content),
            _ => None,
        })
        .expect("tool result is emitted");
    assert!(displayed.len() <= 128);
    assert!(displayed.starts_with("Warning: truncated output"));

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    let canonical_result = requests[1]
        .messages
        .iter()
        .find(|message| message["role"] == "tool")
        .and_then(|message| message["content"].as_str())
        .expect("second attempt contains the canonical tool result");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(canonical_result)
            .expect("canonical result remains typed JSON"),
        serde_json::json!({"payload": "x".repeat(2_000)})
    );
}

#[tokio::test]
async fn smaller_window_failover_rejects_protected_canonical_content_before_http() {
    let server = MockServer::start().await;
    let base_url = format!("{}/v1", server.uri());
    let fallback_model = SYNTHETIC_EXACT_MODEL;
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let fallback_driver = Arc::new(
        LiterLlmDriver::new(config, fallback_model.to_string(), Some(false))
            .expect("build fallback driver")
            .with_endpoint_profile(exact_endpoint_profile(
                &base_url,
                fallback_model,
                EndpointRequestTransform::OpenAiCompatibleChatV1,
                &[
                    "model",
                    "messages",
                    "parallel_tool_calls",
                    "stream_options",
                    "max_completion_tokens",
                    "stream",
                ],
                Some(OutputCeilingField::MaxCompletionTokens),
            ))
            .expect("bind fallback profile"),
    );
    let primary_model = "primary/model";
    let primary = Arc::new(FailingDriver::default());
    let mut fallback_budget = exact_budget_contract(
        &base_url,
        fallback_model,
        OutputCeilingField::MaxCompletionTokens,
        10,
    );
    fallback_budget.limits.context_tokens = 64;
    fallback_budget.limits.independent_input_tokens = Some(54);
    let preparations = Arc::new(
        DestinationRequestPreparations::new(vec![
            destination_preparation(
                &base_url,
                primary_model,
                exact_budget_contract(
                    &base_url,
                    primary_model,
                    OutputCeilingField::MaxCompletionTokens,
                    512,
                ),
                PromptTemplateLayout::Plain,
                Vec::new(),
            ),
            destination_preparation(
                &base_url,
                fallback_model,
                fallback_budget,
                PromptTemplateLayout::StructuredXml,
                Vec::new(),
            ),
        ])
        .expect("primary and fallback preparations are valid"),
    );
    let protected_payload = serde_json::json!({"protected": "x".repeat(8_000)}).to_string();
    let canonical_history = vec![Message {
        role: MessageRole::User,
        content: MessageContent::text(protected_payload.clone()),
        tool_call_id: None,
        tool_calls: None,
    }];
    let failover = FailoverConfig {
        enabled: true,
        fallback_models: vec![FallbackModel {
            model: fallback_model.to_string(),
            api_key: None,
            base_url: Some(base_url.clone()),
        }],
        ..FailoverConfig::default()
    };
    let events = Orchestrator::from_driver(
        LlmConfig {
            model: primary_model.to_string(),
            resolved_provider_id: Some("primary".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        primary.clone(),
    )
    .with_failover(fallback_driver, failover)
    .with_destination_preparations(
        preparations,
        canonical_history,
        vec![canonical_host_fragment("retain every protected byte")],
    )
    .with_resilience_policy(ResiliencePolicy {
        retries_enabled: false,
        ..ResiliencePolicy::default()
    })
    .chat(&protected_payload)
    .await
    .expect("failover run starts")
    .collect::<Vec<_>>()
    .await;

    assert_eq!(primary.calls.load(Ordering::SeqCst), 1);
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::Error { message, .. } if message.contains("ProtectedOverflow")
    )));
    assert!(
        server
            .received_requests()
            .await
            .expect("wiremock recording")
            .is_empty()
    );
}

#[tokio::test]
async fn incompatible_opaque_continuity_blocks_fallback_before_driver_call() {
    let primary_model = "primary/model";
    let fallback_model = "fallback/model";
    let base_url = "http://continuity.invalid/v1";
    let continuity = ProtectedContinuity {
        provider_id: "primary".to_string(),
        endpoint_kind: "fixture-chat".to_string(),
        protocol_revision: "signed-reasoning-v1".to_string(),
    };
    let preparations = Arc::new(
        DestinationRequestPreparations::new(vec![
            destination_preparation(
                base_url,
                primary_model,
                exact_budget_contract(
                    base_url,
                    primary_model,
                    OutputCeilingField::MaxCompletionTokens,
                    512,
                ),
                PromptTemplateLayout::Plain,
                vec![continuity.clone()],
            ),
            destination_preparation(
                base_url,
                fallback_model,
                exact_budget_contract(
                    base_url,
                    fallback_model,
                    OutputCeilingField::MaxCompletionTokens,
                    512,
                ),
                PromptTemplateLayout::Plain,
                Vec::new(),
            ),
        ])
        .expect("continuity fixture preparations are valid"),
    );
    let primary = Arc::new(FailingDriver::default());
    let fallback = Arc::new(CountingSuccessDriver::default());
    let failover = FailoverConfig {
        enabled: true,
        fallback_models: vec![FallbackModel {
            model: fallback_model.to_string(),
            api_key: None,
            base_url: None,
        }],
        ..FailoverConfig::default()
    };
    let events = Orchestrator::from_driver(
        LlmConfig {
            model: primary_model.to_string(),
            resolved_provider_id: Some("primary".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        primary.clone(),
    )
    .with_failover(fallback.clone(), failover)
    .with_destination_preparations(
        preparations,
        vec![Message {
            role: MessageRole::User,
            content: MessageContent::text("continue signed turn"),
            tool_call_id: None,
            tool_calls: None,
        }],
        vec![canonical_host_fragment("preserve opaque continuity")],
    )
    .with_protected_continuity(continuity)
    .with_resilience_policy(ResiliencePolicy {
        retries_enabled: false,
        ..ResiliencePolicy::default()
    })
    .chat("continue signed turn")
    .await
    .expect("continuity run starts")
    .collect::<Vec<_>>()
    .await;

    assert_eq!(primary.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fallback.calls.load(Ordering::SeqCst), 0);
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::Error { message, .. }
            if message.contains("incompatible with protected continuity")
    )));
}

#[tokio::test]
async fn exact_profile_captures_final_outbound_body_and_reserved_output_ceiling() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw("data: [DONE]\n\n", "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let base_url = format!("{}/v1", server.uri());
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let profile = exact_endpoint_profile(
        &base_url,
        SYNTHETIC_EXACT_MODEL,
        EndpointRequestTransform::OpenAiCompatibleChatV1,
        &[
            "model",
            "messages",
            "parallel_tool_calls",
            "stream_options",
            "max_completion_tokens",
            "stream",
        ],
        Some(OutputCeilingField::MaxCompletionTokens),
    );
    let driver = LiterLlmDriver::new(config, SYNTHETIC_EXACT_MODEL.to_string(), Some(false))
        .expect("build exact-profile driver")
        .with_endpoint_profile(profile)
        .expect("bind exact endpoint profile");
    let mut request = direct_liter_request();
    request.budget_contract = Some(RequestBudgetContract::synthetic_exact(&base_url));
    let receipt = driver
        .profiled_wire_receipt(&request)
        .expect("profiled request produces a redacted wire receipt");

    let mut stream = driver
        .stream(request)
        .await
        .expect("profiled exact request dispatches");
    while stream.next().await.is_some() {}

    let requests = server
        .received_requests()
        .await
        .expect("wiremock request recording is enabled");
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value =
        serde_json::from_slice(&requests[0].body).expect("captured request is JSON");
    assert_eq!(
        body,
        serde_json::json!({
            "model": SYNTHETIC_EXACT_MODEL,
            "messages": [{
                "role": "user",
                "content": "exercise the provider HTTP boundary"
            }],
            "parallel_tool_calls": false,
            "stream_options": {"include_usage": true},
            "max_completion_tokens": 512,
            "stream": true
        }),
        "the captured body must equal the complete supported wire fixture"
    );
    let digest = Sha256::digest(&requests[0].body);
    let actual_sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(receipt.sha256, actual_sha256);
    assert_eq!(receipt.serialized_bytes, requests[0].body.len());
    let captured_count = TokenService::count_serialized(
        SerializedCountingContract::ExactCl100kJsonV1,
        &requests[0].body,
    )
    .expect("captured final request uses the exact counting contract");
    assert_eq!(receipt.counted_tokens, Some(captured_count.tokens));
}

#[tokio::test]
async fn exact_profile_rejects_an_unlisted_setting_before_dispatch() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused endpoint");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("endpoint address")
    );
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let profile = exact_endpoint_profile(
        &base_url,
        "openai/settings-fixture",
        EndpointRequestTransform::OpenAiCompatibleChatV1,
        &["model", "messages", "stream_options", "stream"],
        None,
    );
    let driver = LiterLlmDriver::new(config, "openai/settings-fixture".to_string(), None)
        .expect("build settings fixture driver")
        .with_endpoint_profile(profile)
        .expect("bind settings fixture profile");
    let mut request = direct_liter_request();
    request.extra_params = Some(serde_json::json!({"temperature": 0.2}));

    let error = match driver.stream(request).await {
        Ok(_) => panic!("unlisted endpoint settings must fail before dispatch"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("unsupported setting `temperature`")
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "invalid settings must not open a provider connection"
    );
}

#[tokio::test]
async fn anthropic_transform_growth_above_the_reserved_ceiling_is_rejected_before_dispatch() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused Anthropic endpoint");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("endpoint address")
    );
    let model = "anthropic/ceiling-fixture";
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let profile = exact_endpoint_profile(
        &base_url,
        model,
        EndpointRequestTransform::AnthropicMessagesV1,
        &["model", "messages", "max_tokens", "stream"],
        Some(OutputCeilingField::MaxTokens),
    );
    let driver = LiterLlmDriver::new(config, model.to_string(), None)
        .expect("build Anthropic ceiling fixture driver")
        .with_endpoint_profile(profile)
        .expect("bind Anthropic endpoint profile");
    let mut request = direct_liter_request();
    request.extra_params = Some(serde_json::json!({"reasoning_effort": "high"}));
    request.budget_contract = Some(exact_budget_contract(
        &base_url,
        model,
        OutputCeilingField::MaxTokens,
        512,
    ));

    let error = match driver.stream(request).await {
        Ok(_) => panic!("transform growth above the reservation must fail before dispatch"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("would raise max_tokens to 16385, above the reserved output ceiling 512")
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "an under-reserved transformed request must not open a provider connection"
    );
}

#[tokio::test]
async fn profiled_ceiling_without_a_budget_is_rejected_before_dispatch() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused profiled endpoint");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("endpoint address")
    );
    let profile = exact_endpoint_profile(
        &base_url,
        "openai/missing-budget-fixture",
        EndpointRequestTransform::OpenAiCompatibleChatV1,
        &[
            "model",
            "messages",
            "stream_options",
            "max_completion_tokens",
            "stream",
        ],
        Some(OutputCeilingField::MaxCompletionTokens),
    );
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url)
        .max_retries(0)
        .build();
    let driver = LiterLlmDriver::new(config, "openai/missing-budget-fixture".to_string(), None)
        .expect("build missing-budget fixture driver")
        .with_endpoint_profile(profile)
        .expect("bind missing-budget profile");

    let error = match driver.stream(direct_liter_request()).await {
        Ok(_) => panic!("a declared ceiling requires a matching request budget"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("has no budget contract"));
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "a missing reservation must not open a provider connection"
    );
}

#[tokio::test]
async fn endpoint_profile_rejects_wrong_endpoint_and_divergent_wire_model() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused exact endpoint");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("endpoint address")
    );
    let model = "openai/binding-fixture";
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let wrong_endpoint = exact_endpoint_profile(
        "http://127.0.0.1:1/v1",
        model,
        EndpointRequestTransform::OpenAiCompatibleChatV1,
        &["model", "messages", "stream_options", "stream"],
        None,
    );
    let error = LiterLlmDriver::new(config.clone(), model.to_string(), None)
        .expect("build exact endpoint driver")
        .with_endpoint_profile(wrong_endpoint)
        .expect_err("another endpoint fingerprint must be rejected");
    assert!(
        error
            .to_string()
            .contains("does not match the driver endpoint")
    );

    let mut divergent = exact_endpoint_profile(
        &base_url,
        model,
        EndpointRequestTransform::OpenAiCompatibleChatV1,
        &["model", "messages", "stream_options", "stream"],
        None,
    );
    divergent.wire_model = "binding-fixture".to_string();
    let error = LiterLlmDriver::new(config, model.to_string(), None)
        .expect("build divergent model driver")
        .with_endpoint_profile(divergent)
        .expect_err("a separately transformed wire model must remain unsupported");
    assert!(
        error
            .to_string()
            .contains("identical qualified and wire model")
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "profile binding failures must not open a provider connection"
    );
}

#[tokio::test]
async fn anthropic_thinking_budget_overflow_is_rejected_before_dispatch() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind unused Anthropic endpoint");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("endpoint address")
    );
    let model = "anthropic/overflow-fixture";
    let profile = exact_endpoint_profile(
        &base_url,
        model,
        EndpointRequestTransform::AnthropicMessagesV1,
        &["model", "messages", "max_tokens", "stream"],
        Some(OutputCeilingField::MaxTokens),
    );
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let driver = LiterLlmDriver::new(config, model.to_string(), None)
        .expect("build Anthropic overflow fixture driver")
        .with_endpoint_profile(profile)
        .expect("bind Anthropic overflow profile");
    let mut request = direct_liter_request();
    request.extra_params = Some(serde_json::json!({
        "thinking": {"type": "enabled", "budget_tokens": u64::MAX}
    }));
    request.budget_contract = Some(exact_budget_contract(
        &base_url,
        model,
        OutputCeilingField::MaxTokens,
        512,
    ));

    let error = match driver.stream(request).await {
        Ok(_) => panic!("overflowing transformed output must fail before dispatch"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("overflows max_tokens"));
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "an overflowing transform must not open a provider connection"
    );
}

async fn retry_after_case(respect_retry_after: bool) -> (Duration, usize, Vec<NormalizedEvent>) {
    let driver = Arc::new(TypedFailureThenSuccessDriver {
        calls: AtomicUsize::new(0),
        kind: ProviderErrorKind::RateLimited,
        status: 429,
        retry_after: Some(Duration::from_secs(7)),
        message: "fixture rate limit",
    });
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/retry-fixture".to_string(),
            resolved_provider_id: Some("openai".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    )
    .with_resilience_policy(ResiliencePolicy {
        retry_max_attempts: 2,
        retry_base_delay_ms: 1_000,
        retry_backoff_multiplier: 2.0,
        retry_max_delay_ms: 1_000,
        retry_budget_ms: 10_000,
        retry_jitter_mode: "none".to_string(),
        retry_respect_retry_after: respect_retry_after,
        ..ResiliencePolicy::default()
    });

    let started = tokio::time::Instant::now();
    let events = orchestrator
        .chat("retry the rate-limited request")
        .await
        .expect("orchestrator creates a run stream")
        .collect::<Vec<_>>()
        .await;
    let elapsed = started.elapsed();

    (elapsed, driver.calls.load(Ordering::SeqCst), events)
}

#[test]
fn seeded_full_jitter_is_exact_bounded_and_distinct_from_unjittered_backoff() {
    let full = ten_retry_policy("full")
        .retry_backoff_builder()
        .with_jitter_seed(7)
        .build()
        .collect::<Vec<_>>();
    let expected = [
        773_381_829,
        444_971_949,
        2_873_161_554,
        3_849_015_236,
        1_863_689_423,
        5_297_179_699,
        50_664_287_567,
        11_846_517_563,
        35_512_073_517,
        227_439_834_595,
    ]
    .map(Duration::from_nanos);

    assert_eq!(full, expected);
    assert_eq!(full.len(), 10);
    for (index, delay) in full.iter().enumerate() {
        let ceiling = Duration::from_secs(1_u64 << index);
        assert!(!delay.is_zero(), "retry {index} must have nonzero jitter");
        assert!(
            *delay <= ceiling,
            "retry {index} exceeded its exponential ceiling: {delay:?} > {ceiling:?}"
        );
    }
    assert!(full.windows(2).any(|pair| pair[0] != pair[1]));

    let none = ten_retry_policy("none")
        .retry_backoff_builder()
        .with_jitter_seed(7)
        .build()
        .collect::<Vec<_>>();
    let unjittered = (0..10)
        .map(|index| Duration::from_secs(1_u64 << index))
        .collect::<Vec<_>>();
    assert_eq!(none, unjittered);
}

#[tokio::test]
async fn liter_driver_preserves_rate_limit_status_and_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "7")
                .set_body_json(serde_json::json!({
                    "error": { "message": "fixture rate limit" }
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let error = match liter_driver(&server.uri())
        .stream(direct_liter_request())
        .await
    {
        Ok(_) => panic!("HTTP 429 must fail before a model stream is established"),
        Err(error) => error,
    };
    let provider_error = ProviderError::from_anyhow(&error)
        .expect("liter driver retains typed provider metadata through anyhow");

    assert_eq!(provider_error.status, Some(429));
    assert_eq!(provider_error.kind, ProviderErrorKind::RateLimited);
    assert_eq!(provider_error.retry_after, Some(Duration::from_secs(7)));
}

#[tokio::test(start_paused = true)]
async fn retry_after_header_overrides_computed_backoff_when_enabled() {
    let (elapsed, calls, events) = retry_after_case(true).await;

    assert_eq!(elapsed, Duration::from_secs(7));
    assert_eq!(calls, 2, "normalized events: {events:?}");
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "typed retry succeeded"
    )));
}

#[tokio::test(start_paused = true)]
async fn computed_backoff_is_used_when_retry_after_header_is_disabled() {
    let (elapsed, calls, events) = retry_after_case(false).await;

    assert_eq!(elapsed, Duration::from_secs(1));
    assert_eq!(calls, 2, "normalized events: {events:?}");
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "typed retry succeeded"
    )));
}

#[tokio::test]
async fn cooled_down_fallback_is_skipped_before_its_driver_is_attempted() {
    let primary = Arc::new(FailingDriver::default());
    let fallback_a = Arc::new(MockLlmDriver::echo());
    let fallback_b = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "selected fallback b".to_string(),
        },
        NormalizedEvent::Done,
    ]]));
    let health = Arc::new(ProviderHealthMonitor::new());
    health.record_failure("provider-a", 1, 60).await;

    let failover = FailoverConfig {
        enabled: true,
        error_threshold: 1,
        cooldown_secs: 60,
        fallback_models: vec![
            FallbackModel {
                model: "provider-a/model-a".to_string(),
                api_key: None,
                base_url: None,
            },
            FallbackModel {
                model: "provider-b/model-b".to_string(),
                api_key: None,
                base_url: None,
            },
        ],
        ..FailoverConfig::default()
    };
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "primary/model".to_string(),
            resolved_provider_id: Some("primary".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        primary.clone(),
    )
    .with_failovers(
        vec![
            (
                "provider-a/model-a".to_string(),
                fallback_a.clone() as Arc<dyn LlmDriver>,
            ),
            (
                "provider-b/model-b".to_string(),
                fallback_b.clone() as Arc<dyn LlmDriver>,
            ),
        ],
        failover,
    )
    .with_health_monitor(health)
    .with_resilience_policy(ResiliencePolicy {
        retries_enabled: false,
        ..ResiliencePolicy::default()
    });

    let events = orchestrator
        .chat("select a healthy fallback")
        .await
        .expect("orchestrator creates a run stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(primary.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fallback_a.call_count(), 0, "cooldown must prevent the call");
    assert_eq!(fallback_b.call_count(), 1);
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "selected fallback b"
    )));
}

#[tokio::test(start_paused = true)]
async fn idle_stream_timeout_is_retryable_and_the_next_attempt_succeeds() {
    let driver = Arc::new(IdleThenSuccessDriver::default());
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "provider/model".to_string(),
            resolved_provider_id: Some("provider".to_string()),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    )
    .with_resilience_policy(ResiliencePolicy {
        stream_start_timeout_ms: 1_000,
        stream_idle_timeout_ms: 1_000,
        retry_max_attempts: 2,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 100,
        retry_jitter_mode: "none".to_string(),
        retry_budget_ms: 1_000,
        ..ResiliencePolicy::default()
    });

    let events = orchestrator
        .chat("retry an idle stream")
        .await
        .expect("orchestrator creates a run stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(driver.calls.load(Ordering::SeqCst), 2);
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "retry succeeded"
    )));
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, NormalizedEvent::Error { .. }))
    );
}

fn short_idle_policy() -> ResiliencePolicy {
    ResiliencePolicy {
        stream_start_timeout_ms: 1_000,
        stream_idle_timeout_ms: 1_000,
        retry_max_attempts: 2,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 100,
        retry_jitter_mode: "none".to_string(),
        retry_budget_ms: 1_000,
        ..ResiliencePolicy::default()
    }
}

fn metadata_usage(total: u32) -> NormalizedEvent {
    NormalizedEvent::Usage {
        prompt_tokens: total,
        completion_tokens: 0,
        total_tokens: total,
        cached_tokens: None,
        cache_creation_tokens: None,
    }
}

#[tokio::test(start_paused = true)]
async fn metadata_only_stalls_retry_without_resetting_the_first_output_deadline() {
    for repeat_metadata in [false, true] {
        let driver = Arc::new(IdleThenSuccessDriver {
            first_events: vec![
                NormalizedEvent::StreamStart {
                    request_id: "failed-prelude".into(),
                },
                metadata_usage(7),
                NormalizedEvent::MessageDelta {
                    text: String::new(),
                },
                NormalizedEvent::ReasoningDelta {
                    text: String::new(),
                },
                NormalizedEvent::ThinkingDelta {
                    text: String::new(),
                },
            ],
            repeat_metadata,
            ..IdleThenSuccessDriver::default()
        });
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            Arc::new(NativeSkillRegistry::new()),
            driver.clone(),
        )
        .with_resilience_policy(short_idle_policy());
        let events = tokio::time::timeout(Duration::from_secs(4), async {
            orchestrator
                .chat("retry before content")
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        })
        .await
        .expect("metadata cannot keep a content-free attempt alive indefinitely");
        assert_eq!(driver.calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event,
                    NormalizedEvent::MessageDelta { text } if text == "retry succeeded"
                ))
                .count(),
            1
        );
        assert!(!events.iter().any(|event| matches!(event,
            NormalizedEvent::StreamStart { request_id } if request_id.starts_with("failed-prelude")
        )));
        assert!(!events.iter().any(|event| matches!(
            event,
            NormalizedEvent::Usage { .. } | NormalizedEvent::Error { .. }
        )));
    }
}

#[tokio::test]
async fn successful_prelude_preserves_latest_cumulative_usage_once() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::StreamStart {
            request_id: "provider-response".into(),
        },
        metadata_usage(7),
        metadata_usage(11),
        NormalizedEvent::MessageDelta {
            text: "accounted".into(),
        },
        NormalizedEvent::Done,
    ]]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig::default(),
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    )
    .with_resilience_policy(short_idle_policy());
    let events = orchestrator
        .chat("retain usage")
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
    let totals = events
        .iter()
        .filter_map(|event| match event {
            NormalizedEvent::Usage { total_tokens, .. } => Some(*total_tokens),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(totals, vec![11]);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event,
        NormalizedEvent::MessageDelta { text } if text == "accounted"))
            .count(),
        1
    );
    assert!(matches!(events.last(), Some(NormalizedEvent::Done)));
    assert_eq!(driver.call_count(), 1);
}

#[tokio::test]
async fn partial_text_idle_timeout_is_persisted_without_retrying_the_model() {
    let driver = Arc::new(IdleThenSuccessDriver {
        first_events: vec![NormalizedEvent::MessageDelta {
            text: "partial idle answer".into(),
        }],
        ..IdleThenSuccessDriver::default()
    });
    let sessions = SessionStore::new();
    let manager =
        run_manager_with_policy(driver.clone(), sessions.clone(), short_idle_policy()).await;
    let run_id = manager
        .start_run(
            default_agent(),
            "begin".into(),
            Some("partial-idle-session".into()),
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &run_id).await;
    assert_eq!(
        driver.calls.load(Ordering::SeqCst),
        1,
        "partial output must never replay inference"
    );
    let run = manager.get_run(&run_id).await.unwrap();
    assert_eq!(run.context["turn_interrupted"]["authority"], "host");
    let session = sessions.get("partial-idle-session").unwrap();
    assert!(session.messages().iter().any(|message| {
        message.content.as_text().is_some_and(|text| {
            text.contains("partial idle answer")
                && text.contains("[TurnInterrupted: provider_error]")
        })
    }));
}

#[tokio::test(start_paused = true)]
async fn retry_decision_uses_typed_error_kind_instead_of_message_text() {
    let policy = ResiliencePolicy {
        retry_max_attempts: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 100,
        retry_jitter_mode: "none".to_string(),
        retry_budget_ms: 1_000,
        ..ResiliencePolicy::default()
    };
    let invalid = Arc::new(TypedFailureThenSuccessDriver {
        calls: AtomicUsize::new(0),
        kind: ProviderErrorKind::InvalidRequest,
        status: 400,
        retry_after: None,
        message: "429 overloaded timeout retry immediately",
    });
    let invalid_orchestrator = Orchestrator::from_driver(
        LlmConfig::default(),
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        invalid.clone(),
    )
    .with_resilience_policy(policy.clone());
    let invalid_events = invalid_orchestrator
        .chat("do not retry an invalid request")
        .await
        .expect("invalid request still produces a run stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(invalid.calls.load(Ordering::SeqCst), 1);
    assert!(
        invalid_events
            .iter()
            .any(|event| matches!(event, NormalizedEvent::Error { .. }))
    );

    let overloaded = Arc::new(TypedFailureThenSuccessDriver {
        calls: AtomicUsize::new(0),
        kind: ProviderErrorKind::Overloaded,
        status: 503,
        retry_after: None,
        message: "bad request is permanent and must never retry",
    });
    let overloaded_orchestrator = Orchestrator::from_driver(
        LlmConfig::default(),
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
        overloaded.clone(),
    )
    .with_resilience_policy(policy);
    let overloaded_events = overloaded_orchestrator
        .chat("retry an overloaded provider")
        .await
        .expect("overloaded request produces a run stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(overloaded.calls.load(Ordering::SeqCst), 2);
    assert!(overloaded_events.iter().any(|event| matches!(
        event,
        NormalizedEvent::MessageDelta { text } if text == "typed retry succeeded"
    )));
    assert!(
        overloaded_events
            .iter()
            .all(|event| !matches!(event, NormalizedEvent::Error { .. }))
    );
}

#[tokio::test]
async fn interrupted_turn_is_persisted_and_replayed_to_the_next_model_request() {
    let driver = Arc::new(InterruptedThenSuccessDriver::default());
    let sessions = SessionStore::new();
    let manager = run_manager(driver.clone(), sessions.clone()).await;
    let session_id = "interrupted-model-turn";

    let first_run = manager
        .start_run(
            default_agent(),
            "begin a response".to_string(),
            Some(session_id.to_string()),
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &first_run).await;

    let failed_run = manager
        .get_run(&first_run)
        .await
        .expect("failed run remains inspectable");
    assert_eq!(failed_run.context["turn_interrupted"]["authority"], "host");
    assert!(
        failed_run.context["turn_interrupted"]["content"]
            .as_str()
            .is_some_and(|content| content.contains("[TurnInterrupted: provider_error]"))
    );
    let session = sessions
        .get(session_id)
        .expect("anonymous session remains available");
    let interrupted_message = session
        .messages()
        .into_iter()
        .find(|message| {
            message.content.as_text().is_some_and(|text| {
                text.contains("partial assistant answer")
                    && text.contains("[TurnInterrupted: provider_error]")
            })
        })
        .expect("partial assistant content is retained with its interruption marker");
    assert!(interrupted_message.tool_calls.is_none());

    let second_run = manager
        .start_run(
            default_agent(),
            "continue safely".to_string(),
            Some(session_id.to_string()),
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &second_run).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests[1].messages.iter().any(|message| {
        message["content"].as_str().is_some_and(|text| {
            text.contains("partial assistant answer")
                && text.contains("[TurnInterrupted: provider_error]")
        })
    }));
}

#[tokio::test]
async fn partial_tool_call_is_persisted_and_blocks_resume_dispatch() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::ToolCallDelta {
            call_index: 0,
            id: Some("partial-call".to_string()),
            name: Some("partial_tool".to_string()),
            arguments_delta: Some("{\"unfinished\":".to_string()),
        },
        NormalizedEvent::Error {
            message: "provider stream interrupted before tool-call completion".to_string(),
            code: Some("PROVIDER_INTERRUPTED".to_string()),
        },
    ]]));
    let sessions = SessionStore::new();
    let manager = run_manager(driver.clone(), sessions.clone()).await;
    let session_id = "partial-tool-call-turn";

    let first_run = manager
        .start_run(
            default_agent(),
            "start a partial tool call".to_string(),
            Some(session_id.to_string()),
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &first_run).await;

    let session = sessions
        .get(session_id)
        .expect("partial tool-call session remains available");
    let pending = session
        .messages()
        .into_iter()
        .find_map(|message| {
            message.tool_calls.and_then(|calls| {
                calls.into_iter().find(|call| call.id == "partial-call")
            })
        })
        .expect("partial tool call is retained as unresolved history");
    assert_eq!(pending.function.name, "partial_tool");
    assert_eq!(pending.function.arguments, "{\"unfinished\":");

    let resumed_run = manager
        .start_run(
            default_agent(),
            "resume without replaying partial work".to_string(),
            Some(session_id.to_string()),
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &resumed_run).await;

    assert_eq!(
        driver.requests().len(),
        1,
        "unresolved partial calls must block resume before provider dispatch"
    );
}

#[tokio::test]
async fn legacy_unbudgeted_dispatch_is_labeled_in_context_and_artifacts() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "legacy response".to_string(),
        },
        NormalizedEvent::Done,
    ]]));
    let manager = run_manager(driver.clone(), SessionStore::new()).await;

    let run_id = manager
        .start_run(
            default_agent(),
            "use the uncapped legacy provider".to_string(),
            None,
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &run_id).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].budget_contract.is_none());

    let run = manager
        .get_run(&run_id)
        .await
        .expect("completed unbudgeted run remains inspectable");
    let manifests = run.context["attempt_manifests"]
        .as_array()
        .expect("attempt manifests are persisted in run context");
    assert!(manifests.iter().any(|manifest| {
        manifest["budgeting"]["label"] == "legacy_unbudgeted"
            && manifest["budgeting"]["contract_present"] == false
            && manifest["budgeting"]["fit_guarantee"] == false
    }));

    let history = manager
        .history_since(&run_id, None)
        .await
        .expect("completed run retains attempt artifact history");
    let artifact = history
        .iter()
        .find_map(|event| match &event.event {
            RunEvent::Artifact { artifact, .. }
                if artifact.artifact_type == "attempt_manifest" =>
            {
                Some(artifact)
            }
            _ => None,
        })
        .expect("legacy attempt emits a typed attempt manifest artifact");
    let payload: serde_json::Value =
        serde_json::from_str(&artifact.content).expect("attempt manifest is JSON");
    assert_eq!(payload["budgeting"]["label"], "legacy_unbudgeted");
}

#[tokio::test]
async fn last_event_id_replays_only_subsequent_chat_events() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "before cursor".to_string(),
        },
        NormalizedEvent::MessageDelta {
            text: "after cursor".to_string(),
        },
        NormalizedEvent::Done,
    ]]));
    let manager = run_manager(driver, SessionStore::new()).await;
    let run_id = manager
        .start_run(
            default_agent(),
            "stream a replayable response".to_string(),
            None,
            None,
            Vec::new(),
        )
        .await;
    wait_for_run(&manager, &run_id).await;
    let history = manager
        .history_since(&run_id, None)
        .await
        .expect("completed run retains replay history");
    let cursor = history
        .iter()
        .find_map(|event| match &event.event {
            RunEvent::ChatDelta { text_delta, .. } if text_delta == "before cursor" => {
                Some(event.id)
            }
            _ => None,
        })
        .expect("first chat delta supplies the reconnect cursor");
    let after_cursor_id = history
        .iter()
        .find_map(|event| match &event.event {
            RunEvent::ChatDelta { text_delta, .. } if text_delta == "after cursor" => {
                Some(event.id)
            }
            _ => None,
        })
        .expect("second chat delta supplies the replayed event id");

    let user = UserContext {
        user_id: "anonymous".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "anonymous".to_string(),
            name: None,
            roles: None,
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    };
    let app = universal_agent_runtime::uar::api::router()
        .with_state(manager)
        .layer(Extension(user));
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/runs/{run_id}/stream"))
                .header("Last-Event-ID", cursor.to_string())
                .body(Body::empty())
                .expect("SSE reconnect request builds"),
        )
        .await
        .expect("SSE reconnect route responds");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("SSE replay body reads");
    let body = String::from_utf8(body.to_vec()).expect("SSE body is UTF-8");

    assert!(!body.contains("before cursor"));
    assert!(body.contains("after cursor"));
    assert!(body.contains(&format!("id: {after_cursor_id}")));
}
