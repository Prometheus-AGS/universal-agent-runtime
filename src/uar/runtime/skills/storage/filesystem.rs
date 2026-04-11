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
            enabled: true,
            provider_id: self.id.clone(),
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

    let manifest: SkillManifest = serde_yml::from_str(yaml)?;
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
    };
    let yaml = serde_yml::to_string(&manifest)?;
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
        let skills = self.scan_directory(&self.path).await?;

        let mut cache = self.skills_cache.write().await;
        cache.clear();
        for skill in &skills {
            cache.insert(skill.skill_id.clone(), skill.clone());
        }

        info!(
            "FilesystemStorageProvider '{}': loaded {} skills from {:?}",
            self.name,
            skills.len(),
            self.path
        );

        Ok(skills)
    }

    async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()> {
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
