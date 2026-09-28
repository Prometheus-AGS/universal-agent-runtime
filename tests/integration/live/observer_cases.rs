//! C07 phase gate: real UAR HTTP host, durable SurrealKV and independent observers.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};

use super::backend::resolve;
use super::harness::{
    HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server_process_with_instance_id,
    mint_harness_peer_token,
};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint};

const WORKSPACE: &str = "workspace:c03";
const OTHER_WORKSPACE: &str = "workspace:c07-other";
const BINDING_ID: &str = "urn:uar:c03:binding";
const SERVICE_INSTANCE_ID: &str = "agent-instance:c03";
const AGENT: &str = include_str!("../../fixtures/collaboration/agent-definition.json");
const TEAM: &str = include_str!("../../fixtures/collaboration/team-definition.json");
const WORKFLOW: &str = include_str!("../../fixtures/collaboration/workflow-definition.json");
const MANIFEST: &str = include_str!("../../fixtures/collaboration/package-manifest.json");
const GRANT: &str = include_str!("../../fixtures/collaboration/representation-grant-v1.json");
const BINDING: &str = include_str!("../../fixtures/collaboration/deployment-binding.json");

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

fn binding(model: &str) -> Value {
    let mut value: Value = serde_json::from_str(BINDING).expect("C07 binding fixture");
    value["modelBindings"][0]["modelId"] =
        json!(model.split_once('/').map_or(model, |(_, model)| model));
    value
        .as_object_mut()
        .expect("binding object")
        .remove("contentDigest");
    let digest = Sha256::digest(serde_json::to_vec(&canonical(&value)).expect("binding JSON"));
    let mut encoded = String::from("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("hex digest");
    }
    value["contentDigest"] = json!(encoded);
    value
}

async fn request(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    workspace: &str,
    method: Method,
    path: &str,
    body: Option<Value>,
    expected: StatusCode,
) -> Value {
    let mut request = client
        .request(method, format!("{base}{path}"))
        .bearer_auth(token)
        .header("x-uar-workspace-id", workspace);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.expect("C07 host HTTP request");
    let status = response.status();
    let text = response.text().await.expect("C07 host HTTP response");
    assert!(
        !text.contains("api_route_not_found"),
        "unmounted C07 route: {path}"
    );
    let value = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text));
    assert_eq!(status, expected, "C07 {path}: {value}");
    value
}

async fn get(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    workspace: &str,
    path: &str,
    expected: StatusCode,
) -> Value {
    request(
        client,
        base,
        token,
        workspace,
        Method::GET,
        path,
        None,
        expected,
    )
    .await
}

async fn post(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    workspace: &str,
    path: &str,
    body: Value,
    expected: StatusCode,
) -> Value {
    request(
        client,
        base,
        token,
        workspace,
        Method::POST,
        path,
        Some(body),
        expected,
    )
    .await
}

async fn observer(client: &reqwest::Client, base: &str, token: &str, id: &str) -> Value {
    get(
        client,
        base,
        token,
        WORKSPACE,
        &format!("/api/uar/observers/v1/{id}"),
        StatusCode::OK,
    )
    .await
}

fn operator_token() -> String {
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "live-harness-peer".to_owned(),
        name: Some("C07 gate operator".to_owned()),
        roles: Some(vec!["operator".to_owned()]),
        tenant_id: None,
        uar_instance_id: None,
        exp: usize::MAX,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
    )
    .expect("mint C07 operator token")
}

async fn wait_for_inbox_command(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    id: &str,
    source_command: &str,
    status: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        let view = observer(client, base, token, id).await;
        if let Some(entry) = view["subscription"]["inbox"].as_array().and_then(|inbox| {
            inbox.iter().find(|entry| {
                entry["occurrence"]["command_id"] == source_command && entry["status"] == status
            })
        }) {
            return entry.clone();
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "observer {id} did not {status} {source_command}: {view}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_command(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    instance: &str,
    command: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        let view = get(
            client,
            base,
            token,
            WORKSPACE,
            &format!("/api/uar/agent-instances/v1/{instance}"),
            StatusCode::OK,
        )
        .await;
        if let Some(receipt) = view["commands"]
            .as_array()
            .and_then(|commands| commands.iter().find(|item| item["commandId"] == command))
        {
            match receipt["status"].as_str() {
                Some("completed") => return receipt.clone(),
                Some("failed" | "cancelled" | "uncertain") => {
                    panic!("C07 command did not complete: {view}")
                }
                _ => {}
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "C07 command {command} did not settle: {view}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_drained_observer(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    id: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        let view = observer(client, base, token, id).await;
        if view["backlogDepth"] == 0 {
            return view;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "C07 observer did not drain: {view}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn turn(client: &reqwest::Client, base: &str, token: &str, source: &str, command: &str) {
    post(
        client,
        base,
        token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{source}/turns"),
        json!({"commandId":command,"prompt":command}),
        StatusCode::ACCEPTED,
    )
    .await;
    wait_for_command(client, base, token, source, command).await;
}

async fn create_instance(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    retained_events: u32,
) -> Value {
    post(
        client,
        base,
        token,
        WORKSPACE,
        "/api/uar/agent-instances/v1",
        json!({"deploymentBindingId":BINDING_ID,"profile":"on_demand",
            "limits":{"max_inbox":8,"retained_commands":32,"retained_events":retained_events,
                "max_restart_attempts":3,"idle_timeout_secs":300}}),
        StatusCode::CREATED,
    )
    .await
}

fn inbox_has_command(view: &Value, source_command: &str) -> bool {
    view["subscription"]["inbox"]
        .as_array()
        .is_some_and(|inbox| {
            inbox
                .iter()
                .any(|entry| entry["occurrence"]["command_id"] == source_command)
        })
}

#[tokio::test]
#[serial]
async fn c07_observers_complete_live_boundary() {
    assert!(include_str!("../../../versions.toml").contains("surrealdb = \"3.3.0\""));
    let original_skills = std::env::var_os("UAR_BUILTIN_SKILLS_DIR");
    // SAFETY: this is the sole serial C07 gate and restores the process fixture root.
    unsafe {
        std::env::set_var(
            "UAR_BUILTIN_SKILLS_DIR",
            "tests/fixtures/collaboration/builtin-skills",
        )
    };
    let scratch = tempfile::tempdir().expect("C07 SurrealKV scratch");
    let db_path = scratch.path().join("surrealkv");
    let mut fixtures = FixtureSet::new().with_prompt_prefix(
        "{\"observation\":",
        FixtureResponse::Content("C07_OBSERVER_ACK".to_owned()),
    );
    for command in [
        "C07_SOURCE_A",
        "C07_SOURCE_B",
        "C07_SOURCE_C",
        "C07_PAUSED_SOURCE",
        "C07_QUEUED_SOURCE",
    ] {
        fixtures = fixtures.with(
            RequestFingerprint {
                model: "gpt-5.4-mini".to_owned(),
                last_user_message: command.to_owned(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content(format!("{command}_DONE")),
        );
    }
    let backend = resolve(fixtures).await;
    let reserved = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("reserve C07 HTTP port for both process boots");
    let fixed_http_port = reserved.local_addr().expect("reserved C07 address").port();
    drop(reserved);
    let first = boot_test_server_process_with_instance_id(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &db_path,
        Some(SERVICE_INSTANCE_ID),
        Some(fixed_http_port),
        None,
    )
    .await;
    let client = reqwest::Client::new();
    let token = mint_harness_peer_token();
    let capabilities = get(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        "/api/uar/capabilities",
        StatusCode::OK,
    )
    .await;
    assert!(
        capabilities["capabilities"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item == "local_scoped_observers_v1")),
        "C07 durable profile unavailable: {capabilities}"
    );
    post(&client, &first.base_url, &token, WORKSPACE, "/api/v1/collaboration/packages:install",
        json!({"commandId":"c07-package","manifest":MANIFEST,"files":{
            "agent-definition.json":AGENT,"team-definition.json":TEAM,"workflow-definition.json":WORKFLOW}}), StatusCode::CREATED).await;
    post(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        "/api/v1/collaboration/representation-grants",
        json!({"commandId":"c07-grant","expectedRevision":0,
            "grant":serde_json::from_str::<Value>(GRANT).expect("grant fixture")}),
        StatusCode::CREATED,
    )
    .await;
    let installed = post(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        "/api/v1/collaboration/deployment-bindings",
        json!({"commandId":"c07-binding","expectedRevision":0,"binding":binding(&backend.model)}),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(
        installed["preflight"]["activationSupported"], true,
        "C07 binding: {installed}"
    );

    let source_a = create_instance(&client, &first.base_url, &token, 32).await;
    let source_b = create_instance(&client, &first.base_url, &token, 32).await;
    let source_c = create_instance(&client, &first.base_url, &token, 1).await;
    let observer_a = create_instance(&client, &first.base_url, &token, 32).await;
    let observer_b = create_instance(&client, &first.base_url, &token, 32).await;
    let a = source_a["instanceId"].as_str().expect("source A ID");
    let b = source_b["instanceId"].as_str().expect("source B ID");
    let c = source_c["instanceId"].as_str().expect("excluded source ID");
    let oa = observer_a["instanceId"].as_str().expect("observer A ID");
    let ob = observer_b["instanceId"].as_str().expect("observer B ID");
    let conversation_a = source_a["sessionId"]
        .as_str()
        .expect("source A conversation ID");
    let narrow = post(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        "/api/uar/observers/v1",
        json!({"observerInstanceId":oa,"sourceInstanceIds":[a,b],"conversationIds":[conversation_a],
            "limits":{"max_inbox":64,"max_retries":3,"retained_acknowledged":64}}),
        StatusCode::CREATED,
    )
    .await;
    let broad = post(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        "/api/uar/observers/v1",
        json!({"observerInstanceId":ob,"sourceInstanceIds":[a,b],
            "limits":{"max_inbox":64,"max_retries":3,"retained_acknowledged":64}}),
        StatusCode::CREATED,
    )
    .await;
    let narrow_id = narrow["subscription"]["subscription_id"]
        .as_str()
        .expect("narrow subscription ID");
    let broad_id = broad["subscription"]["subscription_id"]
        .as_str()
        .expect("broad subscription ID");
    assert_ne!(narrow_id, broad_id);
    get(
        &client,
        &first.base_url,
        &token,
        OTHER_WORKSPACE,
        &format!("/api/uar/observers/v1/{narrow_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;

    turn(&client, &first.base_url, &token, a, "C07_SOURCE_A").await;
    turn(&client, &first.base_url, &token, b, "C07_SOURCE_B").await;
    turn(&client, &first.base_url, &token, c, "C07_SOURCE_C").await;
    let narrow_a = wait_for_inbox_command(
        &client,
        &first.base_url,
        &token,
        narrow_id,
        "C07_SOURCE_A",
        "acknowledged",
    )
    .await;
    let broad_a = wait_for_inbox_command(
        &client,
        &first.base_url,
        &token,
        broad_id,
        "C07_SOURCE_A",
        "acknowledged",
    )
    .await;
    let broad_b = wait_for_inbox_command(
        &client,
        &first.base_url,
        &token,
        broad_id,
        "C07_SOURCE_B",
        "acknowledged",
    )
    .await;
    assert_ne!(
        narrow_a["observer_command_id"],
        broad_a["observer_command_id"]
    );
    wait_for_command(
        &client,
        &first.base_url,
        &token,
        oa,
        narrow_a["observer_command_id"]
            .as_str()
            .expect("narrow command ID"),
    )
    .await;
    wait_for_command(
        &client,
        &first.base_url,
        &token,
        ob,
        broad_a["observer_command_id"]
            .as_str()
            .expect("broad A command ID"),
    )
    .await;
    wait_for_command(
        &client,
        &first.base_url,
        &token,
        ob,
        broad_b["observer_command_id"]
            .as_str()
            .expect("broad B command ID"),
    )
    .await;
    let narrow_view = observer(&client, &first.base_url, &token, narrow_id).await;
    let broad_view = observer(&client, &first.base_url, &token, broad_id).await;
    for view in [&narrow_view, &broad_view] {
        assert!(
            !inbox_has_command(view, "C07_SOURCE_C"),
            "excluded source leaked: {view}"
        );
        assert!(
            !view.to_string().contains("C07_SOURCE_A_DONE"),
            "source outcome leaked into metadata projection"
        );
    }
    assert!(
        !inbox_has_command(&narrow_view, "C07_SOURCE_B"),
        "excluded conversation leaked: {narrow_view}"
    );

    let paused = post(
        &client,
        &first.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/observers/v1/{narrow_id}/pause"),
        json!({"expectedRevision":narrow_view["subscription"]["revision"]}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(paused["subscription"]["paused"], true);
    turn(&client, &first.base_url, &token, a, "C07_PAUSED_SOURCE").await;
    wait_for_inbox_command(
        &client,
        &first.base_url,
        &token,
        broad_id,
        "C07_PAUSED_SOURCE",
        "acknowledged",
    )
    .await;
    let paused_view = observer(&client, &first.base_url, &token, narrow_id).await;
    assert!(
        !inbox_has_command(&paused_view, "C07_PAUSED_SOURCE"),
        "paused subscription admitted work"
    );
    assert!(
        paused_view["backlogDepth"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "paused backlog hidden: {paused_view}"
    );

    first.shutdown().await;
    let restarted = boot_test_server_process_with_instance_id(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &db_path,
        Some(SERVICE_INSTANCE_ID),
        Some(fixed_http_port),
        None,
    )
    .await;
    let restored = observer(&client, &restarted.base_url, &token, narrow_id).await;
    assert_eq!(restored["subscription"]["paused"], true);
    assert!(!inbox_has_command(&restored, "C07_PAUSED_SOURCE"));
    let resumed = post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/observers/v1/{narrow_id}/resume"),
        json!({"expectedRevision":restored["subscription"]["revision"]}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(resumed["subscription"]["paused"], false);
    let caught_up = wait_for_inbox_command(
        &client,
        &restarted.base_url,
        &token,
        narrow_id,
        "C07_PAUSED_SOURCE",
        "acknowledged",
    )
    .await;
    wait_for_command(
        &client,
        &restarted.base_url,
        &token,
        oa,
        caught_up["observer_command_id"]
            .as_str()
            .expect("catch-up command ID"),
    )
    .await;

    // Revoke an admitted, not yet dispatched observer occurrence. The scheduler
    // admits and delivers on separate cycles; the durable inbox exposes this seam.
    post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{a}/turns"),
        json!({"commandId":"C07_QUEUED_SOURCE","prompt":"C07_QUEUED_SOURCE"}),
        StatusCode::ACCEPTED,
    )
    .await;
    let queued = wait_for_inbox_command(
        &client,
        &restarted.base_url,
        &token,
        narrow_id,
        "C07_QUEUED_SOURCE",
        "admitted",
    )
    .await;
    let before_revoke = observer(&client, &restarted.base_url, &token, narrow_id).await;
    let revoked = request(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        Method::DELETE,
        &format!(
            "/api/uar/observers/v1/{narrow_id}?expectedRevision={}",
            before_revoke["subscription"]["revision"]
        ),
        None,
        StatusCode::OK,
    )
    .await;
    assert_eq!(revoked["subscription"]["revoked"], true);
    tokio::time::sleep(Duration::from_secs(3)).await;
    let after_revoke = observer(&client, &restarted.base_url, &token, narrow_id).await;
    assert_eq!(after_revoke["subscription"]["revoked"], true);
    assert!(
        after_revoke["subscription"]["inbox"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| {
                entry["observer_command_id"] == queued["observer_command_id"]
                    && entry["status"] == "admitted"
            })),
        "queued work was dispatched after revocation: {after_revoke}"
    );
    let observer_instance = get(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{oa}"),
        StatusCode::OK,
    )
    .await;
    assert!(
        !observer_instance["commands"]
            .as_array()
            .is_some_and(|commands| commands
                .iter()
                .any(|command| { command["commandId"] == queued["observer_command_id"] })),
        "revoked observer command entered C06 execution: {observer_instance}"
    );
    wait_for_command(&client, &restarted.base_url, &token, a, "C07_QUEUED_SOURCE").await;

    // The source's public C06 event ring retains one event, while the C07
    // occurrence outbox must preserve every committed lifecycle transition.
    let before_restart = get(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{c}"),
        StatusCode::OK,
    )
    .await;
    post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{c}/restart"),
        json!({"commandId":"c07-gap-restart"}),
        StatusCode::OK,
    )
    .await;
    wait_for_command(&client, &restarted.base_url, &token, c, "c07-gap-restart").await;
    let after_restart = get(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{c}"),
        StatusCode::OK,
    )
    .await;
    assert!(
        after_restart["nextEventSequence"]
            .as_u64()
            .is_some_and(|after| {
                before_restart["nextEventSequence"]
                    .as_u64()
                    .is_some_and(|before| after > before + 1)
            }),
        "lifecycle did not commit multiple durable occurrences: {before_restart} {after_restart}"
    );
    let late = post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        "/api/uar/observers/v1",
        json!({"observerInstanceId":oa,"sourceInstanceIds":[c]}),
        StatusCode::CREATED,
    )
    .await;
    let late_id = late["subscription"]["subscription_id"]
        .as_str()
        .expect("late observer ID");
    wait_for_inbox_command(
        &client,
        &restarted.base_url,
        &token,
        late_id,
        "C07_SOURCE_C",
        "acknowledged",
    )
    .await;
    let restart_entry = wait_for_inbox_command(
        &client,
        &restarted.base_url,
        &token,
        late_id,
        "c07-gap-restart",
        "acknowledged",
    )
    .await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    let late_view = loop {
        let view = observer(&client, &restarted.base_url, &token, late_id).await;
        let restart_count = view["subscription"]["inbox"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter(|entry| entry["occurrence"]["command_id"] == "c07-gap-restart")
                    .count()
            })
            .unwrap_or(0);
        if restart_count >= 2 || tokio::time::Instant::now() >= deadline {
            break view;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert!(
        late_view["subscription"]["inbox"]
            .as_array()
            .is_some_and(|entries| {
                entries
                    .iter()
                    .filter(|entry| entry["occurrence"]["command_id"] == "c07-gap-restart")
                    .count()
                    >= 2
            }),
        "late observer missed committed lifecycle occurrences: {late_view}"
    );
    assert!(restart_entry["source_sequence"].as_u64().is_some());
    assert_eq!(after_restart["events"].as_array().map(Vec::len), Some(1));

    // Cross the real outbox retention boundary while this subscription is
    // paused. Each idle cancel completes through C06 and commits one source
    // occurrence; no synthetic database rows are inserted by the gate.
    let drained = wait_for_drained_observer(&client, &restarted.base_url, &token, late_id).await;
    let last_cursor = drained["subscription"]["cursors"][c]
        .as_u64()
        .expect("late observer source cursor");
    post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &format!("/api/uar/observers/v1/{late_id}/pause"),
        json!({"expectedRevision":drained["subscription"]["revision"]}),
        StatusCode::OK,
    )
    .await;
    for sequence in 0..1_026 {
        post(
            &client,
            &restarted.base_url,
            &token,
            WORKSPACE,
            &format!("/api/uar/agent-instances/v1/{c}/cancel"),
            json!({"commandId":format!("c07-retention-{sequence}")}),
            StatusCode::OK,
        )
        .await;
    }
    let gapped = observer(&client, &restarted.base_url, &token, late_id).await;
    let gap = gapped["subscription"]["gaps"]
        .as_array()
        .and_then(|gaps| {
            gaps.iter()
                .find(|gap| gap["source_instance_id"] == c && gap["acknowledged_at"].is_null())
        })
        .expect("retention prune must persist an exact unacknowledged gap");
    let source = gapped["sources"]
        .as_array()
        .and_then(|sources| {
            sources
                .iter()
                .find(|source| source["sourceInstanceId"] == c)
        })
        .expect("late observer source status");
    assert_eq!(gap["missing_from"].as_u64(), Some(last_cursor + 1));
    assert_eq!(
        gap["missing_through"].as_u64(),
        source["retainedLow"].as_u64().map(|low| low - 1)
    );
    assert!(
        gapped["recoveryActions"].as_array().is_some_and(|actions| {
            actions
                .iter()
                .any(|action| action == "acknowledge_retention_gap_or_resnapshot")
        }),
        "retention gap omitted operator recovery action: {gapped}"
    );
    let acknowledge_path = format!("/api/uar/observers/v1/{late_id}/gaps/acknowledge");
    let acknowledgment = json!({"expectedRevision":gapped["subscription"]["revision"],
        "sourceInstanceId":c,"missingFrom":gap["missing_from"],
        "missingThrough":gap["missing_through"]});
    post(
        &client,
        &restarted.base_url,
        &token,
        WORKSPACE,
        &acknowledge_path,
        acknowledgment.clone(),
        StatusCode::FORBIDDEN,
    )
    .await;
    let acknowledged = post(
        &client,
        &restarted.base_url,
        &operator_token(),
        WORKSPACE,
        &acknowledge_path,
        acknowledgment,
        StatusCode::OK,
    )
    .await;
    assert!(
        acknowledged["subscription"]["gaps"]
            .as_array()
            .is_some_and(|gaps| {
                gaps.iter()
                    .any(|gap| gap["source_instance_id"] == c && !gap["acknowledged_at"].is_null())
            }),
        "operator acknowledgment did not settle the gap: {acknowledged}"
    );

    restarted.shutdown().await;
    match original_skills {
        Some(value) => unsafe { std::env::set_var("UAR_BUILTIN_SKILLS_DIR", value) },
        None => unsafe { std::env::remove_var("UAR_BUILTIN_SKILLS_DIR") },
    }
}
