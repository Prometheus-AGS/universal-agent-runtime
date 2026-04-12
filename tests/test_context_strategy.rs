/// Integration tests for the message-count context strategy.
///
/// These tests verify the strategy types and application logic in isolation.
/// Full pipeline tests (with RunManager) require a running server and are
/// deferred to the E2E suite.

#[path = "common/mod.rs"]
mod common;

use universal_agent_runtime::uar::context::{ContextStrategy, apply_strategy, trim_count};

#[test]
fn sliding_window_trims_old_messages() {
    let msgs: Vec<serde_json::Value> = (0..30)
        .map(|i| serde_json::json!({"role": "user", "content": format!("msg-{i}")}))
        .collect();

    let strategy = ContextStrategy::SlidingWindow { max_messages: 10 };
    let result = apply_strategy(&msgs, &strategy);

    assert_eq!(result.len(), 10, "should keep only the last 10 messages");
    assert_eq!(result[0]["content"], "msg-20", "first kept message should be msg-20");
    assert_eq!(result[9]["content"], "msg-29", "last kept message should be msg-29");
}

#[test]
fn sliding_window_no_op_when_under_limit() {
    let msgs: Vec<serde_json::Value> = (0..5)
        .map(|i| serde_json::json!({"role": "user", "content": format!("msg-{i}")}))
        .collect();

    let strategy = ContextStrategy::SlidingWindow { max_messages: 10 };
    assert_eq!(apply_strategy(&msgs, &strategy).len(), 5);
}

#[test]
fn trim_count_works_with_typed_items() {
    let items: Vec<String> = (0..15).map(|i| format!("item-{i}")).collect();
    let strategy = ContextStrategy::SlidingWindow { max_messages: 5 };
    let result = trim_count(items, &strategy);
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], "item-10");
    assert_eq!(result[4], "item-14");
}

#[test]
fn config_default_is_sliding_window_20() {
    let strategy = ContextStrategy::default();
    match strategy {
        ContextStrategy::SlidingWindow { max_messages } => assert_eq!(max_messages, 20),
        _ => panic!("default should be SlidingWindow(20)"),
    }
}
