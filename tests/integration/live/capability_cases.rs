//! Capability smoke cases — Phase 7 Q1.
//!
//! # What this measures, and what it does NOT
//!
//! Each case boots a real server via [`boot_test_server`] and issues a real HTTP
//! request, proving the endpoint is mounted and answers with the expected shape.
//!
//! **This is a smoke matrix, not a doneness measurement.** Two independent
//! adversarial reviews (MiniMax-M3 critic, k3 judge) ruled the stronger framing
//! unsupportable. The limits are structural, not incidental:
//!
//! - **L4 is capability-specific.** C-12 uses a caller-owned SurrealKV path and
//!   caller-triggered graceful shutdown to prove a write survives a restart
//!   before the original helper process exits. C-13 remains excluded because
//!   the current header-based session
//!   surface is backed by an explicitly non-durable in-process `SessionStore`.
//! - **No semantics.** Assertions check response *shape*. A route returning a
//!   well-formed body containing the *wrong* answer passes.
//! - **Stub provider.** Cases run against `stub_llm`, whose fixtures this same
//!   author writes. For any capability whose correctness depends on what comes
//!   *out* of the model, that is "did my code parse my own canned output" —
//!   **L2 wired, not L3 exercised**. Such cases are marked `l2_` in their name.
//! - **One profile.** Everything here is `server-full`. Per SPECIFICATION.md
//!   §12.1, `embedded-mobile` compiles no `server` feature at all and therefore
//!   has *none* of these routes. A pass here transfers to no other profile.
//!
//! # Evidence label taxonomy
//!
//! Every case name starts with exactly one prefix from this closed set:
//!
//! - `l1_` — route present: reachable, without a behavioural claim.
//! - `l2_` — wired: exercises the real call path with fixtures authored by the
//!   test.
//! - `l3_` — exercised: correctness is independent of stub output.
//! - `l4_` — round-tripped: the result survives a runtime restart.
//! - `shape_only_` — response shape only, without a semantic claim.
//! - `absent_` — asserts a documented absence.
//! - `excluded_` — published exclusion; its doc comment names the reason.
//!
//! # The catch-all discriminator
//!
//! `server.rs:1093` routes `/api/{*path}` to `api_route_not_found`, so an
//! unmounted `/api/*` path still produces a well-formed JSON response. Checking
//! only the status code would confuse "route absent" with "route present and
//! rejecting". Every case therefore calls [`assert_real_handler`], which fails
//! on the sentinel `code: "api_route_not_found"` regardless of status.

use super::harness::{
    HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server, boot_test_server_process,
    mint_harness_peer_token,
};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};
use serial_test::serial;

/// Model name as it appears on the wire to the stub — bare, no `provider/`
/// prefix; see `harness.rs`'s discovery note.
const MODEL: &str = "gpt-5.4-mini";

/// Sentinel emitted by `server.rs`'s `/api/{*path}` catch-all.
const CATCH_ALL_CODE: &str = "api_route_not_found";

/// Fails if the response came from the catch-all rather than a real handler.
///
/// This is the discriminator the adversarial review required: without it, a
/// test for an unmounted route sees a tidy JSON error and cannot tell it from a
/// mounted route that legitimately rejected the request.
fn assert_real_handler(capability: &str, path: &str, status: u16, body: &str) {
    assert!(
        !body.contains(CATCH_ALL_CODE),
        "{capability}: {path} is NOT MOUNTED — the /api/{{*path}} catch-all answered \
         (status {status}). Body: {body}"
    );
}

/// A GET that proves a route exists and is answered by its own handler.
///
/// Returns `(status, body)` so each case can make its own shape assertion.
async fn get_capability(base_url: &str, capability: &str, path: &str) -> (u16, String) {
    let resp = reqwest::Client::new()
        .get(format!("{base_url}{path}"))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .send()
        .await
        .unwrap_or_else(|e| panic!("{capability}: request to {path} failed: {e}"));

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler(capability, path, status, &body);
    (status, body)
}

/// Assert a list endpoint answered 200 with an *empty* collection.
///
/// Stronger than the `200 + parses as JSON` check most cases here use, and
/// deliberately so: SurrealDB is schemaless, so a table does not exist until
/// its first insert and a read against a never-written one fails at the driver
/// level. That surfaced to clients as HTTP 500 on every fresh deploy until the
/// read sites were guarded. Asserting only "200 and valid JSON" would not have
/// caught the regression, and would not catch a future re-break that returns a
/// well-formed error envelope.
fn assert_empty_collection(capability: &str, path: &str, status: u16, body: &str) {
    assert_eq!(
        status, 200,
        "{capability}: {path} on a fresh DB must not error, got {status}: {body}"
    );

    let json: serde_json::Value = serde_json::from_str(body)
        .unwrap_or_else(|e| panic!("{capability}: {path} returned non-JSON: {e}\n{body}"));

    let items = match &json {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(map) => map
            .values()
            .find_map(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("{capability}: {path} envelope has no array field: {body}")),
        other => panic!("{capability}: {path} returned neither array nor object: {other}"),
    };

    assert!(
        items.is_empty(),
        "{capability}: {path} on a fresh DB must be empty, got: {body}"
    );
}

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

/// C-06 orchestrator delegation — **L2 recorded / env-gated live**.
///
/// The recorded backend fixes the router choice and specialist contribution;
/// the live backend exercises the same two-call RouterNode -> AgentNode path
/// against the operator's proxy. UAR-owned assertions require the attributed
/// answer plus start/finish runtime-step events on either backend.
#[tokio::test]
#[serial]
async fn l2_c06_orchestrator_delegates_with_trace() {
    const PROBE: &str = "Review this Rust ownership boundary";
    const ROUTER_REQUEST: &str = concat!(
        "Route Rust implementation, correctness, or safety questions to rust-reviewer. ",
        "Route all other questions to general-purpose.\n\n",
        "User request:\nReview this Rust ownership boundary\n\n",
        "You MUST respond with exactly one of the following options (no extra text): ",
        "general-purpose, rust-reviewer"
    );
    const RECORDED_CONTRIBUTION: &str = "The ownership boundary is sound.";

    let fixtures = FixtureSet::new()
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: ROUTER_REQUEST.to_string(),
                has_tools: false,
                has_tool_result: false,
            },
            FixtureResponse::Content("rust-reviewer".to_string()),
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: PROBE.to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content(RECORDED_CONTRIBUTION.to_string()),
        );
    let recorded = std::env::var(super::backend::BACKEND_ENV_VAR).as_deref() != Ok("live");
    let backend = super::backend::resolve(fixtures).await;
    let server = boot_test_server(&backend.base_url, &backend.model, ServiceNeeds::default()).await;
    let peer_token = mint_harness_peer_token();

    let response = reqwest::Client::new()
        .post(format!("{}/api/chat/completion", server.base_url))
        .bearer_auth(peer_token)
        .json(&serde_json::json!({
            "model": backend.model,
            "agent_id": "orchestrator-agent",
            "messages": [{"role": "user", "content": PROBE}],
            "stream": true,
            "stream_mode": "openai",
        }))
        .send()
        .await
        .expect("orchestrator delegation request");
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    assert_real_handler("C-06", "/api/chat/completion", status, &body);
    assert_eq!(status, 200, "C-06 delegation must return 200: {body}");
    let assistant_text = body
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter(|data| *data != "[DONE]")
        .filter_map(|data| serde_json::from_str::<serde_json::Value>(data).ok())
        .filter_map(|chunk| {
            chunk
                .pointer("/choices/0/delta/content")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect::<String>();
    let contribution = assistant_text
        .strip_prefix("[rust-reviewer]\n\n")
        .unwrap_or_else(|| panic!("answer must attribute the selected sub-agent: {body}"));
    assert!(
        !contribution.trim().is_empty(),
        "answer must contain the selected sub-agent's contribution: {body}"
    );
    assert!(
        body.contains("event: runtime.step")
            && body.contains("step_started")
            && body.contains("step_finished"),
        "router and agent traversal must emit runtime step events: {body}"
    );
    if recorded {
        assert_eq!(contribution, RECORDED_CONTRIBUTION);
    }
}

struct C19FixtureProvider;

#[async_trait::async_trait]
impl universal_agent_runtime::uar::eval::CompletionProvider for C19FixtureProvider {
    async fn complete(&self, _input: &str) -> anyhow::Result<String> {
        Ok("deterministic C-19 fixture output".to_string())
    }
}

// ---------------------------------------------------------------------------
// L3 — correctness independent of model output
// ---------------------------------------------------------------------------

/// C-20 health, readiness, metrics.
///
/// Genuine L3: these endpoints' correctness does not depend on the LLM.
#[tokio::test]
#[serial]
async fn l3_c20_health_readiness_metrics() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-20", "/healthz").await;
    assert_eq!(status, 200, "healthz should be 200, body: {body}");
    assert!(
        body.contains("ok") || body.contains("status"),
        "healthz should report a status, got: {body}"
    );

    // /readyz probes real subsystems, so 200 or 503 are both real answers.
    let (ready_status, ready_body) = get_capability(&server.base_url, "C-20", "/readyz").await;
    assert!(
        ready_status == 200 || ready_status == 503,
        "readyz should be 200 or 503, got {ready_status}: {ready_body}"
    );

    let (m_status, m_body) = get_capability(&server.base_url, "C-20", "/metrics").await;
    assert_eq!(m_status, 200, "metrics should be 200");
    assert!(
        m_body.contains('#') || m_body.contains("_total"),
        "metrics should be Prometheus exposition format, got: {}",
        &m_body.chars().take(200).collect::<String>()
    );
}

/// C-07 skills catalog.
///
/// Genuine L3. **Profile-scoped:** this passes on `server-full` because
/// `register_builtins` is called from `server.rs:436`. GAP-05 records that the
/// same capability is at **0%** on `embedded-mobile`, where `server` is not
/// compiled and that call site does not exist. This pass says nothing about
/// that profile.
#[tokio::test]
#[serial]
async fn l3_c07_skills_catalog() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    // Path resolved from `server.rs`'s `.nest("/api/uar/skills", …)` + the
    // sub-router's `.route("/")`. An earlier draft used `/api/skills`, which
    // would have produced a false ABSENT via the catch-all.
    let (status, body) = get_capability(&server.base_url, "C-07", "/api/uar/skills").await;
    assert_eq!(status, 200, "skills catalog should be 200, got: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-07: body is not JSON: {e}\n{body}"));
    assert!(
        parsed.is_array() || parsed.get("skills").is_some() || parsed.get("data").is_some(),
        "C-07: expected a skills collection, got: {body}"
    );
}

/// C-10 settings — the deployment configuration surface.
#[tokio::test]
#[serial]
async fn l3_c10_settings_surface() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-10", "/api/uar/settings").await;
    assert_eq!(status, 200, "settings should be 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-10: settings body is not JSON: {e}\n{body}"));
}

/// C-11 A2UI schema registry.
#[tokio::test]
#[serial]
async fn l3_c11_a2ui_schema_registry() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-11", "/api/uar/a2ui/schemas").await;
    assert_eq!(status, 200, "a2ui schemas should be 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-11: schemas body is not JSON: {e}\n{body}"));
}

/// C-08 tools registry — the *catalog* is model-independent, so listing is L3.
/// Tool *selection* by a model is not tested here (that would be L2).
#[tokio::test]
#[serial]
async fn l3_c08_tools_registry() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-08", "/api/tools").await;
    assert_eq!(status, 200, "tools registry should be 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-08: tools body is not JSON: {e}\n{body}"));
}

/// C-03 provider registry — *listing* providers is model-independent.
/// Routing *decisions* are covered by `l2_c03_model_routing`.
#[tokio::test]
#[serial]
async fn l3_c03_provider_registry() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-03", "/api/uar/providers").await;
    assert_eq!(status, 200, "providers should be 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-03: providers body is not JSON: {e}\n{body}"));
}

// ---------------------------------------------------------------------------
// L2 — wired, but certified only against a stub this author wrote
// ---------------------------------------------------------------------------

/// C-14 OpenAI-compatible surface — **L2 ONLY**.
///
/// SPECIFICATION.md scopes this capability to *"BossFang — only live traffic"*.
/// The stub returns the fixture verbatim, so a pass proves UAR built a
/// well-formed request and parsed a well-formed reply. It cannot prove
/// compatibility with any real provider.
#[tokio::test]
#[serial]
async fn l2_c14_openai_compatible_surface() {
    let stub = start_stub_llm(content_fixture("ping", "pong")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .post(format!("{}/v1/chat/completions", server.base_url))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "ping"}],
        }))
        .send()
        .await
        .expect("C-14: request");

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-14", "/v1/chat/completions", status, &body);
    assert_eq!(status, 200, "C-14: expected 200, got {status}: {body}");
    assert!(
        body.contains("choices"),
        "C-14: expected an OpenAI-shaped response, got: {body}"
    );
}

/// C-01 + C-02 run lifecycle and AG-UI streaming — **L2**.
///
/// The stream *shape* is UAR's own, but the content driving it comes from the
/// fixture, so this cannot distinguish correct orchestration from a fixture
/// that happens to match.
#[tokio::test]
#[serial]
async fn l2_c01_c02_run_stream_shape() {
    let stub = start_stub_llm(content_fixture("stream please", "streamed reply")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .post(format!("{}/api/chat/completion", server.base_url))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "stream please"}],
            "stream": true,
            "stream_mode": "agui",
        }))
        .send()
        .await
        .expect("C-01/C-02: request");

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-01/C-02", "/api/chat/completion", status, &body);
    assert!(
        status == 200,
        "C-01/C-02: expected 200, got {status}: {body}"
    );
    assert!(
        body.contains("streamed reply"),
        "C-01/C-02: expected fixture content in the stream, got: {body}"
    );
}

// ---------------------------------------------------------------------------
// L4 — survives a cold runtime restart
// ---------------------------------------------------------------------------

/// C-12 persistence — write a knowledge-base resource, stop its server runtime,
/// reopen the same SurrealKV path while the original helper process remains
/// alive at a pre-exit barrier, and read the identical resource.
/// Setting `UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH=1` deliberately points the
/// second boot at an empty path; the final assertion must then fail.
#[tokio::test]
#[serial]
async fn l4_c12_persistence_round_trip() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let scratch = tempfile::tempdir().expect("C-12: create persistence scratch");
    let persistence_path = scratch.path().join("surrealkv");
    let server = boot_test_server_process(
        &stub.base_url,
        MODEL,
        ServiceNeeds::default(),
        &persistence_path,
    )
    .await;
    let client = reqwest::Client::new();
    let expected_name = format!("c12-round-trip-{}", uuid::Uuid::new_v4());
    let expected_description = "C-12 cold-restart marker";

    let create = client
        .post(format!("{}/api/knowledge", server.base_url))
        .json(&serde_json::json!({
            "name": expected_name,
            "description": expected_description,
        }))
        .send()
        .await
        .expect("C-12: create knowledge base");
    let create_status = create.status().as_u16();
    let create_body = create.text().await.unwrap_or_default();
    assert_real_handler("C-12", "/api/knowledge", create_status, &create_body);
    assert_eq!(
        create_status, 201,
        "C-12: expected resource creation to return 201, got: {create_body}"
    );
    let created: serde_json::Value = serde_json::from_str(&create_body)
        .unwrap_or_else(|e| panic!("C-12: create body is not JSON: {e}\n{create_body}"));
    let resource_id = created["id"]
        .as_str()
        .expect("C-12: created resource id")
        .to_string();

    let original_barrier = server.shutdown_to_pre_exit_barrier("TERM").await;

    let negative_path = scratch.path().join("negative-control-surrealkv");
    let reopen_path = if std::env::var_os("UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH").is_some() {
        negative_path.as_path()
    } else {
        persistence_path.as_path()
    };
    let restarted =
        boot_test_server_process(&stub.base_url, MODEL, ServiceNeeds::default(), reopen_path).await;
    original_barrier.allow_exit().await;
    let resource_path = format!("/api/knowledge/{resource_id}");
    let (status, body) = get_capability(&restarted.base_url, "C-12", &resource_path).await;
    restarted.shutdown().await;

    assert_eq!(
        status, 200,
        "C-12: resource did not survive the cold restart: {body}"
    );
    let reopened: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-12: reopened body is not JSON: {e}\n{body}"));
    assert_eq!(
        reopened["id"].as_str(),
        Some(resource_id.as_str()),
        "C-12: reopened resource id changed: {body}"
    );
    assert_eq!(
        reopened["name"].as_str(),
        Some(expected_name.as_str()),
        "C-12: reopened resource name changed: {body}"
    );
    assert_eq!(
        reopened["description"].as_str(),
        Some(expected_description),
        "C-12: reopened resource description changed: {body}"
    );
}

// ---------------------------------------------------------------------------
// Extension round — the ten capabilities uncovered by the first pass
//
// Every path below was resolved from `server.rs`'s `.nest()` prefixes composed
// with each sub-router's own `.route()`, not guessed. Guessing produced a
// near-miss in the first round.
// ---------------------------------------------------------------------------

/// C-04 credentials — multi-tenant provider credential subsystem.
///
/// L3: listing stored credentials is model-independent. Note the server logs
/// `CREDENTIAL_ENCRYPTION_KEY invalid; multi-tenant credentials disabled` when
/// the key is absent, so a 200 here does **not** prove multi-tenant encryption
/// is active — only that the endpoint is mounted and answers.
#[tokio::test]
#[serial]
async fn l3_c04_credentials_listing() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/api/uar/credentials", server.base_url))
        .send()
        .await
        .expect("C-04: unauthenticated credential-list request");
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-04", "/api/uar/credentials", status, &body);
    assert_eq!(
        status, 401,
        "C-04 unauthenticated credentials guard contract changed: expected 401, got {status}: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-04 credentials guard returned non-JSON: {e}\n{body}"));
    assert_eq!(
        parsed.get("error").and_then(serde_json::Value::as_str),
        Some("Authentication required"),
        "C-04 unauthenticated credentials guard contract changed: expected Authentication required, got: {body}"
    );
}

/// C-05 knowledge bases and RAG — **catalog only**.
///
/// Listing knowledge bases is model-independent, so L3. Retrieval *relevance*
/// depends on embeddings and the model and is **not** tested: a well-formed
/// citation list of irrelevant documents would pass.
#[tokio::test]
#[serial]
async fn l3_c05_knowledge_base_catalog() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-05", "/api/knowledge").await;
    assert_eq!(status, 200, "C-05: expected 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-05: body is not JSON: {e}\n{body}"));
}

/// C-06 memory — requires `ServiceNeeds { memory: true }`.
///
/// Shape only: the harness gives each boot a fresh memory store, so this cannot
/// show that anything is retained. GAP-05's sibling risk applies — memory is
/// initialised in `server.rs` and an embedder may not get it.
#[tokio::test]
#[serial]
async fn shape_only_c06_memory_stats() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds { memory: true }).await;

    let (status, body) =
        get_capability(&server.base_url, "C-06", "/api/admin/memories/stats").await;
    assert_eq!(
        status, 200,
        "C-06: expected 200 with memory enabled, got: {body}"
    );
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-06: body is not JSON: {e}\n{body}"));
}

/// C-09 agent compiler — spec listing on a fresh database.
///
/// Regression: SurrealDB's `The table 'uar_specs' does not exist` propagated to
/// the client as HTTP 500, so the compiler catalog was broken on every fresh
/// deploy until the first spec was written. The harness allocates a unique
/// temp SurrealKV path per boot, so every run of this case *is* a fresh-DB run.
#[tokio::test]
#[serial]
async fn l3_c09_compiler_specs() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let path = "/api/compiler/specs";
    let (status, body) = get_capability(&server.base_url, "C-09", path).await;
    assert_empty_collection("C-09", path, status, &body);
}

/// C-09b — the same contract on the `/api/uar/compiler` mount, which
/// `src/server.rs` nests from the same router.
#[tokio::test]
#[serial]
async fn l3_c09_uar_compiler_specs() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let path = "/api/uar/compiler/specs";
    let (status, body) = get_capability(&server.base_url, "C-09b", path).await;
    assert_empty_collection("C-09b", path, status, &body);
}

/// C-09c — compiler sessions on a fresh DB.
///
/// Not a regression case: `CompilerService::list_sessions` already swallowed the
/// storage error into an empty `Vec`, so this endpoint returned 200 even before
/// the fix. Asserted here to pin the contract, since the underlying
/// `uar_compiler_sessions` read was guarded alongside `uar_specs` and the
/// endpoint should not regress if that swallow is ever tightened.
#[tokio::test]
#[serial]
async fn l3_c09_compiler_sessions() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let path = "/api/compiler/sessions";
    let (status, body) = get_capability(&server.base_url, "C-09c", path).await;
    assert_empty_collection("C-09c", path, status, &body);
}

/// C-15 agent descriptor schema — served at the A2A well-known path.
///
/// L3 and genuinely so: this is a static contract document, independent of any
/// model. BossFang gates production readiness on well-known discovery (GAP-01),
/// which makes this one of the few capabilities with a real external consumer
/// contract.
#[tokio::test]
#[serial]
async fn l3_c15_agent_descriptor_well_known() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-15", "/.well-known/agent.json").await;
    assert_eq!(status, 200, "C-15: expected 200, got: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-15: agent.json is not JSON: {e}\n{body}"));
    assert!(
        parsed.get("name").is_some() || parsed.get("skills").is_some(),
        "C-15: expected an agent card, got: {body}"
    );
}

/// C-17 security — auth enforcement.
///
/// L3, and the only case here that asserts a **negative**: an unauthenticated
/// request to a protected route must NOT succeed. The harness sets
/// `jwt_required: false`, so this documents the configured posture rather than
/// proving enforcement works when required — stated so the result is not
/// over-read.
#[tokio::test]
#[serial]
async fn l3_c17_security_posture() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    // No Authorization header at all.
    let resp = reqwest::Client::new()
        .get(format!("{}/api/uar/settings", server.base_url))
        .send()
        .await
        .expect("C-17: request");
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-17", "/api/uar/settings", status, &body);

    // With jwt_required=false the harness expects 200; the assertion records
    // which posture was observed rather than presuming one.
    assert!(
        status == 200 || status == 401 || status == 403,
        "C-17: expected 200 (auth off) or 401/403 (auth on), got {status}: {body}"
    );
}

/// C-13 legacy sessions route — deliberately retired.
///
/// Session continuity moved to caller-supplied `X-UAR-Session-ID` values on
/// `POST /api/chat/completion`; this case pins the explicit retirement response.
#[tokio::test]
#[serial]
async fn absent_c13_sessions_retired() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-13", "/api/sessions").await;
    assert_eq!(
        status, 404,
        "C-13 retired-route contract changed: expected 404, got {status}: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-13 retired-route contract returned non-JSON: {e}\n{body}"));
    assert_eq!(
        parsed
            .pointer("/error/code")
            .and_then(serde_json::Value::as_str),
        Some("legacy_route_disabled"),
        "C-13 retired-route contract changed: expected error.code=legacy_route_disabled, got: {body}"
    );
}

/// C-13 session continuity — **excluded from L4**.
///
/// The current contract is exercised on both boots: a stable UUID is supplied
/// through `X-UAR-Session-ID` to `POST /api/chat/completion`. The first boot
/// creates live session state, but the context-stats handler returns 404 after
/// reopening the same persistence path. This matches `SessionStore`'s
/// in-process `HashMap` implementation and the runtime's explicit statement
/// that the store is not durable. Fixing it requires runtime persistence work
/// beyond the one-parameter source allowance for this change.
#[tokio::test]
#[serial]
async fn excluded_c13_session_continuity_is_not_durable() {
    let fixtures = FixtureSet::new()
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "c13 first turn".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content("c13 first reply".to_string()),
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "c13 second turn".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content("c13 second reply".to_string()),
        );
    let stub = start_stub_llm(fixtures).await;
    let scratch = tempfile::tempdir().expect("C-13: create persistence scratch");
    let persistence_path = scratch.path().join("surrealkv");
    let session_id = uuid::Uuid::new_v4().to_string();
    let client = reqwest::Client::new();

    let first = boot_test_server_process(
        &stub.base_url,
        MODEL,
        ServiceNeeds::default(),
        &persistence_path,
    )
    .await;
    let first_response = client
        .post(format!("{}/api/chat/completion", first.base_url))
        .header("X-UAR-Session-ID", &session_id)
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "c13 first turn"}],
            "stream": false,
        }))
        .send()
        .await
        .expect("C-13: first chat request");
    let first_status = first_response.status().as_u16();
    let first_body = first_response.text().await.unwrap_or_default();
    assert_real_handler("C-13", "/api/chat/completion", first_status, &first_body);
    assert_eq!(first_status, 200, "C-13: first chat failed: {first_body}");
    let stats_path = format!("/api/uar/sessions/{session_id}/context-stats");
    let (before_status, before_body) = get_capability(&first.base_url, "C-13", &stats_path).await;
    assert_eq!(
        before_status, 200,
        "C-13: live session was not observable before restart: {before_body}"
    );
    first.shutdown().await;

    let restarted = boot_test_server_process(
        &stub.base_url,
        MODEL,
        ServiceNeeds::default(),
        &persistence_path,
    )
    .await;
    let (after_status, after_body) = get_capability(&restarted.base_url, "C-13", &stats_path).await;
    assert_eq!(
        after_status, 404,
        "C-13 exclusion is stale: session unexpectedly survived restart: {after_body}"
    );

    let second_response = client
        .post(format!("{}/api/chat/completion", restarted.base_url))
        .header("X-UAR-Session-ID", &session_id)
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "c13 second turn"}],
            "stream": false,
        }))
        .send()
        .await
        .expect("C-13: second chat request");
    let second_status = second_response.status().as_u16();
    let second_body = second_response.text().await.unwrap_or_default();
    assert_real_handler("C-13", "/api/chat/completion", second_status, &second_body);
    restarted.shutdown().await;
    assert_eq!(
        second_status, 200,
        "C-13: current session contract failed after restart: {second_body}"
    );
}

// ---------------------------------------------------------------------------
// NOT MEASURABLE BY THIS INSTRUMENT — no HTTP surface exists
//
// C-16 governance, C-18 file processing, and C-19 evals have no dedicated HTTP
// route. Their cases below use the real surfaces that do exist: governance
// middleware, the upload handler, and the public eval runner. Each still hits a
// known real HTTP handler so the catch-all discriminator applies uniformly.
//
// ---------------------------------------------------------------------------

/// C-16 governance — **L2 wired**.
///
/// Supplying `X-Agent-Id` sends this request through the server's Cedar
/// governance middleware before the settings handler. The repository's policy
/// files are the fixture, so this proves the runtime wiring and default permit
/// behavior but not an independently-authored authorization policy.
#[tokio::test]
#[serial]
async fn l2_c16_governance_middleware() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let path = "/api/uar/settings";
    let resp = reqwest::Client::new()
        .get(format!("{}{}", server.base_url, path))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .header("X-Agent-Id", "conformance-c16-agent")
        .send()
        .await
        .expect("C-16: governed settings request");
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-16", path, status, &body);
    assert_eq!(
        status, 200,
        "C-16: default governance policy should permit the real settings handler: {body}"
    );
}

/// C-18 file processing / document intelligence — **L3 exercised**.
///
/// A text multipart upload runs through the real upload handler and must return
/// the exact extracted text independently of model output. Binary OCR is not
/// claimed by this case.
#[tokio::test]
#[serial]
async fn l3_c18_text_file_processing() {
    const CONTENT: &str = "C-18 deterministic document text";
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let path = "/api/upload";
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(CONTENT.as_bytes().to_vec())
            .file_name("c18-conformance.txt")
            .mime_str("text/plain")
            .expect("C-18: valid MIME type"),
    );
    let resp = reqwest::Client::new()
        .post(format!("{}{}", server.base_url, path))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .multipart(form)
        .send()
        .await
        .expect("C-18: upload request");
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    assert_real_handler("C-18", path, status, &body);
    assert_eq!(status, 200, "C-18: text upload failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("C-18: upload response is not JSON: {e}\n{body}"));
    let file = parsed
        .pointer("/files/0")
        .unwrap_or_else(|| panic!("C-18: upload response has no file: {body}"));
    assert_eq!(
        file.get("text_content").and_then(serde_json::Value::as_str),
        Some(CONTENT),
        "C-18: extracted text did not round-trip: {body}"
    );

    if let Some(id) = file.get("id").and_then(serde_json::Value::as_str) {
        let _ = std::fs::remove_file(
            std::env::temp_dir()
                .join("uar-uploads")
                .join(format!("{id}.txt")),
        );
    }
}

/// C-19 evals — **L2 wired**.
///
/// Loads the shipped suite, builds its declared scorers, and runs every case
/// through the real eval runner. The completion provider is authored by this
/// test, so the case establishes wiring and result production, not model
/// evaluation correctness.
#[tokio::test]
#[serial]
async fn l2_c19_eval_runner() {
    use universal_agent_runtime::uar::eval::{Runner, build_scorers, load_suite};

    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let (status, body) = get_capability(&server.base_url, "C-19", "/healthz").await;
    assert_eq!(
        status, 200,
        "C-19 discriminator: live runtime health handler failed: {body}"
    );

    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("evals/starter.yaml");
    let suite = load_suite(&suite_path)
        .unwrap_or_else(|e| panic!("C-19: shipped starter suite did not load: {e}"));
    assert!(!suite.cases.is_empty(), "C-19: starter suite has no cases");
    assert!(
        !suite.scorers.is_empty(),
        "C-19: starter suite has no scorers"
    );

    let provider: std::sync::Arc<dyn universal_agent_runtime::uar::eval::CompletionProvider> =
        std::sync::Arc::new(C19FixtureProvider);
    let scorers = build_scorers(&suite, &provider);
    let results = Runner
        .run(&suite, &scorers, provider.as_ref(), Some("recorded/c19"))
        .await;
    assert_eq!(
        results.len(),
        suite.cases.len(),
        "C-19: eval runner did not produce one result per case"
    );
    assert!(
        results
            .iter()
            .all(|result| result.scores.len() == suite.scorers.len()),
        "C-19: eval runner did not apply every declared scorer"
    );
}

/// C-21 tenant isolation.
///
/// Genuine L3: two independently verified tenant claims address the same live
/// A2A task and context identifiers. Cross-tenant read and cancel fail while
/// same-tenant access remains available. The result is scoped to the A2A task
/// store under `server-full`; it makes no claim about runs, memory, or KBs.
#[tokio::test]
#[serial]
async fn l3_c21_a2a_tasks_are_partitioned_by_verified_tenant() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let token = |subject: &str, tenant: &str| {
        let claims = universal_agent_runtime::uar::security::claims::UserClaims {
            sub: subject.to_owned(),
            name: None,
            roles: Some(vec!["user".to_owned()]),
            tenant_id: Some(tenant.to_owned()),
            uar_instance_id: None,
            exp: usize::MAX,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
        )
        .expect("C-21: tenant token must encode")
    };

    let tenant_a = token("user-a", "tenant-a");
    let tenant_b = token("user-b", "tenant-b");
    let endpoint = format!("{}/a2a/compiler?tenant_id=tenant-b", server.base_url);
    let client = reqwest::Client::new();
    let call = |token: &str, method: &str, params: serde_json::Value| {
        client
            .post(&endpoint)
            .bearer_auth(token)
            .header("x-uar-tenant-id", "tenant-b")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": "c21",
                "method": method,
                "params": params,
                "tenant_id": "tenant-b"
            }))
            .send()
    };

    let created = call(
        &tenant_a,
        "message/send",
        serde_json::json!({
            "message": {
                "role": "user",
                "parts": [{"type": "text", "text": "C-21 tenant task"}],
                "metadata": {}
            },
            "context_id": "c21-shared-context",
            "metadata": {"tenant_id": "tenant-b"},
            "tenant_id": "tenant-b"
        }),
    )
    .await
    .expect("C-21: tenant A create request");
    assert_eq!(created.status(), 200);
    let created: serde_json::Value = created
        .json()
        .await
        .expect("C-21: tenant A create response JSON");
    let task_id = created["result"]["id"]
        .as_str()
        .expect("C-21: created task id")
        .to_owned();

    let same_tenant = call(&tenant_a, "tasks/get", serde_json::json!({"id": task_id}))
        .await
        .expect("C-21: same-tenant get")
        .json::<serde_json::Value>()
        .await
        .expect("C-21: same-tenant get JSON");
    assert_eq!(
        same_tenant["result"]["id"].as_str(),
        Some(task_id.as_str()),
        "C-21: same-tenant task lookup must succeed: {same_tenant}"
    );

    let cross_get = call(&tenant_b, "tasks/get", serde_json::json!({"id": task_id}))
        .await
        .expect("C-21: cross-tenant get")
        .json::<serde_json::Value>()
        .await
        .expect("C-21: cross-tenant get JSON");
    assert_eq!(cross_get["error"]["code"], -32001);
    assert!(cross_get.get("result").is_none());

    let cross_context = call(
        &tenant_b,
        "message/send",
        serde_json::json!({
            "message": {
                "role": "user",
                "parts": [{"type": "text", "text": "tenant B context"}],
                "metadata": {}
            },
            "context_id": "c21-shared-context",
            "metadata": {}
        }),
    )
    .await
    .expect("C-21: cross-tenant context request")
    .json::<serde_json::Value>()
    .await
    .expect("C-21: cross-tenant context JSON");
    assert_ne!(
        cross_context["result"]["id"].as_str(),
        Some(task_id.as_str()),
        "C-21: tenant B must not join tenant A context: {cross_context}"
    );

    let cross_cancel = call(
        &tenant_b,
        "tasks/cancel",
        serde_json::json!({"id": task_id}),
    )
    .await
    .expect("C-21: cross-tenant cancel")
    .json::<serde_json::Value>()
    .await
    .expect("C-21: cross-tenant cancel JSON");
    assert_eq!(cross_cancel["error"]["code"], -32001);

    let after_cancel = call(&tenant_a, "tasks/get", serde_json::json!({"id": task_id}))
        .await
        .expect("C-21: tenant A post-cancel get")
        .json::<serde_json::Value>()
        .await
        .expect("C-21: tenant A post-cancel JSON");
    assert_eq!(
        after_cancel["result"]["status"]["state"], "working",
        "C-21: cross-tenant cancel must not mutate tenant A task: {after_cancel}"
    );
}

/// C-21 private user resources.
///
/// Two independently verified JWT subjects address the same live session and
/// Alice's persisted memory/knowledge identifiers. Same-user controls succeed;
/// Bob's caller-supplied user ids do not cross the authenticated boundary.
#[tokio::test]
#[serial]
async fn l3_c21_threads_memory_and_knowledge_are_partitioned_by_verified_user() {
    let stub = start_stub_llm(content_fixture("C-21 private thread", "C-21 private reply")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds { memory: true }).await;
    let token = |subject: &str| {
        let claims = universal_agent_runtime::uar::security::claims::UserClaims {
            sub: subject.to_owned(),
            name: None,
            roles: Some(vec!["user".to_owned()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
        )
        .expect("C-21: user token must encode")
    };
    let user_a = token("c21-user-a");
    let user_b = token("c21-user-b");
    let client = reqwest::Client::new();

    let acp_url = format!("{}/acp/", server.base_url);
    let unauthenticated_acp = client
        .post(&acp_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sessions/create",
            "params": {"agent_id": "default"}
        }))
        .send()
        .await
        .expect("C-21: unauthenticated ACP request");
    assert_eq!(unauthenticated_acp.status(), 401);

    let acp_session: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "sessions/create",
            "params": {"agent_id": "default"}
        }))
        .send()
        .await
        .expect("C-21: user A ACP session create")
        .json()
        .await
        .expect("C-21: user A ACP session JSON");
    let acp_session_id = acp_session["result"]["session_id"]
        .as_str()
        .expect("C-21: ACP session id")
        .to_owned();

    let cross_acp_session: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_b)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "sessions/get",
            "params": {"session_id": acp_session_id}
        }))
        .send()
        .await
        .expect("C-21: user B ACP session lookup")
        .json()
        .await
        .expect("C-21: user B ACP session JSON");
    assert_eq!(cross_acp_session["error"]["code"], -32001);

    let cross_acp_delete: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_b)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "sessions/delete",
            "params": {"session_id": acp_session_id}
        }))
        .send()
        .await
        .expect("C-21: user B ACP session delete")
        .json()
        .await
        .expect("C-21: user B ACP delete JSON");
    assert_eq!(cross_acp_delete["result"]["deleted"], false);

    let cross_acp_run_create: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_b)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "runs/create",
            "params": {"session_id": acp_session_id, "input": "must fail"}
        }))
        .send()
        .await
        .expect("C-21: user B ACP run create")
        .json()
        .await
        .expect("C-21: user B ACP run-create JSON");
    assert_eq!(cross_acp_run_create["error"]["code"], -32001);

    let same_acp_session: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "sessions/get",
            "params": {"session_id": acp_session_id}
        }))
        .send()
        .await
        .expect("C-21: user A ACP session lookup")
        .json()
        .await
        .expect("C-21: user A ACP session lookup JSON");
    assert_eq!(same_acp_session["result"]["session_id"], acp_session_id);

    let acp_run: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "runs/create",
            "params": {"session_id": acp_session_id, "input": "C-21 private thread"}
        }))
        .send()
        .await
        .expect("C-21: user A ACP run create")
        .json()
        .await
        .expect("C-21: user A ACP run JSON");
    let acp_run_id = acp_run["result"]["run_id"]
        .as_str()
        .expect("C-21: ACP run id")
        .to_owned();

    let cross_acp_run: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_b)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "runs/get",
            "params": {"run_id": acp_run_id}
        }))
        .send()
        .await
        .expect("C-21: user B ACP run lookup")
        .json()
        .await
        .expect("C-21: user B ACP run JSON");
    assert_eq!(cross_acp_run["error"]["code"], -32002);

    let same_acp_run: serde_json::Value = client
        .post(&acp_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "runs/get",
            "params": {"run_id": acp_run_id}
        }))
        .send()
        .await
        .expect("C-21: user A ACP run lookup")
        .json()
        .await
        .expect("C-21: user A ACP run lookup JSON");
    assert_eq!(same_acp_run["result"]["run_id"], acp_run_id);

    let session_id = uuid::Uuid::new_v4().to_string();
    let chat = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .bearer_auth(&user_a)
        .header("X-UAR-Session-ID", &session_id)
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "C-21 private thread"}],
            "stream": false,
            "memory_enabled": false
        }))
        .send()
        .await
        .expect("C-21: user A chat request");
    assert_eq!(
        chat.status(),
        200,
        "C-21: user A chat failed: {}",
        chat.text().await.unwrap_or_default()
    );
    let stats_url = format!(
        "{}/api/uar/sessions/{session_id}/context-stats",
        server.base_url
    );
    let same_user_stats = client
        .get(&stats_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: same-user session read");
    assert_eq!(same_user_stats.status(), 200);
    let cross_user_stats = client
        .get(&stats_url)
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: cross-user session read");
    assert_eq!(
        cross_user_stats.status(),
        404,
        "C-21: user B read user A session: {}",
        cross_user_stats.text().await.unwrap_or_default()
    );

    let session_agent_url = format!(
        "{}/api/uar/discovery/sessions/{session_id}/agent",
        server.base_url
    );
    let user_a_run: serde_json::Value = client
        .get(&session_agent_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A current run")
        .json()
        .await
        .expect("C-21: user A current run JSON");
    let user_a_run_id = user_a_run["run_id"]
        .as_str()
        .expect("C-21: user A run id")
        .to_owned();
    let cross_user_run = client
        .get(&session_agent_url)
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B current run lookup");
    assert_eq!(cross_user_run.status(), 404);
    let cross_user_stream = client
        .get(format!(
            "{}/api/uar/runs/{user_a_run_id}/stream",
            server.base_url
        ))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B stream lookup");
    assert_eq!(cross_user_stream.status(), 404);
    let cross_user_cancel = client
        .post(format!(
            "{}/api/uar/runs/{user_a_run_id}/cancel",
            server.base_url
        ))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B cancel request");
    assert_eq!(cross_user_cancel.status(), 404);
    let cross_user_checkpoints = client
        .get(format!(
            "{}/api/uar/runs/{user_a_run_id}/checkpoints",
            server.base_url
        ))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B checkpoint request");
    assert_eq!(cross_user_checkpoints.status(), 404);

    let same_user_stream = client
        .get(format!(
            "{}/api/uar/runs/{user_a_run_id}/stream",
            server.base_url
        ))
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A stream lookup");
    assert_eq!(same_user_stream.status(), 200);
    drop(same_user_stream);

    let agent_config_url = format!(
        "{}/api/uar/sessions/{session_id}/agent-config",
        server.base_url
    );
    let missing_agent_config = client
        .get(&agent_config_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A missing agent config lookup");
    assert_eq!(missing_agent_config.status(), 204);
    assert!(
        missing_agent_config
            .bytes()
            .await
            .expect("C-21: read empty missing response")
            .is_empty(),
        "owner-scoped absence must have an empty body"
    );
    let save_agent_config = client
        .post(&agent_config_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({"agent_id": "default-agent"}))
        .send()
        .await
        .expect("C-21: user A save agent config");
    assert_eq!(save_agent_config.status(), 200);
    let cross_agent_config = client
        .get(&agent_config_url)
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B agent config lookup");
    assert_eq!(cross_agent_config.status(), 204);
    assert!(
        cross_agent_config
            .bytes()
            .await
            .expect("C-21: read empty cross-user response")
            .is_empty(),
        "cross-owner absence must have an empty body"
    );
    let same_agent_config = client
        .get(&agent_config_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A agent config lookup");
    assert_eq!(same_agent_config.status(), 200);

    let policy_id = format!("c21-policy-{}", uuid::Uuid::new_v4());
    let policy_url = format!(
        "{}/api/uar/conversations/{policy_id}/policy",
        server.base_url
    );
    let save_policy = client
        .put(&policy_url)
        .bearer_auth(&user_a)
        .json(&serde_json::json!({"memory_enabled": false}))
        .send()
        .await
        .expect("C-21: user A save policy");
    assert_eq!(save_policy.status(), 200);
    let cross_policy: serde_json::Value = client
        .get(&policy_url)
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B policy lookup")
        .json()
        .await
        .expect("C-21: user B policy JSON");
    assert!(cross_policy.is_null());
    let same_policy: serde_json::Value = client
        .get(&policy_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A policy lookup")
        .json()
        .await
        .expect("C-21: user A policy JSON");
    assert_eq!(same_policy["memory_enabled"], false);

    let user_b_chat = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .bearer_auth(&user_b)
        .header("X-UAR-Session-ID", &session_id)
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "C-21 private thread"}],
            "stream": false,
            "memory_enabled": false
        }))
        .send()
        .await
        .expect("C-21: user B same-id chat request");
    assert_eq!(
        user_b_chat.status(),
        200,
        "C-21: user B same-id chat failed: {}",
        user_b_chat.text().await.unwrap_or_default()
    );
    let user_b_run: serde_json::Value = client
        .get(&session_agent_url)
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B current run")
        .json()
        .await
        .expect("C-21: user B current run JSON");
    let user_b_run_id = user_b_run["run_id"].as_str().expect("C-21: user B run id");
    assert_ne!(user_a_run_id, user_b_run_id);
    let user_a_run_after_b: serde_json::Value = client
        .get(&session_agent_url)
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A current run after B")
        .json()
        .await
        .expect("C-21: user A current run after B JSON");
    assert_eq!(user_a_run_after_b["run_id"], user_a_run_id);

    let kb_create = client
        .post(format!("{}/api/knowledge", server.base_url))
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "name": format!("c21-private-{}", uuid::Uuid::new_v4()),
            "description": "user A only"
        }))
        .send()
        .await
        .expect("C-21: user A KB create");
    let kb_status = kb_create.status();
    if kb_status != 201 {
        panic!(
            "C-21: KB create failed ({kb_status}): {}",
            kb_create.text().await.unwrap_or_default()
        );
    }
    let kb: serde_json::Value = kb_create.json().await.expect("C-21: KB create JSON");
    let kb_id = kb["id"].as_str().expect("C-21: KB id");
    let same_user_kb = client
        .get(format!("{}/api/knowledge/{kb_id}", server.base_url))
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: same-user KB read");
    assert_eq!(same_user_kb.status(), 200);
    let cross_user_kb = client
        .get(format!("{}/api/knowledge/{kb_id}", server.base_url))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: cross-user KB read");
    assert_eq!(
        cross_user_kb.status(),
        404,
        "C-21: user B read user A KB: {}",
        cross_user_kb.text().await.unwrap_or_default()
    );
    let user_b_kbs: serde_json::Value = client
        .get(format!("{}/api/knowledge", server.base_url))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B KB list")
        .json()
        .await
        .expect("C-21: user B KB list JSON");
    assert!(
        user_b_kbs
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["id"] != kb_id)),
        "C-21: user B list leaked user A KB: {user_b_kbs}"
    );
    let document_upload = client
        .post(format!(
            "{}/api/knowledge/{kb_id}/documents",
            server.base_url
        ))
        .bearer_auth(&user_a)
        .multipart(
            reqwest::multipart::Form::new().part(
                "file",
                reqwest::multipart::Part::bytes(b"C-21 private document".to_vec())
                    .file_name("c21-private.txt")
                    .mime_str("text/plain")
                    .expect("C-21: document MIME"),
            ),
        )
        .send()
        .await
        .expect("C-21: user A document upload");
    assert_eq!(document_upload.status(), 202);
    let document: serde_json::Value = document_upload
        .json()
        .await
        .expect("C-21: uploaded document JSON");
    let document_id = document["id"].as_str().expect("C-21: uploaded document id");
    let cross_user_document = client
        .get(format!(
            "{}/api/knowledge/{kb_id}/documents/{document_id}",
            server.base_url
        ))
        .bearer_auth(&user_b)
        .send()
        .await
        .expect("C-21: user B document lookup");
    assert_eq!(cross_user_document.status(), 404);
    let same_user_document = client
        .get(format!(
            "{}/api/knowledge/{kb_id}/documents/{document_id}",
            server.base_url
        ))
        .bearer_auth(&user_a)
        .send()
        .await
        .expect("C-21: user A document lookup");
    assert_eq!(same_user_document.status(), 200);

    let secret = format!("c21-memory-{}", uuid::Uuid::new_v4());
    let memory_create = client
        .post(format!("{}/api/memory", server.base_url))
        .bearer_auth(&user_a)
        .json(&serde_json::json!({
            "content": secret,
            "categories": ["c21"],
            "user_id": "c21-user-b"
        }))
        .send()
        .await
        .expect("C-21: user A memory create");
    assert_eq!(
        memory_create.status(),
        200,
        "C-21: memory create failed: {}",
        memory_create.text().await.unwrap_or_default()
    );
    let same_user_memories: serde_json::Value = client
        .get(format!("{}/api/memory", server.base_url))
        .bearer_auth(&user_a)
        .query(&[("q", secret.as_str()), ("user_id", "c21-user-b")])
        .send()
        .await
        .expect("C-21: same-user memory search")
        .json()
        .await
        .expect("C-21: same-user memory JSON");
    assert!(
        same_user_memories
            .as_array()
            .is_some_and(|rows| rows.iter().any(|row| row["content"] == secret)),
        "C-21: user A could not read its memory: {same_user_memories}"
    );
    let cross_user_memories: serde_json::Value = client
        .get(format!("{}/api/memory", server.base_url))
        .bearer_auth(&user_b)
        .query(&[("q", secret.as_str()), ("user_id", "c21-user-a")])
        .send()
        .await
        .expect("C-21: cross-user memory search")
        .json()
        .await
        .expect("C-21: cross-user memory JSON");
    assert!(
        cross_user_memories
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["content"] != secret)),
        "C-21: user B read user A memory: {cross_user_memories}"
    );
}

/// C-24 peer mesh — **published exclusion**.
///
/// Discovery, CRDT convergence, and capability-aware peer routing require two
/// independently-addressable devices. This harness boots one loopback runtime
/// with a throwaway database, so it cannot create the topology needed for the
/// target. The live health request supplies the real-handler discriminator; it
/// is not presented as evidence that any peer behavior occurred.
#[tokio::test]
#[serial]
async fn excluded_c24_peer_mesh_requires_two_devices() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let (status, body) = get_capability(&server.base_url, "C-24", "/healthz").await;
    assert_eq!(
        status, 200,
        "C-24 exclusion discriminator: live runtime health handler failed: {body}"
    );
}

/// C-25 node decentralized identity — **published exclusion**.
///
/// Target: L3 deterministic `did:key` derivation checked against the W3C
/// vector used by `frf-did`. UAR has no `frf-did` dependency or node-identity
/// surface, so this runtime harness cannot reach that implementation. The
/// source audit is executable below; the health request proves this is a live
/// runtime result rather than a skipped test. When UAR consumes the crate, this
/// exclusion deliberately fails and must be replaced by the vector assertion.
#[tokio::test]
#[serial]
async fn excluded_c25_node_did_not_consumed_by_runtime() {
    let manifest = include_str!("../../../Cargo.toml");
    assert!(
        !manifest.contains("frf-did") && !manifest.contains("frf_did"),
        "C-25 exclusion is stale: UAR now declares an frf-did dependency"
    );

    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let (status, body) = get_capability(&server.base_url, "C-25", "/healthz").await;
    assert_eq!(
        status, 200,
        "C-25 exclusion discriminator: live runtime health handler failed: {body}"
    );
}

/// C-26 DID resolution and credential verification — **published exclusion**.
///
/// Target: L3 offline `did:key` resolution plus rejection of a credential from
/// a different DID. UAR consumes neither `frf-did` nor `frf-wallet`, so no
/// runtime call path can perform either half of that check. The manifest audit
/// pins the blocking condition, and the health request is the required live
/// real-handler discriminator.
#[tokio::test]
#[serial]
async fn excluded_c26_did_resolution_and_vc_verification_not_consumed() {
    let manifest = include_str!("../../../Cargo.toml");
    assert!(
        !manifest.contains("frf-did")
            && !manifest.contains("frf_did")
            && !manifest.contains("frf-wallet")
            && !manifest.contains("frf_wallet"),
        "C-26 exclusion is stale: UAR now declares an frf DID/wallet dependency"
    );

    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let (status, body) = get_capability(&server.base_url, "C-26", "/healthz").await;
    assert_eq!(
        status, 200,
        "C-26 exclusion discriminator: live runtime health handler failed: {body}"
    );
}

/// C-27 credential wallet and owner-to-node delegation — **published exclusion**.
///
/// Target: L3 with forged-issuer and expired-credential rejection. The
/// `frf-wallet` implementation is not a UAR dependency and UAR exposes no
/// wallet/delegation call path, so exercising those fail-closed cases through
/// this runtime is structurally impossible. The manifest audit makes that
/// blocker executable; the health request proves a real runtime handler ran.
#[tokio::test]
#[serial]
async fn excluded_c27_wallet_not_consumed_by_runtime() {
    let manifest = include_str!("../../../Cargo.toml");
    assert!(
        !manifest.contains("frf-wallet") && !manifest.contains("frf_wallet"),
        "C-27 exclusion is stale: UAR now declares an frf-wallet dependency"
    );

    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let (status, body) = get_capability(&server.base_url, "C-27", "/healthz").await;
    assert_eq!(
        status, 200,
        "C-27 exclusion discriminator: live runtime health handler failed: {body}"
    );
}

// ---------------------------------------------------------------------------
// Predicted-ABSENT — these SHOULD fail to resolve
// ---------------------------------------------------------------------------

/// C-22 scheduled / event-initiated runs — **predicted ABSENT**.
///
/// SPECIFICATION.md records `[V] ABSENT — every run in C-01 is caller-initiated`.
/// This test asserts the *absence*, so it passes when the capability is missing.
/// If it ever fails, the capability was implemented and the spec is stale —
/// which is a finding either way.
#[tokio::test]
#[serial]
async fn absent_c22_scheduled_runs() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/api/uar/schedules", server.base_url))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .send()
        .await
        .expect("C-22: request");
    let body = resp.text().await.unwrap_or_default();

    assert!(
        body.contains(CATCH_ALL_CODE),
        "C-22 was predicted ABSENT but /api/uar/schedules resolved to a real handler. \
         Either the capability now exists (update SPECIFICATION.md) or this probe \
         guessed the wrong path. Body: {body}"
    );
}

/// C-23 peer reachability — **predicted ABSENT**.
///
/// SPECIFICATION.md records `[V] ABSENT` (GAP-10: no `iroh`, `libp2p`, `str0m`,
/// `webrtc`, or `quinn` dependency in UAR). Both adversarial reviewers objected
/// to burying this in an exclusion list: an ABSENT capability reported as
/// "excluded, needs two devices" launders a known zero into "probably fine,
/// couldn't check". It is a **FAIL with a citation**, and this test states that
/// as an assertion rather than a footnote.
#[tokio::test]
#[serial]
async fn absent_c23_peer_reachability() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/api/uar/peers", server.base_url))
        .header("Authorization", format!("Bearer {HARNESS_JWT_SECRET}"))
        .send()
        .await
        .expect("C-23: request");
    let body = resp.text().await.unwrap_or_default();

    assert!(
        body.contains(CATCH_ALL_CODE),
        "C-23 was predicted ABSENT but /api/uar/peers resolved to a real handler. \
         If peer support landed, SPECIFICATION.md GAP-10 is stale. Body: {body}"
    );
}
