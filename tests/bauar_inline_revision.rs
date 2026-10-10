//! Provider-authoritative inline revision through the real router and tool admission.
//! Source scenarios only until the complete delivery gate is authorized.
#[allow(dead_code)]
#[path = "support/bauar_resource_peer.rs"]
mod peer;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use universal_agent_runtime::uar::domain::artifact::{
    AgentArtifact, AgentArtifactSource, AgentCatalogMetadata, CATALOG_METADATA_EXTENSION,
};
use universal_agent_runtime::uar::runtime::tool_admission::{
    PreparedToolInvocation, ToolExecutionKind, TOOL_ADMISSION_PROTOCOL_VERSION,
};

#[derive(Default)]
struct Admissions {
    invocations: Vec<Value>,
    receipts: HashMap<String, Value>,
}

async fn admission(
    State(state): State<Arc<Mutex<Admissions>>>,
    Path(action): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        != Some("Bearer inline-revision-fixture")
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut state = state.lock().unwrap();
    match action.as_str() {
        "prepare" => {
            let invocation = &body["invocation"];
            let Ok(prepared) = serde_json::from_value::<PreparedToolInvocation>(invocation.clone()) else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            if prepared.validate_authority_envelope().is_err()
                || prepared.execution_kind != ToolExecutionKind::HostMcp
            {
                return StatusCode::CONFLICT.into_response();
            }
            let id = format!("inline-admission-{}", state.invocations.len());
            let receipt = json!({"version":invocation["version"], "admissionId":id,
                "executionKind":invocation["executionKind"],
                "invocationId":invocation["invocationId"], "runtimeEpoch":invocation["runtimeEpoch"],
                "hostEpoch":invocation["hostEpoch"], "authorityRevision":invocation["authorityRevision"],
                "managedMcpMetadata":true});
            state.receipts.insert(id, receipt.clone());
            state.invocations.push(invocation.clone());
            let mut preparation = receipt;
            preparation["hostDisposition"] = json!("auto");
            preparation["actionDisplay"] = json!({"operation":"fixture effect"});
            Json(preparation).into_response()
        }
        "resolve" => match body["admissionId"]
            .as_str()
            .and_then(|id| state.receipts.get(id))
        {
            Some(receipt) => Json(receipt.clone()).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        "claim" => {
            let expected = body["admissionId"]
                .as_str()
                .and_then(|id| state.receipts.get(id));
            if expected != Some(&body["receipt"])
                || !state.invocations.iter().any(|invocation| {
                    invocation == &body["invocation"]
                        && invocation["invocationId"] == body["receipt"]["invocationId"]
                })
            {
                return StatusCode::CONFLICT.into_response();
            }
            Json(body["receipt"].clone()).into_response()
        }
        // This existing revision fixture provides only an MCP receiver.
        "claim-native" => StatusCode::CONFLICT.into_response(),
        "finish" => Json(json!({})).into_response(),
        "cancel" => Json(json!("cancelled")).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

struct AdmissionPeer {
    url: String,
    state: Arc<Mutex<Admissions>>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for AdmissionPeer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl AdmissionPeer {
    async fn start() -> Self {
        let state = Arc::new(Mutex::new(Admissions::default()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/uar/admission/v2", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/uar/admission/v2/{action}", post(admission))
            .with_state(state.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, state, task }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn inline_revision_is_recomputed_before_snapshot_and_tool_authority() {
    let receiver = peer::Peer::start().await;
    let mut host = peer::Host::start(&receiver).await;
    let admission = AdmissionPeer::start().await;
    let source = AgentArtifactSource {
        kind: "boss-projected-artifact".to_owned(),
        id: "source-agent".to_owned(),
        revision: Some("source-r7".to_owned()),
    };
    for (index, input) in ["owner-a-first", "owner-a-second", "owner-b-first"]
        .iter()
        .enumerate()
    {
        let mut artifact: AgentArtifact = serde_json::from_value(sidecar_process::test_agent(
            Some(json!({"tool_approval":"ask"})),
        ))
        .unwrap();
        artifact.id = format!("inline-revision-{index}");
        artifact.extensions.remove(CATALOG_METADATA_EXTENSION);
        if index > 0 {
            artifact.extensions.insert(
                CATALOG_METADATA_EXTENSION.to_owned(),
                serde_json::to_value(AgentCatalogMetadata {
                    schema_version: 1,
                    revision: "stale-client-revision".to_owned(),
                    source: source.clone(),
                })
                .unwrap(),
            );
        }
        // A projected prompt changes content while retaining source provenance.
        if index == 2 {
            artifact
                .prompt
                .system
                .push_str("\nProjected ordinary fixture text.");
        }
        let expected = artifact.clone().with_catalog_metadata("inline");
        let metadata = expected.catalog_metadata().unwrap();
        if index > 0 {
            assert_eq!(metadata.source, source);
        } else {
            assert_eq!(metadata.source.kind, "inline");
        }
        let (status, admitted) = host
            .call(
                Method::POST,
                "/api/uar/runs",
                peer::OWNER_A,
                json!({"artifact":artifact,"input":input,
                "mcp_servers":[peer::grant("a1", &format!("inline-r{index}"), peer::now()+180)],
                "tool_admission":{"version":TOOL_ADMISSION_PROTOCOL_VERSION,"hostEpoch":"inline-host","url":admission.url,
                    "headers":{"Authorization":"Bearer inline-revision-fixture"}}}),
            )
            .await;
        assert_eq!(status, 200, "inline admission failed: {admitted}");
        let run = admitted["run_id"].as_str().unwrap();
        let text = host
            .open(peer::OWNER_A, run)
            .await
            .finish(&host, peer::OWNER_A, run)
            .await;
        assert!(
            text.contains("event: agui.done"),
            "inline run did not finish: {text}"
        );
        let (status, inspection) = host
            .call(
                Method::GET,
                &format!("/api/uar/runs/{run}"),
                peer::OWNER_A,
                Value::Null,
            )
            .await;
        assert_eq!(status, 200);
        assert_eq!(inspection["agent_revision"], metadata.revision);
        let invocations = &admission.state.lock().unwrap().invocations;
        let matching: Vec<_> = invocations
            .iter()
            .filter(|invocation| invocation["rootRunId"] == run)
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "exactly one governed effect per inline run"
        );
        assert_eq!(matching[0]["catalogRevision"], metadata.revision);
        assert_ne!(matching[0]["catalogRevision"], "stale-client-revision");
        assert_eq!(
            receiver.effects().len(),
            index + 1,
            "approved receiver effect is the positive control"
        );
    }
    assert!(host.process.stop_sidecar().await.success());
}
