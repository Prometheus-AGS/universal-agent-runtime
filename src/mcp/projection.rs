//! Pure, authority-aware MCP selection for one model step.
//!
//! Resolve definitions before the host obtains their tool catalogs, then freeze
//! the exact descriptors against those same definitions. This module does no
//! connection or credential lookup. A projection is not an execution grant:
//! binding, authentication, sandbox enforcement and Cedar remain in the host.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use crate::uar::domain::policy::{EffectiveResourceSelection, EffectiveRunPolicy, SelectionMode};
use crate::uar::tools::descriptor::{Exposure, ToolCollision, ToolDescriptor, ToolSource};
use thiserror::Error;

use super::catalog::{McpCatalog, ServerDefinition, ServerSource};

/// Host-resolved activation scope, not fields accepted from MCP annotations.
#[derive(Debug, Clone, Default)]
pub struct McpProjectionScope {
    /// Only active, policy-eligible skills may contribute declarations.
    pub active_skills: BTreeSet<String>,
}

/// A projection could not preserve exact server or tool identity.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum McpProjectionError {
    /// An activated skill is not in the resolved policy's eligible set.
    #[error("MCP projection cannot activate skill {skill_id:?} outside the resolved policy")]
    SkillOutsidePolicy { skill_id: String },
    /// Equally authoritative origins disagree about one server's definition.
    #[error("conflicting equally authoritative MCP definitions for server {server:?}")]
    AmbiguousServer { server: String },
    /// A supplied descriptor is not an identified MCP tool from this server.
    #[error("invalid MCP descriptor {provider_name:?} for server {server:?}")]
    InvalidTool {
        server: String,
        provider_name: String,
    },
    /// A catalog belongs to another configuration or authentication revision.
    #[error("MCP tool catalog no longer matches selected server {server:?}")]
    DefinitionChanged { server: String },
    /// The host has not supplied a tool catalog for a selected definition.
    #[error("MCP server {server:?} requires tool discovery before step projection")]
    MissingCatalog { server: String },
    /// Partial discovery cannot masquerade as a complete, possibly empty catalog.
    #[error("MCP server {server:?} has an incomplete tool catalog")]
    IncompleteCatalog { server: String },
    /// Merging two different snapshots would resurrect removed tools.
    #[error("conflicting tool catalog snapshots for MCP server {server:?}")]
    ConflictingCatalog { server: String },
    /// Availability filtering cannot omit a required or unselected declaration.
    #[error("MCP preflight cannot omit server {server:?} as an unavailable optional server")]
    InvalidOptionalOmission { server: String },
    /// Distinct tools must not share a provider-visible call target.
    #[error(transparent)]
    ToolCollision(#[from] ToolCollision),
}

/// A host-supplied discovery snapshot associated with one exact declaration.
///
/// The host must obtain these descriptors from the matching binding or cache
/// entry, including its owner/auth/environment identity. This association is
/// not a claim an untrusted server may supply about itself.
#[derive(Clone)]
pub struct ServerToolCatalog {
    definition: Arc<ServerDefinition>,
    tools: BTreeMap<String, Arc<ToolDescriptor>>,
    complete: bool,
}

impl ServerToolCatalog {
    /// Capture already-compiled descriptors without modifying their semantics.
    ///
    /// # Errors
    /// Rejects missing identities, non-MCP tools, wrong server associations and
    /// conflicting provider-visible names. Identical repeats are coalesced.
    pub fn new(
        definition: Arc<ServerDefinition>,
        tools: impl IntoIterator<Item = Arc<ToolDescriptor>>,
        complete: bool,
    ) -> Result<Self, McpProjectionError> {
        let mut indexed = BTreeMap::<String, Arc<ToolDescriptor>>::new();
        for descriptor in tools {
            if descriptor.source != ToolSource::Mcp
                || descriptor.server.as_deref() != Some(definition.name())
                || descriptor.id.trim().is_empty()
                || descriptor.provider_name.trim().is_empty()
            {
                return Err(McpProjectionError::InvalidTool {
                    server: definition.name().to_string(),
                    provider_name: descriptor.provider_name.clone(),
                });
            }
            if let Some(existing) = indexed.get(&descriptor.provider_name) {
                if !existing.equivalent_to(&descriptor) {
                    return Err(ToolCollision {
                        provider_name: descriptor.provider_name.clone(),
                    }
                    .into());
                }
            } else {
                indexed.insert(descriptor.provider_name.clone(), descriptor);
            }
        }
        Ok(Self {
            definition,
            tools: indexed,
            complete,
        })
    }

    /// Declaration whose matching binding supplied the discovery snapshot.
    #[must_use]
    pub fn definition(&self) -> &Arc<ServerDefinition> {
        &self.definition
    }

    /// False means discovery must finish before this catalog can define a step.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// Unchanged, source-qualified descriptors, including non-visible entries.
    #[must_use]
    pub fn tools(&self) -> &BTreeMap<String, Arc<ToolDescriptor>> {
        &self.tools
    }

    fn equivalent_tools(&self, other: &Self) -> bool {
        self.complete == other.complete
            && self.tools.len() == other.tools.len()
            && self.tools.iter().all(|(name, tool)| {
                other
                    .tools
                    .get(name)
                    .is_some_and(|other| tool.equivalent_to(other))
            })
    }
}

impl fmt::Debug for ServerToolCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerToolCatalog")
            .field("server", &self.definition.name())
            .field("tool_count", &self.tools.len())
            .field("complete", &self.complete)
            .finish_non_exhaustive()
    }
}

/// Chosen declarations and the immutable tool eligibility used for discovery.
#[derive(Clone)]
pub struct McpServerProjection {
    servers: BTreeMap<String, Arc<ServerDefinition>>,
    eligible_tools: EffectiveResourceSelection,
}

impl McpServerProjection {
    /// Select global over active skill definitions.
    ///
    /// Closed selections use the policy's already-resolved eligible IDs. A
    /// genuine `All`/`Auto` remains open only within this frozen definition
    /// catalog, allowing complete tool discovery without widening server scope.
    /// Authentication failures or missing tool catalogs never make a
    /// lower-authority candidate eligible.
    ///
    /// # Errors
    /// Rejects out-of-policy skill activation and conflicting declarations at
    /// the highest eligible authority. Equal declarations use stable source
    /// order, retaining one origin rather than merging their metadata.
    pub fn resolve(
        catalog: &McpCatalog,
        policy: &EffectiveRunPolicy,
        scope: &McpProjectionScope,
    ) -> Result<Self, McpProjectionError> {
        for skill_id in &scope.active_skills {
            if !eligible(&policy.skills, skill_id) {
                return Err(McpProjectionError::SkillOutsidePolicy {
                    skill_id: skill_id.clone(),
                });
            }
        }
        let mut servers = BTreeMap::new();
        for name in catalog.server_names() {
            if !eligible(&policy.mcp_servers, name) {
                continue;
            }
            let candidates = || {
                catalog
                    .candidates(name)
                    .filter(|candidate| match candidate.source() {
                        ServerSource::Global => true,
                        ServerSource::Skill { skill_id } => scope.active_skills.contains(skill_id),
                    })
            };
            let Some(authority) = candidates().map(|candidate| candidate.authority()).max() else {
                continue;
            };
            let mut highest = candidates().filter(|candidate| candidate.authority() == authority);
            let Some(winner) = highest.next() else {
                continue;
            };
            for candidate in highest {
                if !same_settings(winner, candidate) {
                    return Err(McpProjectionError::AmbiguousServer {
                        server: name.to_string(),
                    });
                }
            }
            servers.insert(name.to_string(), Arc::clone(winner));
        }
        Ok(Self {
            servers,
            eligible_tools: policy.tools.clone(),
        })
    }

    /// The host must bind these declarations without selecting them again.
    #[must_use]
    pub fn servers(&self) -> &BTreeMap<String, Arc<ServerDefinition>> {
        &self.servers
    }

    pub(crate) fn without_unavailable_optional_servers(
        &self,
        omitted: &BTreeSet<String>,
    ) -> Result<Self, McpProjectionError> {
        for name in omitted {
            if !self
                .servers
                .get(name)
                .is_some_and(|server| !server.is_required())
            {
                return Err(McpProjectionError::InvalidOptionalOmission {
                    server: name.clone(),
                });
            }
        }
        let mut narrowed = self.clone();
        narrowed.servers.retain(|name, _| !omitted.contains(name));
        Ok(narrowed)
    }

    /// Freeze exact MCP tools after discovery, preserving the original policy.
    ///
    /// # Errors
    /// Rejects stale, missing, incomplete or conflicting selected catalogs and
    /// provider-name collisions. Catalogs from unselected servers or origins
    /// are ignored, never used as a fallback for the winning origin.
    pub fn with_tools(
        &self,
        catalogs: impl IntoIterator<Item = ServerToolCatalog>,
    ) -> Result<McpStepProjection, McpProjectionError> {
        let mut selected = BTreeMap::<String, ServerToolCatalog>::new();
        for catalog in catalogs {
            let name = catalog.definition.name();
            let Some(definition) = self.servers.get(name) else {
                continue;
            };
            if definition.source() != catalog.definition.source() {
                continue;
            }
            if !same_settings(definition, &catalog.definition) {
                return Err(McpProjectionError::DefinitionChanged {
                    server: name.to_string(),
                });
            }
            if !catalog.complete {
                return Err(McpProjectionError::IncompleteCatalog {
                    server: name.to_string(),
                });
            }
            if let Some(existing) = selected.get(name) {
                if !existing.equivalent_tools(&catalog) {
                    return Err(McpProjectionError::ConflictingCatalog {
                        server: name.to_string(),
                    });
                }
            } else {
                selected.insert(name.to_string(), catalog);
            }
        }

        let mut tools = BTreeMap::<String, ProjectedMcpTool>::new();
        for (name, definition) in &self.servers {
            let catalog = selected
                .get(name)
                .ok_or_else(|| McpProjectionError::MissingCatalog {
                    server: name.clone(),
                })?;
            for descriptor in catalog.tools.values() {
                if descriptor.exposure == Exposure::Hidden
                    || !(eligible(&self.eligible_tools, &descriptor.provider_name)
                        || eligible(&self.eligible_tools, &descriptor.id))
                {
                    continue;
                }
                if tools.contains_key(&descriptor.provider_name) {
                    return Err(ToolCollision {
                        provider_name: descriptor.provider_name.clone(),
                    }
                    .into());
                }
                tools.insert(
                    descriptor.provider_name.clone(),
                    ProjectedMcpTool {
                        server: Arc::clone(definition),
                        descriptor: Arc::clone(descriptor),
                    },
                );
            }
        }
        Ok(McpStepProjection {
            servers: self.servers.clone(),
            tools,
        })
    }
}

impl fmt::Debug for McpServerProjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpServerProjection")
            .field("server_count", &self.servers.len())
            .finish_non_exhaustive()
    }
}

/// One descriptor and its selected server; no identity is parsed from a name.
#[derive(Clone)]
pub struct ProjectedMcpTool {
    server: Arc<ServerDefinition>,
    descriptor: Arc<ToolDescriptor>,
}

impl ProjectedMcpTool {
    /// Exact declaration the host must bind for this tool call.
    #[must_use]
    pub fn server(&self) -> &Arc<ServerDefinition> {
        &self.server
    }

    /// Unmodified governance, schema, scheduling and exposure metadata.
    #[must_use]
    pub fn descriptor(&self) -> &Arc<ToolDescriptor> {
        &self.descriptor
    }
}

impl fmt::Debug for ProjectedMcpTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectedMcpTool")
            .field("server", &self.server.name())
            .field("provider_name", &self.descriptor.provider_name)
            .finish_non_exhaustive()
    }
}

/// Immutable exact server/tool selection, ready for the host's binding stage.
///
/// Deferred descriptors remain eligible but are not initially advertised;
/// hidden and policy-omitted tools cannot enter this projection.
#[derive(Clone)]
pub struct McpStepProjection {
    servers: BTreeMap<String, Arc<ServerDefinition>>,
    tools: BTreeMap<String, ProjectedMcpTool>,
}

impl McpStepProjection {
    /// The same source-qualified server choices used to obtain tool catalogs.
    #[must_use]
    pub fn servers(&self) -> &BTreeMap<String, Arc<ServerDefinition>> {
        &self.servers
    }

    /// Eligible exact tools, including deferred but not hidden descriptors.
    #[must_use]
    pub fn tools(&self) -> &BTreeMap<String, ProjectedMcpTool> {
        &self.tools
    }

    /// Bounded initial model-visible tools; use exposure for later discovery.
    pub fn model_tools(&self) -> impl Iterator<Item = &ProjectedMcpTool> {
        self.tools
            .values()
            .filter(|tool| {
                matches!(
                    tool.descriptor.exposure,
                    Exposure::Eager | Exposure::ModelOnly
                )
            })
            .take(super::exposure::MCP_EAGER_TOOL_LIMIT)
    }

    /// Apply this stream's discovery selections without altering descriptors or
    /// their exact server association. Only host-selected eligible tools enter.
    pub fn exposure(
        &self,
        state: &super::exposure::McpToolExposure,
    ) -> super::exposure::McpExposureSnapshot {
        state.project(
            &self
                .tools
                .iter()
                .map(|(name, tool)| (name.clone(), Arc::clone(&tool.descriptor)))
                .collect(),
        )
    }
}

impl fmt::Debug for McpStepProjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpStepProjection")
            .field("server_count", &self.servers.len())
            .field("tool_count", &self.tools.len())
            .finish_non_exhaustive()
    }
}

fn eligible(selection: &EffectiveResourceSelection, id: &str) -> bool {
    match selection.mode {
        SelectionMode::All | SelectionMode::Auto => true,
        SelectionMode::Selected => selection.ids.iter().any(|allowed| allowed == id),
        SelectionMode::None | SelectionMode::Inherit => false,
    }
}

fn same_settings(left: &ServerDefinition, right: &ServerDefinition) -> bool {
    left.config_hash() == right.config_hash()
        && left.is_required() == right.is_required()
        && left.authentication() == right.authentication()
}
