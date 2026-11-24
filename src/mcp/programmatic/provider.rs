//! The host owns the job process. No guest-supplied command, destination, or environment enters it.
use super::{DemoConfig, JobError, RECIPE, digest, is_ref};
use crate::mcp::{interaction::RunMcpClient, stdio_process::StdioProcessSupervisor};
use rmcp::{
    model::CallToolRequestParams,
    service::{RoleClient, RunningService, ServiceExt},
};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
use tokio::{
    fs,
    io::AsyncReadExt,
    process::Command,
    time::{Instant, sleep, timeout},
};
use tokio_util::sync::CancellationToken;

type Client = RunningService<RoleClient, RunMcpClient>;

pub(crate) struct Receipt {
    pub id: String,
    pub result: Value,
}

pub(crate) async fn run_job(
    config: &DemoConfig,
    owner: &str,
    run: &str,
    input: Value,
    cancel: &CancellationToken,
) -> Result<Receipt, JobError> {
    if !config.bunbox_command.is_absolute()
        || !config.msb_command.is_absolute()
        || !config.msb_home.is_absolute()
        || !config.state_dir.is_absolute()
        || !is_ref(&config.bunbox_sha256)
    {
        return Err(JobError::InvalidJobConfiguration);
    }
    use sha2::{Digest, Sha256};
    let mut binary = fs::File::open(&config.bunbox_command)
        .await
        .map_err(|_| JobError::ExecutorUnavailable)?;
    let mut hash = Sha256::new();
    let mut buf = vec![0; 64 * 1024];
    loop {
        let n = binary
            .read(&mut buf)
            .await
            .map_err(|_| JobError::ExecutorUnavailable)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    let actual: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != config.bunbox_sha256 {
        return Err(JobError::ExecutorPinMismatch);
    }
    let mut cmd = Command::new(&config.bunbox_command);
    cmd.env_clear()
        .env("BUNBOX_OWNER", owner)
        .env("BUNBOX_RUN", run)
        .env("BUNBOX_STATE_DIR", &config.state_dir)
        .env("BUNBOX_MSB", &config.msb_command)
        .env("MSB_HOME", &config.msb_home);
    let supervisor = StdioProcessSupervisor::default();
    let transport = supervisor
        .spawn(cmd)
        .map_err(|_| JobError::ExecutorUnavailable)?;
    let connected = timeout(
        Duration::from_secs(10),
        RunMcpClient::default().serve(transport),
    )
    .await;
    let result = match connected {
        Ok(Ok(client)) => {
            let result = dispatch(&client, config, owner, run, input, cancel).await;
            drop(client);
            result
        }
        _ => Err(JobError::ExecutorUnavailable),
    };
    supervisor.request_force_stop();
    if !matches!(
        timeout(Duration::from_secs(5), supervisor.shutdown()).await,
        Ok(Ok(()))
    ) {
        return Err(JobError::CleanupUnknown);
    }
    result
}

async fn call(
    client: &Client,
    name: &'static str,
    args: Value,
    limit: Duration,
) -> Result<Value, JobError> {
    let params = CallToolRequestParams::new(name)
        .with_arguments(args.as_object().cloned().unwrap_or_default());
    let reply = timeout(limit, client.peer().call_tool(params))
        .await
        .map_err(|_| JobError::OutcomeUnknown)?
        .map_err(|_| JobError::OutcomeUnknown)?;
    if reply.is_error == Some(true) {
        return Err(
            match reply
                .structured_content
                .as_ref()
                .and_then(|value| value["reason"].as_str())
            {
                Some("venue_unavailable") => JobError::ExecutorUnavailable,
                Some("venue_busy") => JobError::ExecutorBusy,
                Some("venue_reconciliation_required") => JobError::CleanupUnknown,
                Some("request_key_conflict") => JobError::IntentScopeChanged,
                Some("unreviewed_recipe") => JobError::RecipePinMismatch,
                Some(
                    "invalid_input"
                    | "invalid_row"
                    | "source_ref_conflict"
                    | "input_limit"
                    | "invalid_request_key",
                ) => JobError::JobRejected,
                _ => JobError::OutcomeUnknown,
            },
        );
    }
    let value = reply
        .structured_content
        .ok_or(JobError::InvalidJobReceipt)?;
    if serde_json::to_vec(&value)
        .map_err(|_| JobError::InvalidJobReceipt)?
        .len()
        > 64 * 1024
    {
        return Err(JobError::InvalidJobReceipt);
    }
    Ok(value)
}

async fn dispatch(
    client: &Client,
    config: &DemoConfig,
    owner: &str,
    run: &str,
    input: Value,
    cancel: &CancellationToken,
) -> Result<Receipt, JobError> {
    // A fsynced prefix survives a lost submit reply. Re-entering this run cannot submit twice.
    let key =
        digest(&serde_json::to_vec(&(owner, run, RECIPE)).map_err(|_| JobError::InvalidJobScope)?);
    let record = config.state_dir.join(format!("uar-intent-{key}"));
    let input_hash =
        digest(&serde_json::to_vec(&input).map_err(|_| JobError::InvalidHeaderProjection)?);
    if cancel.is_cancelled() {
        return Err(JobError::JobCancelled);
    }
    let mut receipt = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&record)
        .await
    {
        Ok(marker) => {
            marker
                .sync_all()
                .await
                .map_err(|_| JobError::IntentUnavailable)?;
            sync_dir(&config.state_dir).await?;
            if cancel.is_cancelled() {
                return Err(JobError::JobCancelled);
            }
            let result = call(
                client,
                "submit",
                json!({"requestKey":key,"recipeSha256":RECIPE,"input":input}),
                Duration::from_secs(45),
            )
            .await?;
            let id = job_id(&result)?;
            let tmp = record.with_extension("receipt.tmp");
            fs::write(
                &tmp,
                serde_json::to_vec(&(id, &input_hash)).map_err(|_| JobError::OutcomeUnknown)?,
            )
            .await
            .map_err(|_| JobError::OutcomeUnknown)?;
            fs::File::open(&tmp)
                .await
                .map_err(|_| JobError::OutcomeUnknown)?
                .sync_all()
                .await
                .map_err(|_| JobError::OutcomeUnknown)?;
            fs::rename(tmp, &record)
                .await
                .map_err(|_| JobError::OutcomeUnknown)?;
            sync_dir(&config.state_dir)
                .await
                .map_err(|_| JobError::OutcomeUnknown)?;
            result
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let bytes = fs::read(&record)
                .await
                .map_err(|_| JobError::OutcomeUnknown)?;
            let (id, prior): (String, String) =
                serde_json::from_slice(&bytes).map_err(|_| JobError::OutcomeUnknown)?;
            if prior != input_hash {
                return Err(JobError::IntentScopeChanged);
            }
            let result = call(client, "get", json!({"jobId":id}), Duration::from_secs(5)).await?;
            if job_id(&result)? != id {
                return Err(JobError::InvalidJobReceipt);
            }
            result
        }
        _ => return Err(JobError::IntentUnavailable),
    };
    let id = job_id(&receipt)?.to_string();
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        if job_id(&receipt)? != id || receipt["recipeSha256"] != RECIPE {
            return Err(JobError::InvalidJobReceipt);
        }
        if cancel.is_cancelled() || Instant::now() >= deadline {
            let stopped = call(
                client,
                "cancel",
                json!({"jobId":id}),
                Duration::from_secs(5),
            )
            .await?;
            return Err(
                if job_id(&stopped)? == id && stopped["status"] == "cancelled" {
                    JobError::JobCancelled
                } else {
                    JobError::CleanupUnknown
                },
            );
        }
        match receipt["status"].as_str() {
            Some("succeeded") => {
                return Ok(Receipt {
                    id,
                    result: receipt
                        .get_mut("result")
                        .ok_or(JobError::InvalidJobResult)?
                        .take(),
                });
            }
            Some("accepted" | "running") => {}
            Some("cancelled") => return Err(JobError::JobCancelled),
            Some("failed") => return Err(JobError::JobFailed),
            Some("cleanup_unknown") => return Err(JobError::CleanupUnknown),
            Some("outcome_unknown") => return Err(JobError::OutcomeUnknown),
            _ => return Err(JobError::InvalidJobReceipt),
        }
        tokio::select! {()=cancel.cancelled()=>{},()=sleep(Duration::from_millis(100))=>{}}
        if !cancel.is_cancelled() {
            receipt = call(client, "get", json!({"jobId":id}), Duration::from_secs(5)).await?;
        }
    }
}

async fn sync_dir(path: &Path) -> Result<(), JobError> {
    fs::File::open(path)
        .await
        .map_err(|_| JobError::IntentUnavailable)?
        .sync_all()
        .await
        .map_err(|_| JobError::IntentUnavailable)
}

fn job_id(v: &Value) -> Result<&str, JobError> {
    v["jobId"]
        .as_str()
        .filter(|s| is_ref(s))
        .ok_or(JobError::InvalidJobReceipt)
}
