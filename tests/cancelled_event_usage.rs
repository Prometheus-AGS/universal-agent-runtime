//! A cancelled run reports the usage it had accumulated.
//!
//! `Cancelled` carried only the run id, so a client that cancelled a run could
//! not settle what the run had already spent.

use serde_json::json;
use universal_agent_runtime::uar::api::sse::to_agui_event;
use universal_agent_runtime::uar::domain::events::{NormalizedEvent, RunUsage};

fn usage() -> RunUsage {
    RunUsage {
        input_tokens: Some(120),
        output_tokens: Some(30),
        total_tokens: Some(150),
        cost_usd_estimate: Some(0.0004),
        model: Some("openai/gpt-4.1-mini".to_string()),
    }
}

#[test]
fn agui_cancelled_carries_usage_when_present() {
    let (name, payload) = to_agui_event(&NormalizedEvent::Cancelled {
        run_id: "r1".into(),
        usage: Some(usage()),
    })
    .expect("cancelled maps to an AG-UI event");

    assert_eq!(name, "agui.cancelled");
    assert_eq!(payload["kind"], "cancelled");
    assert_eq!(payload["request_id"], "r1");
    assert_eq!(payload["usage"]["input_tokens"], 120);
    assert_eq!(payload["usage"]["output_tokens"], 30);
    assert_eq!(payload["usage"]["total_tokens"], 150);
    assert_eq!(payload["usage"]["cost_usd_estimate"], 0.0004);
    assert_eq!(payload["usage"]["model"], "openai/gpt-4.1-mini");
}

#[test]
fn agui_cancelled_is_unchanged_without_usage() {
    let (_, payload) = to_agui_event(&NormalizedEvent::Cancelled {
        run_id: "r1".into(),
        usage: None,
    })
    .unwrap();

    assert_eq!(payload, json!({"kind": "cancelled", "request_id": "r1"}));
}

#[test]
fn a_cancelled_payload_without_usage_still_deserializes() {
    let old = json!({"type": "Cancelled", "data": {"run_id": "r1"}});
    let event: NormalizedEvent = serde_json::from_value(old).unwrap();
    assert_eq!(
        event,
        NormalizedEvent::Cancelled {
            run_id: "r1".into(),
            usage: None
        }
    );
}

#[test]
fn no_usage_serializes_exactly_as_before() {
    let value = serde_json::to_value(NormalizedEvent::Cancelled {
        run_id: "r1".into(),
        usage: None,
    })
    .unwrap();
    assert_eq!(
        value,
        json!({"type": "Cancelled", "data": {"run_id": "r1"}})
    );
}

#[test]
fn usage_survives_a_round_trip() {
    let event = NormalizedEvent::Cancelled {
        run_id: "r1".into(),
        usage: Some(usage()),
    };
    let back: NormalizedEvent =
        serde_json::from_value(serde_json::to_value(&event).unwrap()).unwrap();
    assert_eq!(back, event);
}
