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
//! - **No L4.** The harness uses a fresh temp SurrealKV path per boot and
//!   `start_server` has no shutdown hook, so no write-then-reboot-then-read
//!   cycle is possible. Capabilities whose defining property is persistence
//!   (C-12, C-13) can only be shape-verified — which does not establish that
//!   they persist anything.
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
//! # The catch-all discriminator
//!
//! `server.rs:1093` routes `/api/{*path}` to `api_route_not_found`, so an
//! unmounted `/api/*` path still produces a well-formed JSON response. Checking
//! only the status code would confuse "route absent" with "route present and
//! rejecting". Every case therefore calls [`assert_real_handler`], which fails
//! on the sentinel `code: "api_route_not_found"` regardless of status.

use super::harness::{HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server};
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
// Shape-only — the defining property is NOT verifiable here
// ---------------------------------------------------------------------------

/// C-12 persistence — **shape only. This does NOT establish persistence.**
///
/// The harness gives each boot a fresh temp SurrealKV path and `start_server`
/// exposes no shutdown hook, so a write-then-reboot-then-read cycle cannot be
/// performed. A pass here means the endpoint answered; it does not mean
/// anything was durably stored. Reported as **L4 unverifiable**.
#[tokio::test]
#[serial]
async fn shape_only_c12_persistence_config() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-12", "/api/config/persistence").await;
    assert_eq!(status, 200, "C-12: expected 200, got: {body}");
    assert!(
        body.contains("provider") || body.contains("surreal"),
        "C-12: expected persistence info, got: {body}"
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

    let (status, body) = get_capability(&server.base_url, "C-04", "/api/uar/credentials").await;
    assert_eq!(status, 200, "C-04: expected 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-04: body is not JSON: {e}\n{body}"));
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

/// C-13 sessions and threads — **shape only, same L4 limit as C-12**.
///
/// Caller-supplied thread IDs are the capability's point, but proving a thread
/// *persists* needs a write→reboot→read cycle the harness cannot perform.
#[tokio::test]
#[serial]
async fn shape_only_c13_sessions() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let (status, body) = get_capability(&server.base_url, "C-13", "/api/sessions").await;
    assert_eq!(status, 200, "C-13: expected 200, got: {body}");
    serde_json::from_str::<serde_json::Value>(&body)
        .unwrap_or_else(|e| panic!("C-13: body is not JSON: {e}\n{body}"));
}

// ---------------------------------------------------------------------------
// NOT MEASURABLE BY THIS INSTRUMENT — no HTTP surface exists
//
// C-16 governance, C-18 file processing, C-19 evals have **zero registered
// routes** (verified: no `.route()` anywhere matches governance/file_processing/
// eval). SPECIFICATION.md §3.1 classifies them as internal libraries whose user
// decisions are made through settings keys rather than dedicated endpoints.
//
// There is deliberately no test here. Writing one would mean inventing a path,
// watching the `/api/{*path}` catch-all answer, and recording an ABSENT verdict
// for a capability that was never meant to have an endpoint — manufacturing a
// finding. They are reported in the "not measurable" table instead.
//
// C-21 tenant isolation is also absent from this file: it is a security
// property requiring two tenants and a cross-read attempt, which `#[serial]`
// single-tenant cases structurally cannot express.
// ---------------------------------------------------------------------------

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
