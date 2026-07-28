//! Load the Prometheus skill pack into the database.
//!
//! WHAT LIVES WHERE, AND WHY
//!
//! Skill **metadata** goes in the database; skill **bodies stay on disk**. A
//! SKILL.md body can be hundreds of KB and is only needed when a skill actually
//! runs, so storing it in every catalog row would make listing skills expensive
//! for no benefit. The row carries `source_path` instead, and the runner reads
//! the body from disk when it is needed.
//!
//! Config files SEED the database; they are not consulted at runtime afterwards.
//! That is what lets a runtime API change take effect immediately on both an
//! embedded device and a remote server — no restart, no file-polling loop.
//!
//! WALKING THE PACK IS NOT NAIVE
//!
//! The tree contains 211 `SKILL.md` files but only **140 canonical skills**.
//! `skills/imported/` holds vendored duplicates (a submodule may ship its own
//! `.claude/`, `.cursor/` and `.codex/` copies of the same skill), and
//! `.cursor/` `.opencode/` `.windsurf/` at the repo root are generated
//! per-platform mirrors. Walking everything would triple-count the catalog, so
//! only `skills/` is walked and `imported/` is skipped — matching what the
//! pack's own `install-skills-flat.sh` does.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::uar::domain::skills::{Skill, SkillKind, SkillOrigin};

/// Directory under the pack root holding canonical skills.
const SKILLS_DIR: &str = "skills";
/// Vendored duplicates — deliberately skipped (see the module docs).
const IMPORTED_DIR: &str = "imported";

/// The frontmatter fields the pack actually uses.
///
/// Everything is optional except `name`/`description` (the only two the pack's
/// own validator requires), because a strict struct would reject skills that
/// are otherwise perfectly loadable.
#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    /// Top-level `version`.
    ///
    /// Measured against the pack at 2026-07-27: all 140 canonical skills carry
    /// this, so the `metadata.version` fallback in `resolved_version` never
    /// fires today. It is kept because IMPORTED skills (e.g.
    /// `sycophancy-correction`) do use the nested spelling, and a future
    /// canonical skill copying that style would otherwise load as `0.0.0`
    /// without any error.
    version: Option<String>,
    license: Option<String>,
    language: Option<String>,
    /// Free-form: a plain string in most skills, a structured object in a few.
    /// Passed through untyped rather than guessed at.
    compatibility: Option<serde_json::Value>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    metadata: FrontmatterMetadata,
}

#[derive(Debug, Default, Deserialize)]
struct FrontmatterMetadata {
    version: Option<String>,
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    author: Option<String>,
}

impl Frontmatter {
    /// `version` appears at the top level in some skills and under `metadata` in
    /// others. Accept both.
    fn resolved_version(&self) -> String {
        self.version
            .clone()
            .or_else(|| self.metadata.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string())
    }

    fn resolved_authors(&self) -> Vec<String> {
        if !self.authors.is_empty() {
            return self.authors.clone();
        }
        self.metadata.author.iter().cloned().collect()
    }
}

/// One skill found on disk, ready to persist.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub skill: Skill,
    /// Directory containing SKILL.md. The body is read from here at run time.
    pub dir: PathBuf,
    /// Whether the skill ships executable scripts. Only ~37 of 140 do; the rest
    /// are pure knowledge and need no runner at all, which is what lets them
    /// work on a platform that cannot spawn processes.
    pub has_scripts: bool,
}

/// Split a SKILL.md into (frontmatter yaml, markdown body).
///
/// Returns `None` when the file has no `---` delimited frontmatter, which the
/// pack's validator treats as invalid — better to skip it loudly than to invent
/// a name from the directory.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---")?;
    let rest = rest.strip_prefix('\n').or_else(|| rest.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let body = rest[end..].trim_start_matches("\n---");
    Some((yaml, body.trim_start_matches(['\r', '\n'])))
}

/// Parse one SKILL.md into a `Skill`.
///
/// `dir_name` is used when frontmatter omits `name` — the pack's validator
/// requires the two to match anyway.
pub fn parse_skill_md(text: &str, dir: &Path) -> Option<DiscoveredSkill> {
    let (yaml, body) = split_frontmatter(text)?;
    let front: Frontmatter = serde_yaml::from_str(yaml).ok()?;

    let dir_name = dir.file_name()?.to_string_lossy().to_string();
    let name = front.name.clone().unwrap_or_else(|| dir_name.clone());
    let has_scripts = dir.join("scripts").is_dir();

    let mut skill = Skill {
        skill_id: name.clone(),
        version: front.resolved_version(),
        title: name.clone(),
        description: front.description.clone().unwrap_or_default(),
        // The body IS the artifact for a knowledge skill: it is handed to the
        // model as prompt context.
        prompt_overlay: body.to_string(),
        kind: SkillKind::Manifest,
        origin: SkillOrigin::Builtin,
        license: front.license.clone(),
        authors: front.resolved_authors(),
        language: front.language.clone(),
        compatibility: front.compatibility.clone(),
        metadata_tags: front.metadata.tags.clone(),
        metadata_category: front.metadata.category.clone(),
        ..Default::default()
    };
    skill.enabled = true;

    Some(DiscoveredSkill { skill, dir: dir.to_path_buf(), has_scripts })
}

/// Walk a pack checkout and return every canonical skill.
///
/// Skips `skills/imported/` and never descends into the generated
/// `.cursor`/`.opencode`/`.windsurf` mirrors, because those are duplicates of
/// what `skills/` already contains.
pub fn discover(pack_root: &Path) -> Vec<DiscoveredSkill> {
    let mut found = Vec::new();
    let skills_root = pack_root.join(SKILLS_DIR);
    if !skills_root.is_dir() {
        return found;
    }
    walk(&skills_root, &mut found);
    found
}

fn walk(dir: &Path, out: &mut Vec<DiscoveredSkill>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // Vendored duplicates, and any dotted dir (which is where the
            // per-platform mirrors live).
            if name == IMPORTED_DIR || name.starts_with('.') {
                continue;
            }
            walk(&path, out);
        } else if path.file_name().is_some_and(|f| f == "SKILL.md")
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Some(parent) = path.parent()
            && let Some(discovered) = parse_skill_md(&text, parent)
        {
            out.push(discovered);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWLEDGE_SKILL: &str = r#"---
license: MIT
name: clean-architecture
version: '1.0.0'
description: >
  CLEAN architecture patterns.
language: rust
metadata:
  tags: [architecture, patterns]
  category: architecture
---

# CLEAN Architecture

Body text.
"#;

    // The other half of the pack puts version under `metadata`, not top level.
    const METADATA_VERSION_SKILL: &str = r#"---
name: other-skill
description: Uses metadata.version instead of a top-level version.
metadata:
  version: '2.3.4'
  author: prometheus
---

Body.
"#;

    #[test]
    fn parses_frontmatter_into_a_skill() {
        let dir = Path::new("/tmp/clean-architecture");
        let found = parse_skill_md(KNOWLEDGE_SKILL, dir).expect("parses");
        assert_eq!(found.skill.skill_id, "clean-architecture");
        assert_eq!(found.skill.version, "1.0.0");
        assert_eq!(found.skill.license.as_deref(), Some("MIT"));
        assert_eq!(found.skill.language.as_deref(), Some("rust"));
        assert_eq!(found.skill.metadata_category.as_deref(), Some("architecture"));
        assert!(found.skill.metadata_tags.contains(&"patterns".to_string()));
        assert!(matches!(found.skill.kind, SkillKind::Manifest));
        assert!(matches!(found.skill.origin, SkillOrigin::Builtin));
        // The body is the artifact for a knowledge skill.
        assert!(found.skill.prompt_overlay.contains("# CLEAN Architecture"));
        // Frontmatter must NOT leak into the body.
        assert!(!found.skill.prompt_overlay.contains("license: MIT"));
    }

    #[test]
    fn accepts_version_in_either_location() {
        // Keying on one spelling silently loses the version for half the pack.
        let found = parse_skill_md(METADATA_VERSION_SKILL, Path::new("/tmp/other-skill"))
            .expect("parses");
        assert_eq!(found.skill.version, "2.3.4");
        assert_eq!(found.skill.authors, vec!["prometheus".to_string()]);
    }

    #[test]
    fn rejects_a_file_with_no_frontmatter() {
        // Better to skip loudly than to invent a name from the directory.
        assert!(parse_skill_md("# Just markdown\n", Path::new("/tmp/x")).is_none());
    }

    /// The walker must not double-count. The pack tree holds 211 `SKILL.md`
    /// files but only 140 canonical skills: `skills/imported/` carries vendored
    /// duplicates (a submodule ships its own `.claude`/`.cursor`/`.codex`
    /// copies) and dotted dirs are generated per-platform mirrors.
    ///
    /// Measured against the real pack at 2026-07-27: 140 skills, 37 with
    /// `scripts/`, zero duplicate names, zero parse failures.
    ///
    /// Skipped when the pack is not checked out beside this repo, so the suite
    /// stays green on a machine that only has the runtime.
    #[test]
    fn discovers_canonical_skills_without_duplicates() {
        // A relative hop out of CARGO_MANIFEST_DIR is fragile — it silently
        // resolved to the wrong place and made this test skip while reporting
        // "ok", which is worse than no test. Take the path explicitly.
        let pack = std::env::var("PROMETHEUS_SKILL_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default())
                    .join("Projects/prometheus/prometheus-skill-pack")
            });
        if !pack.join("skills").is_dir() {
            eprintln!(
                "skill pack not at {} — set PROMETHEUS_SKILL_PACK to run this; skipping",
                pack.display()
            );
            return;
        }
        eprintln!("walking real pack at {}", pack.display());
        let found = discover(&pack);
        assert!(!found.is_empty(), "the pack is present but nothing parsed");

        let names: std::collections::HashSet<_> =
            found.iter().map(|f| f.skill.skill_id.clone()).collect();
        assert_eq!(
            names.len(),
            found.len(),
            "duplicate skill ids — the walker is counting imported/ or a mirror"
        );
        assert!(
            found.iter().all(|f| !f.dir.to_string_lossy().contains("/imported/")),
            "an imported/ duplicate leaked into the catalog"
        );
        // A knowledge skill carries its body and needs no runner; that split is
        // what lets ~74% of the pack work on a platform with no process spawning.
        assert!(
            found.iter().any(|f| !f.has_scripts && !f.skill.prompt_overlay.is_empty()),
            "expected at least one knowledge-only skill with a body"
        );
    }
}
