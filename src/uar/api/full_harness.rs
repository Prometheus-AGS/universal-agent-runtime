//! Versioned full-run delegation over UAR's existing native run executor.
//!
//! This adapter owns only process-ephemeral admission and control records. It
//! reserves identities before kernel entry, never logs request material, and
//! delegates all run assembly, tool execution, approval, and cancellation to
//! the native run authorities.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    config::RunsConfig,
    uar::{
        domain::runs::RunStatus,
        runtime::{actor::messages::ActorOwner, manager::RunManager},
    },
};

const TASK_PREFIX: &str = "fh";

#[derive(Clone)]
pub struct FullHarnessTaskAuthority {
    manager: Arc<RunManager>,
    runtime_epoch: String,
    retention: RetentionReceipt,
    records: Arc<Mutex<AuthorityRecords>>,
}

#[derive(Debug, Default)]
struct AuthorityRecords {
    tasks: HashMap<String, TaskRecord>,
    admissions: HashMap<(ActorOwner, String, String), AdmissionRecord>,
}

#[derive(Debug)]
struct AdmissionRecord {
    task_id: String,
    digest: String,
}

#[derive(Debug, Clone)]
struct TaskRecord {
    owner: ActorOwner,
    workspace_id: String,
    receipt: TaskReceipt,
    admission_response: Option<TaskReceipt>,
    response_status: StatusCode,
}

enum Reservation {
    New(TaskReceipt),
    Existing(StatusCode, TaskReceipt),
}

impl FullHarnessTaskAuthority {
    pub fn new(manager: Arc<RunManager>, config: RunsConfig) -> Self {
        Self {
            manager,
            runtime_epoch: Uuid::new_v4().simple().to_string(),
            retention: RetentionReceipt {
                mode: "process_ephemeral",
                terminal_ttl_seconds: config.retention_after_terminal_secs,
                terminal_record_cap: config.max_retained_terminal,
            },
            records: Arc::new(Mutex::new(AuthorityRecords::default())),
        }
    }

    pub fn runtime_descriptor(&self) -> RuntimeDescriptor {
        RuntimeDescriptor {
            event_cursor_profile: "contiguous_cursor_frames_v1",
            profile: "full_harness_v1",
            runtime_epoch: self.runtime_epoch.clone(),
            recovery: "unsupported_after_restart",
            retention: self.retention.clone(),
            steer_supported: false,
            delegated_host_context_v1: true,
        }
    }

    fn reserve(
        &self,
        owner: ActorOwner,
        workspace_id: String,
        admission_id: String,
        native_task_id: String,
        digest: String,
    ) -> Result<Reservation, ApiError> {
        self.prune();
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = (owner.clone(), workspace_id.clone(), admission_id.clone());
        if let Some(admission) = records.admissions.get(&key) {
            if admission.digest != digest {
                return Err(ApiError::conflict(
                    "admission_digest_conflict",
                    "admission_id is already reserved for a different request",
                    Some(admission.task_id.clone()),
                    Some(admission_id),
                ));
            }
            if let Some(record) = records.tasks.get(&admission.task_id) {
                return Ok(Reservation::Existing(
                    record.response_status,
                    record
                        .admission_response
                        .clone()
                        .unwrap_or_else(|| record.receipt.clone()),
                ));
            }
            return Err(ApiError::gone(
                "retention_expired",
                "the exact admission was retained but its process-ephemeral task receipt expired",
                Some(admission.task_id.clone()),
                Some(admission_id),
            ));
        }
        let task_id = format!(
            "{TASK_PREFIX}-{}-{}",
            self.runtime_epoch,
            Uuid::new_v4().simple()
        );
        let run_id = Uuid::new_v4().to_string();
        let base = "/api/uar/full-harness/v1";
        let receipt = TaskReceipt {
            admission_id: admission_id.clone(),
            task_id: task_id.clone(),
            native_task_id,
            run_id,
            root_run_id: None,
            workspace_id: workspace_id.clone(),
            agent_id: None,
            delegated_host_context: None,
            runtime_epoch: self.runtime_epoch.clone(),
            revision: 1,
            cursor: None,
            state: "reserved".to_owned(),
            retention: self.retention.clone(),
            effective_service_binding: None,
            diagnostics: Vec::new(),
            cancellation: CancellationReceipt::default(),
            detached: false,
            detached_observers: Vec::new(),
            created_at: Utc::now(),
            terminal_at: None,
            expires_at: None,
            unsupported_semantics: vec!["steer"],
            links: TaskLinks {
                status: format!("{base}/tasks/{task_id}"),
                stream: format!("{base}/tasks/{task_id}/stream"),
                tool_approval: format!("{base}/tasks/{task_id}/tool-approval"),
                cancel: format!("{base}/tasks/{task_id}/cancel"),
                detach: format!("{base}/tasks/{task_id}/detach"),
                steer: format!("{base}/tasks/{task_id}/steer"),
            },
        };
        records.admissions.insert(
            key,
            AdmissionRecord {
                task_id: task_id.clone(),
                digest: digest.clone(),
            },
        );
        records.tasks.insert(
            task_id,
            TaskRecord {
                owner,
                workspace_id,
                receipt: receipt.clone(),
                admission_response: None,
                response_status: StatusCode::ACCEPTED,
            },
        );
        Ok(Reservation::New(receipt))
    }

    fn update(&self, task_id: &str, update: impl FnOnce(&mut TaskRecord)) {
        if let Some(record) = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tasks
            .get_mut(task_id)
        {
            let previous = record.receipt.clone();
            update(record);
            if record.receipt != previous {
                record.receipt.revision = record.receipt.revision.saturating_add(1);
            }
        }
    }

    async fn capture_root(&self, task_id: &str) -> Option<String> {
        let receipt = self.records.lock().ok()?.tasks.get(task_id)?.receipt.clone();
        if receipt.root_run_id.is_some() {
            return receipt.root_run_id;
        }
        let root_run_id = self.manager.approval_root_run_id(&receipt.run_id).await?;
        self.update(task_id, |record| {
            record.receipt.root_run_id = Some(root_run_id.clone());
        });
        Some(root_run_id)
    }

    fn freeze_admission_response(&self, task_id: &str) -> Result<TaskReceipt, ApiError> {
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = records
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| ApiError::not_found("task_not_found", "task was not found"))?;
        let receipt = record.receipt.clone();
        record.admission_response = Some(receipt.clone());
        Ok(receipt)
    }

    fn owned(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        task_id: &str,
    ) -> Result<TaskReceipt, ApiError> {
        self.prune();
        let records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(record) = records.tasks.get(task_id) {
            return if &record.owner == owner && record.workspace_id == workspace_id {
                Ok(record.receipt.clone())
            } else {
                Err(ApiError::not_found("task_not_found", "task was not found"))
            };
        }
        if task_id.starts_with(&format!("{TASK_PREFIX}-"))
            && !task_id.starts_with(&format!("{TASK_PREFIX}-{}-", self.runtime_epoch))
        {
            return Err(ApiError::conflict(
                "recovery_unsupported",
                "task belongs to a prior runtime epoch and this profile does not support restart recovery",
                Some(task_id.to_owned()),
                None,
            ));
        }
        if task_id.starts_with(&format!("{TASK_PREFIX}-{}-", self.runtime_epoch)) {
            if let Some(((record_owner, record_workspace, _), _)) = records
                .admissions
                .iter()
                .find(|(_, admission)| admission.task_id == task_id)
            {
                return if record_owner == owner && record_workspace == workspace_id {
                    Err(ApiError::gone(
                        "retention_expired",
                        "task receipt exceeded the advertised terminal TTL or record cap",
                        Some(task_id.to_owned()),
                        None,
                    ))
                } else {
                    Err(ApiError::not_found("task_not_found", "task was not found"))
                };
            }
            return Err(ApiError::gone(
                "task_unresolved",
                "task identity is in the current runtime epoch but no authoritative record remains",
                Some(task_id.to_owned()),
                None,
            ));
        }
        Err(ApiError::not_found("task_not_found", "task was not found"))
    }

    fn admission(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        admission_id: &str,
    ) -> Result<TaskReceipt, ApiError> {
        self.prune();
        let records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let admission = records
            .admissions
            .get(&(
                owner.clone(),
                workspace_id.to_owned(),
                admission_id.to_owned(),
            ))
            .ok_or_else(|| {
                ApiError::not_found(
                    "task_unresolved",
                    "admission has no authoritative process-local record",
                )
            })?;
        records
            .tasks
            .get(&admission.task_id)
            .map(|record| record.receipt.clone())
            .ok_or_else(|| {
                ApiError::gone(
                    "retention_expired",
                    "admission record exceeded the advertised terminal retention",
                    Some(admission.task_id.clone()),
                    Some(admission_id.to_owned()),
                )
            })
    }

    fn prune(&self) {
        let now = Utc::now();
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ttl = self.retention.terminal_ttl_seconds as i64;
        let mut terminal = records
            .tasks
            .values()
            .filter_map(|record| {
                record
                    .receipt
                    .terminal_at
                    .as_ref()
                    .map(|at| (record.receipt.task_id.clone(), at.to_owned()))
            })
            .collect::<Vec<_>>();
        terminal.sort_by_key(|(_, at)| at.to_owned());
        let overflow = if self.retention.terminal_record_cap == 0 {
            0
        } else {
            terminal
                .len()
                .saturating_sub(self.retention.terminal_record_cap)
        };
        let expired = terminal
            .into_iter()
            .enumerate()
            .filter(|(index, (_, at))| {
                *index < overflow || now.signed_duration_since(at.to_owned()).num_seconds() >= ttl
            })
            .map(|(_, (id, _))| id)
            .collect::<Vec<_>>();
        for task_id in expired {
            records.tasks.remove(&task_id);
        }
    }

    async fn refresh_receipt(&self, task_id: &str) -> Option<bool> {
        let (run_id, owner) = {
            let records = self.records.lock().ok()?;
            let record = records.tasks.get(task_id)?;
            (record.receipt.run_id.clone(), record.owner.clone())
        };
        let root_run_id = self.capture_root(task_id).await;
        let run = self.manager.get_run(&run_id).await?;
        let terminal = matches!(
            &run.status,
            RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
        );
        // The live root broker owns pending authority; emitting an approval
        // frame does not change the executor's Running status.
        let pending_approval = if !terminal && let Some(root_run_id) = root_run_id {
            self.manager.pending_approval_for_user(owner.user_id(), &root_run_id).await.is_some()
        } else {
            false
        };
        let history = self.manager.history_since(&run_id, None).await;
        let cursor = history.as_ref().and_then(|events| events.last().map(|event| event.id));
        let cleanup_uncertain = terminal && history.as_ref().is_some_and(|events| {
            events.iter().any(|event| matches!(&event.event,
                crate::uar::domain::events::NormalizedEvent::Error { code, .. }
                    if code.ends_with("cleanup_unconfirmed")))
        });
        self.update(task_id, |record| {
            // An older monitor sample must not undo terminal stream settlement.
            if record.receipt.terminal_at.is_some() && !terminal {
                return;
            }
            record.receipt.cursor = cursor;
            record.receipt.state = if pending_approval {
                "input_required"
            } else {
                match &run.status {
                    RunStatus::Pending => "submitted",
                    RunStatus::Running => "working",
                    RunStatus::Paused => "input_required",
                    RunStatus::Done => "completed",
                    RunStatus::Error => "failed",
                    RunStatus::Cancelled => "cancelled",
                }
            }
            .to_owned();
            if terminal {
                let terminal_at = record.receipt.terminal_at.get_or_insert_with(Utc::now).to_owned();
                record.receipt.expires_at = Some(
                    terminal_at + chrono::Duration::seconds(
                        record.receipt.retention.terminal_ttl_seconds as i64,
                    ),
                );
                record.receipt.cancellation.terminal = matches!(&run.status, RunStatus::Cancelled);
                record.receipt.cancellation.cleanup_uncertain |= cleanup_uncertain;
            }
        });
        Some(terminal)
    }

    fn monitor(self: &Arc<Self>, task_id: String) {
        let authority = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(250)).await;
                if authority.refresh_receipt(&task_id).await == Some(true) {
                    authority.prune();
                    break;
                }
            }
        });
    }

    pub(crate) fn is_task_id(&self, task_id: &str) -> bool {
        task_id.starts_with(&format!("{TASK_PREFIX}-"))
    }
    pub(crate) async fn a2a_get(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        agent_id: &str,
        task_id: &str,
    ) -> Result<crate::uar::api::a2a::types::Task, ApiError> {
        let receipt = self.owned(owner, workspace_id, task_id)?;
        if receipt.agent_id.as_deref() != Some(agent_id) {
            return Err(ApiError::not_found("task_not_found", "task was not found"));
        }
        let _ = self.refresh_receipt(task_id).await;
        Ok(project_a2a(agent_id, self.owned(owner, workspace_id, task_id)?))
    }
}

mod a2a_projection;
mod error;
mod handlers;
pub(crate) mod host_context;
mod mutations;
mod stream;
mod types;

pub(crate) use error::ApiError;
pub use handlers::{FullHarnessApiState, build_router};
pub use host_context::{DelegatedHostContexts, DelegatedHostContextReceipt};
pub use types::{
    CancellationReceipt, RetentionReceipt, RuntimeDescriptor, TaskDiagnostic, TaskLinks,
    TaskReceipt,
};

use a2a_projection::project_a2a;
