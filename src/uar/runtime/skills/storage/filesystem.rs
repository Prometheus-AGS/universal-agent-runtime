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

/// Discovers skills from a local filesystem directory.
#[derive(Debug)]
pub struct FilesystemStorageProvider {
    id: String,
    name: String,
    path: PathBuf,
    enabled: bool,
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
        let discovered = self.scan_directory(&self.path).await?;

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
