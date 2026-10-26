//! Fuel- and memory-bounded Component Model intent classification.
//!
//! This module provides an adapter for WASM components that implement
//! the WIT interface below. Guests receive no host filesystem, network, clock,
//! environment or tool imports. Labels are intersected with the real registry.
//!
//! The exported interface is `intent:classifier/classifier@0.1.0`. Context is
//! a JSON string array containing caller dialogue followed by public eligible
//! skill metadata. The component returns the typed result declared below.

use super::{ClassificationResult, IntentClassifier};
#[cfg(feature = "wasm-runtime")]
use super::IntentScore;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;

/// WASM-based intent classifier.
///
/// Uses only the configured component and the current enabled registry.
#[derive(Debug)]
pub struct WasmClassifier {
    /// Path to the WASM component file
    component_path: Option<String>,
}

impl WasmClassifier {
    /// Creates a new WASM classifier.
    ///
    /// # Arguments
    /// * `component_path` - Optional path to the WASM component file
    pub fn new(component_path: Option<String>) -> Self {
        Self { component_path }
    }
}

#[async_trait]
impl IntentClassifier for WasmClassifier {
    async fn classify(
        &self,
        query: &str,
        context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        let path = self.component_path.as_ref().filter(|path| !path.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("WASM classifier not configured: no component_path specified"))?;
        #[cfg(feature = "wasm-runtime")]
        {
            let skills = registry.list_enabled();
            let metadata = skills.iter().map(|skill| serde_json::json!({
                "skill_id":skill.skill_id,"title":skill.title,"description":skill.description,
                "keywords":skill.triggers.keywords,"semantic":skill.triggers.semantic,
            })).collect::<Vec<_>>();
            let mut guest_context = context.to_vec();
            guest_context.push(format!("Available eligible skills (JSON data): {}", serde_json::to_string(&metadata)?));
            let path = path.clone();
            let query = query.to_owned();
            let context_json = serde_json::to_string(&guest_context)?;
            tokio::task::spawn_blocking(move || classify_component(&path, &query, &context_json, &skills))
                .await.map_err(|_| anyhow::anyhow!("wasm_classifier_worker_failed"))?
        }
        #[cfg(not(feature = "wasm-runtime"))]
        {
            let _ = (path, query, context, registry);
            anyhow::bail!("wasm_classifier_runtime_feature_required")
        }
    }
}

#[cfg(feature = "wasm-runtime")]
fn classify_component(path: &str, query: &str, context: &str,
    skills: &[crate::uar::domain::skills::Skill]) -> anyhow::Result<ClassificationResult>
{
    use wasmtime::component::{Component, Linker, Val};
    use wasmtime::{Config, Engine, Store, StoreLimitsBuilder};
    let mut config = Config::new();
    config.wasm_component_model(true).consume_fuel(true);
    let engine = Engine::new(&config).map_err(|_| anyhow::anyhow!("wasm_classifier_engine_failed"))?;
    let component = Component::from_file(&engine, path)
        .map_err(|_| anyhow::anyhow!("wasm_classifier_component_load_failed"))?;
    let limits = StoreLimitsBuilder::new().memory_size(16 * 1024 * 1024)
        .memories(1).table_elements(16_384).tables(4).instances(16).build();
    let mut store = Store::new(&engine, limits);
    store.limiter(|limits| limits);
    store.set_fuel(10_000_000).map_err(|_| anyhow::anyhow!("wasm_classifier_fuel_failed"))?;
    let linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &component)
        .map_err(|_| anyhow::anyhow!("wasm_classifier_import_or_instantiation_failed"))?;
    let interface = instance.get_export_index(&mut store, None, "intent:classifier/classifier@0.1.0")
        .ok_or_else(|| anyhow::anyhow!("wasm_classifier_interface_missing"))?;
    let export = instance.get_export_index(&mut store, Some(&interface), "classify")
        .ok_or_else(|| anyhow::anyhow!("wasm_classifier_export_missing"))?;
    let function = instance.get_func(&mut store, &export)
        .ok_or_else(|| anyhow::anyhow!("wasm_classifier_function_missing"))?;
    let mut results = [Val::Bool(false)];
    function.call(&mut store, &[Val::String(query.to_owned()), Val::String(context.to_owned())], &mut results)
        .map_err(|_| anyhow::anyhow!("wasm_classifier_execution_failed"))?;
    let Val::Result(Ok(Some(result))) = &results[0] else {
        anyhow::bail!("wasm_classifier_guest_error_or_invalid_result");
    };
    let Val::Record(fields) = result.as_ref() else { anyhow::bail!("wasm_classifier_record_invalid"); };
    let field = |name: &str| fields.iter().find(|(key, _)| key == name).map(|(_, value)| value);
    let Some(Val::List(ranks)) = field("scores") else { anyhow::bail!("wasm_classifier_scores_invalid"); };
    let Some(Val::Bool(out_of_scope)) = field("out-of-scope") else { anyhow::bail!("wasm_classifier_scope_invalid"); };
    let mut seen = std::collections::HashSet::new();
    let mut scores = Vec::new();
    for rank in ranks {
        let Val::Record(rank) = rank else { anyhow::bail!("wasm_classifier_rank_invalid"); };
        let label = rank.iter().find_map(|(key, value)| match (key.as_str(), value) {
            ("label", Val::String(label)) => Some(label), _ => None,
        }).ok_or_else(|| anyhow::anyhow!("wasm_classifier_label_invalid"))?;
        let score = rank.iter().find_map(|(key, value)| match (key.as_str(), value) {
            ("score", Val::Float32(score)) => Some(*score), _ => None,
        }).ok_or_else(|| anyhow::anyhow!("wasm_classifier_score_invalid"))?;
        anyhow::ensure!(score.is_finite() && (0.0..=1.0).contains(&score)
            && seen.insert(label.clone()), "wasm_classifier_score_invalid");
        let skill = skills.iter().find(|skill| skill.skill_id == *label)
            .ok_or_else(|| anyhow::anyhow!("wasm_classifier_unknown_selection"))?;
        scores.push(IntentScore::with_skill(label.clone(), score, skill.clone()));
    }
    scores.sort_by(|left, right| right.score.total_cmp(&left.score)
        .then_with(|| left.label.cmp(&right.label)));
    Ok(ClassificationResult { scores, out_of_scope: *out_of_scope })
}

/// WIT interface required by the component classifier.
///
/// This is the interface that WASM components should implement.
/// Save this as `intent-classifier.wit` when building components.
pub const WIT_INTERFACE: &str = r#"
package intent:classifier@0.1.0;

interface classifier {
    /// Classification result for a single skill/intent.
    record intent-score {
        /// The skill ID or intent label
        label: string,
        /// Confidence score (0.0 to 1.0)
        score: float32,
    }

    /// Result of intent classification.
    record classification-result {
        /// Ranked list of intent scores (highest first)
        scores: list<intent-score>,
        /// Whether the query appears to be out-of-scope
        out-of-scope: bool,
    }

    /// Classifies the input query.
    ///
    /// Arguments:
    /// - input: The user's query text
    /// - context: Optional conversation context (JSON array of strings)
    ///
    /// Returns:
    /// - Classification result with ranked scores
    classify: func(input: string, context: string) -> result<classification-result, string>;
}

world intent-classifier {
    export classifier;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_classifier_not_configured() {
        let classifier = WasmClassifier::new(None);
        let registry = SkillRegistry::default();

        let result = classifier.classify("test query", &[], &registry).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no component_path specified")
        );
    }

    #[tokio::test]
    async fn missing_wasm_component_is_an_explicit_failure() {
        let classifier = WasmClassifier::new(Some("/path/to/classifier.wasm".to_string()));
        let registry = SkillRegistry::default();

        let result = classifier.classify("test query", &[], &registry).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("wasm_classifier_")
        );
    }
}
