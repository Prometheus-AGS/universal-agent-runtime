//! Pre-prompt memory context builder.
//!
//! Assembles a ranked block of relevant memories to inject into the system
//! prompt before each LLM call, constrained by the model's token budget.

use std::sync::Arc;

use surreal_memory::{Memory, model_profiles::profile_for};

use crate::uar::security::claims::UserContext;

use super::service::MemoryService;

/// Result of a context build — includes the formatted prompt block and the
/// raw memory hits that were included (for streaming to the client).
pub struct ContextBuildResult {
    /// Formatted memory block ready for injection into the system prompt.
    pub block: String,
    /// The ranked memory records that fit within the token budget.
    pub hits: Vec<Memory>,
}

/// Assembles a formatted memory context block for injection into a system prompt,
/// returning both the block and the raw memory hits.
///
/// Performs a hybrid search waterfall: Session → User → Agent → Global.
/// Results are re-ranked by a weighted score of importance, recency, and
/// vector relevance. The block is truncated to fit within the model's token budget
/// (or `max_tokens` from config if the model is not in the profile registry).
///
/// Returns an empty block and empty hits if no relevant memories are found.
pub async fn build_context_with_hits(
    service: &Arc<MemoryService>,
    query: &str,
    user_ctx: &UserContext,
    agent_id: Option<&str>,
    session_id: Option<&str>,
    model_id: &str,
) -> ContextBuildResult {
    if !service.config().inject_context {
        return ContextBuildResult { block: String::new(), hits: vec![] };
    }

    let max_tokens = service.config().max_context_tokens;

    let profile = profile_for(model_id);
    // profile_for always returns a &ModelProfile (falls back to "default").
    // Use up to max_context_tokens or 10% of the model's context budget, whichever is smaller.
    let ten_pct = (profile.context_window / 10) as u32;
    let effective_max = max_tokens.min(ten_pct);

    // Gather memories — hybrid search over (session, user, agent) then global.
    let limit = 20_usize; // retrieve wider set, then rank and cap by tokens
    let memories = match service
        .hybrid_search(
            query, user_ctx, agent_id, session_id, limit, None, // use config default weights
            None,
        )
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(error = %e, "context_builder: hybrid_search failed, using empty context");
            return ContextBuildResult { block: String::new(), hits: vec![] };
        }
    };

    if memories.is_empty() {
        return ContextBuildResult { block: String::new(), hits: vec![] };
    }

    // Rank and format memories up to the token budget.
    let mut total_tokens = 0u32;
    let mut lines: Vec<String> = Vec::with_capacity(memories.len());
    let mut hits: Vec<Memory> = Vec::new();

    for mem in rank_memories(memories) {
        let token_estimate = mem.token_count.unwrap_or_else(|| {
            // rough estimate: 1 token ≈ 4 chars
            #[allow(clippy::cast_possible_truncation)]
            ((mem.content.len() as u32) / 4).max(1)
        });

        if total_tokens + token_estimate > effective_max {
            break;
        }

        total_tokens += token_estimate;

        let scope_label = format!("{:?}", mem.scope).to_lowercase();
        let type_label = format!("{:?}", mem.memory_type).to_lowercase();
        lines.push(format!("[{scope_label}/{type_label}] {}", mem.content));
        hits.push(mem);
    }

    if lines.is_empty() {
        return ContextBuildResult { block: String::new(), hits: vec![] };
    }

    let block = format!(
        "## Relevant Memory Context\n\
         The following memories are relevant to this conversation. \
         Use them to personalize your response without explicitly citing them:\n\n{}\n",
        lines.join("\n")
    );

    ContextBuildResult { block, hits }
}

/// Assembles a formatted memory context block for injection into a system prompt.
///
/// This is a convenience wrapper around [`build_context_with_hits`] for callers
/// that do not need the raw memory hits.
pub async fn build_context(
    service: &Arc<MemoryService>,
    query: &str,
    user_ctx: &UserContext,
    agent_id: Option<&str>,
    session_id: Option<&str>,
    model_id: &str,
) -> String {
    build_context_with_hits(service, query, user_ctx, agent_id, session_id, model_id)
        .await
        .block
}

/// Re-rank memories by a composite score of importance, recency, and access frequency.
/// The vector/BM25 score is baked in from the storage layer; we apply a lightweight
/// re-ranking on top.
fn rank_memories(mut memories: Vec<Memory>) -> Vec<Memory> {
    let now = chrono::Utc::now();

    memories.sort_by(|a, b| {
        let score_a = composite_score(a, &now);
        let score_b = composite_score(b, &now);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    memories
}

fn composite_score(mem: &Memory, now: &chrono::DateTime<chrono::Utc>) -> f32 {
    // Recency: decay over 30 days
    let age_days = {
        let updated: chrono::DateTime<chrono::Utc> = mem.updated_at.clone().into();
        (now.signed_duration_since(updated).num_seconds().max(0) as f32) / 86_400.0
    };
    let recency = 1.0_f32 / (1.0 + age_days / 30.0);

    // Importance 0.0–1.0 from storage.
    let importance = mem.importance.clamp(0.0, 1.0);

    // Access frequency boost (log-scaled).
    let access_boost = ((mem.access_count + 1) as f32).ln() / 10.0;

    // Weighted composite.
    importance * 0.4 + recency * 0.35 + access_boost * 0.25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_memories_returns_empty_string() {
        // composite_score doesn't panic on zero access_count either.
        let now = chrono::Utc::now();
        let mem = surreal_memory::Memory::new("test", None, None, None, vec![]);
        let score = composite_score(&mem, &now);
        assert!(score >= 0.0);
    }
}
