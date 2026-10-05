//! The OpenAI-shaped `usage` object for a finished run.

use crate::uar::domain::events::NormalizedEvent;
use serde_json::{Value, json};

/// `usage` for an OpenAI-compatible chat completion, taken from the event that
/// ends a run.
///
/// Returns `None` for any event but a run end that reported token counts, and
/// for a run end that reported none, so the response omits `usage` rather than
/// claiming zero tokens were used. A missing total is the sum of the counts that
/// were reported.
#[must_use]
pub fn from_run_done(event: &NormalizedEvent) -> Option<Value> {
    let NormalizedEvent::RunDoneWithUsage {
        input_tokens,
        output_tokens,
        total_tokens,
        ..
    } = event
    else {
        return None;
    };
    if input_tokens.is_none() && output_tokens.is_none() && total_tokens.is_none() {
        return None;
    }
    let prompt = input_tokens.unwrap_or(0);
    let completion = output_tokens.unwrap_or(0);
    let total = total_tokens.unwrap_or_else(|| prompt.saturating_add(completion));
    Some(json!({
        "prompt_tokens": prompt,
        "completion_tokens": completion,
        "total_tokens": total,
    }))
}
