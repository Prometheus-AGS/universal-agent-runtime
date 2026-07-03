//! Skill-pack root detection (CH-16 skill-pack-bundling, docs/uar-next-fable.md §6.2).
//!
//! Resolves which copy of the `prometheus-skill-pack` skill tree is active at
//! startup, in precedence order, and records where it came from so operators
//! can see which pack is live (§6.2's "(pack_version, source, root_path)").
//!
//! Precedence (highest first):
//! 1. `UAR_BUILTIN_SKILLS_DIR` env override — existing behavior, unchanged.
//! 2. Developer sibling checkout — `$PROMETHEUS_SKILL_SYSTEM_DIR` or
//!    `../prometheus-skill-system`, detected by `.claude-plugin/plugin.json`.
//! 3. Installed plugin — platform-install caches (today: `~/.claude/plugins/cache/*/*`;
//!    other harnesses use different paths, see [`INSTALLED_PLUGIN_SEARCH_ROOTS`]).
//!    Prefers the highest semver-ish version among matches named
//!    `"prometheus-skill-pack"`.
//! 4. Embedded submodule (`crates/prometheus-skill-system/skills`) — the
//!    guaranteed floor.
//! 5. Optional network fetch — NOT implemented. §6.2 scopes this as "only
//!    behind an explicit flag; never at first startup (local-first rule)";
//!    fetching + verifying a release artifact is a separate, security-
//!    sensitive change, not a loader-detection change. [`resolve_skill_pack_root`]
//!    falls through past level 4 without attempting it.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Which precedence level produced the resolved skill-pack root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackSource {
    /// `UAR_BUILTIN_SKILLS_DIR` env override.
    EnvOverride,
    /// Developer sibling checkout (`.claude-plugin/plugin.json` detected).
    SiblingCheckout,
    /// Installed plugin cache (highest version among matches).
    InstalledPlugin,
    /// Embedded submodule — the guaranteed floor when nothing else resolves.
    EmbeddedSubmodule,
}

/// Where the active skill-pack root came from, and its declared version.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PackProvenance {
    pub source: PackSource,
    pub root: PathBuf,
    /// From `.claude-plugin/plugin.json`'s `version` field, when found.
    /// `EnvOverride`/`EmbeddedSubmodule` roots may not carry one (a raw
    /// directory override, or a submodule checkout without the plugin
    /// manifest at the expected relative path).
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
}

/// Read `<dir>/.claude-plugin/plugin.json` and return `(name, version)` if
/// present and parseable. Never errors — a missing/malformed manifest just
/// means this candidate doesn't count as a plugin root.
fn read_plugin_manifest(dir: &Path) -> Option<PluginManifest> {
    let manifest_path = dir.join(".claude-plugin").join("plugin.json");
    let raw = std::fs::read_to_string(manifest_path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Very loose version compare: split on `.`, compare numeric segments
/// left-to-right, fall back to string compare on a non-numeric segment.
/// Good enough for `MAJOR.MINOR.PATCH`-style plugin versions; this is not a
/// full semver implementation (no pre-release/build metadata handling) —
/// none of the versions this pack has shipped need that.
fn version_gt(a: &str, b: &str) -> bool {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();
    for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
        match (ap.parse::<u64>(), bp.parse::<u64>()) {
            (Ok(an), Ok(bn)) if an != bn => return an > bn,
            (Ok(_), Ok(_)) => continue,
            _ if ap != bp => return ap > bp,
            _ => continue,
        }
    }
    a_parts.len() > b_parts.len()
}

/// Platform install-cache roots to search for level 3 (installed plugin).
/// Each entry is globbed one level deep for `<root>/<plugin-dir>/<version>/`.
/// `~/.claude/plugins/cache/` is the confirmed real layout (Claude Code);
/// the others are documented conventions from docs/uar-next-fable.md §6.2
/// that this repo hasn't independently verified against a live install —
/// harmless to check and skip if absent.
fn installed_plugin_search_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![
        home.join(".claude/plugins/cache"),
        home.join(".claude-code/skills"),
        home.join(".config/uar/skills"),
    ]
}

/// Level 3: scan installed-plugin cache roots for the highest-version
/// `prometheus-skill-pack` plugin whose directory has a `skills/` subdir.
fn find_installed_plugin() -> Option<(PathBuf, String)> {
    let mut best: Option<(PathBuf, String)> = None;

    for search_root in installed_plugin_search_roots() {
        let Ok(plugin_dirs) = std::fs::read_dir(&search_root) else {
            continue;
        };
        for plugin_dir in plugin_dirs.filter_map(|e| e.ok()) {
            let Ok(version_dirs) = std::fs::read_dir(plugin_dir.path()) else {
                continue;
            };
            for version_dir in version_dirs.filter_map(|e| e.ok()) {
                let candidate = version_dir.path();
                let Some(manifest) = read_plugin_manifest(&candidate) else {
                    continue;
                };
                if manifest.name != "prometheus-skill-pack" {
                    continue;
                }
                if !candidate.join("skills").is_dir() {
                    continue;
                }
                let is_better = best
                    .as_ref()
                    .is_none_or(|(_, best_version)| version_gt(&manifest.version, best_version));
                if is_better {
                    best = Some((candidate.join("skills"), manifest.version));
                }
            }
        }
    }

    best
}

/// Level 2: developer sibling checkout, detected by `.claude-plugin/plugin.json`
/// one directory up from the `skills/` dir.
fn find_sibling_checkout() -> Option<(PathBuf, Option<String>)> {
    let candidate_root = if let Ok(dir) = std::env::var("PROMETHEUS_SKILL_SYSTEM_DIR") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("../prometheus-skill-system")
    };

    if !candidate_root.join("skills").is_dir() {
        return None;
    }
    let manifest = read_plugin_manifest(&candidate_root);
    if manifest.is_none() {
        // §6.2 requires detection BY presence of the plugin manifest — a
        // `../prometheus-skill-system` that happens to exist without one
        // isn't confirmed to be this pack, so don't treat it as a match.
        return None;
    }
    Some((
        candidate_root.join("skills"),
        manifest.map(|m| m.version),
    ))
}

/// Level 4: the embedded submodule — always available as a path (existence
/// is checked by the caller, same as every other root).
fn embedded_submodule_dir() -> PathBuf {
    PathBuf::from("crates/prometheus-skill-system/skills")
}

/// Resolve the active skill-pack root per the §6.2 precedence order.
///
/// Always returns a [`PackProvenance`] even when the resolved directory
/// doesn't exist on disk (falls through to level 4 as the documented floor)
/// — callers decide what to do with a non-existent root (existing behavior:
/// log and skip discovery).
#[must_use]
pub fn resolve_skill_pack_root() -> PackProvenance {
    if let Ok(dir) = std::env::var("UAR_BUILTIN_SKILLS_DIR") {
        let root = PathBuf::from(dir);
        let version = root
            .parent()
            .and_then(read_plugin_manifest)
            .map(|m| m.version);
        return PackProvenance {
            source: PackSource::EnvOverride,
            root,
            version,
        };
    }

    if let Some((root, version)) = find_sibling_checkout() {
        return PackProvenance {
            source: PackSource::SiblingCheckout,
            root,
            version,
        };
    }

    if let Some((root, version)) = find_installed_plugin() {
        return PackProvenance {
            source: PackSource::InstalledPlugin,
            root,
            version: Some(version),
        };
    }

    let root = embedded_submodule_dir();
    let version = root.parent().and_then(read_plugin_manifest).map(|m| m.version);
    PackProvenance {
        source: PackSource::EmbeddedSubmodule,
        root,
        version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gt_compares_numeric_segments() {
        assert!(version_gt("1.5.1", "1.5.0"));
        assert!(version_gt("2.0.0", "1.9.9"));
        assert!(!version_gt("1.5.0", "1.5.0"));
        assert!(!version_gt("1.4.9", "1.5.0"));
        assert!(version_gt("1.5.0.1", "1.5.0"));
    }

    #[test]
    fn env_override_wins_regardless_of_other_roots() {
        // SAFETY: test-only, single-threaded env var mutation.
        unsafe {
            std::env::set_var("UAR_BUILTIN_SKILLS_DIR", "/tmp/__uar_pack_detection_test__");
        }
        let provenance = resolve_skill_pack_root();
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
        }
        assert_eq!(provenance.source, PackSource::EnvOverride);
        assert_eq!(provenance.root, PathBuf::from("/tmp/__uar_pack_detection_test__"));
    }

    #[test]
    fn missing_sibling_manifest_does_not_match() {
        // No env override, no real sibling checkout in a test sandbox —
        // this should fall through to embedded submodule without panicking.
        // SAFETY: test-only; ensure a clean slate for env vars this test reads.
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
            std::env::remove_var("PROMETHEUS_SKILL_SYSTEM_DIR");
        }
        let provenance = resolve_skill_pack_root();
        assert_ne!(provenance.source, PackSource::SiblingCheckout);
    }
}
