//! Schedule transactions share the existing durable catalog CAS authority.
use crate::uar::{domain::schedules::DurableSchedule, runtime::schedules::ScheduleError};
use super::service::{CollaborationCatalogService, MAX_CAS_ATTEMPTS};

fn key(owner: &str, workspace: &str, id: &str) -> String {
    format!("{}:{owner}{}:{workspace}{id}", owner.len(), workspace.len())
}

impl CollaborationCatalogService {
    pub(crate) fn require_schedule_store(&self) -> Result<(), ScheduleError> {
        if self.storage_descriptor().backend == "memory" { return Err(ScheduleError::Unsupported); }
        Ok(())
    }

    pub(crate) async fn create_schedule_record(&self, record: DurableSchedule) -> Result<DurableSchedule, ScheduleError> {
        self.require_schedule_store()?;
        let key = key(&record.owner_key, &record.workspace_id, &record.id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let mut next = self.load_state().await.map_err(|_| ScheduleError::Storage)?;
            if next.schedules.contains_key(&key) { return Err(ScheduleError::Conflict("schedule already exists")); }
            let generation = next.generation;
            next.generation += 1;
            next.schedules.insert(key.clone(), record.clone());
            if self.cas(generation, &next).await.map_err(|_| ScheduleError::Storage)? { return Ok(record); }
        }
        Err(ScheduleError::Conflict("catalog changed during schedule creation"))
    }

    pub(crate) async fn schedule_record(&self, owner: &str, workspace: &str, id: &str) -> Result<DurableSchedule, ScheduleError> {
        self.require_schedule_store()?;
        self.load_state().await.map_err(|_| ScheduleError::Storage)?.schedules
            .get(&key(owner, workspace, id)).cloned().ok_or(ScheduleError::NotFound)
    }

    pub(crate) async fn schedule_records(&self) -> Result<Vec<DurableSchedule>, ScheduleError> {
        self.require_schedule_store()?;
        Ok(self.load_state().await.map_err(|_| ScheduleError::Storage)?.schedules.into_values().collect())
    }

    /// Host-only atomic mutation. Execution operations additionally require the
    /// current nonexpiring supervisor fence in the same catalog snapshot.
    pub(crate) async fn change_schedule<F>(&self, owner: &str, workspace: &str, id: &str,
        execution: bool, mut change: F) -> Result<DurableSchedule, ScheduleError>
    where F: FnMut(&mut DurableSchedule, Option<&crate::uar::domain::team_execution::TeamExecutionFence>) -> Result<bool, ScheduleError> {
        self.require_schedule_store()?;
        let key = key(owner, workspace, id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let mut next = self.load_state().await.map_err(|_| ScheduleError::Storage)?;
            let fence = if execution { Some(self.require_execution_owner(&next, false)
                .map_err(|_| ScheduleError::Conflict("schedule execution owner is fenced"))?) } else { None };
            let record = next.schedules.get_mut(&key).ok_or(ScheduleError::NotFound)?;
            if !change(record, fence.as_ref())? { return Ok(record.clone()); }
            record.updated_at = chrono::Utc::now();
            let result = record.clone();
            let generation = next.generation;
            next.generation += 1;
            if self.cas(generation, &next).await.map_err(|_| ScheduleError::Storage)? { return Ok(result); }
        }
        Err(ScheduleError::Conflict("catalog changed during schedule mutation"))
    }
}
