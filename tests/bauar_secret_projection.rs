//! Authored real-router/provider projection scenarios; execution is deferred.
#[path = "support/bauar_secret_provider.rs"]
mod provider;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;

#[path = "support/bauar_secret_harness.rs"]
mod harness;
use harness::{
    assert_ingress_projected, assert_projected, copied_strings, follow, indexed_kb,
    stream_channels, terminal_diagnostic,
};

use provider::{MCP_SECRET, PROVIDER_SECRET, Provider, ROUTING_SECRET, variants};
use serde_json::{Value, json};
use std::time::Duration;
use universal_agent_runtime::uar::persistence::agent_threads::{
    CanonicalRawSegment, CanonicalReceiptSource, CanonicalSecretProjection, CanonicalToolReceipt,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_provider_and_mcp_copies_hide_variants_without_changing_execution() {
    let token = sidecar_process::random_token();
    let provider = Provider::start(&token).await;
    let workspace = sidecar_process::Workspace::new();
    std::fs::write(
        workspace.work().join("AGENTS.md"),
        format!(
            "World-state-positive-control {}",
            [
                variants(PROVIDER_SECRET),
                variants(MCP_SECRET),
                variants(&token)
            ]
            .concat()
            .join(" | ")
        ),
    )
    .unwrap();
    let mut options = sidecar_process::ConfigOptions::new(&format!("{}/v1", provider.url));
    options.extra_yaml = format!(
        "project_instructions:\n  trusted_workspaces: [{}]\n",
        json!(workspace.work().display().to_string())
    );
    let yaml = sidecar_process::render_config(&workspace, &options).replace("llm:\n", &format!(
        "llm:\n  api_key: {}\n  embedding:\n    backend: openai\n    model: projection-embedding\n    api_key: {}\n    base_url: {}\n    vector_dimension: 3\n",
        json!(PROVIDER_SECRET), json!(ROUTING_SECRET), json!(format!("{}/v1", provider.url))));
    let config = workspace.write_config("config.yaml", &yaml);
    let process = sidecar_process::launch_sidecar(
        &workspace,
        &sidecar_process::LaunchOptions::new(config, "secret-projection").with_token(&token),
    )
    .await;
    let client = reqwest::Client::new();
    let base = process.base_url();
    let kb = indexed_kb(&client, &base, &token).await;
    let mut artifact = sidecar_process::test_agent(Some(json!({"tool_approval":"ask"})));
    artifact["id"] = json!("projection-agent");
    artifact["memory"]["kb"] = json!({"enabled":true,"knowledge_bases":[kb]});
    let registered = client
        .post(format!("{base}/api/agents"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .json(&artifact)
        .send()
        .await
        .unwrap();
    assert_eq!(registered.status(), reqwest::StatusCode::CREATED);
    let resources = json!({
        "mcp_servers":[{"name":"projection","url":format!("{}/mcp",provider.url),
            "headers":{"Authorization":format!("Bearer {MCP_SECRET}")}}]
    });
    let mut request = resources.clone();
    request["agent_id"] = json!("projection-agent");
    request["session_id"] = json!("projection-retained-session");
    // Generation key is emitted later by the provider, after ClientConfig
    // capture. The earlier routing input tests only already captured secrets.
    request["input"] = json!(format!(
        "early-positive-control {}",
        [
            variants(&token),
            variants(&format!("Bearer {token}")),
            variants(MCP_SECRET)
        ]
        .concat()
        .join(" | ")
    ));
    provider
        .seen
        .hold_routing
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let creation = tokio::spawn(
        client
            .post(format!("{base}/api/uar/runs"))
            .bearer_auth(&token)
            .header("X-UAR-Principal", "projection-owner")
            .json(&request)
            .send(),
    );
    tokio::time::timeout(
        Duration::from_secs(30),
        provider.seen.routing_started.notified(),
    )
    .await
    .unwrap();
    let shells: Value = client
        .get(format!("{base}/api/uar/runs"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let shell = shells
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["agent_id"] == "projection-agent")
        .unwrap();
    assert!(
        shell["agent_revision"].is_null(),
        "executable snapshot was retained before complete capture"
    );
    assert_projected(&shell.to_string());
    assert_ingress_projected(&shell.to_string(), &token);
    assert!(
        provider.seen.requests.lock().unwrap().is_empty(),
        "generation preceded routing admission"
    );
    let embeddings = provider.seen.embeddings.lock().unwrap().clone();
    assert!(
        embeddings
            .iter()
            .any(|body| body["input"].to_string().contains("[REDACTED]")),
        "missing captured-secret routing positive control"
    );
    for body in embeddings
        .iter()
        .filter(|body| body["input"].to_string().contains("early-positive-control"))
    {
        let mut text = String::new();
        copied_strings(&body["input"], &mut text);
        assert_ingress_projected(&text, &token);
        for variant in variants(MCP_SECRET) {
            assert!(!text.contains(&variant));
        }
    }
    provider
        .seen
        .hold_routing
        .store(false, std::sync::atomic::Ordering::SeqCst);
    provider.seen.routing_release.notify_one();
    let response = creation.await.unwrap().unwrap();
    assert!(response.status().is_success());
    let admitted: Value = response.json().await.unwrap();
    let run = admitted["run_id"].as_str().unwrap();
    let stream = follow(&client, &base, &token, run).await;
    assert!(
        stream.contains("event: agui.done"),
        "legitimate run did not complete: {stream}"
    );
    assert_projected(&stream);
    assert_ingress_projected(&stream, &token);
    let channels = stream_channels(&stream);
    assert!(
        channels
            .values()
            .any(|text| text.contains("positive-control")),
        "missing streamed positive control"
    );
    for text in channels.values() {
        assert_projected(text);
    }
    let effects = provider.seen.effects.lock().unwrap().clone();
    assert_eq!(
        effects.len(),
        1,
        "expected one genuine receiver effect, no replay"
    );
    assert_eq!(
        effects[0]["echo"], MCP_SECRET,
        "projection mutated live executable arguments"
    );
    assert_eq!(effects[0]["control"], "legitimate-execution-control");
    let requests = provider.seen.requests.lock().unwrap().clone();
    assert!(requests.len() >= 2, "missing post-tool model request");
    for request in &requests {
        let mut content = String::new();
        copied_strings(&request["messages"], &mut content);
        assert_projected(&content);
        assert_ingress_projected(&content, &token);
    }
    let retained: Value = client
        .get(format!("{base}/api/uar/runs/{run}"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut retained_strings = String::new();
    copied_strings(&retained, &mut retained_strings);
    assert_projected(&retained_strings);
    assert_ingress_projected(&retained_strings, &token);
    assert_projected(&process.stderr());
    assert_ingress_projected(&process.stderr(), &token);

    // Reuse the original session through the actual router. Its world-state
    // message and prior projected user turn must survive private admission.
    request["input"] = json!("second-turn-retention-control");
    let response = client
        .post(format!("{base}/api/uar/runs"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .json(&request)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let response: Value = response.json().await.unwrap();
    let second = follow(&client, &base, &token, response["run_id"].as_str().unwrap()).await;
    assert!(
        second.contains("event: agui.done"),
        "same-owner second turn did not complete: {}",
        terminal_diagnostic(&second)
    );
    assert_projected(&second);
    assert_ingress_projected(&second, &token);
    let requests = provider.seen.requests.lock().unwrap().clone();
    let mut reused = String::new();
    copied_strings(&requests.last().unwrap()["messages"], &mut reused);
    assert!(
        reused.contains("early-positive-control"),
        "original append authority lost the first user turn"
    );
    assert!(
        reused.contains("World-state-positive-control"),
        "world-state retention was lost across admission"
    );
    assert!(reused.contains("second-turn-retention-control"));
    assert_projected(&reused);
    assert_ingress_projected(&reused, &token);
    assert_eq!(
        provider.seen.effects.lock().unwrap().len(),
        1,
        "reused history replayed an executed effect"
    );

    // Execution authority is rejected, never rewritten under its prior revision.
    artifact["id"] = json!("projection-secret-artifact");
    artifact["prompt"]["system"] =
        json!(format!("Unsupported executable secret: {PROVIDER_SECRET}"));
    let response = client
        .post(format!("{base}/api/agents"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .json(&artifact)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    request["agent_id"] = json!("projection-secret-artifact");
    request["input"] = json!("must reject before model");
    let before = provider.seen.requests.lock().unwrap().len();
    let response: Value = client
        .post(format!("{base}/api/uar/runs"))
        .bearer_auth(&token)
        .header("X-UAR-Principal", "projection-owner")
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rejected = follow(&client, &base, &token, response["run_id"].as_str().unwrap()).await;
    assert!(rejected.contains("SECRET_IN_EXECUTABLE_ARTIFACT"));
    assert_projected(&rejected);
    assert_eq!(provider.seen.requests.lock().unwrap().len(), before);
    assert_eq!(provider.seen.effects.lock().unwrap().len(), 1);
}

#[test]
fn projected_receipt_metadata_is_explicit_and_hashes_retained_bytes() {
    let mut receipt = CanonicalToolReceipt::acquire(
        "owner",
        "run",
        1,
        "call",
        "tool",
        CanonicalReceiptSource::Terminal,
        json!({"text":"[REDACTED]"}),
        [("stdout".to_owned(), b"[REDACTED]".to_vec())],
        true,
        0,
    )
    .unwrap();
    receipt.secret_projection = Some(CanonicalSecretProjection {
        version: 1,
        redacted: true,
        omitted_raw_segments: 1,
    });
    receipt.validate().unwrap();
    assert_eq!(
        receipt.raw_segments[0].verified_bytes().unwrap().unwrap(),
        b"[REDACTED]"
    );
    assert_eq!(
        receipt.raw_segments[0],
        CanonicalRawSegment::new("stdout", b"[REDACTED]")
    );
    let mut serialized = serde_json::to_value(&receipt).unwrap();
    serialized
        .as_object_mut()
        .unwrap()
        .remove("secret_projection");
    let legacy: CanonicalToolReceipt = serde_json::from_value(serialized).unwrap();
    assert!(
        legacy.secret_projection.is_none(),
        "legacy was falsely labeled projected"
    );
    receipt.secret_projection.as_mut().unwrap().version = 2;
    assert!(
        receipt.validate().is_err(),
        "unknown projection version accepted"
    );
}
