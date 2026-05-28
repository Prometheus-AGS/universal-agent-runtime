//! Loads SKILL.md manifests from the embedded `prometheus-skill-system`
//! submodule into the [`SkillService`] as `kind = Manifest`, `origin = Builtin`
//! skills.
//!
//! Discovery rules:
//! - Walks `$UAR_BUILTIN_SKILLS_DIR` (default `crates/prometheus-skill-system/skills`).
//! - Picks up every `SKILL.md` whose path does not include `imported/` unless
//!   `UAR_LOAD_IMPORTED_SKILLS=true`.
//! - Parses the YAML frontmatter; the markdown body becomes the prompt overlay.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::uar::domain::skills::{
    Skill, SkillConstraints, SkillExecutionConfig, SkillKind, SkillOrigin, SkillTriggers,
};

/// Frontmatter shape we pull out of `SKILL.md`. Skill-system uses several
/// optional fields; we only require name + description + version.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: String,
    #[serde(default)]
    version: Option<String>,
    description: String,
    #[serde(default)]
    triggers: Option<TriggersFrontmatter>,
    #[serde(default, rename = "allowed-tools")]
    allowed_tools: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TriggersFrontmatter {
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    semantic: Option<String>,
}

/// Returns the resolved builtin-skills directory, honouring `UAR_BUILTIN_SKILLS_DIR`.
pub fn builtin_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_BUILTIN_SKILLS_DIR") {
        return PathBuf::from(s);
    }
    PathBuf::from("crates/prometheus-skill-system/skills")
}

fn include_imported() -> bool {
    std::env::var("UAR_LOAD_IMPORTED_SKILLS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Scan the builtin skills directory and return parsed [`Skill`] structs.
pub fn discover_builtin_skills() -> Vec<Skill> {
    let dir = builtin_dir();
    if !dir.exists() {
        debug!(
            path = %dir.display(),
            "builtin skills directory not found; skipping builtin load"
        );
        return Vec::new();
    }
    let allow_imported = include_imported();
    let mut out = Vec::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "SKILL.md")
    {
        let path = entry.path();
        if !allow_imported && path.components().any(|c| c.as_os_str() == "imported") {
            continue;
        }
        match load_one(path) {
            Ok(skill) => out.push(skill),
            Err(err) => {
                warn!(path = %path.display(), error = %err, "failed to load builtin skill");
            }
        }
    }
    info!(count = out.len(), "discovered builtin manifest skills");
    out
}

fn load_one(path: &Path) -> Result<Skill> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let (frontmatter, body) = split_frontmatter(&raw)
        .with_context(|| format!("splitting frontmatter in {}", path.display()))?;
    let meta: Frontmatter = serde_yaml::from_str(frontmatter)
        .with_context(|| format!("parsing yaml in {}", path.display()))?;

    let skill_id = format!("builtin::{}", meta.name);
    let preferred_tools = meta
        .allowed_tools
        .as_deref()
        .map(|s| {
            s.split_whitespace()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let triggers = meta
        .triggers
        .map(|t| SkillTriggers {
            keywords: t.keywords,
            semantic: t.semantic,
            ..Default::default()
        })
        .unwrap_or_default();

    Ok(Skill {
        skill_id,
        version: meta.version.unwrap_or_else(|| "0.0.0".to_string()),
        title: meta.name,
        description: meta.description,
        triggers,
        prompt_overlay: body.to_string(),
        preferred_tools,
        mcp_config: None,
        constraints: SkillConstraints::default(),
        enabled: true,
        provider_id: "builtin".to_string(),
        execution_config: SkillExecutionConfig::default(),
        kind: SkillKind::Manifest,
        origin: SkillOrigin::Builtin,
    })
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str)> {
    let body = raw.strip_prefix("---\n").ok_or_else(|| {
        anyhow::anyhow!("SKILL.md missing leading `---` frontmatter delimiter")
    })?;
    let end = body
        .find("\n---")
        .ok_or_else(|| anyhow::anyhow!("SKILL.md missing trailing `---` frontmatter delimiter"))?;
    let yaml = &body[..end];
    let rest = body[end..]
        .trim_start_matches("\n---")
        .trim_start_matches('\n');
    Ok((yaml, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_works() {
        let raw = "---\nname: foo\ndescription: bar\n---\nbody text";
        let (yaml, body) = split_frontmatter(raw).unwrap();
        assert!(yaml.contains("name: foo"));
        assert_eq!(body, "body text");
    }
}
