//! Host-only schedule provenance for the existing durable inbox.
use std::sync::Arc;
use crate::uar::{domain::schedules::ScheduleGuard, runtime::actor::messages::ActorOwner};
use super::{AgentInstanceController, AgentInstanceCommandView, AgentInstanceError, InstanceCommandStatus};

impl AgentInstanceController {
    pub(crate) async fn submit_scheduled(self: &Arc<Self>, owner: &ActorOwner,
        workspace: &str, schedule_id: &str, guard: &ScheduleGuard,
    ) -> Result<AgentInstanceCommandView, AgentInstanceError> {
        self.submit_inner(owner, workspace, &guard.instance_id, &guard.occurrence.command_id,
            &guard.prompt, Some(schedule_id)).await
    }

    /// Remove the retention pin only after a schedule terminal receipt exists.
    pub(crate) async fn release_schedule_command(&self, owner: &ActorOwner,
        workspace: &str, instance_id: &str, schedule_id: &str, command_id: &str,
    ) -> Result<(), AgentInstanceError> {
        self.update(owner, workspace, instance_id, |record| {
            let Some(command) = record.inbox.iter_mut().find(|c| c.command_id == command_id) else { return Ok(false); };
            if command.schedule_id.is_none() { return Ok(false); }
            if command.schedule_id.as_deref() != Some(schedule_id)
                || !matches!(command.status, InstanceCommandStatus::Completed | InstanceCommandStatus::Failed | InstanceCommandStatus::Cancelled) {
                return Err(AgentInstanceError::Conflict("scheduled command has no terminal disposition"));
            }
            command.schedule_id = None;
            Ok(true)
        }).await.map(|_| ())
    }
}
