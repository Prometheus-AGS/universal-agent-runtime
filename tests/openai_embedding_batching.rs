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
