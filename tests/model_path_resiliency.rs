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
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::config::{FailoverConfig, FallbackModel, LlmConfig};
use universal_agent_runtime::llm::{
    LiterLlmDriver, LlmDriver, LlmRequest, Orchestrator, ProviderError, ProviderErrorKind,
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
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
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
    }
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
