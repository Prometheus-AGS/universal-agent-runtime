//! Redacted manifest for one assembled model turn.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{Authority, PromptFragment, PromptRole, PromptSection, Retention, render};

/// Model-visible fragment metadata. Prompt bodies cannot be represented here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFragment {
    pub id: String,
    pub section: PromptSection,
    pub source: String,
    pub authority: Authority,
    pub role: PromptRole,
    pub retention: Retention,
    pub content_hash: String,
}

impl From<&PromptFragment> for ManifestFragment {
    fn from(fragment: &PromptFragment) -> Self {
        Self {
            id: fragment.id.clone(),
            section: fragment.section,
            source: fragment.source.clone(),
            authority: fragment.authority,
            role: fragment.role,
            retention: fragment.retention,
            content_hash: fragment.content_hash.clone(),
        }
    }
}

/// Fragment counts grouped by stable section and authority names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentCounts {
    pub total: usize,
    pub by_section: BTreeMap<String, usize>,
    pub by_authority: BTreeMap<String, usize>,
}

impl FragmentCounts {
    fn from_fragments(fragments: &[ManifestFragment]) -> Self {
        let mut by_section = BTreeMap::new();
        let mut by_authority = BTreeMap::new();

        for fragment in fragments {
            *by_section
                .entry(fragment.section.as_str().to_string())
                .or_insert(0) += 1;
            *by_authority
                .entry(fragment.authority.as_str().to_string())
                .or_insert(0) += 1;
        }

        Self {
            total: fragments.len(),
            by_section,
            by_authority,
        }
    }
}

/// Observable prompt usage and configured token ceilings for one turn.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptBudgets {
    pub rendered_bytes: usize,
    pub rendered_characters: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<usize>,
}

impl PromptBudgets {
    #[must_use]
    pub fn for_fragments(fragments: &[PromptFragment]) -> Self {
        let rendered = render(fragments);
        Self::for_rendered(&rendered)
    }

    #[must_use]
    pub fn for_rendered(rendered: &str) -> Self {
        Self {
            rendered_bytes: rendered.len(),
            rendered_characters: rendered.chars().count(),
            context_window_tokens: None,
            max_output_tokens: None,
        }
    }
}

/// Redacted, deterministic record of the context assembled for one turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnManifest {
    pub schema_version: u32,
    pub manifest_hash: String,
    pub fragments: Vec<ManifestFragment>,
    pub counts: FragmentCounts,
    pub budgets: PromptBudgets,
    pub selected_skills: Vec<String>,
    pub selected_tools: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<crate::uar::runtime::turn::shadow::ShadowReport>,
}

impl TurnManifest {
    #[must_use]
    pub fn from_fragments(
        fragments: &[PromptFragment],
        mut budgets: PromptBudgets,
        selected_skills: impl IntoIterator<Item = String>,
        selected_tools: impl IntoIterator<Item = String>,
        warnings: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut manifest_fragments = fragments
            .iter()
            .map(ManifestFragment::from)
            .collect::<Vec<_>>();
        manifest_fragments.sort_by(|left, right| {
            left.section
                .cmp(&right.section)
                .then_with(|| left.id.cmp(&right.id))
        });

        if budgets.rendered_bytes == 0 && budgets.rendered_characters == 0 {
            let measured = PromptBudgets::for_fragments(fragments);
            budgets.rendered_bytes = measured.rendered_bytes;
            budgets.rendered_characters = measured.rendered_characters;
        }

        let mut selected_skills = selected_skills.into_iter().collect::<Vec<_>>();
        selected_skills.sort();
        selected_skills.dedup();

        let mut selected_tools = selected_tools.into_iter().collect::<Vec<_>>();
        selected_tools.sort();
        selected_tools.dedup();

        let mut warnings = warnings.into_iter().collect::<Vec<_>>();
        warnings.sort();
        warnings.dedup();

        let counts = FragmentCounts::from_fragments(&manifest_fragments);
        let manifest_hash = manifest_hash(
            &manifest_fragments,
            &counts,
            &budgets,
            &selected_skills,
            &selected_tools,
            &warnings,
        );

        Self {
            schema_version: 1,
            manifest_hash,
            fragments: manifest_fragments,
            counts,
            budgets,
            selected_skills,
            selected_tools,
            warnings,
            shadow: None,
        }
    }

    pub fn with_shadow(mut self, report: crate::uar::runtime::turn::shadow::ShadowReport) -> Self {
        let base = manifest_hash(
            &self.fragments,
            &self.counts,
            &self.budgets,
            &self.selected_skills,
            &self.selected_tools,
            &self.warnings,
        );
        self.manifest_hash = crate::uar::runtime::turn::shadow::fingerprint(&serde_json::json!({
            "base_manifest": base, "shadow": &report,
        }));
        self.shadow = Some(report);
        self
    }
}

fn manifest_hash(
    fragments: &[ManifestFragment],
    counts: &FragmentCounts,
    budgets: &PromptBudgets,
    selected_skills: &[String],
    selected_tools: &[String],
    warnings: &[String],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"uar.turn_manifest.v1\0");

    for fragment in fragments {
        update_hash(&mut hasher, &fragment.id);
        update_hash(&mut hasher, fragment.section.as_str());
        update_hash(&mut hasher, &fragment.source);
        update_hash(&mut hasher, fragment.authority.as_str());
        update_hash(&mut hasher, fragment.role.as_str());
        update_hash(&mut hasher, retention_name(fragment.retention));
        update_hash(&mut hasher, &fragment.content_hash);
    }
    update_hash(&mut hasher, &counts.total.to_string());
    for (key, value) in &counts.by_section {
        update_hash(&mut hasher, key);
        update_hash(&mut hasher, &value.to_string());
    }
    for (key, value) in &counts.by_authority {
        update_hash(&mut hasher, key);
        update_hash(&mut hasher, &value.to_string());
    }
    update_hash(&mut hasher, &budgets.rendered_bytes.to_string());
    update_hash(&mut hasher, &budgets.rendered_characters.to_string());
    update_optional_usize(&mut hasher, budgets.context_window_tokens);
    update_optional_usize(&mut hasher, budgets.max_output_tokens);
    for value in selected_skills {
        update_hash(&mut hasher, value);
    }
    for value in selected_tools {
        update_hash(&mut hasher, value);
    }
    for value in warnings {
        update_hash(&mut hasher, value);
    }

    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn update_hash(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_le_bytes());
    hasher.update(value.as_bytes());
}

fn update_optional_usize(hasher: &mut Sha256, value: Option<usize>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            update_hash(hasher, &value.to_string());
        }
        None => hasher.update([0]),
    }
}

const fn retention_name(retention: Retention) -> &'static str {
    match retention {
        Retention::Session => "session",
        Retention::Turn => "turn",
        Retention::Ephemeral => "ephemeral",
        Retention::Reclaimable => "reclaimable",
    }
}
