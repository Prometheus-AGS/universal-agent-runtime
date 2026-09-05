//! Host-owned, per-stream tool visibility. Discovery never grants execution.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use crate::uar::tools::descriptor::{Exposure, ToolDescriptor, ToolSource};

/// Maximum MCP descriptors advertised in one model step, excluding controls.
pub const MCP_EAGER_TOOL_LIMIT: usize = 32;
/// Maximum deferred descriptors selected by one search call.
pub const MCP_SEARCH_RESULT_LIMIT: usize = 8;
/// Bounded model input for local descriptor matching, in Unicode characters.
pub const MCP_SEARCH_QUERY_LIMIT: usize = 512;

#[derive(Default)]
struct DiscoveryState {
    deferred: BTreeMap<String, Arc<ToolDescriptor>>,
    selected: Vec<Arc<ToolDescriptor>>,
}

/// Fresh for each chat stream; never inherited as a parent's activation handle.
#[derive(Clone, Default)]
pub struct McpToolExposure(Arc<RwLock<DiscoveryState>>);

impl std::fmt::Debug for McpToolExposure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpToolExposure").finish_non_exhaustive()
    }
}

/// Frozen visibility over unchanged descriptors. Safe to retain during a batch.
#[derive(Clone)]
pub struct McpExposureSnapshot {
    visible: BTreeMap<String, Arc<ToolDescriptor>>,
    deferred: BTreeSet<String>,
}

impl std::fmt::Debug for McpExposureSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpExposureSnapshot")
            .field("visible_count", &self.visible.len())
            .field("deferred_count", &self.deferred.len())
            .finish()
    }
}

impl McpExposureSnapshot {
    /// Model-visible and callable descriptors for this step only.
    pub fn visible(&self) -> &BTreeMap<String, Arc<ToolDescriptor>> {
        &self.visible
    }

    /// Whether the host should register and advertise its model-only search tool.
    pub fn has_deferred(&self) -> bool {
        !self.deferred.is_empty()
    }

    /// Effective visibility; absent, hidden and policy-omitted names are Hidden.
    /// The underlying descriptor's governance and declared exposure are unchanged.
    pub fn exposure(&self, name: &str) -> Exposure {
        if let Some(tool) = self.visible.get(name) {
            if tool.exposure == Exposure::ModelOnly {
                Exposure::ModelOnly
            } else {
                Exposure::Eager
            }
        } else if self.deferred.contains(name) {
            Exposure::Deferred
        } else {
            Exposure::Hidden
        }
    }
}

impl McpToolExposure {
    /// Freeze one host-authorized descriptor set, rechecking earlier selections.
    /// Newest search results take priority; remaining eager slots use name order.
    /// Removed or changed descriptors lose prior selection, without fallback.
    pub fn project(
        &self,
        authorized: &BTreeMap<String, Arc<ToolDescriptor>>,
    ) -> McpExposureSnapshot {
        let mut state = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.selected.retain(|selected| {
            authorized
                .get(&selected.provider_name)
                .is_some_and(|current| {
                    current.exposure != Exposure::Hidden && current.equivalent_to(selected)
                })
        });
        let mut selected = state
            .selected
            .iter()
            .map(|tool| tool.provider_name.clone())
            .collect::<BTreeSet<_>>();
        for (name, tool) in authorized {
            if selected.len() >= MCP_EAGER_TOOL_LIMIT {
                break;
            }
            if tool.source == ToolSource::Mcp
                && matches!(tool.exposure, Exposure::Eager | Exposure::ModelOnly)
            {
                selected.insert(name.clone());
            }
        }
        let mut visible = BTreeMap::new();
        let mut deferred = BTreeMap::new();
        for (name, tool) in authorized {
            if tool.exposure == Exposure::Hidden {
                continue;
            }
            if tool.source == ToolSource::Mcp {
                if selected.contains(name) {
                    visible.insert(name.clone(), Arc::clone(tool));
                } else {
                    deferred.insert(name.clone(), Arc::clone(tool));
                }
            } else if matches!(tool.exposure, Exposure::Eager | Exposure::ModelOnly) {
                visible.insert(name.clone(), Arc::clone(tool));
            }
        }
        let snapshot = McpExposureSnapshot {
            visible,
            deferred: deferred.keys().cloned().collect(),
        };
        state.deferred = deferred;
        snapshot
    }

    /// Select matching deferred tools for the next projection, without I/O.
    /// All query terms must match the name, ID, server or description. Exact
    /// provider-name matches rank first, then names, then descriptive matches.
    ///
    /// # Errors
    /// Rejects blank or oversized model input; no query is echoed in the error.
    pub fn search(&self, query: &str) -> anyhow::Result<Vec<Arc<ToolDescriptor>>> {
        anyhow::ensure!(
            !query.trim().is_empty() && query.chars().count() <= MCP_SEARCH_QUERY_LIMIT,
            "query must contain 1 to {MCP_SEARCH_QUERY_LIMIT} characters and not be blank"
        );
        let query = query.trim().to_lowercase();
        let terms = query.split_whitespace().collect::<Vec<_>>();
        let mut state = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut matches = state
            .deferred
            .values()
            .filter_map(|tool| {
                let name = tool.provider_name.to_lowercase();
                let identity = format!(
                    "{} {} {}",
                    name,
                    tool.id,
                    tool.server.as_deref().unwrap_or_default()
                )
                .to_lowercase();
                let searchable = format!("{} {}", identity, tool.description.to_lowercase());
                if !terms.iter().all(|term| searchable.contains(term)) {
                    return None;
                }
                let rank = if name == query {
                    0
                } else if terms.iter().all(|term| identity.contains(term)) {
                    1
                } else {
                    2
                };
                Some((rank, Arc::clone(tool)))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|(left_rank, left), (right_rank, right)| {
            left_rank
                .cmp(right_rank)
                .then_with(|| left.provider_name.cmp(&right.provider_name))
        });
        let matches = matches
            .into_iter()
            .take(MCP_SEARCH_RESULT_LIMIT)
            .map(|(_, tool)| tool)
            .collect::<Vec<_>>();
        for tool in matches.iter().rev() {
            state
                .selected
                .retain(|prior| prior.provider_name != tool.provider_name);
            state.selected.insert(0, Arc::clone(tool));
        }
        state.selected.truncate(MCP_EAGER_TOOL_LIMIT);
        Ok(matches)
    }
}
