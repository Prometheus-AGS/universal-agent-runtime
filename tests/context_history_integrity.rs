//! Behavioral tests for the `context-history-integrity` change.
//!
//! Spec: `openspec/changes/context-history-integrity/specs/conversation-history-integrity/spec.md`.
//! Each test maps to one requirement scenario. They are written before the
//! implementation and are expected to fail to compile until the modules under
//! `universal_agent_runtime::uar::runtime::context` exist.

use universal_agent_runtime::llm::{
    Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
};
use universal_agent_runtime::uar::runtime::context::normalize::{
    SYNTHETIC_CANCELLED_MARKER, normalize_history,
};

fn text(role: MessageRole, s: &str) -> Message {
    Message {
        role,
        content: MessageContent::text(s),
        tool_call_id: None,
        tool_calls: None,
    }
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
    sender: std::sync::Mutex<
        Option<
            tokio::sync::oneshot::Sender<universal_agent_runtime::uar::runtime::graph::GraphState>,
        >,
    >,
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
        if let Some(sender) = self.sender.lock().expect("capture lock").take() {
            let _ = sender.send(state.clone());
        }
        universal_agent_runtime::uar::runtime::graph::NodeResult::Finished(state)
    }
}

/// Scenario: Missing result is synthesized.
#[test]
fn missing_tool_result_is_synthesized_as_cancelled() {
    let mut history = vec![
        text(MessageRole::User, "run two tools"),
        assistant_with_calls(&["c1", "c2"]),
        tool_result("c1", "ok"),
    ];

    let report = normalize_history(&mut history);

    let results: Vec<&Message> = history
        .iter()
        .filter(|m| m.role == MessageRole::Tool)
        .collect();
    assert_eq!(
        results.len(),
        2,
        "every tool call must have exactly one result"
    );

    let c2 = results
        .iter()
        .find(|m| m.tool_call_id.as_deref() == Some("c2"))
        .expect("synthetic result for c2 exists");
    let body = c2.content.as_text().unwrap_or("");
    assert!(
        body.contains(SYNTHETIC_CANCELLED_MARKER),
        "synthetic result is typed cancelled, got: {body}"
    );

    // The real result is untouched and the synthetic one follows the call.
    let c1 = results
        .iter()
        .find(|m| m.tool_call_id.as_deref() == Some("c1"))
        .expect("real result for c1 exists");
    assert_eq!(c1.content.as_text(), Some("ok"));
    let assistant_idx = history
        .iter()
        .position(|m| m.role == MessageRole::Assistant)
        .expect("assistant message present");
    let c2_idx = history
        .iter()
        .position(|m| m.tool_call_id.as_deref() == Some("c2"))
        .expect("c2 result present");
    assert!(
        c2_idx > assistant_idx,
        "synthetic result must follow its call"
    );

    assert_eq!(report.synthesized, vec!["c2".to_string()]);
    assert!(report.removed.is_empty());
}

/// Scenario: Orphaned result is removed.
#[test]
fn orphaned_tool_result_is_removed_before_dispatch() {
    let mut history = vec![
        text(MessageRole::User, "hello"),
        assistant_with_calls(&["c1"]),
        tool_result("c1", "ok"),
        tool_result("ghost", "no call produced me"),
        text(MessageRole::Assistant, "done"),
    ];

    let report = normalize_history(&mut history);

    assert!(
        history
            .iter()
            .all(|m| m.tool_call_id.as_deref() != Some("ghost")),
        "orphaned result must not reach the provider"
    );
    assert_eq!(
        history
            .iter()
            .filter(|m| m.role == MessageRole::Tool)
            .count(),
        1
    );
    assert_eq!(report.removed, vec!["ghost".to_string()]);
    assert!(report.synthesized.is_empty());
    // Non-tool messages are untouched and keep their order.
    assert_eq!(
        history.first().map(|m| m.role.clone()),
        Some(MessageRole::User)
    );
    assert_eq!(
        history.last().and_then(|m| m.content.as_text()),
        Some("done")
    );
}

#[test]
fn misplaced_tool_result_is_removed_and_replaced_beside_its_call() {
    let mut history = vec![
        assistant_with_calls(&["c1"]),
        text(MessageRole::User, "intervening turn"),
        tool_result("c1", "late result"),
    ];

    let report = normalize_history(&mut history);

    assert_eq!(report.removed, vec!["c1".to_string()]);
    assert_eq!(report.synthesized, vec!["c1".to_string()]);
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].role, MessageRole::Assistant);
    assert_eq!(history[1].role, MessageRole::Tool);
    assert_eq!(history[1].tool_call_id.as_deref(), Some("c1"));
    assert!(
        history[1]
            .content
            .as_text()
            .is_some_and(|body| body.contains(SYNTHETIC_CANCELLED_MARKER))
    );
    assert_eq!(history[2].content.as_text(), Some("intervening turn"));
}

#[tokio::test]
async fn direct_orchestrator_normalizes_history_at_provider_dispatch() {
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
    let _: Vec<_> = stream.collect().await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 1);
    let dispatched: Vec<Message> = requests[0]
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("provider request remains typed");
    assert_eq!(dispatched[0].role, MessageRole::Assistant);
    assert_eq!(dispatched[1].role, MessageRole::Tool);
    assert_eq!(dispatched[1].tool_call_id.as_deref(), Some("c1"));
    assert_eq!(dispatched[2].role, MessageRole::User);
    assert_eq!(
        dispatched
            .iter()
            .filter(|message| message.role == MessageRole::Tool)
            .count(),
        1
    );
}

#[tokio::test]
async fn iterative_tool_loop_dispatches_a_complete_call_result_pair() {
    use std::sync::Arc;

    use futures::StreamExt;
    use universal_agent_runtime::config::LlmConfig;
    use universal_agent_runtime::llm::{Orchestrator, mock_driver::MockLlmDriver};
    use universal_agent_runtime::mcp::registry::McpRegistry;
    use universal_agent_runtime::normalized::NormalizedEvent;
    use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;

    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::ToolCallDelta {
                call_index: 0,
                id: Some("c1".to_string()),
                name: Some("missing_tool".to_string()),
                arguments_delta: Some("{}".to_string()),
            },
            NormalizedEvent::ToolCallComplete {
                call_index: 0,
                id: "c1".to_string(),
                name: "missing_tool".to_string(),
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
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        Arc::new(NativeSkillRegistry::new()),
        driver.clone(),
    );

    let stream = orchestrator
        .chat("run the missing tool")
        .await
        .expect("orchestrator starts");
    let _: Vec<_> = stream.collect().await;

    let requests = driver.requests();
    assert_eq!(
        requests.len(),
        2,
        "tool result must trigger another dispatch"
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
async fn reduction_drops_a_tool_pair_as_a_unit_when_the_window_severs_it() {
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::runtime::context::reduce::reduce_history;

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

    let (reduced, _) = reduce_history(
        history,
        &ContextStrategy::SlidingWindow { max_messages: 5 },
        "openai/gpt-4o",
        8_192,
        None,
    )
    .await;

    assert_eq!(reduced[0].role, MessageRole::System);
    assert!(
        reduced
            .iter()
            .all(|message| message.tool_call_id.as_deref() != Some("c1"))
    );
    assert!(reduced.iter().all(|message| {
        message
            .tool_calls
            .iter()
            .flatten()
            .all(|call| call.id != "c1")
    }));
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

    let checkpoint = Checkpoint::new("run-1", "thread-1", "reviewer", &state);

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
    let (capture_tx, capture_rx) = tokio::sync::oneshot::channel();
    let graph = AgentGraph::builder("capture")
        .add_node(CaptureResumeNode {
            sender: std::sync::Mutex::new(Some(capture_tx)),
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

    let original_run_id = manager
        .start_run(
            defaults::default_agent(),
            "message after checkpoint".to_string(),
            Some("thread-1".to_string()),
            Some("alice".to_string()),
            vec![],
        )
        .await;

    let checkpoint_messages = vec![
        text(MessageRole::System, "checkpoint system"),
        text(MessageRole::User, "review this"),
        assistant_with_calls(&["checkpoint-call"]),
        tool_result("checkpoint-call", "checkpoint result"),
    ];
    let mut checkpoint_state = GraphState::default();
    checkpoint_state.iteration = 3;
    checkpoint_state.set("restored-key", "restored-value".to_string());
    checkpoint_state.messages = checkpoint_messages
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .expect("serialize checkpoint history");
    let checkpoint = Checkpoint::new(&original_run_id, "thread-1", "capture", &checkpoint_state);
    persistence
        .save_checkpoint(&checkpoint)
        .await
        .expect("persist checkpoint");

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
    let app = universal_agent_runtime::uar::api::routes::build_router()
        .with_state(manager)
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

    let observed = tokio::time::timeout(std::time::Duration::from_secs(3), capture_rx)
        .await
        .expect("resumed graph executes")
        .expect("capture node returns state");
    assert_eq!(
        observed.iteration, 4,
        "checkpoint iteration resumes at three"
    );
    assert_eq!(
        observed.get::<String>("restored-key").as_deref(),
        Some("restored-value")
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
}
