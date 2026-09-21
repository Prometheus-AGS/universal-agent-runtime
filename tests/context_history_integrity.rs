//! Behavioral tests for the `context-history-integrity` change.
//!
//! Spec: `openspec/changes/context-history-integrity/specs/conversation-history-integrity/spec.md`.
//! Each test maps to one requirement scenario. They are written before the
//! implementation and are expected to fail to compile until the modules under
//! `universal_agent_runtime::uar::runtime::context` exist.

use universal_agent_runtime::llm::{
    Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
};
use universal_agent_runtime::uar::runtime::context::budget::{
    RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
};
use universal_agent_runtime::uar::runtime::context::normalize::{
    HistoryValidationError, SyntheticReason, normalize_history, synthetic_tool_result,
};

#[path = "support/harness_context.rs"]
mod harness_context;

fn text(role: MessageRole, s: &str) -> Message {
    Message {
        role,
        content: MessageContent::text(s),
        tool_call_id: None,
        tool_calls: None,
    }
}

fn summary_contract() -> RequestBudgetContract {
    RequestBudgetContract::synthetic_exact("http://summary.invalid/v1")
}

/// Scenario: Protected blocks between prose.
#[tokio::test]
async fn host_marked_summarization_never_exposes_protected_bodies() {
    use std::sync::Arc;

    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::context::summarizer::{
        HostMarkedProseSpan, summarize_marked_prose,
    };

    let arguments = r#"{"path":"/private/raw.txt","line":17}"#;
    let result_body = "RAW_TOOL_RESULT_DO_NOT_SUMMARIZE";
    let evidence = "EVIDENCE_BLOCK_DO_NOT_SUMMARIZE";
    let history = vec![
        text(MessageRole::User, "eligible earlier narrative"),
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(vec![ToolCall {
                id: "summary-protected-call".to_string(),
                call_type: "function".to_string(),
                function: ToolCallFunction {
                    name: "read_file".to_string(),
                    arguments: arguments.to_string(),
                },
            }]),
        },
        tool_result("summary-protected-call", result_body),
        text(MessageRole::Assistant, evidence),
        text(MessageRole::Assistant, "eligible later narrative"),
    ];
    let protected_before = serde_json::to_value(&history[1..4]).expect("serialize protected body");
    let contract = summary_contract();
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::MessageDelta {
                text: "earlier summary".to_string(),
            },
            NormalizedEvent::Usage {
                prompt_tokens: 8,
                completion_tokens: 2,
                total_tokens: 10,
                cached_tokens: None,
                cache_creation_tokens: None,
            },
            NormalizedEvent::Done,
        ],
        vec![
            NormalizedEvent::MessageDelta {
                text: "later summary".to_string(),
            },
            NormalizedEvent::Usage {
                prompt_tokens: 7,
                completion_tokens: 2,
                total_tokens: 9,
                cached_tokens: None,
                cache_creation_tokens: None,
            },
            NormalizedEvent::Done,
        ],
    ]));

    let summarized = summarize_marked_prose(
        &history,
        &[
            HostMarkedProseSpan::new("earlier-prose", 0, 1),
            HostMarkedProseSpan::new("later-prose", 4, 5),
        ],
        driver.as_ref(),
        SYNTHETIC_EXACT_MODEL,
        &contract,
        64,
    )
    .await
    .expect("explicit eligible prose summarizes");

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    let request_messages = requests
        .iter()
        .map(|request| &request.messages)
        .collect::<Vec<_>>();
    let serialized_requests = serde_json::to_string(&request_messages).expect("serialize requests");
    assert!(serialized_requests.contains("eligible earlier narrative"));
    assert!(serialized_requests.contains("eligible later narrative"));
    for protected in [arguments, result_body, evidence, "summary-protected-call"] {
        assert!(
            !serialized_requests.contains(protected),
            "protected body entered summarizer input: {protected}"
        );
    }
    assert_eq!(summarized[0].role, MessageRole::User);
    assert_eq!(summarized[0].content.as_text(), Some("earlier summary"));
    assert_eq!(
        serde_json::to_value(&summarized[1..4]).expect("serialize retained protected body"),
        protected_before
    );
    assert_eq!(summarized[4].role, MessageRole::Assistant);
    assert_eq!(summarized[4].content.as_text(), Some("later summary"));
}

/// Scenario: Failed summary.
#[tokio::test]
async fn failed_cancelled_empty_and_over_budget_summaries_retain_originals() {
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    use futures::Stream;
    use universal_agent_runtime::llm::{LlmDriver, LlmRequest, mock_driver::MockLlmDriver};
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::context::summarizer::{
        HostMarkedProseSpan, summarize_marked_prose,
    };

    let history = vec![
        text(MessageRole::User, "original eligible prose"),
        assistant_with_calls(&["retain-call"]),
        tool_result("retain-call", "retain-result"),
    ];
    let original = serde_json::to_value(&history).expect("serialize originals");
    let mark = [HostMarkedProseSpan::new("eligible", 0, 1)];
    let contract = summary_contract();

    let incomplete = MockLlmDriver::new(vec![vec![NormalizedEvent::MessageDelta {
        text: "partial".to_string(),
    }]]);
    let incomplete_error = summarize_marked_prose(
        &history,
        &mark,
        &incomplete,
        SYNTHETIC_EXACT_MODEL,
        &contract,
        32,
    )
    .await
    .expect_err("missing Done is a failed summary");
    assert!(format!("{incomplete_error:#}").contains("without Done"));

    let empty = MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "   ".to_string(),
        },
        NormalizedEvent::Done,
    ]]);
    let empty_error = summarize_marked_prose(
        &history,
        &mark,
        &empty,
        SYNTHETIC_EXACT_MODEL,
        &contract,
        32,
    )
    .await
    .expect_err("blank summary is rejected");
    assert!(format!("{empty_error:#}").contains("empty result"));

    let oversized = MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "summary output that is deliberately much longer than one token".to_string(),
        },
        NormalizedEvent::Done,
    ]]);
    let oversized_error = summarize_marked_prose(
        &history,
        &mark,
        &oversized,
        SYNTHETIC_EXACT_MODEL,
        &contract,
        7,
    )
    .await
    .expect_err("over-budget summary is rejected");
    assert!(format!("{oversized_error:#}").contains("exceeded its output budget"));

    let two_spans = vec![
        text(MessageRole::User, "first eligible prose"),
        text(MessageRole::Assistant, "second eligible prose"),
    ];
    let two_spans_before = serde_json::to_value(&two_spans).expect("serialize two spans");
    let aggregate_budget = MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "ok".to_string(),
        },
        NormalizedEvent::Done,
    ]]);
    let aggregate_error = summarize_marked_prose(
        &two_spans,
        &[
            HostMarkedProseSpan::new("first", 0, 1),
            HostMarkedProseSpan::new("second", 1, 2),
        ],
        &aggregate_budget,
        SYNTHETIC_EXACT_MODEL,
        &contract,
        7,
    )
    .await
    .expect_err("summary budget is shared across spans");
    assert!(format!("{aggregate_error:#}").contains("replacement framing"));
    assert_eq!(aggregate_budget.call_count(), 1);
    assert_eq!(serde_json::to_value(&two_spans).unwrap(), two_spans_before);

    #[derive(Debug, Default)]
    struct CancelledDriver {
        requests: Mutex<Vec<LlmRequest>>,
    }

    #[async_trait::async_trait]
    impl LlmDriver for CancelledDriver {
        async fn stream(
            &self,
            request: LlmRequest,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
        {
            self.requests.lock().expect("cancel requests").push(request);
            Ok(Box::pin(futures::stream::iter([Err(anyhow::anyhow!(
                "Model stream cancelled with its root run"
            ))])))
        }
    }

    let cancelled = Arc::new(CancelledDriver::default());
    let cancelled_error = summarize_marked_prose(
        &history,
        &mark,
        cancelled.as_ref(),
        SYNTHETIC_EXACT_MODEL,
        &contract,
        32,
    )
    .await
    .expect_err("cancelled summary is rejected");
    assert!(format!("{cancelled_error:#}").contains("cancelled"));

    let protected_mark = [HostMarkedProseSpan::new("tool-call", 1, 2)];
    let validation_driver = MockLlmDriver::echo();
    assert!(
        summarize_marked_prose(
            &history,
            &protected_mark,
            &validation_driver,
            SYNTHETIC_EXACT_MODEL,
            &contract,
            32,
        )
        .await
        .expect_err("a host marker cannot override tool protection")
        .to_string()
        .contains("protected tool data")
    );
    assert_eq!(validation_driver.call_count(), 0);
    assert_eq!(serde_json::to_value(&history).unwrap(), original);
}

#[tokio::test]
async fn oversized_summary_chunk_is_rejected_before_provider_dispatch() {
    use universal_agent_runtime::llm::LiterLlmDriver;
    use universal_agent_runtime::uar::runtime::context::summarizer::{
        HostMarkedProseSpan, summarize_marked_prose,
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind summary no-dispatch provider");
    let base_url = format!(
        "http://{}/v1",
        listener.local_addr().expect("summary listener address")
    );
    let driver = LiterLlmDriver::new(
        liter_llm::ClientConfigBuilder::new("fixture-key")
            .base_url(base_url.clone())
            .max_retries(0)
            .build(),
        SYNTHETIC_EXACT_MODEL.to_string(),
        None,
    )
    .expect("build summary no-dispatch driver");
    let mut contract = RequestBudgetContract::synthetic_exact(&base_url);
    contract.limits.context_tokens = 64;
    contract.limits.independent_input_tokens = Some(32);
    contract.limits.output_tokens = 8;
    let history = vec![text(MessageRole::User, &"host approved prose ".repeat(200))];

    let error = summarize_marked_prose(
        &history,
        &[HostMarkedProseSpan::new("oversized-prose", 0, 1)],
        &driver,
        SYNTHETIC_EXACT_MODEL,
        &contract,
        14,
    )
    .await
    .expect_err("oversized summary chunk must fail closed");
    assert!(
        format!("{error:#}").contains("proven final-wire allowance"),
        "unexpected error: {error:#}"
    );
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
            .await
            .is_err(),
        "rejected summary chunk must not open a provider connection"
    );
}

#[tokio::test]
async fn failed_summary_returns_explicit_overflow_when_originals_do_not_fit() {
    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::{
        ReduceHistoryError, reduce_history_with_marked_prose,
    };
    use universal_agent_runtime::uar::runtime::context::summarizer::HostMarkedProseSpan;

    let messages = vec![
        text(MessageRole::System, "required system"),
        text(MessageRole::User, &"original eligible prose ".repeat(100)),
    ];
    let original = serde_json::to_value(&messages).expect("serialize overflow originals");
    let driver = MockLlmDriver::new(vec![vec![NormalizedEvent::MessageDelta {
        text: "incomplete".to_string(),
    }]]);
    let contract = summary_contract();

    let error = reduce_history_with_marked_prose(
        messages.clone(),
        &ContextStrategy::Summarize {
            threshold: 0,
            summary_max_tokens: 32,
            model: None,
        },
        SYNTHETIC_EXACT_MODEL,
        1_010,
        Some(&driver),
        Some(&contract),
        &[HostMarkedProseSpan::new("older-prose", 0, 1)],
    )
    .await
    .expect_err("failed summary with oversized originals must return overflow");
    assert!(matches!(
        error,
        ReduceHistoryError::ContextOverflow {
            input_allowance: 10,
            ..
        }
    ));
    assert_eq!(serde_json::to_value(messages).unwrap(), original);
}

fn assistant_with_calls(ids: &[&str]) -> Message {
    Message {
        role: MessageRole::Assistant,
        content: MessageContent::text(""),
        tool_call_id: None,
        tool_calls: Some(
            ids.iter()
                .map(|id| ToolCall {
                    id: (*id).to_string(),
                    call_type: "function".to_string(),
                    function: ToolCallFunction {
                        name: "echo".to_string(),
                        arguments: "{}".to_string(),
                    },
                })
                .collect(),
        ),
    }
}

fn tool_result(call_id: &str, s: &str) -> Message {
    Message {
        role: MessageRole::Tool,
        content: MessageContent::text(s),
        tool_call_id: Some(call_id.to_string()),
        tool_calls: None,
    }
}

struct CaptureResumeNode {
    sender: tokio::sync::mpsc::UnboundedSender<
        universal_agent_runtime::uar::runtime::graph::GraphState,
    >,
    executions: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait::async_trait]
impl universal_agent_runtime::uar::runtime::graph::GraphNode for CaptureResumeNode {
    fn id(&self) -> &str {
        "capture"
    }

    async fn execute(
        &self,
        state: universal_agent_runtime::uar::runtime::graph::GraphState,
        _ctx: &universal_agent_runtime::uar::runtime::graph::GraphContext,
    ) -> universal_agent_runtime::uar::runtime::graph::NodeResult {
        self.executions
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let _ = self.sender.send(state.clone());
        universal_agent_runtime::uar::runtime::graph::NodeResult::Finished(state)
    }
}

/// Scenario: Missing result does not prove cancellation.
#[test]
fn missing_tool_result_blocks_without_mutating_history() {
    let history = vec![
        text(MessageRole::User, "run two tools"),
        assistant_with_calls(&["c1", "c2"]),
        tool_result("c1", "ok"),
    ];
    let before = serde_json::to_value(&history).expect("serialize canonical history");

    assert_eq!(
        normalize_history(&history),
        Err(HistoryValidationError::MissingResult {
            call_id: "c2".to_string(),
        })
    );
    assert_eq!(serde_json::to_value(&history).unwrap(), before);
}

/// Scenario: Orphaned result blocks without being discarded.
#[test]
fn orphaned_tool_result_blocks_without_mutating_history() {
    let history = vec![
        text(MessageRole::User, "hello"),
        assistant_with_calls(&["c1"]),
        tool_result("c1", "ok"),
        tool_result("ghost", "no call produced me"),
        text(MessageRole::Assistant, "done"),
    ];
    let before = serde_json::to_value(&history).expect("serialize canonical history");

    assert_eq!(
        normalize_history(&history),
        Err(HistoryValidationError::OrphanResult {
            call_id: "ghost".to_string(),
        })
    );
    assert_eq!(serde_json::to_value(&history).unwrap(), before);
}

#[test]
fn misplaced_tool_result_blocks_without_mutating_history() {
    let history = vec![
        assistant_with_calls(&["c1"]),
        text(MessageRole::User, "intervening turn"),
        tool_result("c1", "late result"),
    ];
    let before = serde_json::to_value(&history).expect("serialize canonical history");

    assert_eq!(
        normalize_history(&history),
        Err(HistoryValidationError::MisplacedResult {
            call_id: "c1".to_string(),
        })
    );
    assert_eq!(serde_json::to_value(&history).unwrap(), before);
}

#[test]
fn verified_terminal_failure_can_be_recorded_explicitly() {
    let mut history = vec![assistant_with_calls(&["c1"])];
    assert!(matches!(
        normalize_history(&history),
        Err(HistoryValidationError::MissingResult { .. })
    ));

    history.push(synthetic_tool_result(
        "c1",
        &SyntheticReason::Error("durable receipt: process exited 137".to_string()),
    ));

    let report = normalize_history(&history).expect("host-proven terminal result is complete");
    assert_eq!(report.tool_calls, 1);
    assert_eq!(report.tool_results, 1);
    assert!(
        history[1]
            .content
            .as_text()
            .is_some_and(|body| body.contains("durable receipt"))
    );
}

#[tokio::test]
async fn direct_orchestrator_blocks_invalid_history_before_provider_dispatch() {
    use std::sync::Arc;

    use futures::StreamExt;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::{Orchestrator, mock_driver::MockLlmDriver};
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;

    let driver = Arc::new(MockLlmDriver::new(vec![vec![NormalizedEvent::Done]]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    );
    let history = vec![
        assistant_with_calls(&["c1"]),
        text(MessageRole::User, "intervening turn"),
        tool_result("c1", "late result"),
    ];

    let stream = orchestrator
        .chat_with_history(history)
        .await
        .expect("orchestrator accepts typed history");
    let events: Vec<_> = stream.collect().await;

    let requests = driver.requests();
    assert!(
        requests.is_empty(),
        "invalid history must not be dispatched"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::Error { code: Some(code), .. } if code == "INVALID_HISTORY"
    )));
}

#[tokio::test]
async fn iterative_tool_loop_dispatches_a_typed_terminal_error_result() {
    use std::sync::Arc;

    use futures::StreamExt;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::{Orchestrator, mock_driver::MockLlmDriver};
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::domain::policy::{PolicyResolutionInput, resolve_run_policy};
    use universal_agent_runtime::uar::persistence::{
        PersistenceLayer, providers::surreal::SurrealDbProvider,
    };
    use universal_agent_runtime::uar::runtime::context::budget::RequestBudgetContract;
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
    use universal_agent_runtime::uar::runtime::turn::{ResolvedTurn, TurnEnvironment};
    use universal_agent_runtime::uar::tools::terminal_exec::TerminalExecTool;

    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::ToolCallDelta {
                call_index: 0,
                id: Some("c1".to_string()),
                name: Some("terminal_exec".to_string()),
                arguments_delta: Some("{}".to_string()),
            },
            NormalizedEvent::ToolCallComplete {
                call_index: 0,
                id: "c1".to_string(),
                name: "terminal_exec".to_string(),
                arguments_json: "{}".to_string(),
            },
            NormalizedEvent::Done,
        ],
        vec![
            NormalizedEvent::MessageDelta {
                text: "finished".to_string(),
            },
            NormalizedEvent::Done,
        ],
    ]));
    let native_skills = Arc::new(NativeSkillRegistry::new());
    native_skills
        .register(TerminalExecTool {
            shell: "/bin/sh".to_string(),
            timeout_secs: 1,
            use_sandbox: false,
        })
        .await
        .expect("terminal descriptor registers");
    let receipt_tempdir = tempfile::tempdir().expect("terminal receipt tempdir");
    let receipt_url = format!("surrealkv://{}", receipt_tempdir.path().display());
    let store: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(&receipt_url, None, None, None, None)
            .await
            .expect("SurrealDB terminal receipt store"),
    );
    let turn = Arc::new(ResolvedTurn::new(
        universal_agent_runtime::uar::defaults::default_agent(),
        resolve_run_policy(PolicyResolutionInput::default()),
        TurnEnvironment {
            run_id: "terminal-failure-run".to_string(),
            owner_id: "terminal-failure-owner".to_string(),
            session_id: "terminal-failure-session".to_string(),
        },
        LlmConfig::default(),
        Vec::new(),
    ));
    let budget_contract =
        RequestBudgetContract::synthetic_exact("http://iteration-fixture.invalid/v1");
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native_skills,
        driver.clone(),
    )
    .with_request_budget_contract(budget_contract.clone())
    .with_resolved_turn(turn)
    .with_canonical_receipt_store(Some(Arc::clone(&store)));

    let stream = orchestrator
        .chat("run terminal without the required command")
        .await
        .expect("orchestrator starts");
    let _: Vec<_> = stream.collect().await;

    let requests = driver.requests();
    assert_eq!(
        requests.len(),
        2,
        "tool result must trigger another dispatch"
    );
    assert!(
        requests
            .iter()
            .all(|request| request.budget_contract.as_ref() == Some(&budget_contract)),
        "initial and iterative preparations must carry the host-resolved budget contract"
    );
    let second: Vec<Message> = requests[1]
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("second provider request remains typed");
    let assistant_index = second
        .iter()
        .position(|message| {
            message
                .tool_calls
                .iter()
                .flatten()
                .any(|call| call.id == "c1")
        })
        .expect("assistant call reaches second request");
    let result = second
        .get(assistant_index + 1)
        .expect("tool result immediately follows its call");
    assert_eq!(result.role, MessageRole::Tool);
    assert_eq!(result.tool_call_id.as_deref(), Some("c1"));
    let terminal: serde_json::Value = serde_json::from_str(
        result
            .content
            .as_text()
            .expect("typed terminal result has text serialization"),
    )
    .expect("terminal error result is typed JSON");
    assert_eq!(terminal["status"], "error");
    assert_eq!(terminal["tool_call_id"], "c1");
    assert_eq!(terminal["provenance"]["terminal_state"], "failed");
    assert_eq!(terminal["provenance"]["observed_by"], "trusted_host");
    let receipts = store
        .list_canonical_tool_receipts("terminal-failure-owner", "terminal-failure-run")
        .await
        .expect("read canonical terminal receipt");
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0]
            .typed_value
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("error")
    );
    assert_eq!(receipts[0].call_id, "c1");
}

#[tokio::test]
async fn run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch() {
    use std::sync::Arc;

    use tokio::sync::RwLock;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::session::SessionStore;
    use universal_agent_runtime::uar::defaults;
    use universal_agent_runtime::uar::domain::runs::RunStatus;
    use universal_agent_runtime::uar::rag::embeddings::UnavailableEmbeddingBackend;
    use universal_agent_runtime::uar::runtime::manager::RunManager;
    use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
    use universal_agent_runtime::uar::runtime::skills::SkillRegistry;

    let sessions = SessionStore::new();
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::ToolCallComplete {
            call_index: 0,
            id: "pending-1".to_string(),
            name: "pending_fixture".to_string(),
            arguments_json: "{\"value\":1}".to_string(),
        },
        NormalizedEvent::Error {
            message: "provider stream interrupted before execution".to_string(),
            code: Some("PROVIDER_INTERRUPTED".to_string()),
        },
    ]]));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            sessions.clone(),
            Arc::new(RwLock::new(SkillRegistry::new(None, None))),
            Arc::new(VectorMatcher::new(
                Arc::new(UnavailableEmbeddingBackend::new(
                    384,
                    "embeddings are not exercised by this test",
                )),
                0.75,
            )),
            None,
        )
        .await
        .with_llm_driver(driver.clone()),
    );

    let first_run = manager
        .start_run(
            defaults::default_agent(),
            "start pending tool".to_string(),
            Some("pending-session".to_string()),
            None,
            vec![],
        )
        .await;
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if manager
                .get_run(&first_run)
                .await
                .is_some_and(|run| run.status == RunStatus::Error)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("interrupted run reaches Error");

    let session = sessions
        .get("pending-session")
        .expect("pending session remains available");
    let persisted = session.messages();
    let pending = persisted
        .iter()
        .find(|message| {
            message
                .tool_calls
                .iter()
                .flatten()
                .any(|call| call.id == "pending-1")
        })
        .expect("pending assistant call is persisted");
    assert_eq!(pending.content.as_text(), Some(""));
    assert!(matches!(
        normalize_history(&persisted),
        Err(HistoryValidationError::MissingResult { call_id }) if call_id == "pending-1"
    ));

    let second_run = manager
        .start_run(
            defaults::default_agent(),
            "resume".to_string(),
            Some("pending-session".to_string()),
            None,
            vec![],
        )
        .await;
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if manager
                .get_run(&second_run)
                .await
                .is_some_and(|run| run.status == RunStatus::Error)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("resume blocks on invalid pending history");
    assert_eq!(
        driver.requests().len(),
        1,
        "resume with unresolved pending work must not dispatch"
    );
}

#[tokio::test]
async fn graph_llm_node_refuses_unhosted_provider_dispatch() {
    use std::sync::Arc;

    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::graph::{
        GraphContext, GraphNode, GraphState, LlmNode, NodeResult,
    };

    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "done".to_string(),
        },
        NormalizedEvent::Done,
    ]]));
    let context = GraphContext {
        run_id: "graph-history-normalization".to_string(),
        checkpoint_authorization_digest: "0".repeat(64),
        session_id: None,
        llm_config: LlmConfig::default(),
        driver: driver.clone(),
        cache_strategy: None,
        persistence: None,
        thread_delegate: None,
        tool_host: None,
    };
    let mut state = GraphState::default();
    state.messages = vec![
        assistant_with_calls(&["c1"]),
        text(MessageRole::User, "intervening turn"),
        tool_result("c1", "late result"),
    ]
    .iter()
    .map(serde_json::to_value)
    .collect::<Result<_, _>>()
    .expect("serialize graph history");

    let result = LlmNode::new("llm").execute(state, &context).await;

    let requests = driver.requests();
    assert!(requests.is_empty());
    assert!(matches!(
        result,
        NodeResult::Error(_, message) if message == "Graph model host is unavailable"
    ));
}

/// Scenario: Long conversation under a sliding window keeps the system message.
#[test]
fn sliding_window_keeps_system_message_pinned_at_index_zero() {
    use universal_agent_runtime::uar::context::{ContextStrategy, trim_history};

    let system = text(MessageRole::System, "You are the agent.");
    let history: Vec<Message> = (0..59)
        .map(|i| {
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };
            text(role, &format!("turn-{i}"))
        })
        .collect();

    let reduced = trim_history(
        Some(system),
        history,
        &ContextStrategy::SlidingWindow { max_messages: 20 },
    );

    assert_eq!(reduced.len(), 21, "system message plus a 20-turn window");
    assert_eq!(reduced[0].role, MessageRole::System);
    assert_eq!(reduced[0].content.as_text(), Some("You are the agent."));
    assert_eq!(reduced[1].content.as_text(), Some("turn-39"));
    assert_eq!(reduced[20].content.as_text(), Some("turn-58"));
    assert_eq!(
        reduced
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .count(),
        1,
        "the system message is never duplicated by the reducer"
    );
}

/// Scenario: User repeats "continue" and both turns survive the token-budget reducer.
#[tokio::test]
async fn identical_repeated_user_messages_survive_keep_first_last() {
    use universal_agent_runtime::uar::domain::context::{ContextConfig, ContextStrategy};
    use universal_agent_runtime::uar::runtime::context::manager::ContextManager;

    let config = ContextConfig {
        strategy: ContextStrategy::KeepFirstLast,
        max_tokens: Some(120),
        trigger_threshold: 0.1,
        ..ContextConfig::default()
    };
    let manager = ContextManager::new(config);

    let mut messages = vec![
        text(MessageRole::System, "System"),
        text(MessageRole::User, "start the task"),
    ];
    for i in 0..30 {
        messages.push(text(
            MessageRole::Assistant,
            &format!("filler assistant message number {i} with some words"),
        ));
    }
    messages.push(text(MessageRole::User, "continue"));
    messages.push(text(MessageRole::User, "continue"));

    let (reduced, action) = manager.apply(messages, 1_000).await;

    assert!(
        action.is_some(),
        "the budget was exceeded so a reduction ran"
    );
    let n = reduced.len();
    assert!(n >= 4, "system, first user, and the two repeats survive");
    assert_eq!(reduced[0].role, MessageRole::System);
    assert_eq!(reduced[n - 1].content.as_text(), Some("continue"));
    assert_eq!(reduced[n - 2].content.as_text(), Some("continue"));
    assert_eq!(
        reduced
            .iter()
            .filter(|m| m.content.as_text() == Some("continue"))
            .count(),
        2,
        "identical consecutive user turns are not deduplicated"
    );
}

#[tokio::test]
async fn reduction_returns_overflow_instead_of_dropping_a_tool_group() {
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::{
        ReduceHistoryError, reduce_history,
    };

    let history = vec![
        text(MessageRole::System, "system"),
        text(MessageRole::User, "old turn"),
        assistant_with_calls(&["c1"]),
        tool_result("c1", "result"),
        text(MessageRole::User, "new-1"),
        text(MessageRole::Assistant, "new-2"),
        text(MessageRole::User, "new-3"),
        text(MessageRole::Assistant, "new-4"),
    ];

    let error = reduce_history(
        history,
        &ContextStrategy::SlidingWindow { max_messages: 5 },
        "openai/gpt-4o",
        8_192,
        None,
    )
    .await
    .expect_err("protected group cannot be dropped to satisfy a window");

    assert_eq!(
        error,
        ReduceHistoryError::ProtectedHistoryOverflow {
            protected_records: vec!["c1".to_string()],
        }
    );
}

#[tokio::test]
async fn empty_text_multimodal_assistant_returns_overflow_instead_of_being_dropped() {
    use universal_agent_runtime::llm::ContentPart;
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::{
        ReduceHistoryError, reduce_history,
    };

    let history = vec![
        text(MessageRole::System, "system"),
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::parts(vec![
                ContentPart::text(""),
                ContentPart::image_url("data:image/png;base64,AA=="),
            ]),
            tool_call_id: None,
            tool_calls: None,
        },
        text(MessageRole::User, "new-1"),
        text(MessageRole::Assistant, "new-2"),
        text(MessageRole::User, "new-3"),
    ];

    let error = reduce_history(
        history,
        &ContextStrategy::SlidingWindow { max_messages: 2 },
        "openai/gpt-4o",
        8_192,
        None,
    )
    .await
    .expect_err("multimodal assistant record is protected");

    assert_eq!(
        error,
        ReduceHistoryError::ProtectedHistoryOverflow {
            protected_records: vec!["assistant_history_index_1".to_string()],
        }
    );
}

#[tokio::test]
async fn parallel_group_and_repeated_turns_survive_in_original_order() {
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::reduce_history;

    let history = vec![
        text(MessageRole::System, "system"),
        text(MessageRole::User, "continue"),
        assistant_with_calls(&["c1", "c2"]),
        tool_result("c2", "second payload"),
        tool_result("c1", "first payload"),
        text(MessageRole::User, "continue"),
    ];
    let before = serde_json::to_value(&history).expect("serialize canonical history");

    let (reduced, report) = reduce_history(
        history,
        &ContextStrategy::None,
        "openai/gpt-4o",
        8_192,
        None,
    )
    .await
    .expect("protected history fits");

    assert_eq!(serde_json::to_value(&reduced).unwrap(), before);
    assert_eq!(report.normalize.tool_calls, 2);
    assert_eq!(report.normalize.tool_results, 2);
    assert_eq!(
        reduced
            .iter()
            .filter(|message| message.content.as_text() == Some("continue"))
            .count(),
        2
    );
}

#[test]
fn empty_text_parallel_assistant_persists_and_restores_between_repeated_turns() {
    use universal_agent_runtime::session::{Session, SessionStore};

    let store = SessionStore::new();
    let session = store.create();
    let calls = assistant_with_calls(&["c1", "c2"])
        .tool_calls
        .expect("fixture has calls");
    session.add_user_message("continue");
    session.add_assistant_with_tool_calls(None, calls);
    session.add_tool_result("c1", "first");
    session.add_tool_result("c2", "second");
    session.add_user_message("continue");

    let encoded = serde_json::to_string(&session).expect("serialize session");
    let restored: Session = serde_json::from_str(&encoded).expect("restore session");
    let messages = restored.messages();

    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].content.as_text(), Some("continue"));
    assert_eq!(messages[1].role, MessageRole::Assistant);
    assert_eq!(messages[1].content.as_text(), Some(""));
    assert_eq!(messages[1].tool_calls.as_ref().map(Vec::len), Some(2));
    assert_eq!(messages[4].content.as_text(), Some("continue"));
    normalize_history(&messages).expect("restored history remains provider-valid");
}

#[tokio::test]
#[ignore = "frozen-source diagnostic; task 2.2 replaces destructive group dropping"]
async fn baseline_sliding_window_drops_the_protected_tool_group() {
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::reduce_history;

    let mut history = vec![
        harness_context::text(MessageRole::System, "system"),
        harness_context::text(MessageRole::User, "old turn"),
    ];
    history.extend(harness_context::protected_tool_group());
    history.extend([
        harness_context::text(MessageRole::User, "new-1"),
        harness_context::text(MessageRole::Assistant, "new-2"),
        harness_context::text(MessageRole::User, "new-3"),
        harness_context::text(MessageRole::Assistant, "new-4"),
    ]);

    let (reduced, _) = reduce_history(
        history,
        &ContextStrategy::SlidingWindow { max_messages: 5 },
        "openai/gpt-4o",
        8_192,
        None,
    )
    .await
    .expect("frozen baseline expected reduction to succeed");

    assert!(
        !harness_context::contains_complete_tool_group(&reduced),
        "the frozen source is expected to drop the protected group; if this fails, recapture the baseline rather than updating it silently"
    );
}

#[test]
#[ignore = "frozen-source diagnostic; task 2.1 introduces canonical pre-format receipts"]
fn baseline_display_truncation_discards_protected_middle_bytes() {
    use universal_agent_runtime::uar::runtime::context::truncate::{
        TruncationPolicy, formatted_truncate_for_model,
    };

    let raw = harness_context::protected_payload();
    assert!(raw.as_bytes().windows(2).any(|bytes| bytes == b"\r\n"));

    let formatted =
        formatted_truncate_for_model(&raw, TruncationPolicy::Bytes(4_096), "openai/gpt-4o");

    assert_ne!(formatted.as_bytes(), raw.as_bytes());
    assert!(!formatted.contains(harness_context::PAYLOAD_MIDDLE));
}

#[test]
fn canonical_receipt_preserves_value_and_raw_bytes_before_display_formatting() {
    use universal_agent_runtime::uar::persistence::agent_threads::{
        CanonicalReceiptSource, CanonicalToolReceipt,
    };
    use universal_agent_runtime::uar::runtime::context::truncate::{
        TruncationPolicy, formatted_truncate_for_model,
    };

    let raw = harness_context::protected_payload();
    let value = serde_json::json!({"payload": raw});
    let receipt = CanonicalToolReceipt::acquire(
        "owner-1",
        "run-1",
        1,
        harness_context::TOOL_CALL_ID,
        "fixture_tool",
        CanonicalReceiptSource::Terminal,
        value.clone(),
        [("stdout".to_owned(), raw.as_bytes().to_vec())],
        true,
        raw.len() as u64,
    )
    .expect("canonical receipt acquisition");
    let displayed = formatted_truncate_for_model(
        &serde_json::to_string(&value).unwrap(),
        TruncationPolicy::Bytes(4_096),
        "openai/gpt-4o",
    );

    assert!(!displayed.contains(harness_context::PAYLOAD_MIDDLE));
    assert_eq!(receipt.typed_value, Some(value));
    assert_eq!(
        receipt.raw_segments[0]
            .verified_bytes()
            .expect("raw identity validates"),
        Some(raw.into_bytes())
    );
}

#[test]
fn canonical_receipt_over_limit_is_explicitly_incomplete_without_partial_bytes() {
    use universal_agent_runtime::uar::persistence::agent_threads::{
        CanonicalReceiptCompleteness, CanonicalReceiptSource, CanonicalToolReceipt,
        MAX_CANONICAL_RECEIPT_BYTES,
    };

    let raw = vec![b'x'; MAX_CANONICAL_RECEIPT_BYTES as usize];
    let receipt = CanonicalToolReceipt::acquire(
        "owner-1",
        "run-1",
        2,
        "oversized-call",
        "fixture_tool",
        CanonicalReceiptSource::Mcp,
        serde_json::json!({}),
        [("result".to_owned(), raw)],
        true,
        MAX_CANONICAL_RECEIPT_BYTES,
    )
    .expect("incomplete receipt metadata is valid");

    assert!(matches!(
        receipt.completeness,
        CanonicalReceiptCompleteness::AcquisitionLimitExceeded { observed_bytes }
            if observed_bytes > MAX_CANONICAL_RECEIPT_BYTES
    ));
    assert!(receipt.typed_value.is_none());
    assert_eq!(receipt.retained_bytes, 0);
    assert!(receipt.raw_segments[0].bytes_base64.is_none());
    assert_eq!(
        receipt.raw_segments[0]
            .verified_bytes()
            .expect("metadata-only segment validates"),
        None
    );
}

#[tokio::test]
async fn final_request_capture_decodes_empty_text_assistant_tool_turn() {
    use std::sync::Arc;

    use futures::StreamExt;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::{Orchestrator, mock_driver::MockLlmDriver};
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;

    let driver = Arc::new(MockLlmDriver::new(vec![vec![NormalizedEvent::Done]]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    );
    let mut history = vec![harness_context::text(MessageRole::User, "run fixture")];
    history.extend(harness_context::protected_tool_group());

    let stream = orchestrator
        .chat_with_history(history)
        .await
        .expect("orchestrator accepts typed fixture history");
    let _: Vec<_> = stream.collect().await;

    let requests = driver.requests();
    let captured = harness_context::decode_request_messages(
        requests.first().expect("one provider request is captured"),
    );
    assert!(harness_context::contains_complete_tool_group(&captured));
    let assistant = captured
        .iter()
        .find(|message| message.tool_calls.is_some())
        .expect("empty-text assistant tool turn is retained");
    assert_eq!(assistant.content.as_text(), Some(""));
}

#[tokio::test]
async fn exact_budget_contract_dispatches_intact_protected_wire_body() {
    use futures::StreamExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use universal_agent_runtime::llm::{LiterLlmDriver, LlmDriver, LlmRequest};
    use universal_agent_runtime::uar::runtime::context::budget::{
        RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
    };
    use universal_agent_runtime::uar::runtime::context::token_service::TokenService;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind synthetic exact provider");
    let address = listener.local_addr().expect("synthetic provider address");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept budgeted request");
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 2048];
        let (header_end, content_length) = loop {
            let read = socket.read(&mut buffer).await.expect("read request");
            assert!(read > 0, "request ended before complete headers");
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                let header_end = header_end + 4;
                let headers = std::str::from_utf8(&bytes[..header_end]).expect("UTF-8 headers");
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().expect("content length"))
                    })
                    .expect("content-length header");
                break (header_end, content_length);
            }
        };
        while bytes.len() < header_end + content_length {
            let read = socket.read(&mut buffer).await.expect("read request body");
            assert!(read > 0, "request ended before complete body");
            bytes.extend_from_slice(&buffer[..read]);
        }
        let body_bytes = bytes[header_end..header_end + content_length].to_vec();
        let body: serde_json::Value =
            serde_json::from_slice(&body_bytes).expect("captured final JSON body");
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: 14\r\nConnection: close\r\n\r\ndata: [DONE]\n\n",
            )
            .await
            .expect("write synthetic response");
        (body, body_bytes)
    });

    let base_url = format!("http://{address}/v1");
    let config = liter_llm::ClientConfigBuilder::new("fixture-key")
        .base_url(base_url.clone())
        .max_retries(0)
        .build();
    let driver = LiterLlmDriver::new(config, SYNTHETIC_EXACT_MODEL.to_string(), Some(true))
        .expect("build synthetic exact driver");
    let arguments = "{\"path\":\"C:\\\\tmp\\\\payload\",\"lines\":\"a\\r\\nb\"}";
    let result = "{\"status\":\"ok\",\"bytes\":[0,13,10,255]}";
    let request = LlmRequest {
        messages: vec![
            serde_json::json!({"role":"user","content":"preserve the receipt"}),
            serde_json::json!({
                "role":"assistant",
                "content":"",
                "tool_calls":[{
                    "id":"call-budget-1",
                    "type":"function",
                    "function":{"name":"read_fixture","arguments":arguments}
                }]
            }),
            serde_json::json!({
                "role":"tool",
                "tool_call_id":"call-budget-1",
                "content":result
            }),
        ],
        tools: vec![serde_json::json!({
            "type":"function",
            "function":{
                "name":"read_fixture",
                "description":"fixture",
                "parameters":{"type":"object","properties":{}}
            }
        })],
        cache_strategy: None,
        thinking_config: None,
        anthropic_system: None,
        extra_params: Some(serde_json::json!({
            "metadata":{"fixture":"exact"}
        })),
        budget_contract: Some(RequestBudgetContract::synthetic_exact(&base_url)),
    };
    let mut stream = driver
        .stream(request)
        .await
        .expect("exact protected request fits and dispatches");
    while stream.next().await.is_some() {}

    let (body, body_bytes) = server.await.expect("synthetic provider task");
    assert_eq!(body["model"], SYNTHETIC_EXACT_MODEL);
    assert_eq!(body["max_completion_tokens"], 512);
    assert!(body.get("max_tokens").is_none());
    assert_eq!(body["stream"], true);
    assert_eq!(body["metadata"]["fixture"], "exact");
    assert_eq!(body["messages"][1]["tool_calls"][0]["id"], "call-budget-1");
    assert_eq!(
        body["messages"][1]["tool_calls"][0]["function"]["arguments"],
        arguments
    );
    assert_eq!(body["messages"][2]["content"], result);
    let count = TokenService::count_serialized(
        RequestBudgetContract::synthetic_exact(&base_url).counting,
        &body_bytes,
    )
    .expect("captured wire body is countable");
    assert!(count.tokens <= 7_680, "captured final body fits input cap");
}

#[test]
fn pure_budget_planner_returns_fit_compression_overflow_invalid_and_unsupported() {
    use universal_agent_runtime::uar::runtime::context::budget::{
        BudgetInput, BudgetOutcome, DestinationLimits, EligibleProseSpan, plan_budget,
    };
    use universal_agent_runtime::uar::runtime::context::token_service::{
        CountQuality, CountedTokens,
    };

    let exact = |tokens| CountedTokens {
        tokens,
        quality: CountQuality::Exact,
        revision: "fixture-v1".to_string(),
    };
    let base = BudgetInput {
        limits: DestinationLimits {
            context_tokens: 10_000,
            independent_input_tokens: Some(6_000),
            host_input_tokens: Some(7_000),
            output_tokens: 2_000,
            additional_reasoning_tokens: 0,
            count_uncertainty_tokens: 100,
        },
        protected_request: exact(5_000),
        eligible_prose: Vec::new(),
    };
    let unchanged = base.clone();
    let BudgetOutcome::Fit(fit) = plan_budget(&base) else {
        panic!("formula fixture must fit");
    };
    assert_eq!(fit.input_allowance, 6_000);
    assert_eq!(fit.eligible_budget, 1_000);
    assert_eq!(base, unchanged, "pure planning does not mutate its input");

    let mut compression = base.clone();
    compression.eligible_prose.push(EligibleProseSpan {
        id: "host-prose-1".to_string(),
        count: exact(1_001),
    });
    assert!(matches!(
        plan_budget(&compression),
        BudgetOutcome::CompressEligible {
            target_tokens: 1_000,
            ..
        }
    ));

    let mut overflow = base.clone();
    overflow.protected_request = exact(6_001);
    assert_eq!(
        plan_budget(&overflow),
        BudgetOutcome::ProtectedOverflow {
            required_tokens: 6_001,
            input_allowance: 6_000,
        }
    );

    let mut invalid = base.clone();
    invalid.limits.additional_reasoning_tokens = -1;
    assert_eq!(
        plan_budget(&invalid),
        BudgetOutcome::InvalidLimits {
            field: "additional_reasoning_tokens",
            value: -1,
        }
    );

    let mut unsupported = base;
    unsupported.protected_request.quality = CountQuality::Approximate;
    assert!(matches!(
        plan_budget(&unsupported),
        BudgetOutcome::UnsupportedCount { .. }
    ));
}

#[tokio::test]
async fn protected_overflow_and_approximate_count_block_before_dispatch() {
    use universal_agent_runtime::llm::{LiterLlmDriver, LlmDriver, LlmRequest};
    use universal_agent_runtime::uar::runtime::context::budget::{
        RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
    };
    use universal_agent_runtime::uar::runtime::context::token_service::SerializedCountingContract;

    async fn blocked_request(mut contract: RequestBudgetContract, expected: &str) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind no-dispatch provider");
        let address = listener.local_addr().expect("no-dispatch address");
        let base_url = format!("http://{address}/v1");
        contract.destination_endpoint_fingerprint =
            universal_agent_runtime::uar::runtime::context::budget::endpoint_fingerprint(&base_url);
        let config = liter_llm::ClientConfigBuilder::new("fixture-key")
            .base_url(base_url)
            .max_retries(0)
            .build();
        let driver = LiterLlmDriver::new(config, SYNTHETIC_EXACT_MODEL.to_string(), None)
            .expect("build no-dispatch driver");
        contract.destination_model = SYNTHETIC_EXACT_MODEL.to_string();
        let protected = "protected-tool-result-".repeat(200);
        let error = match driver
            .stream(LlmRequest {
                messages: vec![serde_json::json!({
                    "role":"tool",
                    "tool_call_id":"call-overflow",
                    "content":protected
                })],
                tools: Vec::new(),
                cache_strategy: None,
                thinking_config: None,
                anthropic_system: None,
                extra_params: None,
                budget_contract: Some(contract),
            })
            .await
        {
            Ok(_) => panic!("unprovable request must be rejected"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains(expected),
            "unexpected rejection: {error:#}"
        );
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
                .await
                .is_err(),
            "rejected request must not open a provider connection"
        );
    }

    let mut overflow = RequestBudgetContract::synthetic_exact("http://fixture.invalid/v1");
    overflow.limits.context_tokens = 32;
    overflow.limits.independent_input_tokens = Some(16);
    overflow.limits.output_tokens = 8;
    blocked_request(overflow, "ProtectedOverflow").await;

    let mut approximate = RequestBudgetContract::synthetic_exact("http://fixture.invalid/v1");
    approximate.counting = SerializedCountingContract::ApproximateCl100kJsonV1;
    blocked_request(approximate, "UnsupportedCount").await;
}

#[tokio::test]
async fn budget_contracts_fail_closed_on_endpoint_schema_and_unsupported_driver_boundaries() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use universal_agent_runtime::llm::{
        ExternalDriverHandler, ExternalDriverStream, ExternalLlmDriver, LiterLlmDriver, LlmDriver,
        LlmRequest, anthropic_driver::AnthropicDriver,
    };
    use universal_agent_runtime::uar::runtime::context::budget::{
        RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
    };

    fn request(contract: RequestBudgetContract) -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role":"user","content":"protected"})],
            tools: Vec::new(),
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
            budget_contract: Some(contract),
        }
    }

    async fn assert_no_dispatch(listener: &tokio::net::TcpListener) {
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
                .await
                .is_err(),
            "rejected request must not open a provider connection"
        );
    }

    let endpoint_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind endpoint mismatch provider");
    let endpoint_url = format!(
        "http://{}/v1",
        endpoint_listener
            .local_addr()
            .expect("endpoint mismatch address")
    );
    let endpoint_driver = LiterLlmDriver::new(
        liter_llm::ClientConfigBuilder::new("fixture-key")
            .base_url(endpoint_url)
            .max_retries(0)
            .build(),
        SYNTHETIC_EXACT_MODEL.to_string(),
        None,
    )
    .expect("build endpoint mismatch driver");
    let endpoint_error = match endpoint_driver
        .stream(request(RequestBudgetContract::synthetic_exact(
            "http://different.invalid/v1",
        )))
        .await
    {
        Ok(_) => panic!("endpoint mismatch must fail before dispatch"),
        Err(error) => error,
    };
    assert!(endpoint_error.to_string().contains("endpoint mismatch"));
    assert_no_dispatch(&endpoint_listener).await;

    let schema_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind schema rejection provider");
    let schema_url = format!(
        "http://{}/v1",
        schema_listener
            .local_addr()
            .expect("schema provider address")
    );
    let schema_driver = LiterLlmDriver::new(
        liter_llm::ClientConfigBuilder::new("fixture-key")
            .base_url(schema_url.clone())
            .max_retries(0)
            .build(),
        SYNTHETIC_EXACT_MODEL.to_string(),
        None,
    )
    .expect("build schema rejection driver");
    let mut schema_request = request(RequestBudgetContract::synthetic_exact(&schema_url));
    schema_request.tools.push(serde_json::json!({
        "type":"function",
        "function":{"name":17,"parameters":{"type":"object"}}
    }));
    let schema_error = match schema_driver.stream(schema_request).await {
        Ok(_) => panic!("malformed protected tool schema must fail before dispatch"),
        Err(error) => error,
    };
    assert!(schema_error.to_string().contains("tool schema at index 0"));
    assert_no_dispatch(&schema_listener).await;

    let mut reserved_request = request(RequestBudgetContract::synthetic_exact(&schema_url));
    reserved_request.extra_params = Some(serde_json::json!({"messages":[]}));
    let reserved_error = match schema_driver.stream(reserved_request).await {
        Ok(_) => panic!("reserved extra-body collision must fail before dispatch"),
        Err(error) => error,
    };
    assert!(
        reserved_error
            .to_string()
            .contains("reserved field `messages`")
    );
    assert_no_dispatch(&schema_listener).await;

    let mut lossy_message = request(RequestBudgetContract::synthetic_exact(&schema_url));
    lossy_message.messages[0]["protected_metadata"] = serde_json::json!({"receipt_id":"receipt-1"});
    let lossy_error = match schema_driver.stream(lossy_message).await {
        Ok(_) => panic!("lossy message conversion must fail before dispatch"),
        Err(error) => error,
    };
    assert!(lossy_error.to_string().contains("discard protected fields"));
    assert_no_dispatch(&schema_listener).await;

    let external_calls = Arc::new(AtomicUsize::new(0));
    let handler_calls = Arc::clone(&external_calls);
    let handler: ExternalDriverHandler = Arc::new(move |_| {
        handler_calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(Box::pin(futures::stream::empty()) as ExternalDriverStream) })
    });
    let external = ExternalLlmDriver::new("unsupported-budget-fixture", handler);
    let unsupported_contract =
        RequestBudgetContract::synthetic_exact("http://unsupported.invalid/v1");
    assert!(
        external
            .stream(request(unsupported_contract.clone()))
            .await
            .is_err(),
        "external leaf must reject an unsupported budget contract"
    );
    assert_eq!(external_calls.load(Ordering::SeqCst), 0);

    let anthropic = AnthropicDriver::new(
        "fixture-key".to_string(),
        "claude-fixture".to_string(),
        Some("http://127.0.0.1:9".to_string()),
        None,
        None,
        None,
    );
    let anthropic_error = match anthropic.stream(request(unsupported_contract)).await {
        Ok(_) => panic!("Anthropic leaf must reject an unsupported budget contract"),
        Err(error) => error,
    };
    assert!(
        anthropic_error
            .to_string()
            .contains("no supported final-wire budget contract")
    );
}

#[tokio::test]
async fn orchestrator_preserves_budget_contract_on_primary_and_failover_preparation() {
    use std::pin::Pin;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use futures::{Stream, StreamExt};
    use universal_agent_runtime::config::{FailoverConfig, LlmConfig};
    use universal_agent_runtime::llm::{
        LlmDriver, LlmRequest, Orchestrator, ProviderError, mock_driver::MockLlmDriver,
    };
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::context::budget::RequestBudgetContract;
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
    use universal_agent_runtime::uar::settings::resilience_policy::ResiliencePolicy;

    #[derive(Debug, Default)]
    struct FailingCaptureDriver {
        requests: Mutex<Vec<LlmRequest>>,
    }

    #[async_trait::async_trait]
    impl LlmDriver for FailingCaptureDriver {
        async fn stream(
            &self,
            request: LlmRequest,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
        {
            self.requests
                .lock()
                .expect("primary requests")
                .push(request);
            Err(anyhow::anyhow!("fixture primary failure"))
        }
    }

    let contract = RequestBudgetContract::synthetic_exact("http://failover.invalid/v1");
    let primary = Arc::new(FailingCaptureDriver::default());
    let fallback = Arc::new(MockLlmDriver::echo());
    let mut failover = FailoverConfig::default();
    failover.enabled = true;
    let orchestrator = Orchestrator::from_driver(
        LlmConfig::default(),
        Arc::new(McpRegistry::new_empty()),
        Arc::new(NativeSkillRegistry::new()),
        primary.clone(),
    )
    .with_failover(fallback.clone(), failover)
    .with_request_budget_contract(contract.clone());

    let stream = orchestrator
        .chat("fail over with a bound contract")
        .await
        .expect("fallback stream starts");
    futures::pin_mut!(stream);
    while stream.next().await.is_some() {}

    let primary_requests = primary.requests.lock().expect("primary requests");
    assert_eq!(primary_requests.len(), 1);
    assert_eq!(
        primary_requests[0].budget_contract.as_ref(),
        Some(&contract)
    );
    let fallback_requests = fallback.requests();
    assert_eq!(fallback_requests.len(), 1);
    assert_eq!(
        fallback_requests[0].budget_contract.as_ref(),
        Some(&contract)
    );

    #[derive(Debug, Default)]
    struct RetryingCaptureDriver {
        attempts: AtomicUsize,
        requests: Mutex<Vec<LlmRequest>>,
    }

    #[async_trait::async_trait]
    impl LlmDriver for RetryingCaptureDriver {
        async fn stream(
            &self,
            request: LlmRequest,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
        {
            self.requests.lock().expect("retry requests").push(request);
            if self.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(ProviderError::timeout("fixture retry").into());
            }
            Ok(Box::pin(futures::stream::iter([Ok(NormalizedEvent::Done)])))
        }
    }

    let retry_driver = Arc::new(RetryingCaptureDriver::default());
    let mut retry_policy = ResiliencePolicy::default();
    retry_policy.retry_max_attempts = 2;
    retry_policy.retry_base_delay_ms = 100;
    retry_policy.retry_max_delay_ms = 100;
    retry_policy.retry_jitter_mode = "none".to_string();
    retry_policy.retry_budget_ms = 500;
    let retry_orchestrator = Orchestrator::from_driver(
        LlmConfig::default(),
        Arc::new(McpRegistry::new_empty()),
        Arc::new(NativeSkillRegistry::new()),
        retry_driver.clone(),
    )
    .with_resilience_policy(retry_policy)
    .with_request_budget_contract(contract.clone());
    let retry_stream = retry_orchestrator
        .chat("retry with a bound contract")
        .await
        .expect("retry stream starts on the second attempt");
    futures::pin_mut!(retry_stream);
    while retry_stream.next().await.is_some() {}
    let retry_requests = retry_driver.requests.lock().expect("retry requests");
    assert_eq!(retry_requests.len(), 2);
    assert!(
        retry_requests
            .iter()
            .all(|request| request.budget_contract.as_ref() == Some(&contract))
    );
}

/// Scenario: Oversized terminal output is bounded once at ingest with a warning header.
#[tokio::test]
async fn oversized_tool_output_is_truncated_middle_out_with_warning_header() {
    use std::sync::Arc;

    use futures::StreamExt;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::{Orchestrator, mock_driver::MockLlmDriver};
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::context::truncate::{
        TruncationPolicy, WARNING_HEADER_PREFIX,
    };
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
    use universal_agent_runtime::uar::tools::terminal_exec::TerminalExecTool;

    let command = r#"i=0; while [ "$i" -lt 5000 ]; do printf 'line-%04d xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n' "$i"; i=$((i + 1)); done"#;
    let arguments = serde_json::json!({"command": command}).to_string();
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::ToolCallDelta {
                call_index: 0,
                id: Some("terminal-1".to_string()),
                name: Some("terminal_exec".to_string()),
                arguments_delta: Some(arguments.clone()),
            },
            NormalizedEvent::ToolCallComplete {
                call_index: 0,
                id: "terminal-1".to_string(),
                name: "terminal_exec".to_string(),
                arguments_json: arguments,
            },
            NormalizedEvent::Done,
        ],
        vec![
            NormalizedEvent::MessageDelta {
                text: "terminal complete".to_string(),
            },
            NormalizedEvent::Done,
        ],
    ]));
    let native_skills = Arc::new(NativeSkillRegistry::new());
    native_skills
        .register(TerminalExecTool {
            shell: "/bin/sh".to_string(),
            timeout_secs: 10,
            use_sandbox: false,
        })
        .await
        .expect("terminal descriptor registers");
    let policy = TruncationPolicy::Bytes(4_096);
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native_skills,
        driver.clone(),
    )
    .with_tool_output_policy(policy);

    let events: Vec<_> = orchestrator
        .chat("produce a verbose terminal log")
        .await
        .expect("orchestrator starts")
        .collect()
        .await;
    let recorded = events
        .iter()
        .find_map(|event| match event {
            NormalizedEvent::ToolResult {
                id,
                content,
                success,
                ..
            } if id == "terminal-1" => {
                assert!(*success, "terminal command succeeds");
                Some(content.as_str())
            }
            _ => None,
        })
        .expect("tool result event is emitted");

    assert!(
        recorded.len() <= 4_096,
        "recorded terminal result ({} bytes) must be within the configured policy",
        recorded.len()
    );
    assert!(
        recorded.starts_with(WARNING_HEADER_PREFIX),
        "truncated output begins with the warning header, got: {}",
        &recorded[..recorded.len().min(80)]
    );
    assert!(
        recorded.matches(WARNING_HEADER_PREFIX).count() == 1,
        "terminal output is truncated exactly once"
    );
    assert!(
        recorded.contains("original token count: "),
        "header states the original token count"
    );
    assert!(
        recorded.contains("Total output lines: "),
        "header states the total line count"
    );
    assert!(
        recorded.contains("line-0000"),
        "head of the output is retained"
    );
    assert!(
        recorded.contains("line-4999"),
        "tail of the output is retained"
    );
    assert!(
        !recorded.contains("line-2500"),
        "the middle is what gets removed"
    );

    let requests = driver.requests();
    assert_eq!(requests.len(), 2, "tool execution triggers a second turn");
    let second: Vec<Message> = requests[1]
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("second provider request remains typed");
    let history_result = second
        .iter()
        .find(|message| message.tool_call_id.as_deref() == Some("terminal-1"))
        .and_then(|message| message.content.as_text())
        .expect("terminal result reaches the next provider request");
    assert_eq!(
        history_result, recorded,
        "the emitted event and model-visible history record the same single truncation"
    );
}

#[test]
fn token_truncation_uses_the_model_tokenizer_and_enforces_the_limit() {
    use universal_agent_runtime::uar::runtime::context::token_service::TokenService;
    use universal_agent_runtime::uar::runtime::context::truncate::{
        TruncationPolicy, WARNING_HEADER_PREFIX, formatted_truncate_for_model,
    };

    let content = "日🦀".repeat(4_000);
    let budget = 96;
    let recorded =
        formatted_truncate_for_model(&content, TruncationPolicy::Tokens(budget), "openai/gpt-4o");

    assert!(
        TokenService::count("openai/gpt-4o", &recorded) <= budget,
        "token policy must be measured by the selected model tokenizer"
    );
    assert!(recorded.starts_with(WARNING_HEADER_PREFIX));
}

/// Scenario: Known and unknown models are counted by one model-keyed service.
#[test]
fn token_service_is_model_keyed_with_cl100k_fallback() {
    use std::{
        fmt::Write as _,
        sync::{Arc, Mutex},
    };

    use tracing::{
        Subscriber,
        field::{Field, Visit},
    };
    use tracing_subscriber::{Layer, layer::Context, prelude::*};
    use universal_agent_runtime::uar::runtime::context::token_service::{
        TokenEncoding, TokenService,
    };

    #[derive(Clone, Default)]
    struct CapturedEvents(Arc<Mutex<Vec<(String, String)>>>);

    #[derive(Default)]
    struct CapturedFields(String);

    impl Visit for CapturedFields {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            let _ = write!(&mut self.0, "{}={value:?} ", field.name());
        }
    }

    impl<S: Subscriber> Layer<S> for CapturedEvents {
        fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
            let mut fields = CapturedFields::default();
            event.record(&mut fields);
            self.0
                .lock()
                .expect("capture lock")
                .push((event.metadata().name().to_string(), fields.0));
        }
    }

    let text = "Hello, world! The quick brown fox jumps over the lazy dog. 🦀";

    // A model tiktoken maps to o200k_base.
    assert_eq!(
        TokenService::encoding_for("openai/gpt-4o"),
        TokenEncoding::O200kBase
    );
    let expected_o200k = tiktoken_rs::o200k_base_singleton()
        .encode_with_special_tokens(text)
        .len();
    assert_eq!(TokenService::count("openai/gpt-4o", text), expected_o200k);

    // A model tiktoken maps to cl100k_base.
    assert_eq!(
        TokenService::encoding_for("openai/gpt-4"),
        TokenEncoding::Cl100kBase
    );

    // An unknown model uses the documented fallback and says so.
    assert_eq!(
        TokenService::encoding_for("groq/llama-3"),
        TokenEncoding::Cl100kFallback
    );
    let expected_cl100k = tiktoken_rs::cl100k_base_singleton()
        .encode_with_special_tokens(text)
        .len();
    let events = CapturedEvents::default();
    let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(events.clone()));
    let fallback_count =
        tracing::dispatcher::with_default(&dispatch, || TokenService::count("groq/llama-3", text));
    assert_eq!(fallback_count, expected_cl100k);
    let captured = events.0.lock().expect("capture lock");
    let (_, fields) = captured
        .iter()
        .find(|(name, _)| name == "context.token.estimate")
        .expect("token estimate telemetry event");
    assert!(fields.contains("model=\"groq/llama-3\""));
    assert!(fields.contains("token_encoding=Cl100kFallback"));
    assert!(fields.contains("token_estimate_fallback=true"));

    // The message counter goes through the same keyed path.
    let msgs = vec![text_msg(MessageRole::User, text)];
    assert_eq!(
        TokenService::count_messages("openai/gpt-4o", &msgs),
        expected_o200k + 3 + 3,
        "per-message overhead plus reply priming, counted with the model's encoding"
    );
}

fn text_msg(role: MessageRole, s: &str) -> Message {
    text(role, s)
}

/// Scenario: Resume from a graph checkpoint restores its state and messages,
/// and a checkpoint that fails to deserialize is an error rather than empty.
#[test]
fn checkpoint_resume_restores_state_and_messages_or_errors() {
    use universal_agent_runtime::uar::runtime::checkpoint::{Checkpoint, history_from_checkpoint};
    use universal_agent_runtime::uar::runtime::graph::GraphState;

    let mut state = GraphState::default();
    state.iteration = 3;
    state.set("route", "rust-reviewer".to_string());
    state
        .messages
        .push(serde_json::json!({"role": "user", "content": "review this"}));
    state
        .messages
        .push(serde_json::json!({"role": "assistant", "content": "looking"}));

    let checkpoint = Checkpoint::new("run-1", "thread-1", "reviewer", &state, &"0".repeat(64));
    let protection = checkpoint
        .protection
        .as_ref()
        .expect("current checkpoint has protection metadata");
    assert_eq!(protection.raw_history_sha256.len(), 64);
    assert_eq!(protection.raw_state_sha256.len(), 64);

    // Restore is exact.
    let restored = checkpoint
        .try_restore_state()
        .expect("well-formed checkpoint restores");
    assert_eq!(restored.iteration, 3);
    assert_eq!(restored.messages.len(), 2);
    assert_eq!(
        restored.data.get("route").and_then(|v| v.as_str()),
        Some("rust-reviewer")
    );

    // The run seed is the typed history, in order.
    let history = history_from_checkpoint(&checkpoint).expect("messages convert");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].role, MessageRole::User);
    assert_eq!(history[0].content.as_text(), Some("review this"));
    assert_eq!(history[1].role, MessageRole::Assistant);

    // A corrupt state bag is an error, not a silently empty state.
    let mut corrupt = checkpoint.clone();
    corrupt.state = serde_json::json!("not an object");
    assert!(
        corrupt.try_restore_state().is_err(),
        "deserialization failure must surface as an error"
    );

    // A corrupt message entry is an error too.
    let mut bad_messages = checkpoint;
    bad_messages.messages = vec![serde_json::json!({"role": "nonsense"})];
    assert!(history_from_checkpoint(&bad_messages).is_err());

    let mut legacy_json = serde_json::to_value(&bad_messages).expect("serialize checkpoint");
    legacy_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("protection");
    let legacy: Checkpoint = serde_json::from_value(legacy_json).expect("decode legacy row");
    let legacy_error = history_from_checkpoint(&legacy).expect_err("legacy row is incomplete");
    assert!(legacy_error.to_string().contains("legacy-incomplete"));
}

#[tokio::test]
async fn checkpoint_resume_reassembles_trusted_system_and_preserves_checkpoint_dialogue() {
    use std::sync::Arc;

    use axum::Extension;
    use axum_test::TestServer;
    use tokio::sync::RwLock;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::session::SessionStore;
    use universal_agent_runtime::uar::defaults;
    use universal_agent_runtime::uar::persistence::{
        PersistenceLayer, providers::surreal::SurrealDbProvider,
    };
    use universal_agent_runtime::uar::rag::embeddings::{
        EmbeddingBackend, UnavailableEmbeddingBackend,
    };
    use universal_agent_runtime::uar::runtime::checkpoint::Checkpoint;
    use universal_agent_runtime::uar::runtime::graph::{AgentGraph, GraphState};
    use universal_agent_runtime::uar::runtime::manager::RunManager;
    use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
    use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
    use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};

    let tempdir = tempfile::tempdir().expect("checkpoint tempdir");
    let url = format!("surrealkv://{}", tempdir.path().display());
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(&url, None, None, None, None)
            .await
            .expect("SurrealDB checkpoint store"),
    );
    let (capture_tx, mut capture_rx) = tokio::sync::mpsc::unbounded_channel();
    let executions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let graph = AgentGraph::builder("capture")
        .add_node(CaptureResumeNode {
            sender: capture_tx,
            executions: Arc::clone(&executions),
        })
        .build();
    let embedding_backend: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let driver = Arc::new(MockLlmDriver::new(vec![vec![NormalizedEvent::Done]]));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::new(None, None))),
            Arc::new(VectorMatcher::new(embedding_backend, 0.75)),
            Some(persistence.clone()),
        )
        .await
        .with_llm_driver(driver)
        .with_agent_graph(graph),
    );

    let user = UserContext {
        user_id: "alice".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "alice".to_string(),
            name: None,
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    };
    let mut initial_request =
        universal_agent_runtime::uar::runtime::turn::RunExecutionRequest::new(
            defaults::orchestrator_agent(),
            "message after checkpoint".to_string(),
        )
        .with_user_context(&user)
        .expect("verified test user");
    initial_request.session_id = Some("thread-1".to_string());
    let original_run_id = manager.execute_request(initial_request).await;
    let initial_state = tokio::time::timeout(std::time::Duration::from_secs(3), capture_rx.recv())
        .await
        .expect("initial graph executes")
        .expect("capture channel remains open");
    let checkpoint_authorization_digest = initial_state
        .get::<String>(
            universal_agent_runtime::uar::runtime::checkpoint::CHECKPOINT_AUTHORIZATION_DIGEST_KEY,
        )
        .expect("graph state carries immutable checkpoint authorization");

    let checkpoint_messages = vec![
        text(MessageRole::System, "checkpoint system"),
        text(MessageRole::User, "review this"),
        assistant_with_calls(&["checkpoint-call"]),
        tool_result("checkpoint-call", "checkpoint result"),
    ];
    let mut checkpoint_state = GraphState::default();
    checkpoint_state.iteration = 3;
    checkpoint_state.set("restored-key", "restored-value".to_string());
    checkpoint_state.set("normalization-sensitive", "Null".to_string());
    checkpoint_state.messages = checkpoint_messages
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .expect("serialize checkpoint history");
    let checkpoint = Checkpoint::new(
        &original_run_id,
        "thread-1",
        "capture",
        &checkpoint_state,
        &checkpoint_authorization_digest,
    );
    persistence
        .save_checkpoint(&checkpoint)
        .await
        .expect("persist checkpoint");

    let app = universal_agent_runtime::uar::api::routes::build_router()
        .with_state(manager.clone())
        .layer(Extension(user));
    let server = TestServer::new(app);
    let response = server
        .post(&format!("/runs/{original_run_id}/resume/{}", checkpoint.id))
        .json(&serde_json::json!({
            "artifact": defaults::orchestrator_agent(),
            "session_id": "thread-1"
        }))
        .await;
    response.assert_status_ok();

    let observed = tokio::time::timeout(std::time::Duration::from_secs(3), capture_rx.recv())
        .await
        .expect("resumed graph executes")
        .expect("capture channel returns state");
    assert_eq!(
        observed.iteration, 4,
        "checkpoint iteration resumes at three"
    );
    assert_eq!(
        observed.get::<String>("restored-key").as_deref(),
        Some("restored-value")
    );
    assert_eq!(
        observed.get::<String>("normalization-sensitive").as_deref(),
        Some("Null"),
        "protected raw state wins over database value normalization"
    );
    let observed_messages: Vec<Message> = observed
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("resumed graph history remains typed");
    assert_eq!(observed_messages.len(), checkpoint_messages.len() + 4);
    let trusted_system = observed_messages[0]
        .content
        .as_text()
        .expect("resume begins with current trusted system assembly");
    assert!(trusted_system.contains("You coordinate specialist sub-agents."));
    assert!(trusted_system.contains("[EFFECTIVE RUN POLICY]"));
    assert!(trusted_system.contains("<uar-host-content>"));
    assert!(
        observed_messages
            .iter()
            .all(|message| message.content.as_text() != Some("checkpoint system")),
        "checkpoint system text must not override current trusted assembly"
    );
    assert!(
        observed_messages.iter().any(|message| {
            message.role == MessageRole::User && message.content.as_text() == Some("review this")
        }),
        "checkpoint user dialogue must survive resume"
    );
    assert_eq!(
        observed_messages
            .iter()
            .filter(|message| {
                message
                    .content
                    .as_text()
                    .is_some_and(|content| content.contains("[WORLD STATE:"))
            })
            .count(),
        4
    );
    let assistant_index = observed_messages
        .iter()
        .position(|message| {
            message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| calls.iter().any(|call| call.id == "checkpoint-call"))
        })
        .expect("checkpoint assistant tool call survives resume");
    assert_eq!(
        observed_messages
            .get(assistant_index + 1)
            .and_then(|message| message.tool_call_id.as_deref()),
        Some("checkpoint-call")
    );
    assert!(
        observed_messages.iter().all(|message| {
            message.role != MessageRole::User || message.content.as_text() != Some("")
        }),
        "absent resume input must not append an empty user message"
    );
    assert_eq!(executions.load(std::sync::atomic::Ordering::SeqCst), 2);

    let revoked_state = checkpoint_state;
    let revoked = Checkpoint::new(
        &original_run_id,
        "thread-1",
        "capture",
        &revoked_state,
        &"f".repeat(64),
    );
    persistence
        .save_checkpoint(&revoked)
        .await
        .expect("persist revoked checkpoint fixture");
    let revoked_response = server
        .post(&format!("/runs/{original_run_id}/resume/{}", revoked.id))
        .json(&serde_json::json!({
            "artifact": defaults::orchestrator_agent(),
            "session_id": "thread-1"
        }))
        .await;
    revoked_response.assert_status_ok();
    let revoked_body: serde_json::Value = revoked_response.json();
    let revoked_run_id = revoked_body["run_id"]
        .as_str()
        .expect("revoked response run id");
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if manager.get_run(revoked_run_id).await.is_some_and(|run| {
                run.status == universal_agent_runtime::uar::domain::runs::RunStatus::Error
            }) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("revoked resume reaches blocked error");
    assert_eq!(
        executions.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "revoked checkpoint must not reach graph or model dispatch"
    );
}
