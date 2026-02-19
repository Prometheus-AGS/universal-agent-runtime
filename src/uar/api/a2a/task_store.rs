//! In-memory A2A task store.
//!
//! Maps A2A [`Task`] records to [`CompilerSession`] IDs, providing the
//! persistence layer for the A2A endpoint. Tasks are stored in memory and
//! keyed by both task ID and context ID.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{Artifact, Message, Part, Task, TaskState, TaskStatus};

/// Thread-safe in-memory store for A2A tasks.
#[derive(Debug, Default)]
pub struct TaskStore {
    /// task_id → Task
    tasks: RwLock<HashMap<String, Task>>,
    /// context_id → task_id (for multi-turn lookup)
    context_index: RwLock<HashMap<String, String>>,
}

impl TaskStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Create a new task, optionally linked to a context ID.
    pub async fn create(
        &self,
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
            .insert(task_id.clone(), task.clone());
        self.context_index.write().await.insert(ctx_id, task_id);

        task
    }

    /// Retrieve a task by ID.
    pub async fn get(&self, task_id: &str) -> Option<Task> {
        self.tasks.read().await.get(task_id).cloned()
    }

    /// Retrieve a task by context ID.
    pub async fn get_by_context(&self, context_id: &str) -> Option<Task> {
        let task_id = self.context_index.read().await.get(context_id)?.clone();
        self.tasks.read().await.get(&task_id).cloned()
    }

    /// Append a message to a task's history.
    pub async fn append_message(&self, task_id: &str, message: Message) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.history.push(message);
            true
        } else {
            false
        }
    }

    /// Update task state.
    pub async fn set_state(
        &self,
        task_id: &str,
        state: TaskState,
        message: Option<Message>,
    ) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
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
    pub async fn add_artifact(&self, task_id: &str, artifact: Artifact) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
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
    pub async fn cancel(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
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

        let added = self.add_artifact(task_id, artifact).await;
        if added {
            self.set_state(
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
