//! Baseline feature cases for the live integration tier
//! (live-integration-baseline-coverage, task group 2).
//!
//! Each case boots a real server (`harness::boot_test_server`) pointed at a
//! stub LLM (`stub_llm::start_stub_llm`) and makes a real HTTP request,
//! proving the feature works end-to-end through the actual production code
//! path — not a unit-level approximation.

use super::harness::{HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server, mint_harness_peer_token};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};
use serial_test::serial;

/// Model name as it appears on the wire to the stub — bare, no `provider/`
/// prefix; see harness.rs's discovery note on why.
const MODEL: &str = "gpt-5.4-mini";

fn content_fixture(last_user_message: &str, response: &str) -> FixtureSet {
    FixtureSet::new().with(
        RequestFingerprint {
            model: MODEL.to_string(),
            last_user_message: last_user_message.to_string(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content(response.to_string()),
    )
}

/// 2.1 — Streaming chat case for `stream_mode: openai`.
#[tokio::test]
#[serial]
async fn streaming_chat_openai_mode() {
    let stub = start_stub_llm(content_fixture("hello openai mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello openai mode"}],
            "stream": true,
            "stream_mode": "openai",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("chat.completion.chunk"),
        "expected an OpenAI-shaped chunk, got: {body}"
    );
    assert!(
        body.contains("hi there"),
        "expected the fixture content to appear, got: {body}"
    );
    // openai mode must not emit agui.* named events.
    assert!(
        !body.contains("agui."),
        "openai mode should not emit agui events, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.2 — Streaming chat case for `stream_mode: agui`.
#[tokio::test]
#[serial]
async fn streaming_chat_agui_mode() {
    let stub = start_stub_llm(content_fixture("hello agui mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello agui mode"}],
            "stream": true,
            "stream_mode": "agui",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("agui.message.delta") || body.contains("agui.stream.start"),
        "expected an agui.* named event, got: {body}"
    );
    // agui mode must not emit raw OpenAI chunks.
    assert!(
        !body.contains("chat.completion.chunk"),
        "agui mode should not emit OpenAI chunks, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.3 — Streaming chat case for `stream_mode: dual`.
#[tokio::test]
#[serial]
async fn streaming_chat_dual_mode() {
    let stub = start_stub_llm(content_fixture("hello dual mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello dual mode"}],
            "stream": true,
            "stream_mode": "dual",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("chat.completion.chunk"),
        "dual mode should emit OpenAI chunks too, got: {body}"
    );
    assert!(
        body.contains("agui.message.delta") || body.contains("agui.stream.start"),
        "dual mode should emit agui.* events too, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.4 — MCP/native tool-loop round-trip case.
///
/// Uses `native_echo` (`src/uar/runtime/native_skills/echo.rs`) — registered
/// unconditionally by `register_builtins`, so no MCP server or external
/// dependency is needed. Two fixtures: the first request (no tool result in
/// the conversation yet) gets a tool-call fixture; the second request (tool
/// result appended, same user message) gets the final content — see
/// `RequestFingerprint::has_tool_result`'s doc comment for why two fixtures
/// are needed for what looks like "the same" request.
#[tokio::test]
#[serial]
async fn tool_loop_round_trip() {
    let fixtures = FixtureSet::new()
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "echo this back".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::ToolCall {
                name: "native_echo".to_string(),
                arguments: r#"{"message":"echo this back"}"#.to_string(),
            },
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "echo this back".to_string(),
                has_tools: true,
                has_tool_result: true,
            },
            FixtureResponse::Content("the tool echoed it back".to_string()),
        );
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "echo this back"}],
            "stream": false,
        }))
        .send()
        .await
        .expect("request");

    assert!(
        resp.status().is_success(),
        "status: {} body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
    let body: serde_json::Value = resp.json().await.expect("json body");
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(
        content, "the tool echoed it back",
        "expected the tool loop to complete and return the post-tool-call content, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.5 — Agent selection via the `agent_id` request field.
///
/// Scope note (see specs/live-integration-testing/spec.md's "Known gap"):
/// this proves authenticated UAR-peer `agent_id` resolution is fallback-safe
/// for both built-in agents (`resolve_agent_for_run`,
/// `src/uar/api/discovery.rs:259`) — it
/// does NOT prove agent identity changes observable LLM-call behavior.
/// `/api/chat/completion` does not read `agent.prompt.system` (that's only
/// consumed by `RunManager::start_run`, a different code path), and the two
/// built-in agents are behaviorally identical except for id/metadata
/// (`src/uar/defaults.rs:76-82`). The original, stronger claim ("selecting
/// an agent changes behavior") was corrected after checking against code —
/// not assumed.
#[tokio::test]
#[serial]
async fn agent_selection_resolves_both_builtin_agents() {
    const ROUTER_REQUEST: &str = concat!(
        "Route Rust implementation, correctness, or safety questions to rust-reviewer. ",
        "Route all other questions to general-purpose.\n\n",
        "User request:\nhello\n\n",
        "You MUST respond with exactly one of the following options (no extra text): ",
        "general-purpose, rust-reviewer"
    );
    let fixtures = content_fixture("hello", "hi there")
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: ROUTER_REQUEST.to_string(),
                has_tools: false,
                has_tool_result: false,
            },
            FixtureResponse::Content("general-purpose".to_string()),
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "hello".to_string(),
                has_tools: false,
                has_tool_result: false,
            },
            FixtureResponse::Content("hi there".to_string()),
        );
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let peer_token = mint_harness_peer_token();
    let client = reqwest::Client::new();

    for agent_id in ["default-agent", "orchestrator-agent"] {
        let resp = client
            .post(format!("{}/api/chat/completion", server.base_url))
            .bearer_auth(&peer_token)
            .json(&serde_json::json!({
                "model": MODEL,
                "messages": [{"role": "user", "content": "hello"}],
                "agent_id": agent_id,
                "stream": false,
            }))
            .send()
            .await
            .expect("request");
        assert!(
            resp.status().is_success(),
            "agent_id {agent_id:?}: status {} body {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        );
    }

    // An unknown agent_id must fall back to default-agent rather than error.
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .bearer_auth(&peer_token)
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello"}],
            "agent_id": "no-such-agent",
            "stream": false,
        }))
        .send()
        .await
        .expect("request");
    assert!(
        resp.status().is_success(),
        "unknown agent_id should fall back to default-agent, not error: status {}",
        resp.status()
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.6 — Memory write followed by a recall.
///
/// Write: via the `native__memory_save` MCP native tool (`src/uar/tools/
/// memory.rs`, namespaced per `McpRegistry::with_native_tool`'s
/// `native__{name}` convention), same tool-loop mechanism proven in
/// `tool_loop_round_trip`. Recall: NOT a second LLM tool call — that would
/// only prove the orchestrator echoes back whatever content this test's own
/// stub fixture supplies, not that the write actually persisted. Instead,
/// recall is verified via `GET /api/admin/memories/search`, a plain REST
/// endpoint reading directly from the same embedded `MemoryService` instance
/// this server booted with (`ServiceNeeds { memory: true }`) — independent
/// proof the write landed in real (if embedded) storage, not just that the
/// stub's canned answer was echoed.
///
/// Currently ignored: `MemoryService::new` requires an embedding provider.
/// `embedding_provider: "local"` needs `surreal-memory`'s `local-embeddings`
/// Cargo feature, which this workspace does not enable anywhere (checked:
/// `grep -rn local-embeddings Cargo.toml` — zero hits). `start_server`
/// swallows the resulting construction error into `memory_service: None`
/// (a logged `tracing::error!`, not a panic), which is why this test's
/// symptom was a 503 from the search endpoint rather than a boot failure.
/// `"openai"`/`"cohere"` need a real API key, unsuitable for this hermetic
/// tier. See design.md's Risk 1 and appstate-field-plan.md's correction —
/// this was originally (wrongly) marked "resolved" before the test actually
/// ran. Re-enable once `local-embeddings` is available in this build.
#[tokio::test]
#[serial]
#[ignore = "needs surreal-memory's local-embeddings Cargo feature, not enabled in this workspace — see design.md Risk 1"]
async fn memory_write_then_recall() {
    const USER_ID: &str = "live-itest-user";
    const CONTENT: &str = "the sky is blue in this test";

    let fixtures = FixtureSet::new()
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "remember that the sky is blue".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::ToolCall {
                name: "native__memory_save".to_string(),
                arguments: serde_json::json!({
                    "content": CONTENT,
                    "user_id": USER_ID,
                })
                .to_string(),
            },
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "remember that the sky is blue".to_string(),
                has_tools: true,
                has_tool_result: true,
            },
            FixtureResponse::Content("saved it".to_string()),
        );
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds { memory: true }).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let write_resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "remember that the sky is blue"}],
            "stream": false,
        }))
        .send()
        .await
        .expect("write request");
    assert!(
        write_resp.status().is_success(),
        "status: {} body: {}",
        write_resp.status(),
        write_resp.text().await.unwrap_or_default()
    );

    let search_resp = client
        .get(format!(
            "{}/api/admin/memories/search?q=sky&user_id={USER_ID}",
            server.base_url
        ))
        .send()
        .await
        .expect("recall/search request");
    assert!(
        search_resp.status().is_success(),
        "search status: {}",
        search_resp.status()
    );
    let body: serde_json::Value = search_resp.json().await.expect("search json body");
    let items = body["items"].as_array().expect("items array");
    assert!(
        items
            .iter()
            .any(|item| item["content"].as_str() == Some(CONTENT)),
        "expected the saved memory to be recallable via search, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.7 — RAG document ingest followed by a retrieval.
///
/// Unlike memory (2.6), RAG's embedding path is `VectorMatcher.embed_batch`
/// (`src/uar/api/knowledge.rs:541-543`) — the same local Burn/Candle model
/// already used by the intent classifier (`src/uar/runtime/matching/
/// vector.rs`, loaded from the committed `tokenizer.json`), NOT
/// `surreal_memory`'s Cargo-feature-gated `EmbeddingProvider::Local`. So RAG
/// is NOT blocked by the same wall as memory — checked before writing this,
/// not assumed after 2.6's failure.
///
/// Ingestion is asynchronous (`IngestionWorkerPool`, `src/uar/api/
/// knowledge.rs:422-441`): `POST .../documents` returns 202 with
/// `status: "pending"` and processes in the background
/// (pending -> processing -> indexed).
///
/// This test polls the SEARCH endpoint directly, not the document-status
/// endpoint — found (via a `tracing_subscriber` added to `harness.rs` for
/// exactly this kind of diagnosis) that `update_document_status`
/// (`src/uar/persistence/providers/surreal.rs:524`) uses the SurrealQL
/// function `type::thing(...)`, which the pinned SurrealDB version
/// (`=3.0.5`) rejects ("did you maybe mean type::record"). The status write
/// silently fails and is swallowed (`warn!`/`error!`, no propagation), so a
/// document that ingests successfully (chunked, embedded, stored — the
/// worker log confirms "Document ingestion completed") stays reported as
/// `"pending"` forever. Flagged separately (spawn_task), not fixed here.
///
/// Formerly ignored because `VectorMatcher::embed_batch` returned all-zero
/// placeholder vectors, making retrieval structurally impossible. Re-enabled
/// by `fix-embeddings-fastembed` (uar-final-production-hardening-2026-07),
/// which wired real local BGE-small inference via fastembed — this case now
/// validates that fix end-to-end exactly as its original disclosure promised.
#[cfg(feature = "local-models")]
#[tokio::test]
#[serial]
async fn rag_ingest_then_retrieve() {
    let stub = start_stub_llm(FixtureSet::new()).await; // unused — this case never calls the LLM
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let client = reqwest::Client::new();

    let kb_resp = client
        .post(format!("{}/api/knowledge", server.base_url))
        .json(&serde_json::json!({
            "name": format!("live-itest-kb-{}", uuid::Uuid::new_v4()),
            "description": "live integration test KB",
        }))
        .send()
        .await
        .expect("create KB request");
    assert!(
        kb_resp.status().is_success(),
        "create KB status: {} body: {}",
        kb_resp.status(),
        kb_resp.text().await.unwrap_or_default()
    );
    let kb: serde_json::Value = kb_resp.json().await.expect("KB json body");
    let kb_id = kb["id"].as_str().expect("KB id").to_string();

    const CONTENT: &str = "Prometheus universal agent runtime live integration test marker phrase.";
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(CONTENT.as_bytes().to_vec())
            .file_name("marker.txt")
            .mime_str("text/plain")
            .unwrap(),
    );
    let upload_resp = client
        .post(format!(
            "{}/api/knowledge/{kb_id}/documents",
            server.base_url
        ))
        .multipart(form)
        .send()
        .await
        .expect("upload request");
    assert!(
        upload_resp.status().is_success(),
        "upload status: {} body: {}",
        upload_resp.status(),
        upload_resp.text().await.unwrap_or_default()
    );
    let doc: serde_json::Value = upload_resp.json().await.expect("upload json body");
    let doc_id = doc["id"].as_str().expect("document id").to_string();

    // doc_id is captured for completeness (matches the real API response
    // shape) but deliberately unused for polling — see the doc comment on
    // why document-status polling doesn't work in this build.
    let _ = doc_id;

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    let search_body = loop {
        let search_resp = client
            .post(format!("{}/api/knowledge/{kb_id}/search", server.base_url))
            .json(&serde_json::json!({ "query": "live integration test marker phrase" }))
            .send()
            .await
            .expect("search request");
        assert!(
            search_resp.status().is_success(),
            "search status: {} body: {}",
            search_resp.status(),
            search_resp.text().await.unwrap_or_default()
        );
        let body: serde_json::Value = search_resp.json().await.expect("search json body");
        let has_results = body["results"]
            .as_array()
            .is_some_and(|results| !results.is_empty());
        if has_results {
            break body;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "no search results within 20s (last response: {body})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    };

    let results = search_body["results"].as_array().expect("results array");
    assert!(
        results.iter().any(|r| r["content"]
            .as_str()
            .is_some_and(|c| c.contains("marker phrase"))),
        "expected the ingested marker content to be retrievable via search, got: {search_body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.8 — Credential-chain resolution (multi-tenant provider credentials).
///
/// Exercises the per-user encrypted credential store through the FULL booted
/// server (`/api/uar/credentials`), not the narrow sub-router the existing
/// `tests/credentials_api_integration_test.rs` uses — so this additionally
/// proves the real auth middleware + `CREDENTIAL_ENCRYPTION_KEY`-gated
/// `ProviderService` wiring.
///
/// Two pieces of setup the other baseline cases don't need:
///   1. `CREDENTIAL_ENCRYPTION_KEY` (32 ASCII chars) — `ProviderService`
///      only exists when this is set at boot (`from_env`); otherwise the API
///      returns 503. Set it around boot only (server reads it once during
///      startup), then removed — `#[serial]` keeps this from leaking into
///      other tests' boots.
///   2. A real Bearer JWT — the credentials API rejects anonymous callers
///      (401). The middleware parses a provided token even with
///      `jwt_required: false`, so a token signed with the harness's jwt
///      secret yields a genuine non-anonymous `UserContext`.
#[tokio::test]
#[serial]
async fn credential_chain_put_then_list() {
    // SAFETY: process-global env mutation, guarded by #[serial]; removed
    // immediately after boot (below), before any assertion that could panic.
    unsafe {
        std::env::set_var(
            "CREDENTIAL_ENCRYPTION_KEY",
            "0123456789abcdef0123456789abcdef",
        );
    }

    let stub = start_stub_llm(FixtureSet::new()).await; // unused — no LLM call
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    // Server has now read the key into its ProviderService — safe to clear so
    // a later test's boot isn't affected even if an assertion below panics.
    // SAFETY: process-global env mutation, guarded by #[serial].
    unsafe {
        std::env::remove_var("CREDENTIAL_ENCRYPTION_KEY");
    }

    // Mint a Bearer token the booted server will verify (same secret).
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "live-itest-user".to_string(),
        name: Some("Live ITest User".to_string()),
        roles: Some(vec!["user".to_string()]),
        tenant_id: None,
        uar_instance_id: None,
        exp: usize::MAX,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
    )
    .expect("mint jwt");
    let bearer = format!("Bearer {token}");

    let client = reqwest::Client::new();
    const RAW_KEY: &str = "sk-live-itest-super-secret-value-42";

    // Store a provider key.
    let put_resp = client
        .put(format!("{}/api/uar/credentials/openai", server.base_url))
        .header("Authorization", &bearer)
        .json(&serde_json::json!({ "api_key": RAW_KEY }))
        .send()
        .await
        .expect("put credential request");
    assert!(
        put_resp.status().is_success(),
        "put status: {} body: {}",
        put_resp.status(),
        put_resp.text().await.unwrap_or_default()
    );

    // List credentials back — masked metadata only, never the raw key.
    let list_resp = client
        .get(format!("{}/api/uar/credentials", server.base_url))
        .header("Authorization", &bearer)
        .send()
        .await
        .expect("list credentials request");
    assert!(
        list_resp.status().is_success(),
        "list status: {}",
        list_resp.status()
    );
    let list_text = list_resp.text().await.expect("list body");
    assert!(
        list_text.contains("openai"),
        "expected the stored openai credential in the list, got: {list_text}"
    );
    assert!(
        !list_text.contains(RAW_KEY),
        "raw api key must NEVER appear in a credential list response, got: {list_text}"
    );

    // Anonymous request (no Bearer) must be rejected — proves auth is enforced
    // through the full server, not just in the sub-router unit test.
    let anon_resp = client
        .get(format!("{}/api/uar/credentials", server.base_url))
        .send()
        .await
        .expect("anonymous list request");
    assert_eq!(
        anon_resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "anonymous credential access must be 401"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.9 — Backend-parametric parity smoke.
///
/// Cases 2.1-2.8 hardcode the stub and assert exact fixture content, which
/// makes them recorded-only by nature: a live model will not reproduce canned
/// text like "hi there", so those exact-match assertions cannot hold against
/// the real proxy. This case is the exception — it routes through
/// `backend::resolve()` so it honors `UAR_LIVE_INTEGRATION_BACKEND`: recorded
/// (default, CI-safe) hits the in-process stub; `live` (operator, local) hits
/// the real proxy at 127.0.0.1:8181. Its assertions are content-TOLERANT
/// (2xx + non-empty assistant text), so the pass/fail shape is identical on
/// both backends — that shared shape IS the "parity" task 2.9 asks for, and
/// this is the only baseline case actually wired through the dual-backend
/// switch. Live-backend runs are exercised via `scripts/live-integration.sh`
/// (operator; needs the proxy + a real model, hence non-deterministic).
#[tokio::test]
#[serial]
async fn backend_parametric_chat_smoke() {
    const PROBE: &str = "say hello to the live integration tier";
    let fixtures = FixtureSet::new().with(
        RequestFingerprint {
            model: MODEL.to_string(),
            last_user_message: PROBE.to_string(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content("hello from the recorded backend".to_string()),
    );
    // Held for the whole test so its stub (recorded backend) stays alive.
    let backend = super::backend::resolve(fixtures).await;
    let server = boot_test_server(&backend.base_url, &backend.model, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": backend.model,
            "messages": [{"role": "user", "content": PROBE}],
            "stream": false,
        }))
        .send()
        .await
        .expect("chat request");
    assert!(
        resp.status().is_success(),
        "status: {} body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
    let body: serde_json::Value = resp.json().await.expect("json body");
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    assert!(
        !content.trim().is_empty(),
        "expected non-empty assistant content on either backend, got: {body}"
    );
    })).await;
    server.finish_case(fixture_outcome).await

}

/// 2.10 — Anthropic `/v1/messages` binds a registered agent (R7).
///
/// Proves the registered-agent binding on the Anthropic-compatible route:
/// a created agent's `prompt.system` (and host instructions) reach the
/// upstream model request verbatim, the reply reflects the agent, the
/// client-supplied system text is kept after the agent prompt (mirroring
/// the chat-completions run pipeline, where client system messages are not
/// dropped), bound `uar:agent:*` targets are rejected with 400, and an
/// unknown registry id surfaces 404 rather than silently falling back.
#[tokio::test]
#[serial]
async fn anthropic_messages_binds_registered_agent() {
    use serde_json::Value;

    let stub = start_stub_llm(
        FixtureSet::new().with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "hello anthropic agent".to_string(),
                has_tools: false,
                has_tool_result: false,
            },
            FixtureResponse::Content("hi there from the bound agent".to_string()),
        ),
    )
    .await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let peer_token = mint_harness_peer_token();
    let client = reqwest::Client::new();

    let agent_marker = "R7-AGENT-PROMPT-MARKER-2-10";
    let instr_marker = "R7-INSTR-MARKER-2-10";
    let agent = serde_json::json!({
        "version": "1.0.0",
        "kind": "agent",
        "id": "r7-bind-agent",
        "metadata": {
            "title": "R7 bind agent",
            "description": "baseline case 2.10 registered agent",
            "tags": [],
            "author": null,
            "icon": null
        },
        "runtime": {"entry": "chat", "protocols": {}},
        "policy": {
            "provider": {
                "default": {"provider": "stub", "model": MODEL},
                "fallbacks": []
            },
            "tools": {"allow": [], "deny": [], "max_concurrent": 1, "execution_mode": "direct"},
            "skills": {"prefer": [], "max_active": 3}
        },
        "schemas": {"inputs": null, "outputs": null, "state": null},
        "prompt": {
            "system": format!("You are the r7 bind agent. {agent_marker}"),
            "instructions": [format!("Carry the {instr_marker} procedure.")]
        },
        "memory": {},
        "tools": {},
        "ui": {}
    });
    let create = client
        .post(format!("{}/api/agents", server.base_url))
        .bearer_auth(&peer_token)
        .json(&agent)
        .send()
        .await
        .expect("agent create request");
    assert!(
        create.status().is_success(),
        "agent create: status {} body {}",
        create.status(),
        create.text().await.unwrap_or_default()
    );
    let readback = client
        .get(format!("{}/api/agents/r7-bind-agent", server.base_url))
        .bearer_auth(&peer_token)
        .send()
        .await
        .expect("agent read-back request");
    assert!(
        readback.status().is_success(),
        "agent read-back: status {}",
        readback.status()
    );

    let client_marker = "R7-CLIENT-SYS-MARKER-2-10";
    let resp = client
        .post(format!("{}/v1/messages", server.base_url))
        .bearer_auth(&peer_token)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": MODEL,
            "max_tokens": 64,
            "agent_id": "r7-bind-agent",
            "system": format!("client system text {client_marker}"),
            "messages": [{"role": "user", "content": "hello anthropic agent"}]
        }))
        .send()
        .await
        .expect("anthropic messages request");
    assert!(
        resp.status().is_success(),
        "status {} body {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
    let body: Value = resp.json().await.expect("json body");
    assert_eq!(body["type"].as_str(), Some("message"), "body: {body}");
    assert_eq!(body["role"].as_str(), Some("assistant"), "body: {body}");
    let text = body["content"][0]["text"].as_str().unwrap_or_default();
    assert!(
        text.contains("hi there from the bound agent"),
        "expected the fixture reply, got: {body}"
    );

    // The upstream model request must carry the agent prompt verbatim, the
    // host instruction, and the kept client system text.
    let stub_url = format!("{}/_stub/requests", stub.base_url.trim_end_matches("/v1"));
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    let recorded = loop {
        let observed: Value = client
            .get(&stub_url)
            .send()
            .await
            .expect("stub request log")
            .json()
            .await
            .expect("stub request log JSON");
        if observed["requests"].as_array().is_some_and(|requests| {
            requests
                .iter()
                .any(|request| request.to_string().contains(agent_marker))
        }) {
            break observed;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "agent prompt never reached the upstream request: {observed}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };
    let upstream = recorded["requests"]
        .as_array()
        .expect("requests array")
        .iter()
        .find(|request| request.to_string().contains(agent_marker))
        .expect("request with the agent marker")
        .to_string();
    assert!(
        upstream.contains("You are the r7 bind agent."),
        "agent system prompt not verbatim upstream: {upstream}"
    );
    assert!(
        upstream.contains(instr_marker),
        "agent host instruction missing upstream: {upstream}"
    );
    assert!(
        upstream.contains(client_marker),
        "client system text should be kept after the agent prompt: {upstream}"
    );

    // Bound `uar:agent:*` targets are rejected on this route (400), and an
    // unknown registry id surfaces 404 instead of a silent fallback.
    let bound = client
        .post(format!("{}/v1/messages", server.base_url))
        .bearer_auth(&peer_token)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": MODEL,
            "max_tokens": 64,
            "agent_id": "uar:agent:r7-bind-agent",
            "messages": [{"role": "user", "content": "hello anthropic agent"}]
        }))
        .send()
        .await
        .expect("bound-target request");
    assert_eq!(
        bound.status().as_u16(),
        400,
        "bound target must be rejected: status {} body {}",
        bound.status(),
        bound.text().await.unwrap_or_default()
    );
    let unknown = client
        .post(format!("{}/v1/messages", server.base_url))
        .bearer_auth(&peer_token)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": MODEL,
            "max_tokens": 64,
            "agent_id": "no-such-agent-2-10",
            "messages": [{"role": "user", "content": "hello anthropic agent"}]
        }))
        .send()
        .await
        .expect("unknown-agent request");
    assert_eq!(
        unknown.status().as_u16(),
        404,
        "unknown agent must 404: status {} body {}",
        unknown.status(),
        unknown.text().await.unwrap_or_default()
    );
    })).await;
    server.finish_case(fixture_outcome).await

}
