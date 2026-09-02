//! Behavioral tests for the `context-history-integrity` change.
//!
//! Spec: `openspec/changes/context-history-integrity/specs/conversation-history-integrity/spec.md`.
//! Each test maps to one requirement scenario. They are written before the
//! implementation and are expected to fail to compile until the modules under
//! `universal_agent_runtime::uar::runtime::context` exist.

use universal_agent_runtime::llm::{Message, MessageContent, MessageRole, ToolCall, ToolCallFunction};
use universal_agent_runtime::uar::runtime::context::normalize::{normalize_history, SYNTHETIC_CANCELLED_MARKER};

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
    assert_eq!(results.len(), 2, "every tool call must have exactly one result");

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
    assert!(c2_idx > assistant_idx, "synthetic result must follow its call");

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
    assert_eq!(history.first().map(|m| m.role.clone()), Some(MessageRole::User));
    assert_eq!(history.last().and_then(|m| m.content.as_text()), Some("done"));
}

/// Scenario: Long conversation under a sliding window keeps the system message.
#[test]
fn sliding_window_keeps_system_message_pinned_at_index_zero() {
    use universal_agent_runtime::uar::context::{trim_history, ContextStrategy};

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
        reduced.iter().filter(|m| m.role == MessageRole::System).count(),
        1,
        "the system message is never duplicated by the reducer"
    );
}

/// Scenario: User repeats "continue" and both turns survive the token-budget reducer.
#[tokio::test]
async fn identical_repeated_user_messages_survive_keep_first_last() {
    use universal_agent_runtime::uar::context::ContextStrategy;
    use universal_agent_runtime::uar::domain::context::ContextConfig;
    use universal_agent_runtime::uar::runtime::context::manager::ContextManager;

    let config = ContextConfig {
        strategy: ContextStrategy::TruncateMiddle {
            keep_first: 2,
            keep_last: 4,
        },
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

    assert!(action.is_some(), "the budget was exceeded so a reduction ran");
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

/// Scenario: Oversized terminal output is bounded once at ingest with a warning header.
#[test]
fn oversized_tool_output_is_truncated_middle_out_with_warning_header() {
    use universal_agent_runtime::uar::runtime::context::truncate::{
        formatted_truncate, TruncationPolicy, WARNING_HEADER_PREFIX,
    };

    // ~200 KB of numbered lines, the shape of a verbose build log.
    let lines: Vec<String> = (0..5_000).map(|i| format!("line-{i:04} {}", "x".repeat(32))).collect();
    let stdout = lines.join("\n");
    assert!(stdout.len() > 190_000, "fixture is about 200 KB, got {}", stdout.len());

    let budget = 16_000;
    let recorded = formatted_truncate(&stdout, TruncationPolicy::Bytes(budget));

    assert!(
        recorded.len() <= budget,
        "recorded output ({} bytes) must be within the byte budget ({budget})",
        recorded.len()
    );
    assert!(
        recorded.starts_with(WARNING_HEADER_PREFIX),
        "truncated output begins with the warning header, got: {}",
        &recorded[..recorded.len().min(80)]
    );
    assert!(
        recorded.contains("original token count: "),
        "header states the original token count"
    );
    assert!(
        recorded.contains("Total output lines: 5000"),
        "header states the total line count"
    );
    assert!(recorded.contains("line-0000"), "head of the output is retained");
    assert!(recorded.contains("line-4999"), "tail of the output is retained");
    assert!(!recorded.contains("line-2500"), "the middle is what gets removed");

    // Scenario: Output within policy is recorded unchanged.
    let small = "exit 0\nall good\n";
    assert_eq!(
        formatted_truncate(small, TruncationPolicy::Bytes(budget)),
        small,
        "output within the policy is untouched"
    );
}

/// Scenario: Known and unknown models are counted by one model-keyed service.
#[test]
fn token_service_is_model_keyed_with_cl100k_fallback() {
    use universal_agent_runtime::uar::runtime::context::token_service::{TokenEncoding, TokenService};

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
    assert_eq!(TokenService::count("groq/llama-3", text), expected_cl100k);

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
    use universal_agent_runtime::uar::runtime::checkpoint::{history_from_checkpoint, Checkpoint};
    use universal_agent_runtime::uar::runtime::graph::GraphState;

    let mut state = GraphState::default();
    state.iteration = 3;
    state.set("route", "rust-reviewer".to_string());
    state.messages.push(serde_json::json!({"role": "user", "content": "review this"}));
    state.messages.push(serde_json::json!({"role": "assistant", "content": "looking"}));

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
