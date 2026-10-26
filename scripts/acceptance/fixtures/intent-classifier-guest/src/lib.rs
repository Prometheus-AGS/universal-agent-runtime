//! Actual metadata keyword classifier guest; build only in the approved private
//! generated package. cargo-component creates bindings.rs there, never here.

use std::collections::BTreeSet;

mod bindings;

use bindings::exports::intent::classifier::classifier::{ClassificationResult, Guest, IntentScore};

struct Classifier;

const METADATA_PREFIX: &str = "Available eligible skills (JSON data): ";
const MAX_INPUT_BYTES: usize = 64 * 1024;
const MAX_CONTEXT_BYTES: usize = 1024 * 1024;
const MAX_SKILLS: usize = 1024;
const MAX_KEYWORDS: usize = 64;
const MAX_KEYWORD_BYTES: usize = 512;
const MAX_LABEL_BYTES: usize = 1024;
const MAX_RESULTS: usize = 64;

impl Guest for Classifier {
    fn classify(input: String, context: String) -> Result<ClassificationResult, String> {
        if input.len() > MAX_INPUT_BYTES || context.len() > MAX_CONTEXT_BYTES {
            return Err("classifier_input_bound_exceeded".into());
        }
        let context: Vec<String> = serde_json::from_str(&context)
            .map_err(|_| "classifier_context_invalid")?;
        // The real host appends current eligible metadata last. Caller dialogue
        // cannot supply authority by inserting an earlier lookalike context item.
        let metadata = context.last()
            .and_then(|item| item.strip_prefix(METADATA_PREFIX))
            .ok_or("classifier_current_metadata_missing")?;
        let skills: Vec<serde_json::Value> = serde_json::from_str(metadata)
            .map_err(|_| "classifier_metadata_invalid")?;
        if skills.len() > MAX_SKILLS {
            return Err("classifier_skill_bound_exceeded".into());
        }
        let query = input.to_lowercase();
        let mut seen = BTreeSet::new();
        let mut scores = Vec::new();
        for skill in skills {
            let label = skill.get("skill_id").and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty() && value.len() <= MAX_LABEL_BYTES)
                .ok_or("classifier_skill_identity_invalid")?;
            if !seen.insert(label.to_owned()) {
                return Err("classifier_duplicate_skill_identity".into());
            }
            let keywords = skill.get("keywords").and_then(serde_json::Value::as_array)
                .ok_or("classifier_keywords_invalid")?;
            if keywords.len() > MAX_KEYWORDS {
                return Err("classifier_keyword_bound_exceeded".into());
            }
            let mut best = 0.0_f32;
            for keyword in keywords {
                let keyword = keyword.as_str().ok_or("classifier_keyword_invalid")?;
                if keyword.len() > MAX_KEYWORD_BYTES {
                    return Err("classifier_keyword_bound_exceeded".into());
                }
                let keyword = keyword.trim().to_lowercase();
                if keyword.is_empty() || !query.contains(&keyword) {
                    continue;
                }
                // Same legitimate keyword/coverage rule as the host's rules
                // classifier; labels always come from this invocation's registry.
                let score = if query == keyword { 1.0 } else {
                    0.7 + (keyword.len() as f32 / query.len() as f32 * 0.3).min(0.3)
                };
                best = best.max(score);
            }
            if best > 0.0 {
                scores.push(IntentScore { label: label.to_owned(), score: best });
            }
        }
        scores.sort_by(|left, right| right.score.total_cmp(&left.score)
            .then_with(|| left.label.cmp(&right.label)));
        scores.truncate(MAX_RESULTS);
        Ok(ClassificationResult { out_of_scope: scores.is_empty(), scores })
    }
}

bindings::export!(Classifier with_types_in bindings);
