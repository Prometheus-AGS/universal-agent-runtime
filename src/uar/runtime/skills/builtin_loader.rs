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

/// Returns the primary builtin-skills directory, honouring `UAR_BUILTIN_SKILLS_DIR`.
pub fn builtin_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_BUILTIN_SKILLS_DIR") {
        return PathBuf::from(s);
    }
    PathBuf::from("crates/prometheus-skill-system/skills")
}

/// Returns all skill roots: primary dir plus any extras from `UAR_EXTRA_BUILTIN_SKILL_DIRS`
/// (colon-separated list of additional paths to scan).
///
/// Example:
/// ```text
/// UAR_EXTRA_BUILTIN_SKILL_DIRS=crates/kreuzberg/skills:/opt/org-skills
/// ```
pub fn all_builtin_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![builtin_dir()];
    if let Ok(extra) = std::env::var("UAR_EXTRA_BUILTIN_SKILL_DIRS") {
        for path in extra.split(':').filter(|s| !s.is_empty()) {
            dirs.push(PathBuf::from(path));
        }
    }
    dirs
}

fn include_imported() -> bool {
    std::env::var("UAR_LOAD_IMPORTED_SKILLS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Scan all builtin skill directories and return parsed [`Skill`] structs.
///
/// Scans `UAR_BUILTIN_SKILLS_DIR` (default `crates/prometheus-skill-system/skills`)
/// plus any directories listed in `UAR_EXTRA_BUILTIN_SKILL_DIRS` (colon-separated).
/// On name collision across roots, the last-seen skill wins and a warning is logged.
pub fn discover_builtin_skills() -> Vec<Skill> {
    let dirs = all_builtin_dirs();
    let allow_imported = include_imported();
    let mut by_name: std::collections::HashMap<String, Skill> = std::collections::HashMap::new();
    let mut root_count = 0usize;

    for dir in &dirs {
        if !dir.exists() {
            debug!(
                path = %dir.display(),
                "builtin skills directory not found; skipping"
            );
            continue;
        }
        root_count += 1;
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "SKILL.md")
        {
            let path = entry.path();
            if !allow_imported && path.components().any(|c| c.as_os_str() == "imported") {
                continue;
            }
            match load_one(path) {
                Ok(skill) => {
                    if let Some(existing) = by_name.get(&skill.title) {
                        warn!(
                            name = %skill.title,
                            prev_id = %existing.skill_id,
                            new_id  = %skill.skill_id,
                            "builtin skill name collision across roots — new entry wins"
                        );
                    }
                    by_name.insert(skill.title.clone(), skill);
                }
                Err(err) => {
                    warn!(path = %path.display(), error = %err, "failed to load builtin skill");
                }
            }
        }
    }

    let out: Vec<Skill> = by_name.into_values().collect();
    info!(
        count = out.len(),
        roots = root_count,
        "discovered builtin manifest skills"
    );
    out
}

fn load_one(path: &Path) -> Result<Skill> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
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
    let body = raw
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow::anyhow!("SKILL.md missing leading `---` frontmatter delimiter"))?;
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn split_frontmatter_works() {
        let raw = "---\nname: foo\ndescription: bar\n---\nbody text";
        let (yaml, body) = split_frontmatter(raw).unwrap();
        assert!(yaml.contains("name: foo"));
        assert_eq!(body, "body text");
    }

    #[test]
    fn discover_extra_root_loads_skill() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: test skill\n---\nbody",
        )
        .unwrap();

        // Point UAR_EXTRA_BUILTIN_SKILL_DIRS at our temp dir.
        // UAR_BUILTIN_SKILLS_DIR to a non-existent path so only the extra root fires.
        // SAFETY: test-only, single-threaded context.
        unsafe {
            std::env::set_var("UAR_BUILTIN_SKILLS_DIR", "/tmp/__nonexistent_uar_skills__");
            std::env::set_var("UAR_EXTRA_BUILTIN_SKILL_DIRS", dir.path().to_str().unwrap());
        }

        let skills = discover_builtin_skills();

        // SAFETY: test-only cleanup.
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
            std::env::remove_var("UAR_EXTRA_BUILTIN_SKILL_DIRS");
        }

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].title, "my-skill");
        assert_eq!(
            skills[0].origin,
            crate::uar::domain::skills::SkillOrigin::Builtin
        );
    }
}
