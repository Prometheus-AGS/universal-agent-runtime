//! The non-streaming OpenAI-compatible response reports the run's usage.
//!
//! The run's token counts arrive on the event that ends it, but the
//! non-streaming handler matched that event and discarded them, so every
//! client of `/v1/chat/completions` with `stream: false` got no `usage`.

use serde_json::json;
use universal_agent_runtime::uar::api::openai::usage::from_run_done;
use universal_agent_runtime::uar::domain::events::NormalizedEvent;

fn run_done_with_usage(
    input: Option<u32>,
    output: Option<u32>,
    total: Option<u32>,
) -> NormalizedEvent {
    NormalizedEvent::RunDoneWithUsage {
        run_id: "run-1".to_string(),
        input_tokens: input,
        output_tokens: output,
        total_tokens: total,
        cost_usd_estimate: Some(0.0042),
        model: Some("qwen3.8-max".to_string()),
    }
}

#[test]
fn reports_the_counts_in_the_openai_shape() {
    let usage = from_run_done(&run_done_with_usage(Some(120), Some(45), Some(165)));

    assert_eq!(
        usage,
        Some(json!({"prompt_tokens": 120, "completion_tokens": 45, "total_tokens": 165}))
    );
}

#[test]
fn a_missing_total_is_the_sum_of_what_was_reported() {
    let usage = from_run_done(&run_done_with_usage(Some(120), Some(45), None));

    assert_eq!(
        usage,
        Some(json!({"prompt_tokens": 120, "completion_tokens": 45, "total_tokens": 165}))
    );
}

#[test]
fn a_count_that_was_not_reported_is_zero_not_absent() {
    let usage = from_run_done(&run_done_with_usage(None, Some(45), None));

    assert_eq!(
        usage,
        Some(json!({"prompt_tokens": 0, "completion_tokens": 45, "total_tokens": 45}))
    );
}

#[test]
fn the_sum_cannot_overflow() {
    let usage = from_run_done(&run_done_with_usage(Some(u32::MAX), Some(1), None));

    assert_eq!(usage.unwrap()["total_tokens"], json!(u32::MAX));
}

#[test]
fn a_run_end_with_no_counts_reports_no_usage() {
    // Omitting `usage` is honest; zeros would claim nothing was spent.
    assert_eq!(from_run_done(&run_done_with_usage(None, None, None)), None);
}

#[test]
fn a_plain_run_end_reports_no_usage() {
    let event = NormalizedEvent::RunDone {
        run_id: "run-1".to_string(),
    };

    assert_eq!(from_run_done(&event), None);
}

#[test]
fn an_unrelated_event_reports_no_usage() {
    let event = NormalizedEvent::ChatDelta {
        run_id: "run-1".to_string(),
        text_delta: "hi".to_string(),
    };

    assert_eq!(from_run_done(&event), None);
}
