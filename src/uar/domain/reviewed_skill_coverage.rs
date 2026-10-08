//! Private installed-byte evidence; never a portable skill or effect grant.

use serde::{Deserialize, Serialize};

/// Identity verified against the owning host's independently provisioned source.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedSkillCoverage {
    pub source: String,
    pub closure_digest: String,
    pub inventory_digest: String,
    pub location_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_root_digest: Option<String>,
}

/// Live catalog assessment. Persisted skill metadata cannot certify itself.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCoverageAssessment {
    pub status: &'static str,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<ReviewedSkillCoverage>,
}
