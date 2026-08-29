//! Filesystem-based skill storage provider.
//!
//! Discovers skills from local directories by scanning for SKILL.md files.

use super::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::{Skill, SkillManifest};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilesystemDiscoveryMode {
    Project,
    StandardAgentDirectory,
}

/// Discovers skills from a local filesystem directory.
#[derive(Debug)]
pub struct FilesystemStorageProvider {
    id: String,
    name: String,
    path: PathBuf,
    enabled: bool,
    discovery_mode: FilesystemDiscoveryMode,
    writable_dynamic: bool,
    skills_cache: Arc<RwLock<HashMap<String, Skill>>>,
}

impl FilesystemStorageProvider {
    /// Create a new filesystem provider scanning the given directory.
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            enabled: true,
            discovery_mode: FilesystemDiscoveryMode::Project,
            writable_dynamic: true,
            skills_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create the read-only provider for the cross-agent standard directory.
    pub fn standard_agent_directory(path: impl Into<PathBuf>) -> Self {
        Self {
            id: "agent-skills".to_string(),
            name: "Standard Agent Skills".to_string(),
            path: path.into(),
            enabled: true,
            discovery_mode: FilesystemDiscoveryMode::StandardAgentDirectory,
            writable_dynamic: false,
            skills_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Recursively scan a directory for SKILL.md files.
    async fn scan_directory(&self, path: &Path) -> anyhow::Result<Vec<Skill>> {
        let mut skills = Vec::new();

        if !path.exists() {
            warn!("Skills directory not found: {:?}", path);
            return Ok(skills);
        }

        let mut entries = fs::read_dir(path).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let skill_file = entry_path.join("SKILL.md");
                if skill_file.exists() {
                    match self.load_skill_from_file(&skill_file).await {
                        Ok(skill) => skills.push(skill),
                        Err(e) => error!("Failed to load skill from {:?}: {:?}", skill_file, e),
                    }
                } else {
                    // Recurse into subdirectories
                    match Box::pin(self.scan_directory(&entry_path)).await {
                        Ok(sub_skills) => skills.extend(sub_skills),
                        Err(e) => error!("Failed to scan directory {:?}: {:?}", entry_path, e),
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Load a single skill from a SKILL.md file.
    async fn load_skill_from_file(&self, path: &Path) -> anyhow::Result<Skill> {
        let content = fs::read_to_string(path).await?;
        let (manifest, overlay) = parse_skill_file(&content)?;

        let skill_id = manifest.name.to_lowercase().replace(' ', "-");

        // Check for mcp.json in the same directory
        let mut mcp_config = None;
        if let Some(parent) = path.parent() {
            let mcp_path = parent.join("mcp.json");
            if mcp_path.exists() {
                match crate::mcp::config::load_mcp_config(&mcp_path) {
                    Ok(mut cfg) => {
                        info!("Loaded mcp.json for skill: {}", skill_id);
                        let new_servers: HashMap<_, _> = cfg
                            .mcp_servers
                            .drain()
                            .map(|(k, v)| (format!("{skill_id}__{k}"), v))
                            .collect();
                        cfg.mcp_servers = new_servers;
                        mcp_config = Some(cfg);
                    }
                    Err(e) => {
                        error!("Failed to load mcp.json for skill {}: {:?}", skill_id, e);
                    }
                }
            }
        }

        let provider_id = if path
            .strip_prefix(&self.path)
            .is_ok_and(|relative| relative.starts_with("dynamic"))
        {
            "api".to_string()
        } else {
            self.id.clone()
        };

        let skill = Skill {
            skill_id: skill_id.clone(),
            version: manifest.version,
            title: manifest.name,
            description: manifest.description,
            triggers: manifest.triggers,
            prompt_overlay: overlay,
            preferred_tools: manifest.tools,
            mcp_config,
            constraints: Default::default(),
            enabled: manifest.enabled,
            scoped_config: manifest.scoped_config,
            provider_id,
            execution_config: Default::default(),
            kind: crate::uar::domain::skills::SkillKind::Manifest,
            origin: Default::default(),
            ..Default::default()
        };

        info!("Loaded skill from filesystem: {}", skill.title);
        Ok(skill)
    }

    fn scan_standard_tree(
        scan_root: &Path,
        identity_prefix: Option<&Path>,
        max_depth: Option<usize>,
        provider_id: &str,
        discovered: &mut Vec<(PathBuf, Skill)>,
        rejected: &mut usize,
    ) {
        let mut walker = WalkDir::new(scan_root)
            .follow_links(false)
            .follow_root_links(false)
            .sort_by_file_name();
        if let Some(max_depth) = max_depth {
            walker = walker.max_depth(max_depth);
        }
        for entry in walker.into_iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(scan_error) => {
                    *rejected += 1;
                    warn!(
                        path = scan_error
                            .path()
                            .map_or_else(|| "<unknown>".to_string(), |path| path.display().to_string()),
                        error = %scan_error,
                        "could not traverse standard agent skill path"
                    );
                    continue;
                }
            };
            if !entry.file_type().is_file() || entry.file_name() != "SKILL.md" {
                continue;
            }

            let manifest_path = entry.path();
            let relative = match manifest_path.strip_prefix(scan_root) {
                Ok(relative) => relative,
                Err(strip_error) => {
                    *rejected += 1;
                    warn!(
                        path = %manifest_path.display(),
                        error = %strip_error,
                        "standard agent skill path escaped its source root"
                    );
                    continue;
                }
            };
            let tree_directory = relative.parent().unwrap_or_else(|| Path::new(""));
            let relative_directory = identity_prefix.map_or_else(
                || tree_directory.to_path_buf(),
                |prefix| {
                    if tree_directory.as_os_str().is_empty() {
                        prefix.to_path_buf()
                    } else {
                        prefix.join(tree_directory)
                    }
                },
            );
            let normalized_directory = if relative_directory.as_os_str().is_empty() {
                ".".to_string()
            } else if let Some(relative_text) = relative_directory.to_str() {
                relative_text.replace(std::path::MAIN_SEPARATOR, "/")
            } else {
                *rejected += 1;
                warn!(
                    path = %manifest_path.display(),
                    "standard agent skill path is not valid UTF-8"
                );
                continue;
            };
            let skill_id = format!("agents::{normalized_directory}");
            match crate::uar::runtime::skills::builtin_loader::load_external_manifest(
                manifest_path,
                skill_id,
                provider_id,
            ) {
                Ok(skill) => discovered.push((relative_directory, skill)),
                Err(parse_error) => {
                    *rejected += 1;
                    warn!(
                        path = %manifest_path.display(),
                        error_kind = if parse_error.downcast_ref::<std::io::Error>().is_some() {
                            "io"
                        } else {
                            "invalid-manifest"
                        },
                        "failed to load standard agent skill manifest"
                    );
                }
            }
        }
    }

    fn scan_standard_agent_directory(root: &Path, provider_id: &str) -> anyhow::Result<Vec<Skill>> {
        match std::fs::symlink_metadata(root) {
            Ok(_) => {}
            Err(root_error) if root_error.kind() == std::io::ErrorKind::NotFound => {
                warn!(
                    path = %root.display(),
                    "standard agent skills directory not found; preserving durable skills"
                );
                info!(
                    name: "skills.standard.scan",
                    source = %root.display(),
                    discovered = 0,
                    rejected = 0,
                    "scanned standard agent skills directory"
                );
                return Ok(Vec::new());
            }
            Err(root_error) => {
                warn!(
                    path = %root.display(),
                    error_kind = ?root_error.kind(),
                    "standard agent skills directory is unreadable; preserving durable skills"
                );
                info!(
                    name: "skills.standard.scan",
                    source = %root.display(),
                    discovered = 0,
                    rejected = 1,
                    "scanned standard agent skills directory"
                );
                return Ok(Vec::new());
            }
        }

        let canonical_root = match std::fs::canonicalize(root) {
            Ok(canonical_root) => canonical_root,
            Err(root_error) => {
                warn!(
                    path = %root.display(),
                    error_kind = ?root_error.kind(),
                    "standard agent skills directory is unreadable; preserving durable skills"
                );
                info!(
                    name: "skills.standard.scan",
                    source = %root.display(),
                    discovered = 0,
                    rejected = 1,
                    "scanned standard agent skills directory"
                );
                return Ok(Vec::new());
            }
        };

        let mut discovered = Vec::<(PathBuf, Skill)>::new();
        let mut rejected = 0usize;
        Self::scan_standard_tree(
            &canonical_root,
            None,
            None,
            provider_id,
            &mut discovered,
            &mut rejected,
        );

        match std::fs::read_dir(&canonical_root) {
            Ok(entries) => {
                for entry_result in entries {
                    let entry = match entry_result {
                        Ok(entry) => entry,
                        Err(entry_error) => {
                            rejected += 1;
                            warn!(
                                path = %canonical_root.display(),
                                error_kind = ?entry_error.kind(),
                                "could not inspect a standard agent skill directory entry"
                            );
                            continue;
                        }
                    };
                    let alias_path = entry.path();
                    let Ok(metadata) = std::fs::symlink_metadata(&alias_path) else {
                        rejected += 1;
                        warn!(
                            path = %alias_path.display(),
                            "could not inspect standard agent skill alias"
                        );
                        continue;
                    };
                    if !metadata.file_type().is_symlink() {
                        continue;
                    }
                    let linked_target = match std::fs::read_link(&alias_path) {
                        Ok(linked_target) => linked_target,
                        Err(alias_error) => {
                            rejected += 1;
                            warn!(
                                path = %alias_path.display(),
                                error_kind = ?alias_error.kind(),
                                "could not read standard agent skill alias"
                            );
                            continue;
                        }
                    };
                    let one_hop_target = if linked_target.is_absolute() {
                        linked_target
                    } else {
                        alias_path
                            .parent()
                            .unwrap_or_else(|| Path::new(""))
                            .join(linked_target)
                    };
                    let one_hop_metadata = match std::fs::symlink_metadata(&one_hop_target) {
                        Ok(one_hop_metadata) => one_hop_metadata,
                        Err(alias_error) => {
                            rejected += 1;
                            warn!(
                                path = %alias_path.display(),
                                error_kind = ?alias_error.kind(),
                                "could not inspect standard agent skill alias target"
                            );
                            continue;
                        }
                    };
                    if one_hop_metadata.file_type().is_symlink() {
                        rejected += 1;
                        warn!(
                            path = %alias_path.display(),
                            "standard agent skill alias chaining is not allowed"
                        );
                        continue;
                    }
                    if !one_hop_metadata.is_dir() {
                        continue;
                    }
                    let alias_target = match std::fs::canonicalize(&one_hop_target) {
                        Ok(alias_target) => alias_target,
                        Err(alias_error) => {
                            rejected += 1;
                            warn!(
                                path = %alias_path.display(),
                                error_kind = ?alias_error.kind(),
                                "could not resolve standard agent skill alias target"
                            );
                            continue;
                        }
                    };
                    if alias_target == canonical_root || canonical_root.starts_with(&alias_target) {
                        rejected += 1;
                        warn!(
                            path = %alias_path.display(),
                            "standard agent skill alias resolves to the source root or its ancestor"
                        );
                        continue;
                    }
                    let alias_prefix = PathBuf::from(entry.file_name());
                    let root_manifest = alias_target.join("SKILL.md");
                    if root_manifest.is_file() {
                        Self::scan_standard_tree(
                            &alias_target,
                            Some(&alias_prefix),
                            Some(1),
                            provider_id,
                            &mut discovered,
                            &mut rejected,
                        );
                    }

                    let nested_skills = alias_target.join("skills");
                    if nested_skills.is_dir() {
                        let nested_prefix = alias_prefix.join("skills");
                        Self::scan_standard_tree(
                            &nested_skills,
                            Some(&nested_prefix),
                            None,
                            provider_id,
                            &mut discovered,
                            &mut rejected,
                        );
                    } else if !root_manifest.is_file() {
                        match std::fs::read_dir(&alias_target) {
                            Ok(collection_entries) => {
                                for collection_result in collection_entries {
                                    let collection_entry = match collection_result {
                                        Ok(collection_entry) => collection_entry,
                                        Err(collection_error) => {
                                            rejected += 1;
                                            warn!(
                                                path = %alias_path.display(),
                                                error_kind = ?collection_error.kind(),
                                                "could not inspect a standard agent skill collection entry"
                                            );
                                            continue;
                                        }
                                    };
                                    let collection_type = match collection_entry.file_type() {
                                        Ok(collection_type) => collection_type,
                                        Err(collection_error) => {
                                            rejected += 1;
                                            warn!(
                                                path = %collection_entry.path().display(),
                                                error_kind = ?collection_error.kind(),
                                                "could not inspect a standard agent skill collection child"
                                            );
                                            continue;
                                        }
                                    };
                                    if !collection_type.is_dir() {
                                        continue;
                                    }
                                    let collection_path = collection_entry.path();
                                    if !collection_path.join("SKILL.md").is_file() {
                                        continue;
                                    }
                                    let collection_prefix =
                                        alias_prefix.join(collection_entry.file_name());
                                    Self::scan_standard_tree(
                                        &collection_path,
                                        Some(&collection_prefix),
                                        Some(1),
                                        provider_id,
                                        &mut discovered,
                                        &mut rejected,
                                    );
                                    let collection_skills = collection_path.join("skills");
                                    if collection_skills.is_dir() {
                                        let collection_skills_prefix =
                                            collection_prefix.join("skills");
                                        Self::scan_standard_tree(
                                            &collection_skills,
                                            Some(&collection_skills_prefix),
                                            None,
                                            provider_id,
                                            &mut discovered,
                                            &mut rejected,
                                        );
                                    }
                                }
                            }
                            Err(collection_error) => {
                                rejected += 1;
                                warn!(
                                    path = %alias_path.display(),
                                    error_kind = ?collection_error.kind(),
                                    "could not enumerate standard agent skill collection alias"
                                );
                            }
                        }
                    }
                }
            }
            Err(read_error) => {
                rejected += 1;
                warn!(
                    path = %canonical_root.display(),
                    error_kind = ?read_error.kind(),
                    "could not enumerate standard agent skill aliases"
                );
            }
        }

        for current_index in 0..discovered.len() {
            let current_directory = discovered[current_index].0.clone();
            let mut closest_parent: Option<(usize, String)> = None;
            for (candidate_directory, candidate_skill) in &discovered {
                if candidate_directory == &current_directory
                    || !current_directory.starts_with(candidate_directory)
                {
                    continue;
                }
                let depth = candidate_directory.components().count();
                if closest_parent
                    .as_ref()
                    .is_none_or(|(closest_depth, _)| depth > *closest_depth)
                {
                    closest_parent = Some((depth, candidate_skill.skill_id.clone()));
                }
            }
            discovered[current_index].1.parent_skill_id = closest_parent.map(|(_, id)| id);
        }

        let skills = discovered
            .into_iter()
            .map(|(_, skill)| skill)
            .collect::<Vec<_>>();
        info!(
            name: "skills.standard.scan",
            source = %root.display(),
            discovered = skills.len(),
            rejected,
            "scanned standard agent skills directory"
        );
        Ok(skills)
    }
}

/// Resolve the standard cross-agent skill directory for the current user.
pub fn standard_agent_skills_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| standard_agent_skills_dir_for_home(&home))
}

fn standard_agent_skills_dir_for_home(home: &Path) -> PathBuf {
    home.join(".agents").join("skills")
}

/// Parse a SKILL.md file into manifest (YAML frontmatter) and body (markdown).
pub fn parse_skill_file(content: &str) -> anyhow::Result<(SkillManifest, String)> {
    if !content.starts_with("---") {
        return Err(anyhow::anyhow!("Missing frontmatter"));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid frontmatter format"));
    }

    let yaml = parts[1];
    let body = parts[2].trim().to_string();

    let manifest: SkillManifest = serde_norway::from_str(yaml)?;
    Ok((manifest, body))
}

/// Serialize a `Skill` back to SKILL.md format (YAML frontmatter + markdown body).
pub fn serialize_skill_to_md(skill: &Skill) -> anyhow::Result<String> {
    let manifest = SkillManifest {
        name: skill.title.clone(),
        version: skill.version.clone(),
        description: skill.description.clone(),
        authors: Vec::new(),
        triggers: skill.triggers.clone(),
        tools: skill.preferred_tools.clone(),
        enabled: skill.enabled,
        scoped_config: skill.scoped_config.clone(),
    };
    let yaml = serde_norway::to_string(&manifest)?;
    let body = if skill.prompt_overlay.is_empty() {
        format!("# {}\n\n{}", skill.title, skill.description)
    } else {
        skill.prompt_overlay.clone()
    };
    Ok(format!("---\n{yaml}---\n\n{body}\n"))
}

#[async_trait]
impl SkillStorageProvider for FilesystemStorageProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StorageProviderKind {
        StorageProviderKind::Filesystem
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn list_skills(&self) -> anyhow::Result<Vec<Skill>> {
        let cache = self.skills_cache.read().await;
        if cache.is_empty() {
            drop(cache);
            return self.refresh().await;
        }
        Ok(cache.values().cloned().collect())
    }

    async fn refresh(&self) -> anyhow::Result<Vec<Skill>> {
        let discovered = match self.discovery_mode {
            FilesystemDiscoveryMode::Project => self.scan_directory(&self.path).await?,
            FilesystemDiscoveryMode::StandardAgentDirectory => {
                let path = self.path.clone();
                let provider_id = self.id.clone();
                tokio::task::spawn_blocking(move || {
                    Self::scan_standard_agent_directory(&path, &provider_id)
                })
                .await??
            }
        };

        let mut cache = self.skills_cache.write().await;
        cache.clear();
        for skill in discovered {
            match cache.get(&skill.skill_id) {
                Some(existing) if existing.provider_id == self.id && skill.provider_id == "api" => {
                    warn!(
                        skill_id = %skill.skill_id,
                        "Ignoring dynamic skill copy because a configuration-managed source exists"
                    );
                }
                Some(existing) if existing.provider_id == "api" && skill.provider_id == self.id => {
                    warn!(
                        skill_id = %skill.skill_id,
                        "Replacing dynamic skill copy with configuration-managed source"
                    );
                    cache.insert(skill.skill_id.clone(), skill);
                }
                _ => {
                    cache.insert(skill.skill_id.clone(), skill);
                }
            }
        }
        let skills = cache.values().cloned().collect::<Vec<_>>();

        info!(
            "FilesystemStorageProvider '{}': loaded {} skills from {:?}",
            self.name,
            skills.len(),
            self.path
        );

        Ok(skills)
    }

    async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.writable_dynamic,
            "filesystem provider '{}' is read-only",
            self.id
        );
        anyhow::ensure!(
            skill.provider_id == "api",
            "filesystem dynamic storage accepts only API-managed skills"
        );
        // Write to skills/dynamic/<skill_id>/SKILL.md so the skill persists
        // across restarts and is picked up by future filesystem scans.
        let skill_dir = self.path.join("dynamic").join(&skill.skill_id);
        fs::create_dir_all(&skill_dir).await?;
        let content = serialize_skill_to_md(skill)?;
        fs::write(skill_dir.join("SKILL.md"), content).await?;
        self.skills_cache
            .write()
            .await
            .insert(skill.skill_id.clone(), skill.clone());
        info!(
            "FilesystemStorageProvider '{}': wrote skill '{}' to {:?}",
            self.name, skill.skill_id, skill_dir
        );
        Ok(())
    }

    async fn delete_skill(&self, id: &str) -> anyhow::Result<()> {
        if !self.writable_dynamic {
            self.skills_cache.write().await.remove(id);
            return Ok(());
        }
        let skill_dir = self.path.join("dynamic").join(id);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir).await?;
            info!(
                "FilesystemStorageProvider '{}': deleted skill dir {:?}",
                self.name, skill_dir
            );
        }
        self.skills_cache.write().await.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::domain::skills::SkillScope;

    async fn write_agent_manifest(directory: &Path, name: &str, description: &str) {
        fs::create_dir_all(directory).await.unwrap();
        fs::write(
            directory.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n"),
        )
        .await
        .unwrap();
    }

    #[test]
    fn standard_directory_is_resolved_below_the_supplied_home() {
        assert_eq!(
            standard_agent_skills_dir_for_home(Path::new("/operator")),
            PathBuf::from("/operator/.agents/skills")
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn standard_directory_follows_root_link_and_keeps_nested_duplicate_names() {
        use std::os::unix::fs::symlink;

        let source = tempfile::tempdir().unwrap();
        let link_parent = tempfile::tempdir().unwrap();
        let linked_root = link_parent.path().join("skills");
        symlink(source.path(), &linked_root).unwrap();

        write_agent_manifest(&source.path().join("first"), "duplicate", "first source").await;
        write_agent_manifest(&source.path().join("second"), "duplicate", "second source").await;
        write_agent_manifest(&source.path().join("pack"), "parent", "parent source").await;
        write_agent_manifest(
            &source.path().join("pack/skills/child"),
            "child",
            "nested source",
        )
        .await;
        let invalid_directory = source.path().join("invalid");
        fs::create_dir_all(&invalid_directory).await.unwrap();
        fs::write(invalid_directory.join("SKILL.md"), "not frontmatter")
            .await
            .unwrap();

        let provider = FilesystemStorageProvider::standard_agent_directory(&linked_root);
        let mut loaded = provider.refresh().await.unwrap();
        loaded.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));

        assert_eq!(loaded.len(), 4);
        assert!(
            loaded
                .iter()
                .all(|skill| skill.provider_id == "agent-skills")
        );
        assert!(loaded.iter().all(|skill| skill.version == "0.0.0"));
        assert!(loaded.iter().any(|skill| {
            skill.skill_id == "agents::first"
                && skill.title == "duplicate"
                && skill.description == "first source"
        }));
        assert!(loaded.iter().any(|skill| {
            skill.skill_id == "agents::second"
                && skill.title == "duplicate"
                && skill.description == "second source"
        }));
        let child = loaded
            .iter()
            .find(|skill| skill.skill_id == "agents::pack/skills/child")
            .unwrap();
        assert_eq!(child.parent_skill_id.as_deref(), Some("agents::pack"));
    }

    #[tokio::test]
    async fn standard_directory_assigns_distinct_ids_to_root_and_named_root_directory() {
        let source = tempfile::tempdir().unwrap();
        write_agent_manifest(source.path(), "source-root", "root source").await;
        write_agent_manifest(
            &source.path().join("__root__"),
            "named-root",
            "named directory source",
        )
        .await;

        let provider = FilesystemStorageProvider::standard_agent_directory(source.path());
        let loaded = provider.refresh().await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert!(loaded.iter().any(|skill| skill.skill_id == "agents::."));
        assert!(
            loaded
                .iter()
                .any(|skill| skill.skill_id == "agents::__root__")
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn standard_directory_follows_top_level_aliases_without_chaining_or_cycles() {
        use std::os::unix::fs::symlink;

        let source = tempfile::tempdir().unwrap();
        let aliased_skill = tempfile::tempdir().unwrap();
        let chained_skill = tempfile::tempdir().unwrap();
        let chain_holder = tempfile::tempdir().unwrap();
        let intermediate_chain = tempfile::tempdir().unwrap();
        let linked_pack = tempfile::tempdir().unwrap();
        let linked_pack_skills = tempfile::tempdir().unwrap();
        write_agent_manifest(&source.path().join("inside"), "inside", "inside source").await;
        write_agent_manifest(aliased_skill.path(), "aliased", "explicit alias source").await;
        write_agent_manifest(chained_skill.path(), "chained", "chained source").await;
        write_agent_manifest(
            &intermediate_chain.path().join("skills/child"),
            "intermediate-chain",
            "intermediate symlink source",
        )
        .await;
        write_agent_manifest(linked_pack.path(), "linked-pack", "linked pack root").await;
        write_agent_manifest(
            &linked_pack_skills.path().join("child"),
            "linked-pack-child",
            "must not follow linked skills root",
        )
        .await;
        symlink(
            chained_skill.path(),
            aliased_skill.path().join("chained-link"),
        )
        .unwrap();
        let chained_alias_target = chain_holder.path().join("alias-hop");
        symlink(
            std::fs::canonicalize(chained_skill.path()).unwrap(),
            &chained_alias_target,
        )
        .unwrap();
        symlink(&chained_alias_target, source.path().join("top-level-chain")).unwrap();
        let intermediate_hop = chain_holder.path().join("current");
        symlink(
            std::fs::canonicalize(intermediate_chain.path()).unwrap(),
            &intermediate_hop,
        )
        .unwrap();
        symlink(
            intermediate_hop.join("skills/child"),
            source.path().join("intermediate-component-chain"),
        )
        .unwrap();
        symlink(
            std::fs::canonicalize(linked_pack_skills.path()).unwrap(),
            linked_pack.path().join("skills"),
        )
        .unwrap();
        symlink(
            std::fs::canonicalize(aliased_skill.path()).unwrap(),
            source.path().join("linked-skill"),
        )
        .unwrap();
        symlink(
            std::fs::canonicalize(linked_pack.path()).unwrap(),
            source.path().join("linked-pack"),
        )
        .unwrap();
        symlink(source.path(), source.path().join("cycle")).unwrap();

        let provider = FilesystemStorageProvider::standard_agent_directory(source.path());
        let mut loaded = provider.refresh().await.unwrap();
        loaded.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));

        assert_eq!(loaded.len(), 4);
        assert_eq!(loaded[0].skill_id, "agents::inside");
        assert_eq!(loaded[0].title, "inside");
        assert_eq!(loaded[1].skill_id, "agents::intermediate-component-chain");
        assert_eq!(loaded[1].title, "intermediate-chain");
        assert_eq!(loaded[2].skill_id, "agents::linked-pack");
        assert_eq!(loaded[2].title, "linked-pack");
        assert_eq!(loaded[3].skill_id, "agents::linked-skill");
        assert_eq!(loaded[3].title, "aliased");
        assert!(loaded.iter().all(|skill| skill.title != "chained"));
        assert!(
            loaded
                .iter()
                .all(|skill| skill.title != "linked-pack-child")
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn standard_directory_aliases_scan_only_declared_skill_surfaces() {
        use std::os::unix::fs::symlink;

        let source = tempfile::tempdir().unwrap();
        let pack = tempfile::tempdir().unwrap();
        let collection = tempfile::tempdir().unwrap();
        write_agent_manifest(
            &pack.path().join("skills/child"),
            "pack-child",
            "declared pack skill",
        )
        .await;
        write_agent_manifest(
            &pack.path().join(".build/checkouts/dependency"),
            "build-dependency",
            "not a declared pack skill",
        )
        .await;
        write_agent_manifest(
            &collection.path().join("direct"),
            "collection-child",
            "declared collection skill",
        )
        .await;
        write_agent_manifest(
            &collection.path().join("unrelated/deep"),
            "deep-unrelated",
            "not an immediate collection skill",
        )
        .await;
        symlink(
            std::fs::canonicalize(pack.path()).unwrap(),
            source.path().join("pack"),
        )
        .unwrap();
        symlink(
            std::fs::canonicalize(collection.path()).unwrap(),
            source.path().join("collection"),
        )
        .unwrap();

        let provider = FilesystemStorageProvider::standard_agent_directory(source.path());
        let mut loaded = provider.refresh().await.unwrap();
        loaded.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].skill_id, "agents::collection/direct");
        assert_eq!(loaded[1].skill_id, "agents::pack/skills/child");
        assert!(loaded.iter().all(|skill| skill.title != "build-dependency"));
        assert!(loaded.iter().all(|skill| skill.title != "deep-unrelated"));
    }

    #[tokio::test]
    async fn standard_directory_provider_is_read_only_and_missing_source_is_nonfatal() {
        let missing_root = tempfile::tempdir().unwrap().path().join("missing");
        let provider = FilesystemStorageProvider::standard_agent_directory(&missing_root);

        assert!(provider.refresh().await.unwrap().is_empty());
        let mut skill = Skill::default();
        skill.skill_id = "agents::cannot-write".to_string();
        skill.provider_id = "api".to_string();
        let error = provider.save_skill(&skill).await.unwrap_err();
        assert!(error.to_string().contains("read-only"));
    }

    #[test]
    fn scoped_config_round_trips_through_skill_markdown() {
        let mut skill = Skill {
            skill_id: "round-trip".to_string(),
            version: "1.0.0".to_string(),
            title: "Round Trip".to_string(),
            description: "Scoped persistence".to_string(),
            prompt_overlay: "# Round Trip".to_string(),
            ..Skill::default()
        };
        skill.set_enabled_for(SkillScope::Global, false);
        skill.set_enabled_for(SkillScope::Agent("agent-a".to_string()), true);
        skill.set_enabled_for(
            SkillScope::Conversation("conversation-a".to_string()),
            false,
        );

        let markdown = serialize_skill_to_md(&skill).expect("serialize skill");
        let (manifest, overlay) = parse_skill_file(&markdown).expect("parse skill");

        assert_eq!(manifest.enabled, false);
        assert_eq!(manifest.scoped_config, skill.scoped_config);
        assert_eq!(overlay, skill.prompt_overlay);
    }

    #[tokio::test]
    async fn cold_reload_preserves_api_and_config_provenance() {
        let directory = tempfile::tempdir().expect("temporary skills directory");
        let provider =
            FilesystemStorageProvider::new("fs-skills", "Local Skills", directory.path());
        let api_skill = Skill {
            skill_id: "api-cold-reload".to_string(),
            version: "1.0.0".to_string(),
            title: "API Cold Reload".to_string(),
            description: "API-managed skill".to_string(),
            provider_id: "api".to_string(),
            ..Skill::default()
        };
        provider
            .save_skill(&api_skill)
            .await
            .expect("save API skill");

        let config_directory = directory.path().join("config-cold-reload");
        fs::create_dir_all(&config_directory)
            .await
            .expect("create config skill directory");
        let config_skill = Skill {
            skill_id: "config-cold-reload".to_string(),
            version: "1.0.0".to_string(),
            title: "Config Cold Reload".to_string(),
            description: "Configuration-managed skill".to_string(),
            ..Skill::default()
        };
        fs::write(
            config_directory.join("SKILL.md"),
            serialize_skill_to_md(&config_skill).expect("serialize config skill"),
        )
        .await
        .expect("write config skill");
        drop(provider);

        let restarted =
            FilesystemStorageProvider::new("fs-skills", "Local Skills", directory.path());
        let reloaded = restarted.list_skills().await.expect("cold reload skills");
        let api = reloaded
            .iter()
            .find(|skill| skill.skill_id == "api-cold-reload")
            .expect("API skill reloaded");
        let config = reloaded
            .iter()
            .find(|skill| skill.skill_id == "config-cold-reload")
            .expect("config skill reloaded");

        assert_eq!(api.provider_id, "api");
        assert_eq!(config.provider_id, "fs-skills");
    }

    #[tokio::test]
    async fn dynamic_storage_rejects_non_api_skills() {
        let directory = tempfile::tempdir().expect("temporary skills directory");
        let provider =
            FilesystemStorageProvider::new("fs-skills", "Local Skills", directory.path());
        let config_skill = Skill {
            skill_id: "config-only".to_string(),
            provider_id: "fs-skills".to_string(),
            ..Skill::default()
        };

        let error = provider
            .save_skill(&config_skill)
            .await
            .expect_err("configuration-managed skill must not enter dynamic storage");

        assert!(error.to_string().contains("only API-managed skills"));
        assert!(!directory.path().join("dynamic/config-only").exists());
    }

    #[tokio::test]
    async fn configuration_source_wins_over_a_stale_dynamic_copy() {
        let directory = tempfile::tempdir().expect("temporary skills directory");
        let provider =
            FilesystemStorageProvider::new("fs-skills", "Local Skills", directory.path());
        let config_skill = Skill {
            skill_id: "shared-skill".to_string(),
            title: "Shared Skill".to_string(),
            description: "current configuration".to_string(),
            provider_id: "fs-skills".to_string(),
            ..Skill::default()
        };
        let mut stale_copy = config_skill.clone();
        stale_copy.description = "stale dynamic copy".to_string();
        stale_copy.provider_id = "api".to_string();

        let config_directory = directory.path().join("shared-skill");
        let dynamic_directory = directory.path().join("dynamic/shared-skill");
        fs::create_dir_all(&config_directory).await.unwrap();
        fs::create_dir_all(&dynamic_directory).await.unwrap();
        fs::write(
            config_directory.join("SKILL.md"),
            serialize_skill_to_md(&config_skill).unwrap(),
        )
        .await
        .unwrap();
        fs::write(
            dynamic_directory.join("SKILL.md"),
            serialize_skill_to_md(&stale_copy).unwrap(),
        )
        .await
        .unwrap();

        let loaded = provider.refresh().await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].provider_id, "fs-skills");
        assert_eq!(loaded[0].description, "current configuration");
    }
}
