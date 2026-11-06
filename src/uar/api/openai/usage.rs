//! The OpenAI-shaped `usage` object for a finished run.

use crate::uar::domain::events::{NormalizedEvent, RunUsage};
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
        cache_read_input_tokens,
        cache_creation_input_tokens,
        ..
    } = event
    else {
        return None;
    };
    project_usage(
        *input_tokens,
        *output_tokens,
        *total_tokens,
        *cache_read_input_tokens,
        *cache_creation_input_tokens,
    )
}

/// Project reported cancelled-run usage with the same optional cache semantics.
#[must_use]
pub fn from_usage(usage: &RunUsage) -> Option<Value> {
    project_usage(
        usage.input_tokens,
        usage.output_tokens,
        usage.total_tokens,
        usage.cache_read_input_tokens,
        usage.cache_creation_input_tokens,
    )
}

fn project_usage(
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    total_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
    cache_creation_input_tokens: Option<u32>,
) -> Option<Value> {
    if input_tokens.is_none()
        && output_tokens.is_none()
        && total_tokens.is_none()
        && cache_read_input_tokens.is_none()
        && cache_creation_input_tokens.is_none()
    {
        return None;
    }
    let prompt = input_tokens.unwrap_or(0);
    let completion = output_tokens.unwrap_or(0);
    let total = total_tokens.unwrap_or_else(|| prompt.saturating_add(completion));
    let mut usage = json!({
        "prompt_tokens": prompt,
        "completion_tokens": completion,
        "total_tokens": total,
    });
    let mut details = serde_json::Map::new();
    if let Some(count) = cache_read_input_tokens {
        details.insert("cached_tokens".into(), count.into());
    }
    // UAR extension: keep cache creation distinct from standard cached reads.
    if let Some(count) = cache_creation_input_tokens {
        details.insert("cache_creation_tokens".into(), count.into());
    }
    if !details.is_empty() {
        usage["prompt_tokens_details"] = details.into();
    }
    Some(usage)
}
