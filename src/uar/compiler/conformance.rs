//! CH-14 conformance harness.
//!
//! Validates that a compiled agent descriptor's declared v2 sections
//! (`model_requirements`, `prompt_dialect`, `context_strategy`) are
//! satisfiable at load time and honored at run time — by driving the exact
//! runtime functions a real request uses ([`ModelRouter::route`],
//! [`PromptDialect::detect`], [`apply_strategy`]), not a re-implementation of
//! them. This is genuinely new test infrastructure (per `assessment.md`,
//! there was no "conformance" concept anywhere in the repo before this).
//!
//! `rag_configuration` and `api_harness` are declarative-only sections with
//! no corresponding runtime decision to conform against yet (RAG posture is
//! read by the retrieval pipeline directly; API harness just advertises
//! transport support) — out of scope for this harness, consistent with
//! plan.md's own framing ("declared model/dialect/context are satisfiable at
//! load and honored at run").
//!
//! A section left at its parsed default (which is indistinguishable from
//! "the author declared nothing here", since `#[serde(default)]` produces
//! the same value either way) is reported as [`CheckResult::NotDeclared`]
//! rather than checked — there's nothing to conform *to*.

use crate::llm::prompt_dialect::PromptDialect;
use crate::llm::router::{ModelRouter, RouteRequirements};
use crate::uar::context::strategy::{
    ContextStrategy, apply_strategy, default_keep_first, default_keep_last,
    default_long_term_tokens, default_max_messages, default_mid_term_tokens,
    default_short_term_turns, default_summarize_threshold, default_summary_max_tokens,
};

use super::ir::{
    AgentDescriptorIR, ContextStrategySection, ModelRequirementsSection, PromptDialectSection,
};

/// Outcome of one section's conformance check.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// The section was left at its parsed default — nothing was declared to
    /// check against.
    NotDeclared,
    /// The declared requirement is satisfiable (load) or was honored (run).
    Satisfied(String),
    /// The declared requirement could not be satisfied, or was not honored.
    Unsatisfied(String),
}

impl CheckResult {
    /// `true` for [`Self::NotDeclared`] and [`Self::Satisfied`]; `false` only
    /// for [`Self::Unsatisfied`] — a section nobody declared can't fail
    /// conformance.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        !matches!(self, Self::Unsatisfied(_))
    }
}

/// Full conformance report for one compiled descriptor's IR payload.
#[derive(Debug, Clone)]
pub struct ConformanceReport {
    /// §19 `model_requirements` — checked against [`ModelRouter::route`].
    pub model_requirements: CheckResult,
    /// §20 `prompt_dialect` — checked against [`PromptDialect::detect`].
    pub prompt_dialect: CheckResult,
    /// §22 `context_strategy` — checked against [`apply_strategy`].
    pub context_strategy: CheckResult,
    /// The model resolved while checking `model_requirements` (or from an
    /// explicit deployment-profile pin), used as the input to the
    /// `prompt_dialect` check. `None` if nothing could be resolved.
    pub resolved_model: Option<String>,
}

impl ConformanceReport {
    /// `true` iff every section is [`CheckResult::NotDeclared`] or
    /// [`CheckResult::Satisfied`].
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.model_requirements.is_ok() && self.prompt_dialect.is_ok() && self.context_strategy.is_ok()
    }
}

/// Run the full conformance check against a compiled descriptor's IR.
///
/// `router` should be the same [`ModelRouter`] instance (backed by the same
/// [`crate::llm::registry::ProviderRegistry`]) a real request would use —
/// conformance is only meaningful against the actual configured deployment,
/// not an idealized one.
pub async fn check_conformance(ir: &AgentDescriptorIR, router: &ModelRouter) -> ConformanceReport {
    let (model_requirements, router_resolved_model) =
        check_model_requirements(&ir.model_requirements, router).await;

    // An explicit deployment-profile model pin takes priority over the
    // router's pick as "the resolved model" for the dialect check below — an
    // operator who pinned a model means that pin, not whatever the router
    // would otherwise choose for the declared capability floor.
    let resolved_model = ir
        .deployment
        .profiles
        .iter()
        .find_map(|p| p.provider.as_ref().map(|pc| pc.model.clone()))
        .or(router_resolved_model);

    let prompt_dialect = check_prompt_dialect(&ir.prompt_dialect, resolved_model.as_deref());
    let context_strategy = check_context_strategy(&ir.context_strategy);

    ConformanceReport {
        model_requirements,
        prompt_dialect,
        context_strategy,
        resolved_model,
    }
}

async fn check_model_requirements(
    section: &ModelRequirementsSection,
    router: &ModelRouter,
) -> (CheckResult, Option<String>) {
    if *section == ModelRequirementsSection::default() {
        return (CheckResult::NotDeclared, None);
    }
    let requirements = RouteRequirements {
        needs_tools: section.needs_tools,
        needs_reasoning: section.needs_reasoning,
        needs_vision: section.needs_vision,
        needs_structured_output: section.needs_structured_output,
        min_context: section.min_context,
        max_cost_per_1m_input: section.max_cost_per_1m_input,
        preferred_provider: section.preferred_provider.clone(),
    };
    match router.route(&requirements).await {
        Some(model) => (
            CheckResult::Satisfied(format!("model_requirements satisfiable: routed to {model}")),
            Some(model),
        ),
        None => (
            CheckResult::Unsatisfied(
                "no configured, healthy provider/model satisfies the declared model_requirements"
                    .into(),
            ),
            None,
        ),
    }
}

fn check_prompt_dialect(section: &PromptDialectSection, resolved_model: Option<&str>) -> CheckResult {
    let Some(declared) = section.dialect.as_deref() else {
        return CheckResult::NotDeclared;
    };
    let Some(model) = resolved_model else {
        return CheckResult::Unsatisfied(
            "prompt_dialect declares an explicit override but no model could be resolved \
             (model_requirements unsatisfiable and no deployment profile pins one) to check it against"
                .into(),
        );
    };
    let detected = PromptDialect::detect(model);
    if detected.name() == declared {
        CheckResult::Satisfied(format!(
            "declared dialect '{declared}' matches PromptDialect::detect(\"{model}\")"
        ))
    } else {
        CheckResult::Unsatisfied(format!(
            "declared dialect '{declared}' does not match PromptDialect::detect(\"{model}\") = '{}'",
            detected.name()
        ))
    }
}

/// Convert the declared IR section into the runtime [`ContextStrategy`]
/// type, filling any unset field with the exact same default the runtime
/// itself uses (via the `pub(crate) default_*` functions in
/// `uar::context::strategy` — the single source of truth, not a duplicated
/// magic number).
fn to_runtime_strategy(section: &ContextStrategySection) -> Option<ContextStrategy> {
    match section {
        ContextStrategySection::Auto => None,
        ContextStrategySection::None => Some(ContextStrategy::None),
        ContextStrategySection::SlidingWindow { max_messages } => {
            Some(ContextStrategy::SlidingWindow {
                max_messages: max_messages.unwrap_or_else(default_max_messages),
            })
        }
        ContextStrategySection::Summarize {
            threshold,
            summary_max_tokens,
        } => Some(ContextStrategy::Summarize {
            threshold: threshold.unwrap_or_else(default_summarize_threshold),
            summary_max_tokens: summary_max_tokens.unwrap_or_else(default_summary_max_tokens),
            model: None,
        }),
        ContextStrategySection::TruncateMiddle {
            keep_first,
            keep_last,
        } => Some(ContextStrategy::TruncateMiddle {
            keep_first: keep_first.unwrap_or_else(default_keep_first),
            keep_last: keep_last.unwrap_or_else(default_keep_last),
        }),
        ContextStrategySection::Hierarchical {
            short_term_turns,
            mid_term_summary_tokens,
            long_term_facts_tokens,
        } => Some(ContextStrategy::Hierarchical {
            short_term_turns: short_term_turns.unwrap_or_else(default_short_term_turns),
            mid_term_summary_tokens: mid_term_summary_tokens.unwrap_or_else(default_mid_term_tokens),
            long_term_facts_tokens: long_term_facts_tokens.unwrap_or_else(default_long_term_tokens),
        }),
    }
}

/// The number of messages a 10-message synthetic transcript should be
/// trimmed to by each concrete strategy — derived independently of
/// `apply_strategy`'s own logic so the test is a real check, not a tautology.
fn expected_trim_len(strategy: &ContextStrategy, total: usize) -> usize {
    match strategy {
        ContextStrategy::None => total,
        ContextStrategy::SlidingWindow { max_messages } => total.min(*max_messages),
        ContextStrategy::TruncateMiddle {
            keep_first,
            keep_last,
        } => {
            let total_keep = keep_first + keep_last;
            if total <= total_keep { total } else { total_keep }
        }
        // `apply_strategy` deliberately falls back to a 50-message sliding
        // window for Summarize/Hierarchical (see its own doc comment) — a
        // 10-message transcript is under that fallback's threshold either way.
        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => total,
        ContextStrategy::Auto => unreachable!("Auto is filtered out before this is called"),
    }
}

fn check_context_strategy(section: &ContextStrategySection) -> CheckResult {
    // `Auto` is this section's parsed default, and is indistinguishable from
    // "not declared" (see module doc) — the runtime's own model-aware
    // selection (CH-05) applies, with nothing fixed declared to conform to.
    let Some(runtime_strategy) = to_runtime_strategy(section) else {
        return CheckResult::NotDeclared;
    };

    let messages: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            serde_json::json!({
                "role": if i % 2 == 0 { "user" } else { "assistant" },
                "content": format!("message {i}"),
            })
        })
        .collect();
    let expected = expected_trim_len(&runtime_strategy, messages.len());
    let trimmed = apply_strategy(&messages, &runtime_strategy);

    if trimmed.len() == expected {
        CheckResult::Satisfied(format!(
            "declared context_strategy honored: apply_strategy() produced {} of {} messages as expected",
            trimmed.len(),
            messages.len()
        ))
    } else {
        CheckResult::Unsatisfied(format!(
            "declared context_strategy NOT honored: apply_strategy() produced {} message(s), expected {expected}",
            trimmed.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::config::LlmConfig;
    use crate::llm::registry::ProviderRegistry;
    use crate::uar::compiler::parser::{minimal_agent_md, parse};

    /// Build an IR from `minimal_agent_md()` plus the given v2-section YAML
    /// appended as new `##` headings (parser accepts sections in any order).
    fn ir_with_v2(extra_headings_yaml: &str) -> AgentDescriptorIR {
        let md = format!("{}\n{extra_headings_yaml}", minimal_agent_md());
        parse(&md).unwrap_or_else(|e| panic!("fixture failed to parse: {e}"))
    }

    async fn router_with_anthropic() -> ModelRouter {
        let registry = ProviderRegistry::new();
        registry
            .seed_from_llm_config(&LlmConfig {
                model: "anthropic/claude-opus-4-8".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            })
            .await;
        ModelRouter::new(Arc::new(registry))
    }

    #[tokio::test]
    async fn all_default_sections_are_not_declared() {
        let ir = ir_with_v2("");
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert_eq!(report.model_requirements, CheckResult::NotDeclared);
        assert_eq!(report.prompt_dialect, CheckResult::NotDeclared);
        assert_eq!(report.context_strategy, CheckResult::NotDeclared);
        assert!(report.all_satisfied());
    }

    #[tokio::test]
    async fn model_requirements_satisfiable_with_configured_provider() {
        let ir = ir_with_v2(
            r#"
## Model Requirements
```yaml
needs_tools: true
```
"#,
        );
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(report.model_requirements, CheckResult::Satisfied(_)));
        // The router picks the cheapest tools-capable Anthropic model, not
        // necessarily the exact one `router_with_anthropic()` configured —
        // seeding a provider makes its whole catalog available.
        assert!(report.resolved_model.as_deref().is_some_and(|m| m.starts_with("anthropic/")));
    }

    #[tokio::test]
    async fn model_requirements_unsatisfiable_with_no_matching_provider() {
        // No provider is configured at all -> nothing can satisfy any
        // declared requirement.
        let ir = ir_with_v2(
            r#"
## Model Requirements
```yaml
needs_tools: true
needs_vision: true
```
"#,
        );
        let router = ModelRouter::new(Arc::new(ProviderRegistry::new()));
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(
            report.model_requirements,
            CheckResult::Unsatisfied(_)
        ));
        assert!(!report.all_satisfied());
    }

    #[tokio::test]
    async fn prompt_dialect_matches_resolved_model() {
        let ir = ir_with_v2(
            r#"
## Model Requirements
```yaml
needs_tools: true
```

## Prompt Dialect
```yaml
dialect: "anthropic_xml"
```
"#,
        );
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(report.prompt_dialect, CheckResult::Satisfied(_)));
    }

    #[tokio::test]
    async fn prompt_dialect_mismatch_is_unsatisfied() {
        let ir = ir_with_v2(
            r#"
## Model Requirements
```yaml
needs_tools: true
```

## Prompt Dialect
```yaml
dialect: "openai_json"
```
"#,
        );
        // Registry only has an Anthropic model configured -> router resolves
        // to it -> declared "openai_json" does not match.
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(
            report.prompt_dialect,
            CheckResult::Unsatisfied(_)
        ));
        assert!(!report.all_satisfied());
    }

    #[tokio::test]
    async fn prompt_dialect_declared_without_resolvable_model_is_unsatisfied() {
        let ir = ir_with_v2(
            r#"
## Prompt Dialect
```yaml
dialect: "anthropic_xml"
```
"#,
        );
        // No model_requirements declared and no deployment profile pin ->
        // nothing to resolve a model from.
        let router = ModelRouter::new(Arc::new(ProviderRegistry::new()));
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(
            report.prompt_dialect,
            CheckResult::Unsatisfied(_)
        ));
    }

    #[tokio::test]
    async fn context_strategy_sliding_window_honored() {
        let ir = ir_with_v2(
            r#"
## Context Strategy
```yaml
type: "sliding_window"
max_messages: 3
```
"#,
        );
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        match &report.context_strategy {
            CheckResult::Satisfied(detail) => assert!(detail.contains('3')),
            other => panic!("expected Satisfied, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn context_strategy_truncate_middle_honored() {
        let ir = ir_with_v2(
            r#"
## Context Strategy
```yaml
type: "truncate_middle"
keep_first: 2
keep_last: 2
```
"#,
        );
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert!(matches!(
            report.context_strategy,
            CheckResult::Satisfied(_)
        ));
    }

    #[tokio::test]
    async fn context_strategy_auto_is_not_declared() {
        let ir = ir_with_v2(
            r#"
## Context Strategy
```yaml
type: "auto"
```
"#,
        );
        let router = router_with_anthropic().await;
        let report = check_conformance(&ir, &router).await;
        assert_eq!(report.context_strategy, CheckResult::NotDeclared);
    }

    #[test]
    fn check_result_is_ok() {
        assert!(CheckResult::NotDeclared.is_ok());
        assert!(CheckResult::Satisfied("x".into()).is_ok());
        assert!(!CheckResult::Unsatisfied("x".into()).is_ok());
    }
}
