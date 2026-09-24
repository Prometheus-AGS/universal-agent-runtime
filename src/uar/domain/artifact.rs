use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write as _;

pub const CATALOG_METADATA_EXTENSION: &str = "uar.catalog";

/// Stable origin information retained with a catalog definition and every run
/// snapshot. `revision` identifies the source document, while the enclosing
/// catalog metadata revision identifies the complete UAR artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentArtifactSource {
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

/// Content-addressed catalog identity stored in the artifact extension map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCatalogMetadata {
    pub schema_version: u32,
    pub revision: String,
    pub source: AgentArtifactSource,
}

/// Immutable definition captured when a run is admitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentArtifactSnapshot {
    pub schema_version: u32,
    pub revision: String,
    pub source: AgentArtifactSource,
    pub artifact: AgentArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentArtifact {
    pub version: String,
    pub kind: String, // must be "agent"
    pub id: String,
    pub metadata: AgentMetadata,
    pub runtime: AgentRuntimeConfig,
    pub policy: AgentPolicy,
    pub schemas: AgentSchemas,
    pub prompt: AgentPrompt,
    pub memory: AgentMemoryConfig,
    pub tools: AgentToolConfig,
    pub ui: AgentUiConfig,
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl AgentArtifact {
    /// Stamp a deterministic catalog revision while retaining caller-supplied
    /// source provenance. Existing definitions without provenance receive the
    /// supplied source kind and their own id/version as source identity.
    #[must_use]
    pub fn with_catalog_metadata(mut self, default_source_kind: &str) -> Self {
        let source = self.catalog_metadata().map_or_else(
            || AgentArtifactSource {
                kind: default_source_kind.to_string(),
                id: self.id.clone(),
                revision: Some(self.version.clone()),
            },
            |metadata| metadata.source,
        );
        self.extensions.insert(
            CATALOG_METADATA_EXTENSION.to_string(),
            serde_json::to_value(AgentCatalogMetadata {
                schema_version: 1,
                revision: String::new(),
                source,
            })
            .expect("catalog metadata is serializable"),
        );
        let revision = self.content_revision();
        self.extensions
            .get_mut(CATALOG_METADATA_EXTENSION)
            .and_then(serde_json::Value::as_object_mut)
            .expect("catalog metadata is an object")
            .insert("revision".to_string(), serde_json::Value::String(revision));
        self
    }

    #[must_use]
    pub fn catalog_metadata(&self) -> Option<AgentCatalogMetadata> {
        self.extensions
            .get(CATALOG_METADATA_EXTENSION)
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    /// Content address of the complete definition excluding its self-referential
    /// catalog revision field. Source provenance remains part of the digest.
    #[must_use]
    pub fn content_revision(&self) -> String {
        let mut value = serde_json::to_value(self).expect("agent artifact is serializable");
        if let Some(extension) = value
            .get_mut("extensions")
            .and_then(serde_json::Value::as_object_mut)
            .and_then(|extensions| extensions.get_mut(CATALOG_METADATA_EXTENSION))
            .and_then(serde_json::Value::as_object_mut)
        {
            extension.remove("revision");
        }
        let digest = Sha256::digest(
            serde_json::to_vec(&value).expect("agent artifact JSON value is serializable"),
        );
        encode_digest(&digest)
    }

    /// Definition digest used only to compare a compatibility artifact with a
    /// stored snapshot. Catalog provenance is intentionally excluded.
    #[must_use]
    pub fn definition_revision(&self) -> String {
        let mut value = serde_json::to_value(self).expect("agent artifact is serializable");
        if let Some(extensions) = value
            .get_mut("extensions")
            .and_then(serde_json::Value::as_object_mut)
        {
            extensions.remove(CATALOG_METADATA_EXTENSION);
        }
        let digest = Sha256::digest(
            serde_json::to_vec(&value).expect("agent artifact JSON value is serializable"),
        );
        encode_digest(&digest)
    }

    #[must_use]
    pub fn snapshot(&self, default_source_kind: &str) -> AgentArtifactSnapshot {
        let artifact = self.clone().with_catalog_metadata(default_source_kind);
        let metadata = artifact
            .catalog_metadata()
            .expect("stamped artifact has catalog metadata");
        AgentArtifactSnapshot {
            schema_version: 1,
            revision: metadata.revision,
            source: metadata.source,
            artifact,
        }
    }
}

fn encode_digest(digest: &[u8]) -> String {
    let mut revision = String::from("sha256:");
    for byte in digest {
        write!(&mut revision, "{byte:02x}").expect("writing to a String cannot fail");
    }
    revision
}

impl AgentArtifactSnapshot {
    /// Decode and verify a snapshot before it can control a continuation.
    pub fn from_run_context(context: &serde_json::Value) -> Result<Self, &'static str> {
        let snapshot = context
            .get("agent_snapshot")
            .cloned()
            .ok_or("source run has no agent artifact snapshot; start a new run")
            .and_then(|value| {
                serde_json::from_value::<Self>(value)
                    .map_err(|_| "source run agent artifact snapshot is invalid; start a new run")
            })?;
        if snapshot.schema_version != 1 || snapshot.artifact.content_revision() != snapshot.revision
        {
            return Err("source run agent artifact snapshot cannot be verified; start a new run");
        }
        Ok(snapshot)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub entry: String,
    #[serde(default)]
    pub protocols: HashMap<String, ProtocolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPolicy {
    pub provider: ProviderPolicy,
    pub tools: ToolPolicy,
    pub skills: SkillPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPolicy {
    pub default: ProviderSelection,
    #[serde(default)]
    pub fallbacks: Vec<ProviderSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
    /// How tool calls should be executed: directly via MCP, sandboxed in a VM, or auto-detect.
    #[serde(default)]
    pub execution_mode: ToolExecutionMode,
}

fn default_max_concurrent() -> u32 {
    1
}

/// Controls how tool calls are routed during execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionMode {
    /// Execute tools directly via MCP (default).
    #[default]
    Direct,
    /// Execute all tool calls inside a microsandbox VM.
    Sandboxed,
    /// Automatically sandbox code-execution tools; use direct MCP for data/API tools.
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicy {
    #[serde(default)]
    pub prefer: Vec<String>,
    #[serde(default = "default_max_active")]
    pub max_active: u32,
}

fn default_max_active() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSchemas {
    #[serde(default)]
    pub inputs: Option<serde_json::Value>,
    #[serde(default)]
    pub outputs: Option<serde_json::Value>,
    #[serde(default)]
    pub state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPrompt {
    pub system: String,
    #[serde(default)]
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemoryConfig {
    #[serde(default)]
    pub conversation: ConversationMemory,
    #[serde(default)]
    pub kb: KbMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KbMemory {
    #[serde(default)]
    pub enabled: bool,
    /// Knowledge base names this agent has access to (empty = Default KB only)
    #[serde(default)]
    pub knowledge_bases: Vec<String>,
    #[serde(default)]
    pub citation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolConfig {
    #[serde(default)]
    pub bundles: Vec<ToolBundle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBundle {
    pub id: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUiConfig {
    #[serde(default)]
    pub forms: FeatureFlag,
    #[serde(default)]
    pub artifacts: ArtifactsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlag {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactsConfig {
    pub enabled: bool,
    #[serde(default)]
    pub preferred_types: Vec<String>,
}
