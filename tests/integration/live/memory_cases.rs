//! Live cases for the memory subsystem.
//!
//! These are the only cases that boot with `ServiceNeeds { memory: true }`.
//! Until they existed, `ServiceNeeds.memory` was dead code — every call site
//! passed `ServiceNeeds::default()` — so the memory boot path was never
//! exercised and three independent defects shipped together:
//!
//!   1. `surreal-memory` was built without `local-embeddings`, making
//!      `embedding_provider: "local"` a hard `bail!` inside
//!      `MemoryService::new`.
//!   2. The harness wrote a scheme-qualified `db_path`, which
//!      `MemoryService::new` prefixed a second time.
//!   3. `mcp.json`'s `${UAR_MEMORY_MCP_URL:-…}` never expanded.
//!
//! Each of those surfaced identically to an operator: the service was left as
//! `None` and every memory endpoint answered `503 Memory system not enabled`,
//! which reads as "you didn't turn it on" rather than "it failed to start".

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureSet, start_stub_llm};
use serial_test::serial;

/// Model name as it appears on the wire to the stub — bare, no `provider/`
/// prefix; see harness.rs's discovery note on why.
const MODEL: &str = "gpt-5.4-mini";

/// A server booted with `memory.enabled: true` must actually serve memory,
/// not report itself disabled.
///
/// This asserts the *specific* 503 body cannot come back, because that is the
/// exact symptom every one of the defects above produced.
#[tokio::test]
#[serial]
async fn memory_stats_is_served_when_memory_is_enabled() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds { memory: true }).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/api/admin/memories/stats", server.base_url))
        .send()
        .await
        .expect("request");

    let status = resp.status();
    let body = resp.text().await.expect("body");

    assert_ne!(
        status,
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "memory.enabled: true but the service was left as None — \
         MemoryService::new failed at startup. Body: {body}"
    );
    assert!(
        status.is_success(),
        "expected memory stats, got {status}: {body}"
    );

    let stats: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("stats should be JSON ({e}), got: {body}"));
    assert!(
        stats.get("total").is_some_and(serde_json::Value::is_number),
        "stats should carry a numeric `total`, got: {body}"
    );
    assert!(
        stats.get("by_scope").is_some(),
        "stats should carry a `by_scope` breakdown, got: {body}"
    );
}
