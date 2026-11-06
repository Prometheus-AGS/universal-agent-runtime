//! The OpenAI-compatible embedding backend splits inputs into `batch_size`
//! requests. DashScope text-embedding-v4, for one, rejects more than 10 inputs
//! per request. Exercised at the real HTTP boundary with wiremock.

use universal_agent_runtime::uar::rag::embeddings::openai::OpenAiEmbeddingBackend;
use universal_agent_runtime::uar::rag::embeddings::{EmbeddingBackend, EmbeddingConfig};
/// Echoes one embedding per input, encoding the input's number so the test
/// can check order, and records each request's batch size.
struct EchoEmbeddings;

impl wiremock::Respond for EchoEmbeddings {
    fn respond(&self, request: &wiremock::Request) -> wiremock::ResponseTemplate {
        let body: serde_json::Value = serde_json::from_slice(&request.body).expect("json request");
        let inputs = body["input"].as_array().expect("input array");
        let data: Vec<serde_json::Value> = inputs
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let n: f32 = text
                    .as_str()
                    .and_then(|t| t.strip_prefix("text-"))
                    .and_then(|n| n.parse().ok())
                    .expect("text-N input");
                serde_json::json!({ "index": i, "embedding": [n] })
            })
            .rev() // providers may return items out of order
            .collect();
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": data }))
    }
}

#[tokio::test]
async fn inputs_are_sent_in_batches_of_batch_size_and_returned_in_order() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/embeddings"))
        .respond_with(EchoEmbeddings)
        .expect(3)
        .mount(&server)
        .await;

    let cfg = EmbeddingConfig {
        backend: "openai".to_string(),
        api_key: Some("sk-test".to_string()),
        base_url: Some(server.uri()),
        vector_dimension: 1,
        batch_size: 10,
        ..EmbeddingConfig::default()
    };
    let backend = OpenAiEmbeddingBackend::new(&cfg).expect("build backend");
    let owned: Vec<String> = (0..25).map(|n| format!("text-{n}")).collect();
    let texts: Vec<&str> = owned.iter().map(String::as_str).collect();

    let out = backend.embed(&texts).await.expect("embed");

    let got: Vec<f32> = out.iter().map(|e| e[0]).collect();
    let want: Vec<f32> = (0..25).map(|n| n as f32).collect();
    assert_eq!(got, want);
    let sizes: Vec<usize> = server
        .received_requests()
        .await
        .expect("recorded requests")
        .iter()
        .map(|r| {
            let v: serde_json::Value = serde_json::from_slice(&r.body).expect("json");
            v["input"].as_array().expect("input").len()
        })
        .collect();
    assert_eq!(sizes, vec![10, 10, 5]);
}

fn configured_backend(server: &wiremock::MockServer, batch_size: usize) -> OpenAiEmbeddingBackend {
    OpenAiEmbeddingBackend::new(&EmbeddingConfig {
        backend: "openai".into(),
        api_key: Some("owned-synthetic-key".into()),
        base_url: Some(server.uri()),
        vector_dimension: 1,
        batch_size,
        ..EmbeddingConfig::default()
    })
    .expect("configured owned peer")
}

#[tokio::test]
async fn usage_counters_require_every_chunk_independently() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let server = wiremock::MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            let usage = match call {
                0 => serde_json::json!({"prompt_tokens": 7, "total_tokens": 9}),
                1 => serde_json::json!({"total_tokens": 11}),
                _ => serde_json::json!({"prompt_tokens": 13, "total_tokens": 17}),
            };
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"index": 0, "embedding": [1.0]}], "usage": usage
            }))
        })
        .expect(3)
        .mount(&server)
        .await;
    let batch = configured_backend(&server, 1)
        .embed_with_usage(&["a", "b", "c"])
        .await
        .expect("batch");
    assert_eq!(batch.vectors.len(), 3);
    assert_eq!(batch.usage.prompt_tokens, None);
    assert_eq!(batch.usage.total_tokens, Some(37));
}

#[tokio::test]
async fn provider_zero_absence_and_overflow_are_distinct() {
    for (usage, expected_prompt, expected_total) in [
        (
            serde_json::json!({"prompt_tokens": 3, "total_tokens": 5}),
            Some(6),
            Some(10),
        ),
        (
            serde_json::json!({"prompt_tokens": 0, "total_tokens": 0}),
            Some(0),
            Some(0),
        ),
        (serde_json::Value::Null, None, None),
        (
            serde_json::json!({"prompt_tokens": u64::MAX, "total_tokens": 2}),
            None,
            Some(4),
        ),
        (serde_json::json!({"prompt_tokens": 3}), Some(6), None),
    ] {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [{"index": 0, "embedding": [1.0]}], "usage": usage
                })),
            )
            .expect(2)
            .mount(&server)
            .await;
        let batch = configured_backend(&server, 1)
            .embed_with_usage(&["a", "b"])
            .await
            .expect("batch");
        assert_eq!(batch.usage.prompt_tokens, expected_prompt);
        assert_eq!(batch.usage.total_tokens, expected_total);
    }
}

#[tokio::test]
async fn pinned_backend_keeps_vectors_metadata_and_usage_across_a_swap() {
    use std::sync::Arc;
    use universal_agent_runtime::uar::rag::embeddings::swappable::SwappableEmbeddingBackend;
    let first = wiremock::MockServer::start().await;
    let second = wiremock::MockServer::start().await;
    for (server, value, tokens) in [(&first, 1.0, 7), (&second, 2.0, 11)] {
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [{"index": 0, "embedding": [value]}],
                    "usage": {"prompt_tokens": tokens, "total_tokens": tokens}
                })),
            )
            .expect(1)
            .mount(server)
            .await;
    }
    let backend = SwappableEmbeddingBackend::new(Arc::new(configured_backend(&first, 1)));
    let pinned = backend.pin();
    backend.swap(Arc::new(configured_backend(&second, 1)));
    let old = pinned
        .embed_with_usage(&["owned"])
        .await
        .expect("pinned first backend");
    let next = backend
        .pin()
        .embed_with_usage(&["owned"])
        .await
        .expect("next backend");
    assert_eq!(pinned.endpoint(), Some(first.uri().as_str()));
    assert_eq!(old.vectors, vec![vec![1.0]]);
    assert_eq!(old.usage.prompt_tokens, Some(7));
    assert_eq!(next.vectors, vec![vec![2.0]]);
    assert_eq!(next.usage.prompt_tokens, Some(11));
}

#[tokio::test]
async fn malformed_indices_usage_and_dimensions_refuse_the_whole_batch() {
    let sentinel = "synthetic-provider-secret-must-not-escape";
    for body in [
        serde_json::json!({"data": [{"index": 0, "embedding": [1.0]}, {"index": 0, "embedding": [2.0]}]}),
        serde_json::json!({"data": [{"embedding": [1.0]}, {"index": 1, "embedding": [2.0]}]}),
        serde_json::json!({"data": [{"index": 0, "embedding": [1.0]}, {"index": 2, "embedding": [2.0]}]}),
        serde_json::json!({"data": [{"index": 0, "embedding": [1.0]}]}),
        serde_json::json!({"data": [{"index": 0, "embedding": [1.0, 2.0]}, {"index": 1, "embedding": [2.0]}]}),
        serde_json::json!({"data": [{"index": 0, "embedding": [1.0]}, {"index": 1, "embedding": [2.0]}], "usage": {"prompt_tokens": sentinel}}),
    ] {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let error = configured_backend(&server, 2)
            .embed_with_usage(&["a", "b"])
            .await
            .expect_err("malformed refusal");
        assert!(!error.to_string().contains(sentinel));
    }
}

#[tokio::test]
async fn provider_failure_body_and_url_never_enter_backend_error() {
    let server = wiremock::MockServer::start().await;
    let sentinel = "synthetic-provider-credential-body";
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(wiremock::ResponseTemplate::new(401).set_body_string(sentinel))
        .expect(1)
        .mount(&server)
        .await;
    let error = configured_backend(&server, 1)
        .embed_with_usage(&["a"])
        .await
        .expect_err("provider refuses");
    let message = error.to_string();
    assert!(message.contains("401"));
    assert!(!message.contains(sentinel));
    assert!(!message.contains(&server.uri()));
}

#[tokio::test]
async fn voyage_preserves_only_reported_total_and_sorts_complete_indices() {
    use universal_agent_runtime::uar::rag::embeddings::voyage::VoyageEmbeddingBackend;
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"index": 1, "embedding": [2.0]}, {"index": 0, "embedding": [1.0]}],
                "usage": {"total_tokens": 13}
            })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let backend = VoyageEmbeddingBackend::new(&EmbeddingConfig {
        backend: "voyage".into(),
        api_key: Some("owned-synthetic-key".into()),
        base_url: Some(server.uri()),
        vector_dimension: 1,
        ..EmbeddingConfig::default()
    })
    .expect("owned voyage peer");
    let batch = backend
        .embed_with_usage(&["a", "b"])
        .await
        .expect("voyage batch");
    assert_eq!(batch.vectors, vec![vec![1.0], vec![2.0]]);
    assert_eq!(batch.usage.total_tokens, Some(13));
    assert_eq!(batch.usage.prompt_tokens, None);
}

#[tokio::test]
async fn cohere_native_billing_units_are_not_invented_processed_counts() {
    use universal_agent_runtime::uar::rag::embeddings::cohere::CohereEmbeddingBackend;
    for (meta, expected) in [
        (
            serde_json::json!({"billed_units": {"input_tokens": 17}}),
            Some(17),
        ),
        (
            serde_json::json!({"billed_units": {"input_tokens": 0}}),
            Some(0),
        ),
        (serde_json::Value::Null, None),
    ] {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "embeddings": {"float": [[1.0]]}, "meta": meta
                })),
            )
            .expect(1)
            .mount(&server)
            .await;
        let backend = CohereEmbeddingBackend::new(&EmbeddingConfig {
            backend: "cohere".into(),
            api_key: Some("owned-synthetic-key".into()),
            base_url: Some(server.uri()),
            vector_dimension: 1,
            ..EmbeddingConfig::default()
        })
        .expect("owned cohere peer");
        let batch = backend
            .embed_with_usage(&["a"])
            .await
            .expect("cohere batch");
        assert_eq!(batch.usage.billed_input_tokens, expected);
        assert_eq!(batch.usage.prompt_tokens, None);
        assert_eq!(batch.usage.total_tokens, None);
    }
}
