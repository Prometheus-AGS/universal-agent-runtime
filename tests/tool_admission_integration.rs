use std::sync::Arc;

use universal_agent_runtime::uar::runtime::tool_admission::{
    HostAdmissionDisposition, HostToolAdmissionPort, LocalAdmissionDisposition,
    PreparedToolInvocation, StandaloneToolAdmissionPort, TOOL_ADMISSION_PROTOCOL_VERSION,
};
use universal_agent_runtime::uar::tools::descriptor::ApprovalClass;

fn invocation(host_epoch: String) -> Arc<PreparedToolInvocation> {
    Arc::new(PreparedToolInvocation {
        version: TOOL_ADMISSION_PROTOCOL_VERSION,
        invocation_id: uuid::Uuid::new_v4().to_string(),
        model_tool_call_id: "standalone-model-call".to_owned(),
        attempt: 1,
        root_run_id: "standalone-root".to_owned(),
        executing_run_id: "standalone-root".to_owned(),
        owner_id: "standalone".to_owned(),
        workspace: "standalone-workspace".to_owned(),
        runtime_epoch: "standalone-runtime".to_owned(),
        host_epoch,
        catalog_revision: "standalone-catalog".to_owned(),
        mounted_server_id: "builtin".to_owned(),
        native_tool_name: "read".to_owned(),
        provider_tool_name: "read".to_owned(),
        run_policy_revision: "standalone-policy".to_owned(),
        tool_policy_revision: "standalone-tool-policy".to_owned(),
        approval_class: ApprovalClass::NotRequired,
        call_index: 0,
        validated_arguments: serde_json::json!({"path": "README.md"}),
    })
}

#[tokio::test]
async fn standalone_exact_admission_remains_compatible() {
    let port = StandaloneToolAdmissionPort::new("standalone-runtime");
    let prepared = invocation(port.binding().host_epoch);
    let host = port
        .prepare(Arc::clone(&prepared))
        .await
        .expect("standalone preparation succeeds");
    assert_eq!(host.host_disposition, HostAdmissionDisposition::Auto);
    assert!(!host.managed_mcp_metadata);

    let admitted = port
        .resolve(
            Arc::clone(&prepared),
            host,
            LocalAdmissionDisposition::Allowed,
            true,
        )
        .await
        .expect("standalone resolution succeeds")
        .expect("approved standalone invocation has a receipt");
    assert_eq!(admitted.prepared.invocation_id, prepared.invocation_id);
    assert!(admitted.host_receipt.mcp_request_meta().is_none());

    let denied_invocation = invocation(port.binding().host_epoch);
    let denied_preparation = port
        .prepare(Arc::clone(&denied_invocation))
        .await
        .expect("a new standalone preparation succeeds");
    assert!(
        port.resolve(
            denied_invocation,
            denied_preparation,
            LocalAdmissionDisposition::Denied,
            false,
        )
        .await
        .expect("standalone denial resolves")
        .is_none()
    );
}
