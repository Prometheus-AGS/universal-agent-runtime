//! COUNT-01 evidence for the existing trusted Rust embedding/builder surface.
//! This is deliberately NOT proof of standalone-main exact-fit support.
//! Authored for the separately approved boundary packet; no self-running test.

use std::{fs::OpenOptions, io::Write, path::Path, sync::{Arc, Mutex}, time::Duration};
use anyhow::{Context, Result, anyhow, ensure};
use axum::{Router, extract::State, response::IntoResponse, routing::post};
use futures::StreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;
use universal_agent_runtime::{
    config::LlmConfig,
    llm::{DestinationRequestPreparation, DestinationRequestPreparations,
        EndpointRequestProfile, EndpointRequestTransform, LiterLlmDriver, LlmDriver,
        LlmRequest, Message, MessageContent, MessageRole, Orchestrator},
    mcp::registry::McpRegistry,
    normalized::NormalizedEvent,
    uar::runtime::{
        context::{budget::{DestinationLimits, OutputCeilingField, RequestBudgetContract,
            SYNTHETIC_EXACT_MODEL, WireContract, endpoint_fingerprint},
            token_service::{SerializedCountingContract, TokenService}},
        native_skill::NativeSkillRegistry,
        prompt::{Authority, PromptFragment, PromptRole, PromptSection, PromptTemplateLayout,
            PromptTemplateProfile, PromptTemplateSelector, Retention, render_with_template},
    },
};

type Captures = Arc<Mutex<Vec<Vec<u8>>>>;

async fn reply(State(captures): State<Captures>, body: String) -> impl IntoResponse {
    // Authorization headers are intentionally not extracted or recorded.
    if let Ok(mut captures) = captures.lock() {
        captures.push(body.into_bytes());
    } else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "text/event-stream")], String::new());
    }
    let chunk = |delta: Value, finish: Value| json!({
        "id":"count-builder-peer", "object":"chat.completion.chunk", "created":1,
        "model":SYNTHETIC_EXACT_MODEL,
        "choices":[{"index":0,"delta":delta,"finish_reason":finish}]
    });
    let frames = format!("data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        chunk(json!({"role":"assistant","content":"count-builder-positive"}), Value::Null),
        chunk(json!({}), json!("stop")));
    (axum::http::StatusCode::OK, [("content-type", "text/event-stream")], frames)
}

fn received(captures: &Captures) -> Result<Vec<Vec<u8>>> {
    captures.lock().map(|rows| rows.clone()).map_err(|_| anyhow!("peer_capture_poisoned"))
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn endpoint(base: &str) -> EndpointRequestProfile {
    EndpointRequestProfile {
        profile_id: "post-lifecycle-controlled-count".into(),
        profile_revision: "controlled-count-v1".into(),
        provider_id: "openai".into(), endpoint_kind: "fixture-chat".into(),
        endpoint_fingerprint: endpoint_fingerprint(base),
        qualified_model: SYNTHETIC_EXACT_MODEL.into(), wire_model: SYNTHETIC_EXACT_MODEL.into(),
        model_revision: "controlled-model-v1".into(), settings_revision: "controlled-settings-v1".into(),
        transform: EndpointRequestTransform::OpenAiCompatibleChatV1,
        allowed_request_fields: ["model", "messages", "tools", "tool_choice", "parallel_tool_calls",
            "stream_options", "max_completion_tokens", "stream"].into_iter().map(str::to_owned).collect(),
        output_ceiling: Some(OutputCeilingField::MaxCompletionTokens),
    }
}

fn contract(base: &str) -> RequestBudgetContract {
    RequestBudgetContract {
        destination_model: SYNTHETIC_EXACT_MODEL.into(),
        destination_endpoint_fingerprint: endpoint_fingerprint(base),
        limits: DestinationLimits {context_tokens:8192, independent_input_tokens:Some(7680),
            host_input_tokens:None, output_tokens:512, additional_reasoning_tokens:19,
            count_uncertainty_tokens:7},
        counting: SerializedCountingContract::ExactCl100kJsonV1,
        wire: WireContract::SyntheticOpenAiCompatibleChatV1,
        output_ceiling: OutputCeilingField::MaxCompletionTokens,
    }
}

fn template() -> PromptTemplateProfile {
    PromptTemplateProfile {
        id: "post-lifecycle-count-template".into(), revision: "controlled-template-v1".into(),
        selector: PromptTemplateSelector::Exact {
            provider_id:"openai".into(), endpoint_kind:"fixture-chat".into(),
            model_id:"uar-exact-cl100k-json-v1".into(), model_revision:"controlled-model-v1".into()},
        layout:PromptTemplateLayout::Plain,
        required_slots:vec![PromptSection::HostInstructions],
        supported_roles:vec![PromptRole::System], wire_contract_id:"controlled-wire-v1".into(),
    }
}

fn preparation(base: &str, budget: RequestBudgetContract) -> DestinationRequestPreparation {
    DestinationRequestPreparation {
        destination: universal_agent_runtime::llm::prompt_dialect::TemplateDestination {
            provider_id:"openai".into(), endpoint_kind:"fixture-chat".into(),
            model_id:"uar-exact-cl100k-json-v1".into(), model_revision:"controlled-model-v1".into(),
            verified_family_revision:None, generic_contract_eligible:false},
        template:template(), request_profile:endpoint(base), budget_contract:budget,
        exact_settings:None, compatible_continuity:Vec::new(),
    }
}

fn request(messages: Vec<Value>, tools: Vec<Value>, budget: RequestBudgetContract) -> LlmRequest {
    LlmRequest {messages, tools, cache_strategy:None, thinking_config:None,
        anthropic_system:None, extra_params:None, budget_contract:Some(budget)}
}

fn bind_at_limit(request: &mut LlmRequest, count: u64) -> Result<()> {
    let count = i64::try_from(count).context("count exceeds limit representation")?;
    let budget = request.budget_contract.as_mut().context("missing fixture contract")?;
    budget.limits.context_tokens = count.checked_add(512 + 19 + 7).context("capacity overflow")?;
    budget.limits.independent_input_tokens = Some(count);
    budget.limits.host_input_tokens = Some(count);
    Ok(())
}

async fn consume(driver: &dyn LlmDriver, request: LlmRequest) -> Result<()> {
    let mut stream = driver.stream(request).await?;
    let mut done = false;
    let mut positive = false;
    while let Some(event) = stream.next().await {
        match event? {
            NormalizedEvent::Done => done = true,
            NormalizedEvent::MessageDelta {text} if text == "count-builder-positive" => positive = true,
            NormalizedEvent::Error {..} => return Err(anyhow!("peer_request_returned_normalized_error")),
            _ => {},
        }
    }
    ensure!(done && positive, "missing controlled positive response/terminal sentinel");
    Ok(())
}

async fn cases(base: &str, captures: &Captures) -> Result<Value> {
    // Empty credential: the loopback peer requires none. Never synthesize a
    // key or read/copy an existing provider credential to make this case pass.
    let config = liter_llm::ClientConfigBuilder::new("")
        .base_url(base.to_owned()).max_retries(0).build();
    let driver = Arc::new(LiterLlmDriver::new(config, SYNTHETIC_EXACT_MODEL.into(), Some(false))?
        .with_endpoint_profile(endpoint(base))?);
    let arguments = "{\"path\":\"owned-fixture\",\"text\":\"λ\\r\\nkeep\"}";
    let result = "{\"owned\":true,\"bytes\":[0,13,10,255]}";
    let tools = vec![json!({"type":"function","function":{"name":"count_fixture_tool",
        "description":"Protected tool schema must consume request capacity",
        "parameters":{"type":"object","properties":{"path":{"type":"string"}}}}})];
    let mut at = request(vec![json!({"role":"user","content":"preserve the owned receipt"}),
        json!({"role":"assistant","content":"","tool_calls":[{"id":"count-call-1",
            "type":"function","function":{"name":"count_fixture_tool","arguments":arguments}}]}),
        json!({"role":"tool","tool_call_id":"count-call-1","content":result})], tools.clone(), contract(base));
    let calibration = driver.profiled_wire_receipt(&at)?;
    let count = calibration.counted_tokens.context("missing exact synthetic count")?;
    ensure!(count > 1, "invalid synthetic calibration");
    bind_at_limit(&mut at, count)?;
    let exact = driver.profiled_wire_receipt(&at)?;
    ensure!(exact.counted_tokens == Some(count), "limit binding changed final serialization");
    let mut without_tools = at.clone();
    without_tools.tools.clear();
    let lower_count = driver.profiled_wire_receipt(&without_tools)?.counted_tokens
        .context("missing tool-free comparison count")?;
    ensure!(lower_count < count, "tool definitions were not included in the count");
    tokio::time::timeout(Duration::from_secs(10), consume(driver.as_ref(), at.clone()))
        .await.context("at-limit request timed out")??;
    let first = received(captures)?;
    ensure!(first.len() == 1, "at-limit request did not dispatch exactly once");
    ensure!(sha(&first[0]) == exact.sha256, "recorded final body differs from validated serialization");
    let body: Value = serde_json::from_slice(&first[0])?;
    ensure!(body["tools"] == json!(tools), "tool definitions changed at transport");
    ensure!(body["messages"][1]["tool_calls"][0]["function"]["arguments"] == arguments,
        "protected tool arguments changed");
    ensure!(body["messages"][2]["content"] == result && body["messages"][2]["tool_call_id"] == "count-call-1",
        "protected tool result changed");
    ensure!(body["max_completion_tokens"] == 512 && body.get("max_tokens").is_none(),
        "output reserve not enforced by final wire field");
    let actual = TokenService::count_serialized(SerializedCountingContract::ExactCl100kJsonV1, &first[0])?;
    ensure!(actual.tokens == count, "captured wire count differs from prepared count");
    let mut over = at.clone();
    over.budget_contract.as_mut().context("missing over-limit contract")?.limits.host_input_tokens =
        Some(i64::try_from(count)? - 1);
    ensure!(driver.stream(over).await.is_err(), "one-over request was not refused");
    ensure!(received(captures)?.len() == 1, "one-over refusal caused a model effect");
    let mut approximate = at.clone();
    approximate.budget_contract.as_mut().context("missing approximate contract")?.counting =
        SerializedCountingContract::ApproximateCl100kJsonV1;
    ensure!(driver.stream(approximate).await.is_err(), "approximate count certified a dispatch");
    ensure!(received(captures)?.len() == 1, "unsupported count caused a model effect");
    let mut unsupported = contract(base);
    unsupported.counting = SerializedCountingContract::ApproximateCl100kJsonV1;
    ensure!(DestinationRequestPreparations::new(vec![preparation(base, unsupported)]).is_err(),
        "host registry accepted an unsupported exact-fit contract");

    // Exercise the existing trusted Rust builder through the same real peer.
    let fragments = vec![PromptFragment::new("count-host-policy", PromptSection::HostInstructions,
        "owned-count-fixture", Authority::Host, PromptRole::System, Retention::Session,
        "Keep this exact host policy in the destination request")];
    let rendered = render_with_template(&fragments, &template()).map_err(|error| anyhow!("{error:?}"))?;
    let history = vec![Message {role:MessageRole::User,
        content:MessageContent::text("trusted Rust builder sentinel"), tool_call_id:None, tool_calls:None}];
    let mut builder_request = request(vec![json!({"role":"system","content":rendered}),
        serde_json::to_value(&history[0])?], Vec::new(), contract(base));
    let builder_count = driver.profiled_wire_receipt(&builder_request)?.counted_tokens
        .context("missing builder calibration")?;
    bind_at_limit(&mut builder_request, builder_count)?;
    let builder_receipt = driver.profiled_wire_receipt(&builder_request)?;
    let budget = builder_request.budget_contract.clone().context("missing builder contract")?;
    let preparations = Arc::new(DestinationRequestPreparations::new(vec![preparation(base, budget)])?);
    let orchestrator = Orchestrator::from_driver(LlmConfig {model:SYNTHETIC_EXACT_MODEL.into(),
        resolved_provider_id:Some("openai".into()), ..LlmConfig::default()},
        Arc::new(McpRegistry::empty()), Arc::new(NativeSkillRegistry::new()), driver.clone())
        .with_destination_preparations(preparations, history.clone(), fragments)
        .with_resilience_policy(universal_agent_runtime::uar::settings::resilience_policy::ResiliencePolicy {
            retries_enabled:false, ..Default::default()});
    let events = tokio::time::timeout(Duration::from_secs(10), async {
        let stream = orchestrator.chat_with_history(history).await?;
        Ok::<_, anyhow::Error>(stream.collect::<Vec<_>>().await)
    }).await.context("builder request timed out")??;
    ensure!(!events.iter().any(|event| matches!(event, NormalizedEvent::Error {..})),
        "trusted builder returned an error");
    ensure!(events.iter().any(|event| matches!(event, NormalizedEvent::Done)),
        "builder missing actual terminal sentinel");
    ensure!(events.iter().any(|event| matches!(event, NormalizedEvent::MessageDelta {text}
        if text == "count-builder-positive")), "builder missing positive sentinel");
    let all = received(captures)?;
    ensure!(all.len() == 2, "trusted builder did not produce exactly one further model request");
    ensure!(sha(&all[1]) == builder_receipt.sha256, "builder changed calibrated final serialization");
    let builder_body: Value = serde_json::from_slice(&all[1])?;
    ensure!(builder_body["messages"] == json!(builder_request.messages), "builder lost protected host/history content");
    Ok(json!({"outcome":"PASS", "evidenceScope":"trusted_rust_builder",
        "standaloneMainExactFit":"UNSUPPORTED", "trustedRustBuilder":"PASS", "standaloneMain":"UNSUPPORTED",
        "standaloneReason":"No serialized/runtime trusted preparation seam; not added by this phase",
        "proofScope":"Existing Rust builder plus real controlled loopback HTTP peer; synthetic cl100k JSON agreement only",
        "modelRequests":all.len(), "atLimitCount":count, "atLimitWireSha256":exact.sha256,
        "toolFreeCount":lower_count, "reservedOutput":512, "reservedReasoning":19,
        "reservedUncertainty":7, "oneOverNoEffect":true, "approximateNoEffect":true,
        "unsupportedPreparationRejected":true, "protectedRecordsUnchanged":true,
        "builderAtLimitCount":builder_count, "builderWireSha256":builder_receipt.sha256,
        "bodyHashes":all.iter().map(|bytes| sha(bytes)).collect::<Vec<_>>()}))
}

/// Execute only as part of the operator-approved whole-phase boundary packet.
/// Persist a fresh labelled receipt even when a case or owned peer teardown fails.
pub async fn exercise_count_boundary(evidence_dir: &Path) -> Result<Value> {
    std::fs::create_dir_all(evidence_dir)?;
    let mut receipt = OpenOptions::new().write(true).create_new(true)
        .open(evidence_dir.join("count-trusted-builder.json"))?;
    let captures: Captures = Arc::new(Mutex::new(Vec::new()));
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => {
            let blocked = json!({"kind":"uar.count-evidence-split/v1", "outcome":"BLOCKED",
                "evidenceScope":"trusted_rust_builder", "standaloneMainExactFit":"UNSUPPORTED",
                "setup":"BLOCKED", "bodyStatus":"NOT_EXERCISED", "teardown":"NOT_STARTED",
                "reason":"owned_loopback_bind_failed"});
            receipt.write_all(serde_json::to_string_pretty(&blocked)?.as_bytes())?;
            return Err(error.into());
        }
    };
    let base = format!("http://{}/v1", listener.local_addr()?);
    let app = Router::new().route("/v1/chat/completions", post(reply)).with_state(captures.clone());
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let mut peer = tokio::spawn(async move {
        axum::serve(listener, app).with_graceful_shutdown(async {let _ = shutdown_rx.await;}).await
    });
    let body_result = cases(&base, &captures).await;
    let _ = shutdown_tx.send(());
    let teardown = match tokio::time::timeout(Duration::from_secs(5), &mut peer).await {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(_) => Err(anyhow!("owned_count_peer_exit_failed")),
        Err(_) => {
            peer.abort();
            let _ = peer.await;
            Err(anyhow!("owned_count_peer_graceful_exit_timed_out"))
        }
    };
    let outcome = match (&body_result, &teardown) {
        (Ok(value), Ok(())) => json!({"kind":"uar.count-evidence-split/v1", "body":value,
            "outcome":"PASS", "evidenceScope":"trusted_rust_builder", "standaloneMainExactFit":"UNSUPPORTED",
            "setup":"PASS", "bodyStatus":"PASS", "teardown":"PASS"}),
        (body, cleanup) => json!({"kind":"uar.count-evidence-split/v1",
            "outcome":"FAIL_OR_BLOCKED", "evidenceScope":"trusted_rust_builder",
            "standaloneMainExactFit":"UNSUPPORTED",
            "standaloneMain":"UNSUPPORTED", "trustedRustBuilder":"FAIL_OR_BLOCKED",
            "setup":"PASS", "bodyStatus":if body.is_ok(){"PASS"}else{"FAIL_OR_BLOCKED"},
            "bodyError":body.as_ref().err().map(|error| error.to_string()),
            "teardown":if cleanup.is_ok(){"PASS"}else{"BLOCKED"},
            "teardownError":cleanup.as_ref().err().map(|error| error.to_string()),
            "modelRequests":received(&captures).map(|rows|rows.len()).ok()}),
    };
    receipt.write_all(serde_json::to_string_pretty(&outcome)?.as_bytes())?;
    body_result?;
    teardown?;
    Ok(outcome)
}
