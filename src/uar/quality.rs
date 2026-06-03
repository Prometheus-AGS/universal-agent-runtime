//! Response-quality (sycophancy) detection wiring.
//!
//! Runs the local, rule-based `sycophancy-core` detector on completed assistant
//! responses. Detection is sync and makes no LLM or network call. This module
//! exposes a single [`detect`] entry point plus the pure decision helpers it is
//! built from (so the strictness mapping and flag threshold are unit-testable).
//!
//! Detection-only: this does not correct or regenerate responses — flagged
//! results are surfaced via the `SycophancyFlagged` event and metrics.

use sycophancy_core::Strictness;
use sycophancy_core::skill::types::{DetectionResult, Severity};

use crate::config::SycophancyConfig;
use crate::uar::domain::events::SycophancyClassification;

/// Outcome of running detection on a completed response.
#[derive(Debug)]
pub struct SycophancyOutcome {
    /// 0.0 (clean) – 1.0 (fully sycophantic).
    pub score: f32,
    pub has_critical: bool,
    pub correction_mandatory: bool,
    /// True when the response should be flagged (score ≥ threshold or critical).
    pub flagged: bool,
    pub classifications: Vec<SycophancyClassification>,
}

/// Map the configured strictness string to the detector's level.
///
/// Total and forgiving: unknown values (including `"standard"`) map to Standard.
fn strictness_from(s: &str) -> Strictness {
    match s.to_lowercase().as_str() {
        "permissive" => Strictness::Permissive,
        "strict" => Strictness::Strict,
        _ => Strictness::Standard,
    }
}

/// Whether a detection result should be flagged: score at/above the configured
/// threshold, or a critical classification was found.
fn should_flag(result: &DetectionResult, auto_correct_threshold: f32) -> bool {
    result.sycophancy_score >= auto_correct_threshold || result.has_critical
}

/// Lower-case severity label for the serializable classification summary.
fn severity_str(severity: &Severity) -> &'static str {
    match severity {
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

/// Run sycophancy detection on a completed assistant response.
///
/// Returns `None` when detection is disabled or the text is empty/whitespace.
/// Otherwise returns the score, the flag decision, and a compact list of pattern
/// classifications (never the response text). Sync and rule-based — no LLM call.
#[must_use]
pub fn detect(config: &SycophancyConfig, text: &str) -> Option<SycophancyOutcome> {
    if !config.enabled || text.trim().is_empty() {
        return None;
    }

    let detector = sycophancy_core::skill::detector::Detector::new(
        sycophancy_core::config::SkillConfig::default(),
    );
    let strictness = strictness_from(&config.strictness);
    let result = detector.detect(text, &[], &strictness);

    let flagged = should_flag(&result, config.auto_correct_threshold);
    let classifications = result
        .classifications
        .iter()
        .map(|c| SycophancyClassification {
            pattern_id: c.pattern_id.clone(),
            severity: severity_str(&c.severity).to_string(),
            rationale: c.rationale.clone(),
        })
        .collect();

    Some(SycophancyOutcome {
        score: result.sycophancy_score,
        has_critical: result.has_critical,
        correction_mandatory: result.correction_mandatory,
        flagged,
        classifications,
    })
}

#[cfg(test)]
mod tests {
    use super::{should_flag, strictness_from};
    use sycophancy_core::Strictness;
    use sycophancy_core::skill::types::DetectionResult;

    fn result(score: f32, critical: bool) -> DetectionResult {
        DetectionResult {
            sycophancy_score: score,
            classifications: Vec::new(),
            has_critical: critical,
            correction_mandatory: false,
        }
    }

    #[test]
    fn strictness_mapping() {
        assert_eq!(strictness_from("permissive"), Strictness::Permissive);
        assert_eq!(strictness_from("strict"), Strictness::Strict);
        assert_eq!(strictness_from("standard"), Strictness::Standard);
        // Unknown / mixed-case → Standard.
        assert_eq!(strictness_from("StRiCt"), Strictness::Strict);
        assert_eq!(strictness_from("bogus"), Strictness::Standard);
    }

    #[test]
    fn flag_above_threshold() {
        assert!(should_flag(&result(0.6, false), 0.5));
        assert!(should_flag(&result(0.5, false), 0.5)); // at threshold
    }

    #[test]
    fn no_flag_below_threshold() {
        assert!(!should_flag(&result(0.49, false), 0.5));
        assert!(!should_flag(&result(0.0, false), 0.5));
    }

    #[test]
    fn critical_always_flags() {
        // Below threshold but critical → still flagged.
        assert!(should_flag(&result(0.1, true), 0.5));
    }
}
