//! CH-20: concurrent-agent load test.
//!
//! Boots one real server (`harness::boot_test_server`) against the
//! in-process stub LLM, then fires many concurrent chat-completion requests
//! at it — simulating N agents/conversations running at once — and reports
//! latency/throughput. This exercises the real production request path (the
//! same one the baseline feature cases use), not a synthetic microbenchmark.
//!
//! Deliberately a soak/smoke test, not a strict perf-regression gate: CI
//! runner hardware varies too much for a hard latency SLA to be meaningful
//! here (that's what `benches/hot_path.rs`'s Criterion benchmarks are for,
//! on functions cheap enough to run thousands of iterations locally). What
//! this test guarantees is the one thing that matters at this level: N
//! concurrent requests against the real Axum app + orchestrator + stub
//! driver all complete successfully with no deadlock, panic, or dropped
//! connection — and it prints real numbers for a human to judge.

use std::time::{Duration, Instant};

use serial_test::serial;

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};

const MODEL: &str = "gpt-5.4-mini";
const CONCURRENCY: usize = 50;

#[tokio::test]
#[serial]
async fn concurrent_agents_load_test() {
    let mut fixtures = FixtureSet::new();
    for i in 0..CONCURRENCY {
        fixtures = fixtures.with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: format!("load test message {i}"),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content(format!("load test response {i}")),
        );
    }
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let base_url = server.base_url.clone();

    let client = reqwest::Client::new();
    let overall_start = Instant::now();

    let mut handles = Vec::with_capacity(CONCURRENCY);
    for i in 0..CONCURRENCY {
        let client = client.clone();
        let base_url = base_url.clone();
        handles.push(tokio::spawn(async move {
            let start = Instant::now();
            let resp = client
                .post(format!("{base_url}/api/chat/completion"))
                .json(&serde_json::json!({
                    "model": MODEL,
                    "messages": [{"role": "user", "content": format!("load test message {i}")}],
                    "stream": false,
                }))
                .send()
                .await;
            let elapsed = start.elapsed();
            match resp {
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    (
                        elapsed,
                        status.is_success() && body.contains(&format!("load test response {i}")),
                    )
                }
                Err(_) => (elapsed, false),
            }
        }));
    }

    let mut latencies: Vec<Duration> = Vec::with_capacity(CONCURRENCY);
    let mut failures = 0usize;
    for handle in handles {
        let (elapsed, ok) = handle.await.expect("task should not panic");
        latencies.push(elapsed);
        if !ok {
            failures += 1;
        }
    }
    let total_elapsed = overall_start.elapsed();

    latencies.sort();
    let n = latencies.len();
    let p50 = latencies[n / 2];
    let p95 = latencies[((n * 95) / 100).min(n - 1)];
    let max = latencies[n - 1];
    let mean: Duration = latencies.iter().sum::<Duration>() / n as u32;
    let throughput_rps = CONCURRENCY as f64 / total_elapsed.as_secs_f64();

    println!(
        "concurrent_agents_load_test: {CONCURRENCY} concurrent requests in {total_elapsed:?} \
         ({throughput_rps:.1} req/s) — latency mean={mean:?} p50={p50:?} p95={p95:?} max={max:?}, \
         failures={failures}/{CONCURRENCY}"
    );

    assert_eq!(
        failures, 0,
        "{failures}/{CONCURRENCY} concurrent requests failed or returned the wrong fixture response"
    );
}
