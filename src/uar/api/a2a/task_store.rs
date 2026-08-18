//! In-memory A2A task store.
//!
//! Maps A2A [`Task`] records to [`CompilerSession`] IDs, providing the
//! persistence layer for the A2A endpoint. Tasks are stored in memory and
//! keyed by both task ID and context ID.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{Artifact, Message, Part, Task, TaskState, TaskStatus};
use crate::uar::security::claims::TenantId;

type TenantTaskKey = (Option<TenantId>, String);

/// Thread-safe in-memory store for A2A tasks.
#[derive(Debug, Default)]
pub struct TaskStore {
    // (tenant, task_id) → Task
    tasks: RwLock<HashMap<TenantTaskKey, Task>>,
    // (tenant, context_id) → task_id (for multi-turn lookup)
    context_index: RwLock<HashMap<TenantTaskKey, String>>,
}

impl TaskStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Create a new task, optionally linked to a context ID.
    pub async fn create(
        &self,
        tenant_id: Option<&TenantId>,
        context_id: Option<String>,
        session_id: &str,
        initial_message: Message,
    ) -> Task {
        let task_id = Uuid::new_v4().to_string();
        let ctx_id = context_id.unwrap_or_else(|| session_id.to_string());

        let task = Task {
            id: task_id.clone(),
            context_id: Some(ctx_id.clone()),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
            },
            history: vec![initial_message],
            artifacts: vec![],
            metadata: HashMap::new(),
        };

        self.tasks
            .write()
            .await
            .insert((tenant_id.cloned(), task_id.clone()), task.clone());
        self.context_index
            .write()
            .await
            .insert((tenant_id.cloned(), ctx_id), task_id);

        task
    }

    /// Retrieve a task by ID.
    pub async fn get(&self, tenant_id: Option<&TenantId>, task_id: &str) -> Option<Task> {
        self.tasks
            .read()
            .await
            .get(&(tenant_id.cloned(), task_id.to_owned()))
            .cloned()
    }

    /// Retrieve a task by context ID.
    pub async fn get_by_context(
        &self,
        tenant_id: Option<&TenantId>,
        context_id: &str,
    ) -> Option<Task> {
        let task_id = self
            .context_index
            .read()
            .await
            .get(&(tenant_id.cloned(), context_id.to_owned()))?
            .clone();
        self.get(tenant_id, &task_id).await
    }

    /// Append a message to a task's history.
    pub async fn append_message(
        &self,
        tenant_id: Option<&TenantId>,
        task_id: &str,
        message: Message,
    ) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&(tenant_id.cloned(), task_id.to_owned())) {
            task.history.push(message);
            true
        } else {
            false
        }
    }

    /// Update task state.
    pub async fn set_state(
        &self,
        tenant_id: Option<&TenantId>,
        task_id: &str,
        state: TaskState,
        message: Option<Message>,
    ) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&(tenant_id.cloned(), task_id.to_owned())) {
            task.status = TaskStatus {
                state,
                message,
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
            };
            true
        } else {
            false
        }
    }

    /// Add an artifact to a task.
    pub async fn add_artifact(
        &self,
        tenant_id: Option<&TenantId>,
        task_id: &str,
        artifact: Artifact,
    ) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&(tenant_id.cloned(), task_id.to_owned())) {
            task.artifacts.push(artifact);
            true
        } else {
            false
        }
    }

    /// Cancel a task (sets state to Canceled if currently cancellable).
    ///
    /// Returns `true` if the task was cancelled, `false` if not found or
    /// already in a terminal state.
    pub async fn cancel(&self, tenant_id: Option<&TenantId>, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&(tenant_id.cloned(), task_id.to_owned())) {
            match task.status.state {
                TaskState::Submitted | TaskState::Working | TaskState::InputRequired => {
                    task.status = TaskStatus {
                        state: TaskState::Canceled,
                        message: None,
                        timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    };
                    true
                }
                _ => false, // already terminal
            }
        } else {
            false
        }
    }

    /// Build a completed task with a compiled descriptor artifact.
    pub async fn complete_with_descriptor(
        &self,
        tenant_id: Option<&TenantId>,
        task_id: &str,
        descriptor_json: serde_json::Value,
    ) -> bool {
        let artifact = Artifact {
            artifact_id: Uuid::new_v4().to_string(),
            name: Some("compiled-descriptor.json".into()),
            description: Some("Compiled UAR agent descriptor".into()),
            parts: vec![Part::Data {
                data: descriptor_json,
            }],
            metadata: HashMap::new(),
        };

        let added = self.add_artifact(tenant_id, task_id, artifact).await;
        if added {
            self.set_state(
                tenant_id,
                task_id,
                TaskState::Completed,
                Some(Message::agent_text(
                    "Compilation complete. The descriptor artifact is attached.",
                )),
            )
            .await;
        }
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(text: &str) -> Message {
        Message::user_text(text)
    }

    #[tokio::test]
    async fn partitions_task_and_context_lookup_by_tenant() {
        let store = TaskStore::new();
        let tenant_a = TenantId::for_test("tenant-a");
        let tenant_b = TenantId::for_test("tenant-b");
        let task = store
            .create(
                Some(&tenant_a),
                Some("shared-context".to_owned()),
                "session-a",
                message("hello"),
            )
            .await;

        assert!(store.get(Some(&tenant_a), &task.id).await.is_some());
        assert!(store.get(Some(&tenant_b), &task.id).await.is_none());
        assert!(
            store
                .get_by_context(Some(&tenant_a), "shared-context")
                .await
                .is_some()
        );
        assert!(
            store
                .get_by_context(Some(&tenant_b), "shared-context")
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn cross_tenant_cancel_does_not_mutate_task() {
        let store = TaskStore::new();
        let tenant_a = TenantId::for_test("tenant-a");
        let tenant_b = TenantId::for_test("tenant-b");
        let task = store
            .create(Some(&tenant_a), None, "session-a", message("hello"))
            .await;

        assert!(!store.cancel(Some(&tenant_b), &task.id).await);
        let unchanged = store
            .get(Some(&tenant_a), &task.id)
            .await
            .expect("own tenant must retain task");
        assert_eq!(unchanged.status.state, TaskState::Working);
    }
}
