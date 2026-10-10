//! Verify current skill closures against launch-provisioned host trust roots.

mod inventory;
mod sources;

use crate::uar::domain::{
    collaboration::SkillRef,
    reviewed_skill_coverage::{ReviewedSkillCoverage, SkillCoverageAssessment},
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) const EXTENSION: &str = "urn:prometheus:uar:reviewed-skill-coverage:1";

/// Add only independently verified full-generation roots to normal discovery.
/// Primary mini and explicit extra roots retain their existing name precedence.
pub(crate) async fn discover_host_builtin_skills() -> (
    Vec<crate::uar::domain::skills::Skill>,
    super::pack_detection::PackProvenance,
) {
    let roots = match sources::verified_full_roots().await {
        Ok(roots) => roots,
        Err(_) => {
            tracing::warn!(
                "Configured signed full skill generation could not be verified; its skills were not added"
            );
            Vec::new()
        }
    };
    super::builtin_loader::discover_builtin_skills_with_verified_roots(&roots)
}

/// One catalog/admission operation shares a parsed inventory and current-byte
/// observations. A new operation starts fresh; no authority cache outlives it.
#[derive(Default)]
pub(crate) struct ReviewSession {
    inventories: BTreeMap<
        std::path::PathBuf,
        Result<(Arc<inventory::PreparedInventory>, ReviewedSkillCoverage), String>,
    >,
}

impl ReviewSession {
    /// No request field or persisted Skill row contributes a trust root.
    pub(crate) async fn assess(
        &mut self,
        skill: &SkillRef,
        location: &str,
    ) -> SkillCoverageAssessment {
        match self.verify(skill, location).await {
            Ok(Some(coverage)) => SkillCoverageAssessment {
                status: "reviewed",
                reason: "verified-current-closure",
                coverage: Some(coverage),
            },
            Ok(None) => SkillCoverageAssessment {
                status: "unreviewed",
                reason: "trusted-source-unavailable",
                coverage: None,
            },
            Err(_) => SkillCoverageAssessment {
                status: "blocked",
                reason: "reviewed-closure-verification-failed",
                coverage: None,
            },
        }
    }

    pub(crate) async fn verify(
        &mut self,
        skill: &SkillRef,
        location: &str,
    ) -> anyhow::Result<Option<ReviewedSkillCoverage>> {
        let Some(source) = sources::select(location)? else {
            return Ok(None);
        };
        if !self.inventories.contains_key(&source.root) {
            let prepared = async {
                let (document, coverage) = source.inventory().await?;
                let inventory = tokio::task::spawn_blocking(move || {
                    inventory::PreparedInventory::new(document)
                })
                .await??;
                Ok::<_, anyhow::Error>((Arc::new(inventory), coverage))
            }
            .await;
            self.inventories
                .insert(source.root.clone(), prepared.map_err(|e| e.to_string()));
        }
        let (document, mut coverage) = self
            .inventories
            .get(&source.root)
            .ok_or_else(|| anyhow::anyhow!("trusted source inventory unavailable"))?
            .clone()
            .map_err(anyhow::Error::msg)?;
        let root = source.root;
        let skill = skill.clone();
        let location = location.to_owned();
        let checked =
            tokio::task::spawn_blocking(move || document.verify(&root, &skill, &location))
                .await??;
        coverage.closure_digest = checked.0;
        coverage.inventory_digest = checked.1;
        coverage.location_digest = checked.2;
        Ok(Some(coverage))
    }
}
