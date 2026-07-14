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
    Some((candidate_root.join("skills"), manifest.map(|m| m.version)))
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
    let version = root
        .parent()
        .and_then(read_plugin_manifest)
        .map(|m| m.version);
    PackProvenance {
        source: PackSource::EmbeddedSubmodule,
        root,
        version,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
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
        assert_eq!(
            provenance.root,
            PathBuf::from("/tmp/__uar_pack_detection_test__")
        );
    }

    #[test]
    fn missing_sibling_manifest_does_not_match() {
        // Point PROMETHEUS_SKILL_SYSTEM_DIR at a path that definitely has no
        // plugin.json, rather than relying on `../prometheus-skill-system`
        // not existing relative to cwd — on a dev box with sibling checkouts
        // of other Prometheus repos (this one included), that relative path
        // can genuinely resolve to a real sibling with a valid manifest, so
        // asserting "no sibling checkout" without controlling this env var
        // is not hermetic (observed: false failure on exactly such a box).
        // SAFETY: test-only; ensure a clean slate for env vars this test reads.
        unsafe {
            std::env::remove_var("UAR_BUILTIN_SKILLS_DIR");
            std::env::set_var(
                "PROMETHEUS_SKILL_SYSTEM_DIR",
                "/tmp/__uar_pack_detection_no_such_sibling__",
            );
        }
        let provenance = resolve_skill_pack_root();
        unsafe {
            std::env::remove_var("PROMETHEUS_SKILL_SYSTEM_DIR");
        }
        assert_ne!(provenance.source, PackSource::SiblingCheckout);
    }

    fn write_plugin_manifest(dir: &Path, name: &str, version: &str) {
        std::fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
        std::fs::write(
            dir.join(".claude-plugin").join("plugin.json"),
            format!(r#"{{"name": "{name}", "version": "{version}"}}"#),
        )
        .unwrap();
    }

    #[test]
    fn sibling_checkout_found_via_prometheus_skill_system_dir_env_var() {
        let dir = tempfile::TempDir::new().unwrap();
        let sibling_root = dir.path().join("sibling");
        std::fs::create_dir_all(sibling_root.join("skills")).unwrap();
        write_plugin_manifest(&sibling_root, "prometheus-skill-pack", "9.9.9");

        // SAFETY: test-only, single-threaded env var mutation.
        unsafe {
            std::env::set_var(
                "PROMETHEUS_SKILL_SYSTEM_DIR",
                sibling_root.to_str().unwrap(),
            );
        }
        let found = find_sibling_checkout();
        unsafe {
            std::env::remove_var("PROMETHEUS_SKILL_SYSTEM_DIR");
        }

        let (root, version) = found.expect("sibling checkout with a valid manifest must match");
        assert_eq!(root, sibling_root.join("skills"));
        assert_eq!(version.as_deref(), Some("9.9.9"));
    }

    #[test]
    fn sibling_checkout_without_skills_dir_does_not_match() {
        let dir = tempfile::TempDir::new().unwrap();
        // Manifest present, but no `skills/` subdirectory — not a valid pack root.
        write_plugin_manifest(dir.path(), "prometheus-skill-pack", "1.0.0");

        // SAFETY: test-only, single-threaded env var mutation.
        unsafe {
            std::env::set_var("PROMETHEUS_SKILL_SYSTEM_DIR", dir.path().to_str().unwrap());
        }
        let found = find_sibling_checkout();
        unsafe {
            std::env::remove_var("PROMETHEUS_SKILL_SYSTEM_DIR");
        }

        assert!(found.is_none());
    }

    #[test]
    fn installed_plugin_prefers_highest_version_among_matches() {
        let fake_home = tempfile::TempDir::new().unwrap();
        let cache_root = fake_home.path().join(".claude/plugins/cache");

        for version in ["1.0.0", "1.5.0", "1.2.0"] {
            let version_dir = cache_root.join("prometheus-skill-pack").join(version);
            std::fs::create_dir_all(version_dir.join("skills")).unwrap();
            write_plugin_manifest(&version_dir, "prometheus-skill-pack", version);
        }
        // A same-named-org plugin that ISN'T the skill pack must be ignored
        // even if its version would otherwise "win".
        let other_plugin_dir = cache_root.join("some-other-plugin").join("9.0.0");
        std::fs::create_dir_all(other_plugin_dir.join("skills")).unwrap();
        write_plugin_manifest(&other_plugin_dir, "some-other-plugin", "9.0.0");

        // HOME is process-global and potentially read by unrelated
        // concurrently-running tests elsewhere in this binary (unlike
        // UAR_BUILTIN_SKILLS_DIR/PROMETHEUS_SKILL_SYSTEM_DIR, which only
        // this module's tests touch) — save and restore the real value
        // rather than removing it.
        // SAFETY: test-only env var mutation.
        let real_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", fake_home.path().to_str().unwrap());
        }
        let found = find_installed_plugin();
        unsafe {
            match &real_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }

        let (root, version) = found.expect("highest-version prometheus-skill-pack must be found");
        assert_eq!(version, "1.5.0");
        assert_eq!(
            root,
            cache_root
                .join("prometheus-skill-pack")
                .join("1.5.0")
                .join("skills")
        );
    }
}
