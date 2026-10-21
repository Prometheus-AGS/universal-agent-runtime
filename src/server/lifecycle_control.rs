//! Trusted host control outside database-backed request authorization.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::uar::{realtime::lifecycle::BusLifecycle, runtime::lifecycle::RuntimeHandle};
use super::lifecycle_reconnect::{ReconnectTransport, TransportSnapshot};

#[derive(Default)]
struct ControlState {
    runtime: Option<RuntimeHandle>,
    buses: BTreeMap<String, Arc<BusLifecycle>>,
    transports: BTreeMap<String, Arc<ReconnectTransport>>,
}

/// A host capability. Sidecars expose it only over their inherited launch pipe.
#[derive(Clone, Default)]
pub struct ServerLifecycleControl(Arc<Mutex<ControlState>>);

impl std::fmt::Debug for ServerLifecycleControl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ServerLifecycleControl(***redacted***)")
    }
}

/// Versioned command sent by the already authenticated launching parent.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRequest {
    pub version: u8,
    pub request_id: String,
    pub operation: LifecycleOperation,
}

impl std::fmt::Debug for LifecycleRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LifecycleRequest")
            .field("version", &self.version)
            .field("request_id", &"***redacted***")
            .field("operation", &self.operation)
            .finish()
    }
}

/// No operation changes durable ownership or bypasses business authorization.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LifecycleOperation {
    Snapshot,
    Reconnect { resource_id: String, expected_generation: u64 },
}

impl std::fmt::Debug for LifecycleOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Snapshot => formatter.write_str("LifecycleOperation::Snapshot"),
            Self::Reconnect {
                expected_generation,
                ..
            } => formatter
                .debug_struct("LifecycleOperation::Reconnect")
                .field("resource_id", &"***redacted***")
                .field("expected_generation", expected_generation)
                .finish(),
        }
    }
}

impl ServerLifecycleControl {
    pub(super) fn attach(&self, runtime: RuntimeHandle) {
        self.0.lock().unwrap_or_else(|error| error.into_inner()).runtime = Some(runtime);
    }

    pub(super) fn register_bus(&self, name: &str, bus: Arc<BusLifecycle>) {
        self.0.lock().unwrap_or_else(|error| error.into_inner()).buses.insert(name.to_owned(), bus);
    }

    pub(super) fn register_transport(&self, name: &str, transport: Arc<ReconnectTransport>) {
        self.0.lock().unwrap_or_else(|error| error.into_inner()).transports.insert(name.to_owned(), transport);
    }

    fn connection_snapshot(&self) -> serde_json::Value {
        let (buses, transports) = {
            let state = self.0.lock().unwrap_or_else(|error| error.into_inner());
            (state.buses.clone(), state.transports.clone())
        };
        let subscription_generations = buses.iter().map(|(name, bus)| (name.clone(), bus.generation()))
            .collect::<BTreeMap<_, _>>();
        let subscriptions = buses.into_iter().map(|(name, bus)| (name, bus.states().borrow().clone()))
            .collect::<BTreeMap<_, _>>();
        let transports = transports.into_iter().map(|(name, transport)| (name, transport.snapshot()))
            .collect::<BTreeMap<String, TransportSnapshot>>();
        serde_json::json!({"subscriptions": subscriptions, "subscription_generations": subscription_generations,
            "transports": transports})
    }

    /// Execute a host command without consulting an unavailable database.
    pub async fn execute(&self, request: LifecycleRequest) -> anyhow::Result<serde_json::Value> {
        anyhow::ensure!(request.version == 1, "unsupported lifecycle protocol version");
        anyhow::ensure!(!request.request_id.is_empty() && request.request_id.len() <= 128,
            "invalid lifecycle request identity");
        let result = match request.operation {
            LifecycleOperation::Snapshot => {
                let runtime = {
                    let state = self.0.lock().unwrap_or_else(|error| error.into_inner());
                    state.runtime.clone()
                };
                let runtime = runtime.ok_or_else(|| anyhow::anyhow!("runtime is initializing"))?;
                let mut resources = Vec::new();
                for resource in runtime.resources().await {
                    let report = runtime.resource_report(&resource.resource_id).await?;
                    resources.push(serde_json::json!({
                    "resource_id": resource.resource_id, "owner": resource.owner,
                    "generation": resource.generation, "state": format!("{:?}", resource.state),
                    "in_flight_operations": resource.in_flight_operations,
                    "outstanding_operation_ids": resource.outstanding_operation_ids,
                    "ownership": format!("{:?}", resource.ownership),
                    "dependencies": resource.dependencies,
                    "tracked_tasks": resource.tracked_tasks,
                    "failure": resource.failure.as_ref().map(|_| "resource_failure"),
                    "failure_categories": resource.failure_categories,
                    "unresolved_effects": resource.unresolved_effects,
                    "dependency_failures": resource.dependency_failures.iter().map(dependency_failure).collect::<Vec<_>>(),
                    "close_report": report.as_ref().map(close_report),
                    }));
                }
                let mut connections = self.connection_snapshot();
                connections["resources"] = serde_json::json!(resources);
                connections
            }
            LifecycleOperation::Reconnect { resource_id, expected_generation } => {
                let runtime = self.0.lock().unwrap_or_else(|error| error.into_inner()).runtime.clone()
                    .ok_or_else(|| anyhow::anyhow!("runtime is initializing"))?;
                let generation = match runtime.reconnect(&resource_id, expected_generation).await {
                    Ok(generation) => generation,
                    Err(_) => {
                        let mut details = self.connection_snapshot();
                        details["resource_id"] = serde_json::json!(resource_id);
                        details["expected_generation"] = serde_json::json!(expected_generation);
                        return Ok(serde_json::json!({"version": 1, "request_id": request.request_id,
                            "error": "lifecycle_reconnect_failed", "details": details}));
                    }
                };
                serde_json::json!({"resource_id": resource_id, "generation": generation, "outcome": "acknowledged"})
            }
        };
        Ok(serde_json::json!({"version": 1, "request_id": request.request_id, "result": result}))
    }
}

fn dependency_failure(failure: &crate::uar::runtime::lifecycle::LifecycleDependencyFailure) -> serde_json::Value {
    use crate::uar::runtime::lifecycle::LifecycleDependencyFailure;
    match failure {
        LifecycleDependencyFailure::Missing { resource_id, dependency_id } =>
            serde_json::json!({"category": "missing_dependency", "resource_id": resource_id, "dependency_id": dependency_id}),
        LifecycleDependencyFailure::Cycle { resource_ids } =>
            serde_json::json!({"category": "dependency_cycle", "resource_ids": resource_ids}),
    }
}

fn close_report(report: &crate::uar::runtime::lifecycle::CloseReport) -> serde_json::Value {
    serde_json::json!({
        "resource_id": report.resource_id, "generation": report.generation,
        "released": report.released, "completed_operations": report.completed_operations,
        "completed_tasks": report.completed_tasks, "failure_count": report.failures.len(),
        "unresolved_effects": report.unresolved_effects,
        "dependency_failures": report.dependency_failures.iter().map(dependency_failure).collect::<Vec<_>>(),
        "child_reports": report.child_reports.iter().map(close_report).collect::<Vec<_>>(),
    })
}
