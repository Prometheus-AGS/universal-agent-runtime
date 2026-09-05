//! Primary HTTP replay over a real UAR and a controlled streaming provider.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use axum::{
    Router,
    extract::State,
    response::{Sse, sse::Event},
    routing::post,
};
use futures::StreamExt;
use serde_json::json;
use serial_test::serial;
use tokio::sync::Semaphore;

use super::harness::{HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server, mint_harness_peer_token};

struct ProviderState {
    calls: AtomicUsize,
    release: Semaphore,
    burst: AtomicUsize,
}

async fn provider_chat(
    State(state): State<Arc<ProviderState>>,
) -> impl axum::response::IntoResponse {
    state.calls.fetch_add(1, Ordering::SeqCst);
    let burst = state.burst.load(Ordering::SeqCst);
    Sse::new(async_stream::stream! {
        for index in 0..burst {
            // Pace the producer so this tests history eviction, not receiver lag.
            tokio::time::sleep(Duration::from_millis(2)).await;
            yield Ok::<_, std::convert::Infallible>(Event::default().data(json!({
                "id":"chatcmpl-replay-provider", "object":"chat.completion.chunk", "created":0,
                "model":"gpt-5.4-mini", "choices":[{"index":0,
                    "delta":{"content":format!("burst-{index}")}, "finish_reason":null}]
            }).to_string()));
        }
        for text in ["BEFORE", "AFTER"] {
            if text == "AFTER" {
                state.release.acquire().await.unwrap().forget();
            }
            yield Ok::<_, std::convert::Infallible>(Event::default().data(json!({
                "id":"chatcmpl-replay-provider", "object":"chat.completion.chunk", "created":0,
                "model":"gpt-5.4-mini", "choices":[{"index":0,
                    "delta":{"content":text}, "finish_reason":null}]
            }).to_string()));
        }
        yield Ok(Event::default().data(json!({
            "id":"chatcmpl-replay-provider", "object":"chat.completion.chunk", "created":0,
            "model":"gpt-5.4-mini", "choices":[{"index":0,"delta":{},"finish_reason":"stop"}]
        }).to_string()));
        yield Ok(Event::default().data("[DONE]"));
    })
}

fn frame_id(frame: &str) -> Option<&str> {
    frame
        .lines()
        .find_map(|line| line.strip_prefix("id:").map(str::trim))
}

fn cursor_position(cursor: &str) -> (u64, usize, u8) {
    let fields = cursor.split(':').collect::<Vec<_>>();
    assert_eq!(fields.len(), 3);
    (
        fields[0].parse().unwrap(),
        fields[1].parse().unwrap(),
        fields[2].parse().unwrap(),
    )
}

fn foreign_tenant_token() -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &json!({"sub":"live-harness-peer", "tenant_id":"00000000-0000-4000-8000-000000000001", "exp":usize::MAX}),
        &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
    ).unwrap()
}

#[tokio::test]
#[serial]
async fn primary_chat_reconnects_mid_run_without_repeating_execution_or_frames() {
    let provider = Arc::new(ProviderState {
        calls: AtomicUsize::new(0),
        release: Semaphore::new(0),
        burst: AtomicUsize::new(0),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let provider_url = format!("http://{}/v1", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/chat/completions", post(provider_chat))
        .with_state(provider.clone());
    let shutdown = tokio_util::sync::CancellationToken::new();
    let provider_shutdown = shutdown.clone();
    let provider_job = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(provider_shutdown.cancelled_owned())
            .await
            .unwrap();
    });
    let server = boot_test_server(&provider_url, "gpt-5.4-mini", ServiceNeeds::default()).await;
    let client = reqwest::Client::new();
    let token = mint_harness_peer_token();
    let endpoint = format!("{}/api/chat/completion", server.base_url);

    for (index, mode) in ["openai", "agui", "dual", "agui_spec"]
        .into_iter()
        .enumerate()
    {
        let original = client
            .post(&endpoint)
            .bearer_auth(&token)
            .json(&json!({
                "model":"gpt-5.4-mini", "message":"stream two chunks",
                "stream":true, "stream_mode":mode, "memory_enabled":false,
            }))
            .send()
            .await
            .unwrap();
        assert!(original.status().is_success(), "{}", original.status());
        let run_id = original.headers()["x-uar-run-id"]
            .to_str()
            .unwrap()
            .to_owned();
        let mut original_stream = original.bytes_stream();
        let cursor = tokio::time::timeout(Duration::from_secs(10), async {
            let mut pending = String::new();
            loop {
                let bytes = original_stream
                    .next()
                    .await
                    .expect("stream remains live")
                    .unwrap();
                pending.push_str(std::str::from_utf8(&bytes).unwrap());
                while let Some(end) = pending.find("\n\n") {
                    let frame = pending[..end].to_owned();
                    pending.drain(..end + 2);
                    // Stop within a multi-frame source event in spec mode.
                    // Other modes acknowledge the first rendered text frame.
                    if (mode == "agui_spec" && frame.contains("event: TEXT_MESSAGE_START"))
                        || (mode != "agui_spec" && frame.contains("BEFORE"))
                    {
                        return frame_id(&frame)
                            .expect("every rendered frame has a cursor")
                            .to_owned();
                    }
                }
            }
        })
        .await
        .expect("first provider text arrives");
        assert_eq!(provider.calls.load(Ordering::SeqCst), index + 1);

        let foreign = client
            .post(&endpoint)
            .bearer_auth(foreign_tenant_token())
            .header("x-uar-run-id", &run_id)
            .header("Last-Event-ID", &cursor)
            .json(&json!({"stream":true,"stream_mode":mode}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            foreign.status(),
            reqwest::StatusCode::NOT_FOUND,
            "tenant must be part of ownership"
        );
        let wrong_format = client
            .post(&endpoint)
            .bearer_auth(&token)
            .header("x-uar-run-id", &run_id)
            .header("Last-Event-ID", &cursor)
            .json(
                &json!({"stream":true,"stream_mode":if mode == "openai" {"agui"} else {"openai"}}),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(wrong_format.status(), reqwest::StatusCode::BAD_REQUEST);

        for invalid in ["not-a-cursor", "18446744073709551615"] {
            let rejected = client
                .post(&endpoint)
                .bearer_auth(&token)
                .header("x-uar-run-id", &run_id)
                .header("Last-Event-ID", invalid)
                .json(&json!({"stream":true,"stream_mode":mode}))
                .send()
                .await
                .unwrap();
            assert_eq!(rejected.status(), reqwest::StatusCode::BAD_REQUEST);
        }
        let missing_cursor = client
            .post(&endpoint)
            .bearer_auth(&token)
            .header("x-uar-run-id", &run_id)
            .json(&json!({"stream":true,"stream_mode":mode}))
            .send()
            .await
            .unwrap();
        assert_eq!(missing_cursor.status(), reqwest::StatusCode::BAD_REQUEST);
        assert_eq!(
            provider.calls.load(Ordering::SeqCst),
            index + 1,
            "rejected replay requests must not start a model call"
        );

        // Keep the first subscriber until the second response is attached, so
        // the unchanged last-viewer cancellation grace cannot race this proof.
        let replay = client
            .post(format!("{}/v1/chat/completions", server.base_url))
            .bearer_auth(&token)
            .header("x-uar-run-id", &run_id)
            .header("Last-Event-ID", &cursor)
            .json(&json!({"stream":true,"stream_mode":mode,"message":"must never execute again"}))
            .send()
            .await
            .unwrap();
        assert!(replay.status().is_success(), "{}", replay.status());
        assert_eq!(replay.headers()["x-uar-run-id"].to_str().unwrap(), run_id);
        assert_eq!(provider.calls.load(Ordering::SeqCst), index + 1);
        drop(original_stream);
        provider.release.add_permits(1);
        let body = tokio::time::timeout(Duration::from_secs(10), replay.text())
            .await
            .unwrap()
            .unwrap();
        let frames = body
            .split("\n\n")
            .filter(|frame| frame_id(frame).is_some())
            .collect::<Vec<_>>();
        let ids = frames
            .iter()
            .map(|frame| frame_id(frame).unwrap())
            .collect::<Vec<_>>();
        assert!(!ids.is_empty());
        let acknowledged = cursor_position(&cursor);
        let mut previous = (acknowledged.0, acknowledged.1);
        for id in &ids {
            let position = cursor_position(id);
            assert_eq!(position.2, acknowledged.2);
            assert!(
                (position.0, position.1) > previous,
                "replay must advance strictly: {id}"
            );
            previous = (position.0, position.1);
        }
        assert!(
            !ids.contains(&cursor.as_str()),
            "acknowledged frame must not replay"
        );
        assert_eq!(
            ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
            ids.len()
        );
        let text_frames = frames
            .iter()
            .filter(|frame| frame.contains("AFTER"))
            .count();
        assert_eq!(text_frames, if mode == "dual" { 2 } else { 1 });
        assert_eq!(
            frames
                .iter()
                .filter(|frame| frame.contains("BEFORE"))
                .count(),
            if mode == "agui_spec" || mode == "dual" {
                1
            } else {
                0
            }
        );
        assert!(
            !body.contains("event: TEXT_MESSAGE_START"),
            "already acknowledged lifecycle must not restart"
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), index + 1);

        let terminal_cursor = ids.last().unwrap();
        let terminal = client
            .post(&endpoint)
            .bearer_auth(&token)
            .header("x-uar-run-id", &run_id)
            .header("Last-Event-ID", *terminal_cursor)
            .json(&json!({"stream":true,"stream_mode":mode}))
            .send()
            .await
            .unwrap();
        assert!(terminal.status().is_success());
        let terminal_body = tokio::time::timeout(Duration::from_secs(3), terminal.text())
            .await
            .unwrap()
            .unwrap();
        assert!(
            terminal_body.trim().is_empty(),
            "terminal cursor must close without replay: {terminal_body}"
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), index + 1);
        let numeric = terminal_cursor.split(':').next().unwrap();
        let legacy = client
            .post(&endpoint)
            .bearer_auth(&token)
            .header("x-uar-run-id", &run_id)
            .header("Last-Event-ID", numeric)
            .json(&json!({"stream":true,"stream_mode":mode}))
            .send()
            .await
            .unwrap();
        assert!(legacy.status().is_success());
        assert!(
            tokio::time::timeout(Duration::from_secs(3), legacy.text())
                .await
                .unwrap()
                .unwrap()
                .trim()
                .is_empty()
        );
    }

    // The cursor itself remains in the 512-event window, but its projection
    // prefix has gone. This must return 410, not restart message lifecycles.
    provider.burst.store(520, Ordering::SeqCst);
    let original = client.post(&endpoint).bearer_auth(&token)
        .json(&json!({"message":"fill replay history","stream":true,"stream_mode":"agui_spec","memory_enabled":false}))
        .send().await.unwrap();
    assert!(original.status().is_success());
    let run_id = original.headers()["x-uar-run-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let mut stream = original.bytes_stream();
    let retained_cursor = tokio::time::timeout(Duration::from_secs(15), async {
        let mut pending = String::new();
        loop {
            pending.push_str(std::str::from_utf8(&stream.next().await.unwrap().unwrap()).unwrap());
            while let Some(end) = pending.find("\n\n") {
                let frame = pending[..end].to_owned();
                pending.drain(..end + 2);
                if frame.contains("BEFORE") {
                    return frame_id(&frame).unwrap().to_owned();
                }
            }
        }
    })
    .await
    .unwrap();
    assert!(cursor_position(&retained_cursor).0 > 512);
    let expired = client
        .post(&endpoint)
        .bearer_auth(&token)
        .header("x-uar-run-id", &run_id)
        .header("Last-Event-ID", retained_cursor)
        .json(&json!({"stream":true,"stream_mode":"agui_spec"}))
        .send()
        .await
        .unwrap();
    assert_eq!(expired.status(), reqwest::StatusCode::GONE);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 5);
    provider.release.add_permits(1);
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(bytes) = stream.next().await {
            bytes.unwrap();
        }
    })
    .await
    .unwrap();
    server.trigger_shutdown();
    server.wait_for_exit().await;
    shutdown.cancel();
    provider_job.await.unwrap();
}
