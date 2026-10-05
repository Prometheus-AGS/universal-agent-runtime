use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use serde::Deserialize;
use tokio::sync::Mutex;

use super::TeamExecutionRuntime;
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::team_execution::TeamExecutionAttempt,
    runtime::{
        tool_admission::{HttpHostToolAdmissionPort, RunToolAdmissionInput},
        turn::{RunExecutionRequest, host::RunMcpServerInput},
    },
    security::claims::UserContext,
};

pub const CAPABILITY: &str = "team_execution_host_workspace_v1";
pub const EXTENSION: &str = "urn:prometheus:uar:team-host-workspace:1";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamHostSelection {
    pub required: bool,
    pub version: u32,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub servers: Vec<String>,
    pub workspace_path: Option<PathBuf>,
}

impl TeamHostSelection {
    pub fn valid(&self) -> bool {
        self.required
            && self.version == 1
            && self.tools.iter().all(|id| !id.is_empty())
            && self.servers.iter().all(|id| !id.is_empty())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeamHostContextInput {
    pub expected_binding_revision: u64,
    pub working_directory: PathBuf,
    pub mcp_servers: Vec<RunMcpServerInput>,
    pub tool_admission: RunToolAdmissionInput,
}

#[derive(Clone)]
pub(super) struct TeamHostContext {
    binding_revision: u64,
    directory: PathBuf,
    resources: crate::mcp::runtime::McpRunResources,
    admission: Arc<HttpHostToolAdmissionPort>,
    scrubber: crate::uar::runtime::turn::host::RunSecretScrubber,
    servers: Vec<String>,
}

pub(super) type TeamHostContexts = Mutex<BTreeMap<String, TeamHostContext>>;

fn key(owner: &str, workspace: &str, team: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{team}")
}

impl TeamExecutionRuntime {
    pub async fn attach_host_context(
        &self,
        user: &UserContext,
        workspace: &str,
        team_id: &str,
        input: TeamHostContextInput,
    ) -> Result<(), CollaborationError> {
        let owner = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
            .map_err(|_| CollaborationError::Conflict("TEAM_SCOPE_DENIED".into()))?;
        let owner_id = owner.presentation_owner_key();
        let team = self
            .catalog
            .get_team_instance(&owner_id, workspace, team_id)
            .await?;
        let binding = self
            .catalog
            .get_binding(&owner_id, workspace, &team.binding.id)
            .await?;
        let selection: TeamHostSelection =
            serde_json::from_value(binding.document["extensions"][EXTENSION].clone())
                .map_err(|_| CollaborationError::Conflict("TEAM_HOST_CONTEXT_REQUIRED".into()))?;
        let directory = std::fs::canonicalize(&input.working_directory)
            .map_err(|_| CollaborationError::Conflict("TEAM_WORKSPACE_DENIED".into()))?;
        if !selection.valid()
            || !directory.is_dir()
            || directory.parent().is_none()
            || selection.workspace_path.as_ref() != Some(&directory)
            || input.expected_binding_revision != team.binding.revision
            || binding.revision != team.binding.revision
        {
            return Err(CollaborationError::Conflict("TEAM_WORKSPACE_DENIED".into()));
        }
        let names = input
            .mcp_servers
            .iter()
            .map(|server| server.name.clone())
            .collect::<Vec<_>>();
        if names.iter().any(|name| !selection.servers.contains(name))
            || selection.servers.iter().any(|name| !names.contains(name))
        {
            return Err(CollaborationError::Conflict("TEAM_SCOPE_DENIED".into()));
        }
        let (servers, resources) = self
            .manager
            .admit_run_mcp_servers(input.mcp_servers, user, owner, directory.clone())
            .await
            .map_err(|_| CollaborationError::Conflict("TEAM_HOST_CONTEXT_REQUIRED".into()))?;
        let admission = HttpHostToolAdmissionPort::from_input(input.tool_admission)
            .map_err(|_| CollaborationError::Conflict("TEAM_HOST_CONTEXT_REQUIRED".into()))?;
        self.host_contexts.lock().await.insert(
            key(&owner_id, workspace, team_id),
            TeamHostContext {
                binding_revision: team.binding.revision,
                directory,
                resources,
                admission: Arc::new(admission),
                scrubber: servers.scrubber(),
                servers: names,
            },
        );
        self.catalog.team_execution_notify.notify_one();
        Ok(())
    }

    pub(super) async fn require_host_context(
        &self,
        attempt: &TeamExecutionAttempt,
        effective: &serde_json::Value,
    ) -> Result<(), CollaborationError> {
        if effective.get("teamHostWorkspaceRequired") != Some(&serde_json::Value::Bool(true)) {
            return Ok(());
        }
        let contexts = self.host_contexts.lock().await;
        let context = contexts
            .get(&key(
                &attempt.owner_id,
                &attempt.workspace_id,
                &attempt.team_id,
            ))
            .ok_or_else(|| CollaborationError::Conflict("TEAM_HOST_CONTEXT_REQUIRED".into()))?;
        if context.binding_revision != attempt.binding_revision {
            return Err(CollaborationError::Conflict(
                "TEAM_REVISION_CONFLICT".into(),
            ));
        }
        Ok(())
    }

    pub(super) async fn apply_host_context(
        &self,
        attempt: &TeamExecutionAttempt,
        request: &mut RunExecutionRequest,
    ) -> anyhow::Result<()> {
        if request
            .collaboration_binding
            .as_ref()
            .is_none_or(|binding| {
                binding.receipt.effective.get("teamHostWorkspaceRequired")
                    != Some(&serde_json::Value::Bool(true))
            })
        {
            return Ok(());
        }
        let context = self
            .host_contexts
            .lock()
            .await
            .get(&key(
                &attempt.owner_id,
                &attempt.workspace_id,
                &attempt.team_id,
            ))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("TEAM_HOST_CONTEXT_REQUIRED"))?;
        anyhow::ensure!(
            context.binding_revision == attempt.binding_revision,
            "TEAM_REVISION_CONFLICT"
        );
        request.working_directory = Some(context.directory);
        request.mcp_resources = Some(context.resources);
        request.host_tool_admission = Some(context.admission);
        request.host_secret_scrubber.extend(context.scrubber);
        request.host_resources_marker.mcp_servers = context.servers;
        Ok(())
    }
}
