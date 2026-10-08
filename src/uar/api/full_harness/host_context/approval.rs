//! Host records the exact decision before the native approval waiter is resolved.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;
use super::{ApiError, DelegatedHostContext, FullHarnessApiState, TaskReceipt, UserContext};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Acknowledgement {
    version: u32,
    context_id: String,
    run_id: String,
    issuer_id: String,
    challenge_id: String,
    admission_id: Option<String>,
    approved: bool,
    recorded: bool,
}

pub(super) async fn coordinate(context: &DelegatedHostContext, state: &FullHarnessApiState,
    user: &UserContext, receipt: &TaskReceipt, approval_id: &str, approved: bool) -> Result<(), ApiError> {
    context.require_live()?;
    let pending = state.authority.manager.pending_approval_for_user(&user.user_id, &receipt.run_id)
        .await.ok_or_else(mismatch)?;
    if pending.approval.approval_id != approval_id
        || pending.approval.root_run_id != receipt.run_id
        || pending.approval.admission_id.is_none()
        || pending.approval.admission_owner != crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost
    { return Err(mismatch()); }
    // The base and authorization were validated by the existing private adapter
    // at host registration. No URL, headers or transport errors are disclosed.
    let mut url = url::Url::parse(context.admission_input.url.expose_secret()).map_err(|_| failed("private callback unavailable"))?;
    url.set_path("/uar/admission/v1/delegated-approval");
    let mut headers = HeaderMap::new();
    for (name, value) in &context.admission_input.headers {
        headers.insert(HeaderName::from_bytes(name.as_bytes()).map_err(|_| failed("private callback unavailable"))?,
            HeaderValue::from_str(value.expose_secret()).map_err(|_| failed("private callback unavailable"))?);
    }
    let client = reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none())
        .build().map_err(|_| failed("private callback unavailable"))?;
    let response = client.post(url).headers(headers).json(&json!({
        "version": 1, "contextId": context.receipt.context_id,
        "grantId": context.receipt.grant_id, "runtimeEpoch": context.receipt.runtime_epoch,
        "workspaceId": context.receipt.workspace_id, "workingDirectory": context.working_directory,
        "definition": context.receipt.definition, "binding": context.receipt.binding,
        "taskId": receipt.task_id, "nativeTaskId": receipt.native_task_id, "runId": receipt.run_id,
        "approval": pending, "approved": approved,
    })).send().await.map_err(|_| failed("POST private approval callback transport failed"))?;
    if !response.status().is_success() {
        return Err(failed(&format!("POST private approval callback HTTP {}", response.status().as_u16())));
    }
    let ack = response.json::<Acknowledgement>().await.map_err(|_| failed("POST private approval callback response invalid"))?;
    if ack.version != 1 || !ack.recorded || ack.context_id != context.receipt.context_id
        || ack.run_id != receipt.run_id || ack.issuer_id != pending.approval.issuer_id
        || ack.challenge_id != pending.approval.challenge_id
        || ack.admission_id != pending.approval.admission_id || ack.approved != approved
    { return Err(mismatch()); }
    context.require_live()?;
    let current = state.authority.manager.pending_approval_for_user(&user.user_id, &receipt.run_id)
        .await.ok_or_else(mismatch)?;
    if current.approval.approval_id != pending.approval.approval_id
        || current.approval.issuer_id != pending.approval.issuer_id
        || current.approval.challenge_id != pending.approval.challenge_id
        || current.approval.admission_id != pending.approval.admission_id
        || current.approval.tool_call_id != pending.approval.tool_call_id
    { return Err(mismatch()); }
    Ok(())
}

fn mismatch() -> ApiError {
    ApiError::conflict("delegated_host_approval_mismatch", "exact live host approval challenge must match", None, None)
}
fn failed(message: &str) -> ApiError {
    ApiError::unprocessable("delegated_host_approval_failed", message, None)
}
