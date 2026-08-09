//! Which version of the Prometheus Skill Pack is loaded.
//!
//! # Why this exists
//!
//! UAR pinned the skill pack at a commit that was **359 commits and two months
//! stale**, exposing 161 skills where the pack had 220 — and nothing detected
//! it, because nothing recorded which version had been loaded. A consumer
//! cannot notice drift from a version it never reads.
//!
//! # Why it never shells out to git
//!
//! The obvious implementation is `git rev-parse HEAD` against the submodule.
//! That is wrong for the case this runtime exists to serve: **a phone has no
//! git binary and no `.git` directory**, and an embedded build may ship the
//! skills as bundled assets with no repository at all.
//!
//! So the *pack* records its own commit at generation time — the one moment a
//! git binary is guaranteed present — into `SKILLS.md`'s YAML frontmatter. This
//! module only reads that. It is a file read, and works anywhere a file can be
//! read.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Provenance of the loaded skill pack.
///
/// Every field is optional because a pack predating `change-uhe-005` has none
/// of them. An older pack must still load — reporting "unknown" is correct, and
/// refusing to start would make provenance a breaking change rather than an
/// observability one.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackProvenance {
    /// Semver from the pack's `package.json`, e.g. `1.6.0`.
    pub version: Option<String>,
    /// Full 40-char commit SHA the pack was generated at.
    pub commit: Option<String>,
    /// Skills the pack believes it ships, counted the way this loader counts:
    /// every `SKILL.md` outside `imported/`, `node_modules/`, tests, fixtures.
    pub skill_count: Option<usize>,
    /// RFC3339 timestamp of manifest generation.
    pub generated_at: Option<String>,
    /// Where the manifest was read from — so an operator can tell a missing
    /// manifest from one that was found but empty.
    pub manifest_path: Option<PathBuf>,
}

impl PackProvenance {
    /// True when the pack recorded nothing at all.
    ///
    /// Distinguishes "old pack, no manifest" from "manifest present but
    /// unreadable"; both are reported, but they mean different things to
    /// whoever has to fix it.
    pub fn is_unknown(&self) -> bool {
        self.version.is_none() && self.commit.is_none() && self.skill_count.is_none()
    }

    /// Short commit for display. `None` when no commit was recorded.
    pub fn short_commit(&self) -> Option<&str> {
        self.commit.as_deref().map(|c| &c[..c.len().min(10)])
    }
}

/// Frontmatter keys we care about. `SKILLS.md` carries many more (name,
/// platforms, description…); `serde` ignores what is not declared here, so the
/// pack can add keys without breaking this reader.
#[derive(Debug, Deserialize)]
struct ManifestFrontmatter {
    version: Option<String>,
    commit: Option<String>,
    skill_count: Option<usize>,
    generated_at: Option<String>,
}

/// Read provenance from a pack root.
///
/// `pack_root` is the directory *containing* `SKILLS.md` — the parent of the
/// `skills/` directory that [`super::builtin_loader::builtin_dir`] returns.
///
/// Never fails: an absent, unreadable, or malformed manifest yields a
/// `PackProvenance` where [`PackProvenance::is_unknown`] is true. **A skill pack
/// that cannot state its version must still load** — otherwise adding
/// provenance would break every existing deployment.
pub fn read_provenance(pack_root: &Path) -> PackProvenance {
    let manifest = pack_root.join("SKILLS.md");
    let Ok(text) = std::fs::read_to_string(&manifest) else {
        return PackProvenance::default();
    };

    let Some(front) = extract_frontmatter(&text) else {
        // Manifest exists but has no frontmatter: record where we looked, so
        // this is distinguishable from "no manifest".
        return PackProvenance {
            manifest_path: Some(manifest),
            ..Default::default()
        };
    };

    match serde_yaml::from_str::<ManifestFrontmatter>(front) {
        Ok(fm) => PackProvenance {
            version: fm.version,
            commit: fm.commit,
            skill_count: fm.skill_count,
            generated_at: fm.generated_at,
            manifest_path: Some(manifest),
        },
        Err(_) => PackProvenance {
            manifest_path: Some(manifest),
            ..Default::default()
        },
    }
}

/// Return the YAML between the leading `---` fences, or `None`.
fn extract_frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_manifest(dir: &Path, body: &str) {
        let mut f = std::fs::File::create(dir.join("SKILLS.md")).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    /// Unique temp dir per call.
    ///
    /// A nanosecond timestamp is NOT unique enough: these tests run in
    /// parallel, several call this within the same tick, and two of them then
    /// share a directory — so one test's manifest is read by another. That
    /// showed up as a DIFFERENT test failing on each run, which is the
    /// signature of a race rather than a broken assertion. An atomic counter
    /// makes collision impossible instead of unlikely.
    fn tmpdir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let d = std::env::temp_dir().join(format!(
            "uar-prov-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn reads_version_commit_and_count() {
        let d = tmpdir();
        write_manifest(
            &d,
            "---\nname: prometheus-skill-pack\nversion: 1.6.0\n\
             commit: 3370852b8688652f2c08281d9db820d1e3d0c4fd\n\
             skill_count: 146\ngenerated_at: 2026-07-31T15:00:00Z\n---\n\n# Skills\n",
        );
        let p = read_provenance(&d);
        assert_eq!(p.version.as_deref(), Some("1.6.0"));
        assert_eq!(p.skill_count, Some(146));
        assert_eq!(p.short_commit(), Some("3370852b86"));
        assert!(!p.is_unknown());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn a_pack_with_no_manifest_still_loads_as_unknown() {
        // The compatibility guarantee: provenance is observability, not a
        // breaking change. A pack predating change-uhe-005 must not fail here.
        let d = tmpdir();
        let p = read_provenance(&d);
        assert!(p.is_unknown());
        assert!(
            p.manifest_path.is_none(),
            "no manifest means no path recorded"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn a_manifest_without_frontmatter_is_unknown_but_records_where_we_looked() {
        let d = tmpdir();
        write_manifest(&d, "# Skills\n\nNo frontmatter here.\n");
        let p = read_provenance(&d);
        assert!(p.is_unknown());
        assert!(
            p.manifest_path.is_some(),
            "found-but-empty must be distinguishable from absent"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn unknown_keys_do_not_break_the_reader() {
        // The pack must be able to add frontmatter keys without a UAR release.
        let d = tmpdir();
        write_manifest(
            &d,
            "---\nversion: 2.0.0\ncommit: abc\nskill_count: 7\n\
             some_future_key: [a, b]\n---\n",
        );
        let p = read_provenance(&d);
        assert_eq!(p.version.as_deref(), Some("2.0.0"));
        assert_eq!(p.skill_count, Some(7));
        std::fs::remove_dir_all(&d).ok();
    }

    /// The exact JSON `SkillOrigin::Builtin` serialises to.
    ///
    /// A DB trigger guarding builtin deletion must match this string EXACTLY.
    /// `#[serde(rename_all = "lowercase")]` on the enum means the wire value is
    /// `"builtin"`, not `"Builtin"` — a trigger written against the Rust
    /// variant's spelling would silently never fire, which is worse than no
    /// trigger because it reads as protection.
    #[test]
    fn builtin_origin_serialises_lowercase() {
        use crate::uar::domain::skills::SkillOrigin;
        let json = serde_json::to_string(&SkillOrigin::Builtin).unwrap();
        assert_eq!(json, "\"builtin\"", "DB guards must match this literal");
    }

    /// A real `Skill` must serialise to JSON the DB trigger can act on.
    ///
    /// The trigger in `20260731000000_builtin_skill_delete_guard.sql` reads
    /// `definition->>'origin' = 'builtin'`. `postgres.rs:77` writes that column
    /// with `serde_json::to_value(skill)`. This test closes the loop: if the
    /// serialised shape ever stops producing a top-level lowercase `origin`,
    /// the guard silently stops matching and builtins become deletable — a
    /// regression that would otherwise surface only as data loss.
    #[test]
    fn a_serialised_skill_carries_the_origin_the_db_guard_matches() {
        use crate::uar::domain::skills::{Skill, SkillOrigin};

        let mut skill = Skill::default();
        skill.skill_id = "b1".into();
        skill.origin = SkillOrigin::Builtin;
        skill.enabled = true;

        let v = serde_json::to_value(&skill).expect("skill serialises");
        assert_eq!(
            v.get("origin").and_then(|o| o.as_str()),
            Some("builtin"),
            "the DB trigger matches definition->>'origin' = 'builtin'"
        );
        assert_eq!(
            v.get("enabled").and_then(|e| e.as_bool()),
            Some(true),
            "enabled must round-trip so a builtin can be disabled but not deleted"
        );
    }

    #[test]
    fn the_359_commit_drift_would_have_been_visible() {
        // The regression this whole change exists to prevent, reproduced from
        // the real history rather than a synthetic fixture.
        //
        // UAR pinned the pack at 8ddac9a (2026-06-01). That commit predates the
        // manifest entirely — `git show 8ddac9a:SKILLS.md` returns nothing — so
        // provenance reads as UNKNOWN.
        //
        // The subtle part: unknown must NOT read as "fine". A caller that
        // treated a missing manifest as "no drift" would have reported healthy
        // for two months, which is exactly what happened when nothing reported
        // at all. `is_unknown()` is therefore a positive signal an operator can
        // alert on, not an absence.
        let stale = tmpdir(); // no SKILLS.md — the state at 8ddac9a
        let stale_prov = read_provenance(&stale);
        assert!(
            stale_prov.is_unknown(),
            "a pack with no manifest must report unknown, not a plausible default"
        );

        // Today's pack states version, commit, and count.
        let current = tmpdir();
        write_manifest(
            &current,
            "---\nversion: 1.6.0\ncommit: 3370852b8688652f2c08281d9db820d1e3d0c4fd\n\
             skill_count: 146\n---\n",
        );
        let current_prov = read_provenance(&current);
        assert!(!current_prov.is_unknown());

        // The two are distinguishable — which is the entire requirement. Before
        // this change both states looked identical: silence.
        assert_ne!(stale_prov, current_prov);
        assert_eq!(current_prov.skill_count, Some(146));

        std::fs::remove_dir_all(&stale).ok();
        std::fs::remove_dir_all(&current).ok();
    }

    #[test]
    fn the_reported_version_changes_when_the_manifest_changes() {
        // THE assertion this change exists for. If this cannot fail, the
        // 359-commit drift would still be invisible.
        let d = tmpdir();
        write_manifest(
            &d,
            "---\nversion: 1.0.0\ncommit: aaa\nskill_count: 161\n---\n",
        );
        let before = read_provenance(&d);
        write_manifest(
            &d,
            "---\nversion: 1.6.0\ncommit: bbb\nskill_count: 220\n---\n",
        );
        let after = read_provenance(&d);

        assert_ne!(before, after, "provenance must track manifest changes");
        assert_eq!(before.skill_count, Some(161));
        assert_eq!(after.skill_count, Some(220));
        std::fs::remove_dir_all(&d).ok();
    }
}
