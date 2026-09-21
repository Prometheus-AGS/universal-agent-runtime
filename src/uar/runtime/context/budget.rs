//! Pure destination budget planning.
//!
//! The planner never edits request data. Callers identify any host-marked prose
//! spans separately; everything else belongs in `protected_request`.

use super::token_service::{CountQuality, CountedTokens, SerializedCountingContract};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Synthetic destination used by deterministic request-boundary fixtures.
pub const SYNTHETIC_EXACT_MODEL: &str = "openai/uar-exact-cl100k-json-v1";

/// Destination limits used by the pure planner.
///
/// Signed values make invalid negative configuration representable so it is
/// rejected explicitly instead of being hidden by saturating arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationLimits {
    pub context_tokens: i64,
    pub independent_input_tokens: Option<i64>,
    pub host_input_tokens: Option<i64>,
    pub output_tokens: i64,
    pub additional_reasoning_tokens: i64,
    pub count_uncertainty_tokens: i64,
}

/// A contiguous prose span that trusted-host metadata has marked eligible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibleProseSpan {
    pub id: String,
    pub count: CountedTokens,
}

/// Immutable inputs to [`plan_budget`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetInput {
    pub limits: DestinationLimits,
    /// Complete protected request cost, including its fixed framing.
    pub protected_request: CountedTokens,
    /// Only prose carrying explicit trusted-host eligibility metadata.
    pub eligible_prose: Vec<EligibleProseSpan>,
}

/// A successful arithmetic plan. It contains no transformed request data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetPlan {
    pub limits: DestinationLimits,
    pub input_allowance: u64,
    pub protected_tokens: u64,
    pub eligible_budget: u64,
    pub eligible_tokens: u64,
    pub eligible_span_ids: Vec<String>,
    pub counting_revision: String,
}

/// Exhaustive outcome from the pure planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetOutcome {
    Fit(BudgetPlan),
    CompressEligible {
        plan: BudgetPlan,
        target_tokens: u64,
    },
    ProtectedOverflow {
        required_tokens: u64,
        input_allowance: u64,
    },
    InvalidLimits {
        field: &'static str,
        value: i64,
    },
    UnsupportedCount {
        subject: String,
        quality: CountQuality,
        revision: String,
    },
}

/// Wire field that must carry the reserved output ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputCeilingField {
    MaxTokens,
    MaxCompletionTokens,
}

/// Final wire representation whose framing has been proven for counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireContract {
    /// Liter's generic OpenAI-compatible chat JSON for an explicitly bound
    /// base URL, after stream insertion and `extra_body` merging.
    SyntheticOpenAiCompatibleChatV1,
}

/// Immutable contract carried to the final production request boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestBudgetContract {
    pub destination_model: String,
    pub destination_endpoint_fingerprint: String,
    pub limits: DestinationLimits,
    pub counting: SerializedCountingContract,
    pub wire: WireContract,
    pub output_ceiling: OutputCeilingField,
}

impl RequestBudgetContract {
    /// Deterministic exact contract for the supported synthetic fixture at
    /// one explicit base URL.
    #[must_use]
    pub fn synthetic_exact(base_url: &str) -> Self {
        Self {
            destination_model: SYNTHETIC_EXACT_MODEL.to_string(),
            destination_endpoint_fingerprint: endpoint_fingerprint(base_url),
            limits: DestinationLimits {
                context_tokens: 8_192,
                independent_input_tokens: Some(7_680),
                host_input_tokens: None,
                output_tokens: 512,
                additional_reasoning_tokens: 0,
                count_uncertainty_tokens: 0,
            },
            counting: SerializedCountingContract::ExactCl100kJsonV1,
            wire: WireContract::SyntheticOpenAiCompatibleChatV1,
            output_ceiling: OutputCeilingField::MaxCompletionTokens,
        }
    }
}

/// Stable, non-secret identity for an explicitly configured endpoint.
///
/// Liter's builder removes trailing slashes, so the fingerprint mirrors that
/// normalization. The raw URL is never carried by the request contract.
#[must_use]
pub fn endpoint_fingerprint(base_url: &str) -> String {
    let normalized = base_url.trim_end_matches('/');
    let digest = Sha256::digest(normalized.as_bytes());
    let mut fingerprint = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut fingerprint, "{byte:02x}").expect("writing to a String cannot fail");
    }
    fingerprint
}

fn reject_negative(field: &'static str, value: i64) -> Option<BudgetOutcome> {
    (value < 0).then_some(BudgetOutcome::InvalidLimits { field, value })
}

fn require_bounded(subject: impl Into<String>, count: &CountedTokens) -> Option<BudgetOutcome> {
    (count.revision.trim().is_empty()
        || !matches!(
            count.quality,
            CountQuality::Exact | CountQuality::ValidatedUpperBound
        ))
    .then(|| BudgetOutcome::UnsupportedCount {
        subject: subject.into(),
        quality: count.quality,
        revision: count.revision.clone(),
    })
}

/// Compute `I = min(L, H, C - O - R - M)` and `B = I - F` without I/O or
/// request mutation.
#[must_use]
pub fn plan_budget(input: &BudgetInput) -> BudgetOutcome {
    let limits = &input.limits;
    for (field, value) in [
        ("context_tokens", limits.context_tokens),
        ("output_tokens", limits.output_tokens),
        (
            "additional_reasoning_tokens",
            limits.additional_reasoning_tokens,
        ),
        ("count_uncertainty_tokens", limits.count_uncertainty_tokens),
    ] {
        if let Some(outcome) = reject_negative(field, value) {
            return outcome;
        }
    }
    for (field, value) in [
        ("independent_input_tokens", limits.independent_input_tokens),
        ("host_input_tokens", limits.host_input_tokens),
    ] {
        if let Some(value) = value
            && let Some(outcome) = reject_negative(field, value)
        {
            return outcome;
        }
    }

    let Some(reserves) = limits
        .output_tokens
        .checked_add(limits.additional_reasoning_tokens)
        .and_then(|value| value.checked_add(limits.count_uncertainty_tokens))
    else {
        return BudgetOutcome::InvalidLimits {
            field: "reserve_sum",
            value: i64::MAX,
        };
    };
    if reserves > limits.context_tokens {
        return BudgetOutcome::InvalidLimits {
            field: "reserve_sum",
            value: reserves,
        };
    }

    if let Some(outcome) = require_bounded("protected_request", &input.protected_request) {
        return outcome;
    }
    for span in &input.eligible_prose {
        if let Some(outcome) = require_bounded(format!("eligible_prose:{}", span.id), &span.count) {
            return outcome;
        }
    }

    let mut input_allowance = limits.context_tokens - reserves;
    if let Some(limit) = limits.independent_input_tokens {
        input_allowance = input_allowance.min(limit);
    }
    if let Some(limit) = limits.host_input_tokens {
        input_allowance = input_allowance.min(limit);
    }
    let input_allowance = u64::try_from(input_allowance).expect("non-negative limits validated");

    if input.protected_request.tokens > input_allowance {
        return BudgetOutcome::ProtectedOverflow {
            required_tokens: input.protected_request.tokens,
            input_allowance,
        };
    }
    let eligible_budget = input_allowance - input.protected_request.tokens;
    let Some(eligible_tokens) = input
        .eligible_prose
        .iter()
        .try_fold(0_u64, |total, span| total.checked_add(span.count.tokens))
    else {
        return BudgetOutcome::InvalidLimits {
            field: "eligible_token_sum",
            value: i64::MAX,
        };
    };
    let plan = BudgetPlan {
        limits: limits.clone(),
        input_allowance,
        protected_tokens: input.protected_request.tokens,
        eligible_budget,
        eligible_tokens,
        eligible_span_ids: input
            .eligible_prose
            .iter()
            .map(|span| span.id.clone())
            .collect(),
        counting_revision: input.protected_request.revision.clone(),
    };
    if eligible_tokens <= eligible_budget {
        BudgetOutcome::Fit(plan)
    } else {
        BudgetOutcome::CompressEligible {
            plan,
            target_tokens: eligible_budget,
        }
    }
}

/// Recount the complete final serialization and prove it fits the plan.
#[must_use]
pub fn validate_final_count(plan: &BudgetPlan, final_request: &CountedTokens) -> BudgetOutcome {
    if let Some(outcome) = require_bounded("final_serialized_request", final_request) {
        return outcome;
    }
    if final_request.tokens > plan.input_allowance {
        return BudgetOutcome::ProtectedOverflow {
            required_tokens: final_request.tokens,
            input_allowance: plan.input_allowance,
        };
    }
    let mut final_plan = plan.clone();
    final_plan.protected_tokens = final_request.tokens;
    final_plan.eligible_budget = plan.input_allowance - final_request.tokens;
    final_plan.eligible_tokens = 0;
    final_plan.eligible_span_ids.clear();
    final_plan.counting_revision = final_request.revision.clone();
    BudgetOutcome::Fit(final_plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(tokens: u64) -> CountedTokens {
        CountedTokens {
            tokens,
            quality: CountQuality::Exact,
            revision: "fixture-v1".into(),
        }
    }

    #[test]
    fn formula_uses_independent_limit_and_reserves() {
        let outcome = plan_budget(&BudgetInput {
            limits: DestinationLimits {
                context_tokens: 10_000,
                independent_input_tokens: Some(6_000),
                host_input_tokens: Some(7_000),
                output_tokens: 2_000,
                additional_reasoning_tokens: 0,
                count_uncertainty_tokens: 100,
            },
            protected_request: exact(5_000),
            eligible_prose: Vec::new(),
        });
        let BudgetOutcome::Fit(plan) = outcome else {
            panic!("expected fit");
        };
        assert_eq!(plan.input_allowance, 6_000);
        assert_eq!(plan.eligible_budget, 1_000);
    }

    #[test]
    fn invalid_reserve_and_protected_overflow_are_distinct() {
        let invalid = plan_budget(&BudgetInput {
            limits: DestinationLimits {
                context_tokens: 100,
                independent_input_tokens: None,
                host_input_tokens: None,
                output_tokens: 80,
                additional_reasoning_tokens: 30,
                count_uncertainty_tokens: 0,
            },
            protected_request: exact(1),
            eligible_prose: Vec::new(),
        });
        assert!(matches!(invalid, BudgetOutcome::InvalidLimits { .. }));

        let overflow = plan_budget(&BudgetInput {
            limits: DestinationLimits {
                context_tokens: 100,
                independent_input_tokens: None,
                host_input_tokens: None,
                output_tokens: 20,
                additional_reasoning_tokens: 0,
                count_uncertainty_tokens: 0,
            },
            protected_request: exact(81),
            eligible_prose: Vec::new(),
        });
        assert_eq!(
            overflow,
            BudgetOutcome::ProtectedOverflow {
                required_tokens: 81,
                input_allowance: 80,
            }
        );
    }

    #[test]
    fn approximate_count_cannot_certify_fit() {
        let outcome = plan_budget(&BudgetInput {
            limits: DestinationLimits {
                context_tokens: 100,
                independent_input_tokens: None,
                host_input_tokens: None,
                output_tokens: 20,
                additional_reasoning_tokens: 0,
                count_uncertainty_tokens: 0,
            },
            protected_request: CountedTokens {
                tokens: 10,
                quality: CountQuality::Approximate,
                revision: "fallback".into(),
            },
            eligible_prose: Vec::new(),
        });
        assert!(matches!(outcome, BudgetOutcome::UnsupportedCount { .. }));
    }
}
