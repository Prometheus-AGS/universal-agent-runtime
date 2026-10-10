use serde::Deserialize;
use serde_json::Value;

use super::service::CollaborationError;
use crate::uar::{
    domain::{
        collaboration::{ConversionDisposition, FieldDiagnostic, SkillRef},
        reviewed_skill_coverage::ReviewedSkillCoverage,
    },
    runtime::skills::reviewed_coverage,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Extension {
    required: bool,
    value: Claims,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Claims {
    schema_version: u32,
    entries: Vec<Claim>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Claim {
    skill_ref: SkillRef,
    source: String,
    closure_digest: String,
    inventory_digest: String,
    location_digest: String,
    generation_digest: Option<String>,
    signer_key_id: Option<String>,
    trust_root_digest: Option<String>,
}

impl Claim {
    fn matches(&self, coverage: &ReviewedSkillCoverage) -> bool {
        self.source == coverage.source
            && self.closure_digest == coverage.closure_digest
            && self.inventory_digest == coverage.inventory_digest
            && self.location_digest == coverage.location_digest
            && self.generation_digest == coverage.generation_digest
            && self.signer_key_id == coverage.signer_key_id
            && self.trust_root_digest == coverage.trust_root_digest
    }
}

pub(super) async fn resolve(
    reviews: &mut reviewed_coverage::ReviewSession,
    binding: &Value,
    skill: &SkillRef,
    location: &str,
    pointer: &str,
    diagnostics: &mut Vec<FieldDiagnostic>,
) -> Result<(bool, Option<ReviewedSkillCoverage>), CollaborationError> {
    let extension = binding["extensions"].get(reviewed_coverage::EXTENSION);
    let Some(extension) = extension else {
        // Prior bindings retain their original admission semantics. New Boss
        // authoring requires this extension; an old host marker is not a migration.
        // Keep legacy receipt bytes stable during member revalidation. Absence
        // of reviewedCoverage means unreviewed, as the live catalog states.
        return Ok((true, None));
    };
    let extension: Extension = serde_json::from_value(extension.clone())?;
    if !extension.required || extension.value.schema_version != 1 {
        return Err(CollaborationError::Invalid(
            "invalid reviewed skill coverage extension".to_owned(),
        ));
    }
    let mut matches = extension
        .value
        .entries
        .iter()
        .filter(|claim| claim.skill_ref == *skill);
    let claim = matches.next();
    if matches.next().is_some() {
        return Err(CollaborationError::Invalid(
            "duplicate reviewed skill coverage claim".to_owned(),
        ));
    }
    let verified = reviews.verify(skill, location).await;
    let coverage = match (claim, verified) {
        (Some(claim), Ok(Some(coverage))) if claim.matches(&coverage) => Some(coverage),
        _ => None,
    };
    if coverage.is_none() {
        diagnostics.push(diagnostic(
            pointer,
            if skill.required {
                ConversionDisposition::RequiredUnsupported
            } else {
                ConversionDisposition::OptionalUnsupported
            },
            "skill.reviewed-coverage-invalid",
            "Selected skill lacks matching trusted coverage of its current installed closure.",
        ));
        return Ok((false, None));
    }
    diagnostics.push(diagnostic(
        pointer,
        ConversionDisposition::Exact,
        "skill.reviewed-closure-verified",
        "Current installed skill closure matches the independently trusted host baseline.",
    ));
    Ok((true, coverage))
}

fn diagnostic(
    pointer: &str,
    disposition: ConversionDisposition,
    code: &str,
    message: &str,
) -> FieldDiagnostic {
    FieldDiagnostic {
        pointer: pointer.to_owned(),
        disposition,
        reason_code: code.to_owned(),
        message: message.to_owned(),
        effective_binding_ref: None,
        source_kind: None,
        source_definition: None,
    }
}
