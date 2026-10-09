//! Acceptance-artifact observation of the real post-claim native dispatch path.

#[cfg(not(feature = "server-full"))]
compile_error!("bauar-native-admission-gate requires the unchanged server-full profile");

use cap_std::fs::{Dir, OpenOptions};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::uar::runtime::tool_admission::{AdmittedToolInvocation, ToolExecutionKind};

const DIRECTORY: &str = ".bauar-post-ack-gate";
const LIMIT: u64 = 4096;
type GateResult<T> = Result<T, &'static str>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arm {
    version: u32,
    purpose: String,
    control_id: String,
    correlation_digest: String,
    execution_kind: String,
    tool_name: String,
    deadline_unix_ms: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Release {
    version: u32,
    control_id: String,
    correlation_digest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Reached<'a> {
    version: u32,
    control_id: &'a str,
    correlation_digest: &'a str,
    phase: &'static str,
    body_entries: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Final<'a> {
    version: u32,
    control_id: &'a str,
    correlation_digest: &'a str,
    phase: &'static str,
    checkpoint_outcome: &'static str,
    body_entries: u64,
    run_cancelled: bool,
    run_failed: bool,
    observer_error: Option<&'static str>,
}

#[derive(Clone, Debug)]
struct Bound {
    directory: Arc<Dir>,
    arm: Arm,
}

#[derive(Debug, Default)]
struct Observation {
    bound: Option<Bound>,
    body_entries: u64,
    outcome: &'static str,
    error: Option<&'static str>,
    sealed: bool,
}

/// Explicitly shared only by one orchestrator and its native discovery body.
#[derive(Debug, Default)]
pub(crate) struct NativeAdmissionGateObserver {
    observation: Mutex<Observation>,
    io_tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}

struct CheckpointDrop<'a>(&'a NativeAdmissionGateObserver);

impl Drop for CheckpointDrop<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.observation.lock() {
            match state.outcome {
                "held" => state.outcome = "dropped_while_held",
                "publishing" => {
                    state.outcome = "failed";
                    state.error = Some("checkpoint_publication_interrupted");
                }
                _ => {}
            }
        }
    }
}

impl NativeAdmissionGateObserver {
    /// Called only after the real consuming claim returned, before the guard.
    pub(crate) async fn checkpoint(
        self: &Arc<Self>,
        admitted: &AdmittedToolInvocation,
        execution_kind: ToolExecutionKind,
    ) -> GateResult<()> {
        if admitted.prepared.provider_tool_name != "search_tools" {
            return Ok(());
        }
        let workspace = admitted.prepared.workspace.clone();
        let bound = self.control_io(move || open_arm(&workspace)).await?;
        let Some(bound) = bound else { return Ok(()); };
        {
            let mut state = self.observation.lock().map_err(|_| "observer_poisoned")?;
            if state.bound.is_some() || state.sealed {
                state.error = Some("duplicate_checkpoint");
                return Err("duplicate_checkpoint");
            }
            state.bound = Some(bound.clone());
            state.outcome = "publishing";
        }
        let drop_guard = CheckpointDrop(self);
        let result = self.hold(admitted, execution_kind, &bound).await;
        if let Err(code) = result {
            if let Ok(mut state) = self.observation.lock() {
                state.outcome = "failed";
                state.error = Some(code);
            }
        }
        drop(drop_guard);
        result
    }

    async fn hold(
        self: &Arc<Self>,
        admitted: &AdmittedToolInvocation,
        execution_kind: ToolExecutionKind,
        bound: &Bound,
    ) -> GateResult<()> {
        let call = admitted.prepared.as_ref();
        let receipt = &admitted.host_receipt;
        if execution_kind != ToolExecutionKind::RuntimeNative
            || call.execution_kind != execution_kind || receipt.execution_kind != execution_kind
            || receipt.invocation_id != call.invocation_id
            || receipt.runtime_epoch != call.runtime_epoch || receipt.host_epoch != call.host_epoch
            || receipt.authority_revision != call.authority_revision
        {
            return Err("claim_correlation_mismatch");
        }
        let encoded = serde_json::to_vec(&[
            "bauar-post-ack-gate/1", call.executing_run_id.as_str(), call.invocation_id.as_str(),
            call.model_tool_call_id.as_str(), call.runtime_epoch.as_str(), call.host_epoch.as_str(),
            "runtime_native", receipt.admission_id.as_str(), call.authority_revision.as_str(),
        ]).map_err(|_| "correlation_encoding_failed")?;
        let digest = Sha256::digest(encoded).iter()
            .map(|byte| format!("{byte:02x}")).collect::<String>();
        if digest != bound.arm.correlation_digest {
            return Err("claim_correlation_mismatch");
        }
        let now = now_ms()?;
        if bound.arm.deadline_unix_ms <= now || bound.arm.deadline_unix_ms - now > 120_000 {
            return Err("control_deadline_invalid");
        }
        let deadline = tokio::time::Instant::now()
            + Duration::from_millis(bound.arm.deadline_unix_ms - now);
        let entries = self.observation.lock().map_err(|_| "observer_poisoned")?.body_entries;
        if entries != 0 { return Err("body_entered_before_checkpoint"); }
        let payload = serde_json::to_vec(&Reached {
            version: 1, control_id: &bound.arm.control_id,
            correlation_digest: &bound.arm.correlation_digest,
            phase: "post_ack_pre_guard", body_entries: entries,
        }).map_err(|_| "control_encoding_failed")?;
        // Publication may become visible before its blocking task's join wakes.
        // The future is already held here; cancellation in that interval is a
        // dropped checkpoint, not a failure to reach the post-ack boundary.
        self.observation.lock().map_err(|_| "observer_poisoned")?.outcome = "held";
        let directory = Arc::clone(&bound.directory);
        tokio::time::timeout_at(deadline, self.control_io(move || publish(&directory, "reached.json", &payload)))
            .await.map_err(|_| "control_deadline_expired")??;
        loop {
            if tokio::time::Instant::now() >= deadline { return Err("control_deadline_expired"); }
            let directory = Arc::clone(&bound.directory);
            let release = tokio::time::timeout_at(deadline, self.control_io(move || {
                read_document::<Release>(&directory, "release.json")
            })).await.map_err(|_| "control_deadline_expired")??;
            if let Some(release) = release {
                if release.version != 1 || release.control_id != bound.arm.control_id
                    || release.correlation_digest != bound.arm.correlation_digest
                { return Err("release_correlation_mismatch"); }
                self.observation.lock().map_err(|_| "observer_poisoned")?.outcome = "released";
                return Ok(());
            }
            tokio::time::sleep_until((tokio::time::Instant::now() + Duration::from_millis(20)).min(deadline)).await;
        }
    }

    /// Records which existing dispatch guard branch ran after checkpoint release.
    pub(crate) fn observe_guard(&self, admitted: &AdmittedToolInvocation, cancelled: bool) {
        if admitted.prepared.provider_tool_name != "search_tools" { return; }
        if let Ok(mut state) = self.observation.lock() {
            if state.bound.is_some() {
                if state.outcome != "released" || state.sealed {
                    state.error = Some("guard_outside_release");
                } else {
                    state.outcome = if cancelled { "released_then_guard_observed" }
                        else { "released_then_guard_allowed" };
                }
            }
        }
    }

    /// First statement of the real SearchToolsTool body, including calibration.
    pub(crate) fn record_native_body_entry(&self) -> GateResult<()> {
        let mut state = self.observation.lock().map_err(|_| "observer_poisoned")?;
        state.body_entries = state.body_entries.saturating_add(1);
        if state.sealed || (state.bound.is_some() && state.outcome != "released_then_guard_allowed") {
            state.error = Some("body_outside_dispatch");
            return Err("body_outside_dispatch");
        }
        Ok(())
    }

    /// Seals explicit body evidence after the actual stream is dropped and cleaned.
    pub(crate) async fn finalize(self: &Arc<Self>, cancelled: bool, failed: bool) -> GateResult<()> {
        let bound = self.observation.lock().map_err(|_| "observer_poisoned")?.bound.clone();
        let Some(bound) = bound else { return Ok(()); };
        // A dropped checkpoint can leave one finite filesystem operation in
        // flight. Join it before publishing a final count or I/O outcome.
        let remaining = bound.arm.deadline_unix_ms.saturating_sub(now_ms()?);
        let deadline = tokio::time::Instant::now() + Duration::from_millis(remaining);
        let tasks = std::mem::take(&mut *self.io_tasks.lock().map_err(|_| "observer_poisoned")?);
        for task in tasks {
            tokio::time::timeout_at(deadline, task).await
                .map_err(|_| "control_finalization_timeout")?
                .map_err(|_| "control_io_panicked")?;
        }
        let (directory, payload) = {
            let mut state = self.observation.lock().map_err(|_| "observer_poisoned")?;
            if state.sealed { return Err("duplicate_finalization"); }
            state.sealed = true;
            if !matches!(state.outcome, "dropped_while_held" | "released_then_guard_observed" | "released_then_guard_allowed" | "failed") {
                state.error = Some("checkpoint_incomplete");
                state.outcome = "failed";
            }
            let payload = serde_json::to_vec(&Final {
                version: 1, control_id: &bound.arm.control_id,
                correlation_digest: &bound.arm.correlation_digest, phase: "final",
                checkpoint_outcome: state.outcome, body_entries: state.body_entries,
                run_cancelled: cancelled, run_failed: failed, observer_error: state.error,
            }).map_err(|_| "control_encoding_failed")?;
            (bound.directory, payload)
        };
        tokio::time::timeout_at(deadline, self.control_io(move || publish(&directory, "final.json", &payload)))
            .await.map_err(|_| "control_finalization_timeout")?
    }

    async fn control_io<T: Send + 'static>(
        self: &Arc<Self>,
        operation: impl FnOnce() -> GateResult<T> + Send + 'static,
    ) -> GateResult<T> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let observer = Arc::clone(self);
        let task = tokio::task::spawn_blocking(move || {
            let result = operation();
            if let Err(code) = &result {
                if let Ok(mut state) = observer.observation.lock() {
                    state.error.get_or_insert(*code);
                }
            }
            // Cancellation can drop the receiver; the manager still joins the
            // operation and inspects the recorded outcome before finalization.
            let _ = sender.send(result);
        });
        {
            let mut tasks = self.io_tasks.lock().map_err(|_| "observer_poisoned")?;
            tasks.retain(|task| !task.is_finished());
            tasks.push(task);
        }
        receiver.await.map_err(|_| "control_io_interrupted")?
    }
}

fn now_ms() -> GateResult<u64> {
    SystemTime::now().duration_since(UNIX_EPOCH).ok()
        .and_then(|value| u64::try_from(value.as_millis()).ok()).ok_or("control_clock_invalid")
}

fn open_arm(workspace: &str) -> GateResult<Option<Bound>> {
    let path = Path::new(workspace);
    if !path.is_absolute() || std::fs::canonicalize(path).map_err(|_| "workspace_unavailable")? != path {
        return Err("workspace_not_canonical");
    }
    let root = Dir::open_ambient_dir(path, cap_std::ambient_authority()).map_err(|_| "workspace_unavailable")?;
    let metadata = match root.symlink_metadata(DIRECTORY) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("control_directory_unavailable"),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() { return Err("control_directory_invalid"); }
    let directory = Arc::new(root.open_dir(DIRECTORY).map_err(|_| "control_directory_unavailable")?);
    let arm = read_document::<Arm>(&directory, "arm.json")?.ok_or("arm_missing")?;
    if arm.version != 1 || arm.purpose != "bauar-native-post-ack"
        || arm.execution_kind != "runtime_native" || arm.tool_name != "search_tools"
        || uuid::Uuid::parse_str(&arm.control_id).is_err()
        || arm.correlation_digest.len() != 64
        || !arm.correlation_digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    { return Err("arm_invalid"); }
    for name in ["reached.json", "release.json", "final.json"] {
        match directory.symlink_metadata(name) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("control_not_fresh"),
        }
    }
    Ok(Some(Bound { directory, arm }))
}

fn read_document<T: DeserializeOwned>(directory: &Dir, name: &str) -> GateResult<Option<T>> {
    let metadata = match directory.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("control_read_failed"),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > LIMIT {
        return Err("control_file_invalid");
    }
    let file = directory.open(name).map_err(|_| "control_read_failed")?;
    if !file.metadata().map_err(|_| "control_read_failed")?.is_file() { return Err("control_file_invalid"); }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes).map_err(|_| "control_read_failed")?;
    if bytes.len() as u64 > LIMIT { return Err("control_file_invalid"); }
    serde_json::from_slice(&bytes).map(Some).map_err(|_| "control_json_invalid")
}

fn publish(directory: &Dir, name: &str, bytes: &[u8]) -> GateResult<()> {
    if bytes.len() as u64 > LIMIT { return Err("control_file_invalid"); }
    match directory.symlink_metadata(name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return Err("control_output_exists"),
    }
    let temporary = format!("{name}.tmp");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = directory.open_with(&temporary, &options).map_err(|_| "control_write_failed")?;
    file.write_all(bytes).map_err(|_| "control_write_failed")?;
    file.sync_all().map_err(|_| "control_write_failed")?;
    drop(file);
    directory.rename(&temporary, directory, name).map_err(|_| "control_write_failed")
}
