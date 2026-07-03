//! Loads SKILL.md manifests from the active Prometheus Skill Pack root into
//! the [`SkillService`] as `kind = Manifest`, `origin = Builtin` skills.
//!
//! Root resolution: [`super::pack_detection::resolve_skill_pack_root`]
//! (CH-16 skill-pack-bundling, docs/uar-next-fable.md §6.2) — env override,
//! developer sibling checkout, installed plugin, or the embedded submodule
//! floor, in that precedence order.
//!
//! Discovery rules:
//! - Walks the resolved root plus any extra roots from
//!   `UAR_EXTRA_BUILTIN_SKILL_DIRS` (colon-separated, lower precedence than
//!   the resolved primary root).
//! - Picks up every `SKILL.md` whose path does not include `imported/` unless
//!   `UAR_LOAD_IMPORTED_SKILLS=true`.
//! - Parses the full agentskills.io frontmatter (§6.3 item 1): `license`,
//!   `authors`, `language`, `compatibility`, `metadata.tags/category`,
//!   `model_routing`, in addition to the fields the loader already read.
//! - Preserves nested-skill parent/child relationships (§6.3 item 3).
//! - Collision policy is precedence-wins, not last-wins (§6.3 item 6): the
//!   first root in precedence order to define a name keeps it; later roots'
//!   same-named skills are dropped with a warning, UNLESS the collision is
//!   listed in the pack's `scripts/skill-collision-allowlist.json`.
//!
//! NOT implemented in this pass (see docs/uar-next-fable.md §6.3 for the
//! full item list) — each is either a separate concern or too risky to wire
//! in without the compile/integration-test verification this pass
//! deliberately defers (implementation-first-then-test):
//! - §6.3 item 2 (progressive disclosure / lazy body load) — `prompt_overlay`
//!   is still read eagerly. Changing this ripples into every consumer of
//!   `Skill.prompt_overlay`; needs its own focused pass.
//! - §6.3 item 4 (pack MCP servers) — [`detect_pack_mcp_servers`] parses and
//!   logs which servers the pack's `.mcp.json` declares, but does NOT
//!   register them into UAR's live MCP registry yet.
//! - §6.3 item 5 (hooks mapping) — explicitly out of scope; not attempted at
//!   all (matches the plan's own "or explicitly document as unsupported;
//!   don't half-load").
//! - §6.3 item 7 (activation metrics) — already shipped separately as
//!   CH-08 skill-activation-metrics.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::uar::domain::skills::{
    Skill, SkillConstraints, SkillExecutionConfig, SkillKind, SkillModelRouting, SkillOrigin,
    SkillTriggers,
};

use super::pack_detection::{self, PackProvenance};

/// Frontmatter shape we pull out of `SKILL.md`. Skill-system uses several
/// optional fields beyond these; unrecognized ones are ignored (`serde`
/// default struct behavior — no `deny_unknown_fields`).
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
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    compatibility: Option<serde_json::Value>,
    #[serde(default)]
    metadata: Option<MetadataFrontmatter>,
    #[serde(default)]
    model_routing: Option<ModelRoutingFrontmatter>,
}

#[derive(Debug, Default, Deserialize)]
struct TriggersFrontmatter {
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    semantic: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct MetadataFrontmatter {
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    category: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ModelRoutingFrontmatter {
    #[serde(default)]
    policy_source: Option<String>,
    #[serde(default)]
    phases: HashMap<String, String>,
    #[serde(default)]
    routing_reference: Option<String>,
}

/// A single allowlisted collision that should not be logged as a warning.
/// Reused from the pack's own `scripts/skill-collision-allowlist.json` —
/// each entry's `skills` names the (possibly repeated, per the pack's own
/// convention for a same-name-different-path collision) skill title(s)
/// this entry covers.
#[derive(Debug, Deserialize)]
struct AllowlistEntry {
    skills: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    reason: String,
}

/// Returns the primary builtin-skills directory, honouring `UAR_BUILTIN_SKILLS_DIR`.
///
/// Kept for backward compatibility with any external callers/tests that
/// depended on the raw path; [`discover_builtin_skills`] uses
/// [`pack_detection::resolve_skill_pack_root`] internally, which applies the
/// full §6.2 precedence (this function only reflects level 1's env var and
/// level 4's embedded-submodule floor, matching this function's pre-CH-16
/// behavior exactly).
pub fn builtin_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_BUILTIN_SKILLS_DIR") {
        return PathBuf::from(s);
    }
    PathBuf::from("crates/prometheus-skill-system/skills")
}

/// Returns all skill roots: the resolved primary root (§6.2 precedence) plus
/// any extras from `UAR_EXTRA_BUILTIN_SKILL_DIRS` (colon-separated list of
/// additional lower-precedence paths to scan).
///
/// Example:
/// ```text
/// UAR_EXTRA_BUILTIN_SKILL_DIRS=crates/kreuzberg/skills:/opt/org-skills
/// ```
pub fn all_builtin_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![pack_detection::resolve_skill_pack_root().root];
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

/// Load the pack's `scripts/skill-collision-allowlist.json` (relative to the
/// skill-pack root's parent), if present, and return the flat set of skill
/// titles it covers. Returns an empty set (not an error) when absent or
/// malformed — the allowlist only suppresses warning noise, it's never
/// load-bearing for correctness.
fn load_collision_allowlist(pack_root: &Path) -> HashSet<String> {
    let Some(pack_parent) = pack_root.parent() else {
        return HashSet::new();
    };
    let allowlist_path = pack_parent
        .join("scripts")
        .join("skill-collision-allowlist.json");
    let Ok(raw) = std::fs::read_to_string(&allowlist_path) else {
        return HashSet::new();
    };
    let Ok(entries) = serde_json::from_str::<Vec<AllowlistEntry>>(&raw) else {
        warn!(
            path = %allowlist_path.display(),
            "failed to parse skill-collision-allowlist.json; treating as empty"
        );
        return HashSet::new();
    };

    entries.into_iter().flat_map(|e| e.skills).collect()
}

/// Parse the pack's `.mcp.json` (relative to the skill-pack root's parent)
/// and return the MCP server names it declares. Detection/logging only —
/// does NOT register these into UAR's live MCP registry (see module docs,
/// §6.3 item 4 is intentionally not fully wired in this pass).
#[must_use]
pub fn detect_pack_mcp_servers(pack_root: &Path) -> Vec<String> {
    let Some(pack_parent) = pack_root.parent() else {
        return Vec::new();
    };
    let mcp_json_path = pack_parent.join(".mcp.json");
    let Ok(raw) = std::fs::read_to_string(&mcp_json_path) else {
        return Vec::new();
    };
    match serde_json::from_str::<crate::mcp::config::McpConfig>(&raw) {
        Ok(config) => config.mcp_servers.into_keys().collect(),
        Err(e) => {
            warn!(path = %mcp_json_path.display(), error = %e, "failed to parse pack .mcp.json");
            Vec::new()
        }
    }
}

/// Scan all builtin skill directories and return parsed [`Skill`] structs
/// plus the [`PackProvenance`] of the primary (highest-precedence) root.
///
/// On name collision across roots, the FIRST-seen skill wins (precedence-
/// wins, §6.3 item 6) — roots are scanned in precedence order, so this
/// means the highest-precedence root's skill is kept and lower-precedence
/// duplicates are dropped with a warning, unless the pair is listed in
/// `skill-collision-allowlist.json`.
pub fn discover_builtin_skills() -> (Vec<Skill>, PackProvenance) {
    let provenance = pack_detection::resolve_skill_pack_root();
    let mut dirs = vec![provenance.root.clone()];
    if let Ok(extra) = std::env::var("UAR_EXTRA_BUILTIN_SKILL_DIRS") {
        for path in extra.split(':').filter(|s| !s.is_empty()) {
            dirs.push(PathBuf::from(path));
        }
    }

    let allowlist = load_collision_allowlist(&provenance.root);
    let allow_imported = include_imported();

    // Track skills by directory path (for nested-parent resolution) as well
    // as by name (for collision handling) — WalkDir doesn't guarantee any
    // particular traversal order for siblings, so parent resolution happens
    // as a second pass over ALL discovered paths, not during the walk.
    let mut by_name: HashMap<String, Skill> = HashMap::new();
    let mut paths_by_name: HashMap<String, PathBuf> = HashMap::new();
    let mut root_count = 0usize;
    let mcp_servers_detected = detect_pack_mcp_servers(&provenance.root);

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
                        if !allowlist.contains(&skill.title) {
                            warn!(
                                name = %skill.title,
                                prev_id = %existing.skill_id,
                                new_id  = %skill.skill_id,
                                "builtin skill name collision across roots — first-seen (highest precedence) wins"
                            );
                        }
                        // Precedence-wins: keep the existing (first-seen,
                        // higher-precedence) entry, drop this one.
                        continue;
                    }
                    paths_by_name.insert(skill.title.clone(), path.to_path_buf());
                    by_name.insert(skill.title.clone(), skill);
                }
                Err(err) => {
                    warn!(path = %path.display(), error = %err, "failed to load builtin skill");
                }
            }
        }
    }

    resolve_nested_parents(&mut by_name, &paths_by_name);

    let out: Vec<Skill> = by_name.into_values().collect();
    info!(
        count = out.len(),
        roots = root_count,
        pack_source = ?provenance.source,
        pack_root = %provenance.root.display(),
        pack_version = provenance.version.as_deref().unwrap_or("<unknown>"),
        mcp_servers_detected = mcp_servers_detected.join(","),
        "discovered builtin manifest skills"
    );
    (out, provenance)
}

/// §6.3 item 3: for each skill, find the closest ancestor directory that is
/// also the directory of another discovered skill, and record that skill's
/// `skill_id` as `parent_skill_id`. Top-level skills (no such ancestor) keep
/// `parent_skill_id: None`.
fn resolve_nested_parents(
    by_name: &mut HashMap<String, Skill>,
    paths_by_name: &HashMap<String, PathBuf>,
) {
    // (skill directory, skill_id) pairs, so each skill can search for the
    // deepest OTHER skill directory that is a strict ancestor of its own.
    let dirs_with_ids: Vec<(PathBuf, String)> = paths_by_name
        .iter()
        .filter_map(|(name, path)| {
            let dir = path.parent()?.to_path_buf();
            let skill_id = by_name.get(name)?.skill_id.clone();
            Some((dir, skill_id))
        })
        .collect();

    let mut parent_by_name: HashMap<String, String> = HashMap::new();
    for (name, path) in paths_by_name {
        let Some(own_dir) = path.parent() else {
            continue;
        };
        let mut best: Option<(&Path, &str)> = None;
        for (other_dir, other_id) in &dirs_with_ids {
            if other_dir == own_dir {
                continue;
            }
            if !own_dir.starts_with(other_dir) {
                continue;
            }
            // Deepest (longest path) ancestor wins.
            let is_deeper = best.is_none_or(|(best_dir, _)| {
                other_dir.as_path().components().count() > best_dir.components().count()
            });
            if is_deeper {
                best = Some((other_dir.as_path(), other_id.as_str()));
            }
        }
        if let Some((_, parent_id)) = best {
            parent_by_name.insert(name.clone(), parent_id.to_string());
        }
    }

    for (name, parent_id) in parent_by_name {
        if let Some(skill) = by_name.get_mut(&name) {
            skill.parent_skill_id = Some(parent_id);
        }
    }
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

    let model_routing = meta.model_routing.map(|mr| SkillModelRouting {
        policy_source: mr.policy_source,
        phases: mr.phases,
        routing_reference: mr.routing_reference,
    });

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
        license: meta.license,
        authors: meta.authors,
        language: meta.language,
        compatibility: meta.compatibility,
        metadata_tags: meta
            .metadata
            .as_ref()
            .map(|m| m.tags.clone())
            .unwrap_or_default(),
        metadata_category: meta.metadata.and_then(|m| m.category),
        model_routing,
        parent_skill_id: None,
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

        let (skills, provenance) = discover_builtin_skills();

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
        assert_eq!(
            provenance.source,
            crate::uar::runtime::skills::pack_detection::PackSource::EnvOverride
        );
    }

    #[test]
    fn full_frontmatter_fields_are_parsed() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("rich-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: rich-skill
description: test skill with full frontmatter
license: MIT
authors:
  - Someone
language: rust
compatibility: requires foo ^1.0
model_routing:
  policy_source: some-config
  phases:
    my-phase: small
  routing_reference: references/routing.md
metadata:
  tags: [a, b]
  category: testing
---
body"#,
        )
        .unwrap();

        // SAFETY: test-only, single-threaded context.
        unsafe {
            std::env::set_var("UAR_BUILTIN_SKILLS_DIR", "/tmp/__nonexistent_uar_skills__");
            std::env::set_var("UAR_EXTRA_BUILTIN_SKILL_DIRS", dir.path().to_str().unwrap());
        }
        let (skills, _) = discover_builtin_skills();
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
            std::env::remove_var("UAR_EXTRA_BUILTIN_SKILL_DIRS");
        }

        let skill = skills
            .into_iter()
            .find(|s| s.title == "rich-skill")
            .unwrap();
        assert_eq!(skill.license.as_deref(), Some("MIT"));
        assert_eq!(skill.authors, vec!["Someone".to_string()]);
        assert_eq!(skill.language.as_deref(), Some("rust"));
        assert_eq!(skill.metadata_tags, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(skill.metadata_category.as_deref(), Some("testing"));
        let routing = skill.model_routing.expect("model_routing should be parsed");
        assert_eq!(
            routing.phases.get("my-phase").map(String::as_str),
            Some("small")
        );
    }

    #[test]
    fn nested_skill_records_parent() {
        let dir = TempDir::new().unwrap();
        let parent_dir = dir.path().join("parent-skill");
        fs::create_dir_all(&parent_dir).unwrap();
        fs::write(
            parent_dir.join("SKILL.md"),
            "---\nname: parent-skill\ndescription: parent\n---\nbody",
        )
        .unwrap();
        let child_dir = parent_dir.join("skills").join("child-skill");
        fs::create_dir_all(&child_dir).unwrap();
        fs::write(
            child_dir.join("SKILL.md"),
            "---\nname: child-skill\ndescription: child\n---\nbody",
        )
        .unwrap();

        // SAFETY: test-only, single-threaded context.
        unsafe {
            std::env::set_var("UAR_BUILTIN_SKILLS_DIR", "/tmp/__nonexistent_uar_skills__");
            std::env::set_var("UAR_EXTRA_BUILTIN_SKILL_DIRS", dir.path().to_str().unwrap());
        }
        let (skills, _) = discover_builtin_skills();
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
            std::env::remove_var("UAR_EXTRA_BUILTIN_SKILL_DIRS");
        }

        let parent = skills.iter().find(|s| s.title == "parent-skill").unwrap();
        let child = skills.iter().find(|s| s.title == "child-skill").unwrap();
        assert_eq!(
            child.parent_skill_id.as_deref(),
            Some(parent.skill_id.as_str())
        );
        assert_eq!(parent.parent_skill_id, None);
    }
}
