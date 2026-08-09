//! `/metrics` endpoint cases for the live integration tier.
//!
//! These cases exist because `/metrics` is registered unconditionally in
//! `start_server` (`src/server.rs`), but the Prometheus recorder used to be
//! initialised only from the two binaries (`src/main.rs`,
//! `src/bin/uar-sidecar.rs`). Any host that embedded UAR via `start_server` —
//! including SDK consumers going through `sdks/rust/src/runtime.rs`'s
//! re-export — therefore served a `/metrics` route whose handler panicked its
//! request thread on the first scrape.
//!
//! Booting through `boot_test_server` reproduces the embedder's situation
//! exactly: it calls the real `start_server` and never calls
//! `metrics::init()`, which is precisely what the binaries do and embedders
//! do not.

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureSet, start_stub_llm};
use serial_test::serial;

const MODEL: &str = "gpt-5.4-mini";

/// `/metrics` must serve Prometheus exposition format for an embedded host
/// that never called `metrics::init()`, rather than panicking its request
/// thread.
#[tokio::test]
#[serial]
async fn metrics_endpoint_serves_without_explicit_init() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/metrics", server.base_url))
        .send()
        .await
        .expect("request to /metrics failed — the handler panicked its request thread");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "/metrics should return 200 on an embedded host"
    );

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("text/plain"),
        "expected Prometheus text exposition content type, got {content_type:?}"
    );

    // Body is served rather than the connection being torn down. A freshly
    // booted server may not have recorded any series yet, so assert on the
    // response being intact rather than on specific metric names.
    resp.text().await.expect("failed to read /metrics body");
}

/// The recorder backing `/metrics` must be the *global* one the `counter!` /
/// `gauge!` macros write to. A lazy fallback that built a detached recorder
/// would stop the panic while rendering an empty page forever — a silent
/// failure strictly worse than the crash. Serving a request increments
/// `uar_requests_total`, so that series must appear in a subsequent scrape.
#[tokio::test]
#[serial]
async fn metrics_endpoint_reflects_recorded_series() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();

    // Drive at least one request through the metrics-recording middleware.
    let _ = client
        .get(format!("{}/health", server.base_url))
        .send()
        .await
        .expect("health request failed");

    let body = client
        .get(format!("{}/metrics", server.base_url))
        .send()
        .await
        .expect("request to /metrics failed")
        .text()
        .await
        .expect("failed to read /metrics body");

    assert!(
        body.contains("uar_requests_total"),
        "expected the global recorder's series in /metrics output; a detached \
         recorder would render an empty page. Body was:\n{body}"
    );
}
