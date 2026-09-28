use std::collections::HashMap;

use serde_json::Value;

use super::TaskReceipt;

pub(super) fn project_a2a(
    agent_id: &str,
    receipt: TaskReceipt,
) -> crate::uar::api::a2a::types::Task {
    use crate::uar::api::a2a::types::{Task, TaskState, TaskStatus};
    let state = match receipt.state.as_str() {
        "completed" => TaskState::Completed,
        "cancelled" => TaskState::Canceled,
        "failed" | "rejected" => TaskState::Failed,
        "input_required" => TaskState::InputRequired,
        "reserved" | "submitted" => TaskState::Submitted,
        _ => TaskState::Working,
    };
    let receipt_metadata = serde_json::to_value(&receipt).ok();
    let mut metadata = HashMap::new();
    metadata.insert("run_id".to_owned(), Value::String(receipt.run_id));
    metadata.insert(
        "native_task_id".to_owned(),
        Value::String(receipt.native_task_id),
    );
    metadata.insert(
        "runtime_epoch".to_owned(),
        Value::String(receipt.runtime_epoch),
    );
    metadata.insert("agent_id".to_owned(), Value::String(agent_id.to_owned()));
    metadata.insert(
        "cleanup_unconfirmed".to_owned(),
        Value::Bool(receipt.cancellation.cleanup_uncertain),
    );
    if let Some(receipt_metadata) = receipt_metadata {
        metadata.insert("uar_full_harness".to_owned(), receipt_metadata);
    }
    Task {
        id: receipt.task_id,
        context_id: None,
        status: TaskStatus {
            state,
            message: None,
            timestamp: receipt.terminal_at.map(|at| at.to_rfc3339()),
        },
        history: Vec::new(),
        artifacts: Vec::new(),
        metadata,
    }
}
